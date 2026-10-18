use std::cmp::{Ordering, Reverse};
use std::ops::Range;
use std::path::{MAIN_SEPARATOR, PathBuf};

use file_categories::FileCategory;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::lowercase_names::lowercase_name;
use super::search_index_node::SearchNodeFlags;
use super::search_query::{ExtensionAlternatives, ParsedQuery, TermScope};
use super::text_pattern::TextPattern;
use super::volume_search_index::VolumeSearchIndex;
use crate::file_tree::{ExtensionId, NodeIndex, NodeKind, is_plausible_extension};

/// Nodes are matched in chunks of this many in parallel; large enough that the per-task
/// overhead is negligible, small enough that the work spreads evenly over all cores.
const SEARCH_CHUNK_SIZE: usize = 16 * 1024;
const PATH_SEPARATOR_BYTE: u8 = MAIN_SEPARATOR as u8;

/// The order of search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SearchSortOrder {
    NameAscending,
    SizeDescending,
    /// Newest first; items without a known modification time come last.
    ModifiedDescending,
    PathAscending,
}

/// One file or folder found by a search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    /// The volume the item is on, for example `C:\`.
    pub volume_mount_point: PathBuf,
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    /// The file size; for a folder, the total size of everything inside it.
    pub logical_size: u64,
    pub modified_at_unix_milliseconds: Option<u64>,
    /// The file type category by extension; `None` for folders and unknown extensions.
    pub category: Option<FileCategory>,
    /// The item's node in that volume's index. Only meaningful together with the volume, and
    /// only until the index is rebuilt.
    pub node: NodeIndex,
}

/// The best results of a search in the requested order and how many items matched in total.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchMatches {
    pub hits: Vec<SearchHit>,
    pub total_match_count: u64,
}

/// Searches one volume and returns its `limit` best matches in `sort` order.
///
/// Every node is checked, in parallel on the rayon thread pool, so the results are the true
/// first `limit` items in that order and `total_match_count` is exact. Takes a few tens of
/// milliseconds for three million nodes; call it from a background thread.
#[must_use]
pub fn search_volume_index(
    index: &VolumeSearchIndex,
    query: &ParsedQuery,
    sort: SearchSortOrder,
    limit: usize,
) -> SearchMatches {
    if query.is_empty() || limit == 0 {
        return SearchMatches::default();
    }

    let plan = VolumeQueryPlan::new(index, query);
    let node_count = index.nodes.len();
    let chunk_count = node_count.div_ceil(SEARCH_CHUNK_SIZE);
    let compare = |first: &u32, second: &u32| compare_positions(index, sort, *first, *second);

    let best_matches = (0..chunk_count)
        .into_par_iter()
        .map(|chunk| {
            let chunk_start = chunk * SEARCH_CHUNK_SIZE;
            let chunk_end = (chunk_start + SEARCH_CHUNK_SIZE).min(node_count);
            let mut chunk_matches = plan.matching_positions(chunk_start..chunk_end);
            keep_best(&mut chunk_matches.positions, limit, compare);
            chunk_matches
        })
        .reduce(ChunkMatches::default, |mut combined, next| {
            combined.match_count += next.match_count;
            combined.positions.extend(next.positions);
            if combined.positions.len() > limit.saturating_mul(2) {
                keep_best(&mut combined.positions, limit, compare);
            }
            combined
        });

    let mut best_positions = best_matches.positions;
    keep_best(&mut best_positions, limit, compare);
    best_positions.sort_unstable_by(compare);

    let hits = best_positions
        .par_iter()
        .filter_map(|&position| search_hit_for(index, position as usize))
        .collect();

    SearchMatches {
        hits,
        total_match_count: best_matches.match_count,
    }
}

/// Searches every volume and merges their results. Give the volumes ordered by mount point:
/// path order is volume by volume in the order given, and ties in the other orders keep it.
#[must_use]
pub fn search_volume_indexes<'index>(
    indexes: impl IntoIterator<Item = &'index VolumeSearchIndex>,
    query: &ParsedQuery,
    sort: SearchSortOrder,
    limit: usize,
) -> SearchMatches {
    let mut merged = SearchMatches::default();
    for index in indexes {
        let volume_matches = search_volume_index(index, query, sort, limit);
        merged.total_match_count += volume_matches.total_match_count;
        merged.hits.extend(volume_matches.hits);
    }

    sort_hits_of_several_volumes(&mut merged.hits, sort);
    merged.hits.truncate(limit);
    merged
}

/// Stable sorts, so each volume's own order (which breaks ties by path) survives.
pub(super) fn sort_hits_of_several_volumes(hits: &mut [SearchHit], sort: SearchSortOrder) {
    match sort {
        SearchSortOrder::NameAscending => hits.sort_by_cached_key(|hit| hit.name.to_lowercase()),
        SearchSortOrder::SizeDescending => {
            hits.sort_by_key(|hit| Reverse(hit.logical_size));
        }
        SearchSortOrder::ModifiedDescending => {
            hits.sort_by_key(|hit| Reverse(hit.modified_at_unix_milliseconds));
        }
        SearchSortOrder::PathAscending => {}
    }
}

#[derive(Default)]
struct ChunkMatches {
    match_count: u64,
    positions: Vec<u32>,
}

/// Keeps the `limit` smallest positions according to `compare`, in no particular order.
fn keep_best(positions: &mut Vec<u32>, limit: usize, compare: impl Fn(&u32, &u32) -> Ordering) {
    if positions.len() <= limit {
        return;
    }
    if limit == 0 {
        positions.clear();
        return;
    }
    positions.select_nth_unstable_by(limit - 1, compare);
    positions.truncate(limit);
}

/// A total order: no two searchable nodes have the same place in path order, which breaks all
/// ties.
fn compare_positions(
    index: &VolumeSearchIndex,
    sort: SearchSortOrder,
    first: u32,
    second: u32,
) -> Ordering {
    let (first, second) = (first as usize, second as usize);
    let by_path = || index.compare_in_path_order(first, second);
    let first_node = &index.nodes[first];
    let second_node = &index.nodes[second];

    match sort {
        SearchSortOrder::NameAscending => index
            .compare_names_ignoring_case(first, second)
            .then_with(by_path),
        SearchSortOrder::SizeDescending => second_node
            .logical_size
            .cmp(&first_node.logical_size)
            .then_with(by_path),
        SearchSortOrder::ModifiedDescending => second_node
            .modified_time
            .cmp(&first_node.modified_time)
            .then_with(by_path),
        SearchSortOrder::PathAscending => by_path(),
    }
}

fn search_hit_for(index: &VolumeSearchIndex, position: usize) -> Option<SearchHit> {
    let node = index.nodes.get(position)?;
    let path = index.full_path(position)?;

    Some(SearchHit {
        volume_mount_point: index.mount_point().to_path_buf(),
        path,
        name: index.name_at(position).to_owned(),
        is_directory: node.kind() == NodeKind::Directory,
        logical_size: node.logical_size,
        modified_at_unix_milliseconds: node.modified_time.unix_milliseconds(),
        category: index.names.extension_category(node.extension),
        node: NodeIndex::from_usize(position),
    })
}

/// One `ext:` filter resolved against a volume's interned extensions.
struct ResolvedExtensionFilter {
    extension_ids: Vec<ExtensionId>,
    /// `.tar.gz` for `ext:tar.gz`: extensions that are never interned are matched by name.
    lowercase_name_suffixes: Vec<Vec<u8>>,
}

impl ResolvedExtensionFilter {
    fn resolve(filter: &ExtensionAlternatives, index: &VolumeSearchIndex) -> Self {
        let mut extension_ids = Vec::new();
        let mut lowercase_name_suffixes = Vec::new();

        for extension in &filter.lowercase_extensions {
            if is_plausible_extension(extension) {
                // An extension no file on the volume has is simply not found.
                extension_ids.extend(index.names.interned_extension_id(extension));
            } else {
                lowercase_name_suffixes.push(format!(".{extension}").into_bytes());
            }
        }

        Self {
            extension_ids,
            lowercase_name_suffixes,
        }
    }

    fn matches_extension_id(&self, extension: ExtensionId) -> bool {
        self.extension_ids.contains(&extension)
    }

    fn matches_name_suffix(&self, lowercase_name: &[u8]) -> bool {
        self.lowercase_name_suffixes
            .iter()
            .any(|suffix| lowercase_name.ends_with(suffix))
    }
}

/// A query prepared for one volume.
struct VolumeQueryPlan<'search> {
    index: &'search VolumeSearchIndex,
    query: &'search ParsedQuery,
    extension_filters: Vec<ResolvedExtensionFilter>,
    name_patterns: Vec<&'search TextPattern>,
    path_patterns: Vec<&'search TextPattern>,
    /// False when only sizes, kinds and interned extensions are compared, so names are not
    /// even lowercased.
    needs_lowercase_name: bool,
}

impl<'search> VolumeQueryPlan<'search> {
    fn new(index: &'search VolumeSearchIndex, query: &'search ParsedQuery) -> Self {
        let patterns_in = |scope: TermScope| {
            query
                .terms
                .iter()
                .filter(|term| term.scope == scope)
                .map(|term| &term.pattern)
                .collect::<Vec<_>>()
        };

        let extension_filters: Vec<ResolvedExtensionFilter> = query
            .extension_filters
            .iter()
            .map(|filter| ResolvedExtensionFilter::resolve(filter, index))
            .collect();
        let name_patterns = patterns_in(TermScope::Name);
        let needs_lowercase_name = !name_patterns.is_empty()
            || extension_filters
                .iter()
                .any(|filter| !filter.lowercase_name_suffixes.is_empty());

        Self {
            index,
            query,
            extension_filters,
            name_patterns,
            path_patterns: patterns_in(TermScope::FullPath),
            needs_lowercase_name,
        }
    }

    fn matching_positions(&self, positions: Range<usize>) -> ChunkMatches {
        let mut chunk_matches = ChunkMatches::default();
        let mut path_builder = LowercasePathBuilder::default();
        let mut name_buffer = Vec::new();

        for position in positions {
            if self.matches_without_path(position, &mut name_buffer)
                && self.matches_path_terms(position, &mut path_builder)
            {
                chunk_matches.match_count += 1;
                chunk_matches
                    .positions
                    .push(u32::try_from(position).unwrap_or(u32::MAX));
            }
        }
        chunk_matches
    }

    /// The cheap checks first: flags, kind, size, extension, then the name terms.
    fn matches_without_path(&self, position: usize, name_buffer: &mut Vec<u8>) -> bool {
        let node = &self.index.nodes[position];
        if !node.is_searchable() {
            return false;
        }

        let kind = node.kind();
        if !self.query.kind_filters.iter().all(|&wanted| wanted == kind) {
            return false;
        }
        if !self
            .query
            .size_ranges
            .iter()
            .all(|range| range.contains(node.logical_size))
        {
            return false;
        }

        if !self.extension_filters.is_empty() && kind != NodeKind::File {
            return false;
        }
        if !self.needs_lowercase_name {
            return self
                .extension_filters
                .iter()
                .all(|filter| filter.matches_extension_id(node.extension));
        }

        let lowercase_name = lowercase_name(self.index.name_at(position), node.flags, name_buffer);
        let matches_every_extension_filter = self.extension_filters.iter().all(|filter| {
            filter.matches_extension_id(node.extension)
                || filter.matches_name_suffix(lowercase_name)
        });
        if !matches_every_extension_filter {
            return false;
        }

        self.name_patterns
            .iter()
            .all(|pattern| pattern.matches(lowercase_name))
    }

    fn matches_path_terms(&self, position: usize, path_builder: &mut LowercasePathBuilder) -> bool {
        if self.path_patterns.is_empty() {
            return true;
        }
        let lowercase_path = path_builder.lowercase_path_of(self.index, position);
        self.path_patterns
            .iter()
            .all(|pattern| pattern.matches(lowercase_path))
    }
}

/// Builds lowercase full paths for path terms. The arena stores siblings next to each other,
/// so consecutive nodes nearly always share a parent and its path is built only once.
struct LowercasePathBuilder {
    cached_parent: usize,
    cached_parent_path: Vec<u8>,
    full_path: Vec<u8>,
    ancestors: Vec<usize>,
    name_buffer: Vec<u8>,
}

impl Default for LowercasePathBuilder {
    fn default() -> Self {
        Self {
            cached_parent: usize::MAX,
            cached_parent_path: Vec::new(),
            full_path: Vec::new(),
            ancestors: Vec::new(),
            name_buffer: Vec::new(),
        }
    }
}

impl LowercasePathBuilder {
    fn lowercase_path_of(&mut self, index: &VolumeSearchIndex, position: usize) -> &[u8] {
        let node = &index.nodes[position];
        let parent = node.parent().map_or(usize::MAX, NodeIndex::as_usize);
        if parent != self.cached_parent {
            self.cached_parent = parent;
            self.rebuild_parent_path(index, parent);
        }

        self.full_path.clear();
        self.full_path.extend_from_slice(&self.cached_parent_path);
        let name = lowercase_name(index.name_at(position), node.flags, &mut self.name_buffer);
        push_path_component(&mut self.full_path, name);
        &self.full_path
    }

    /// Synthetic grouping folders are not part of a path, as in `VolumeSearchIndex::full_path`.
    fn rebuild_parent_path(&mut self, index: &VolumeSearchIndex, parent: usize) {
        self.cached_parent_path.clear();
        if parent == usize::MAX {
            return;
        }

        self.ancestors.clear();
        let mut current = Some(NodeIndex::from_usize(parent));
        while let Some(ancestor) = current {
            let ancestor_node = &index.nodes[ancestor.as_usize()];
            let is_root = ancestor_node.parent().is_none();
            if is_root || !ancestor_node.flags.contains(SearchNodeFlags::SYNTHETIC) {
                self.ancestors.push(ancestor.as_usize());
            }
            current = ancestor_node.parent();
        }

        for &ancestor in self.ancestors.iter().rev() {
            let flags = index.nodes[ancestor].flags;
            let name = lowercase_name(index.name_at(ancestor), flags, &mut self.name_buffer);
            if self.cached_parent_path.is_empty() {
                self.cached_parent_path.extend_from_slice(name);
            } else {
                push_path_component(&mut self.cached_parent_path, name);
            }
        }
    }
}

fn push_path_component(path: &mut Vec<u8>, component: &[u8]) {
    if path.last().is_some_and(|&last| last != PATH_SEPARATOR_BYTE) {
        path.push(PATH_SEPARATOR_BYTE);
    }
    path.extend_from_slice(component);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::Path;
    use std::time::{Duration, SystemTime};

    use super::*;
    use crate::example_paths::{
        example_path, example_volume_root, example_volume_root_query, path_below,
        second_example_volume_root,
    };
    use crate::file_tree::test_trees::entry;
    use crate::file_tree::{DirectoryKey, FileTreeBuilder, NewEntry};
    use crate::scan_method::ScanMethod;

    fn modified_at(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
    }

    /// C:\
    /// ├─ Users/
    /// │  └─ me/
    /// │     ├─ Photos/
    /// │     │  ├─ Beach.JPG      4000  (t=300)
    /// │     │  ├─ city.png       3000  (t=100)
    /// │     │  └─ holiday 2024/
    /// │     │     └─ dive.jpg    9000  (t=200)
    /// │     └─ report.pdf        500
    /// ├─ NTFS metadata/ (synthetic)
    /// │  └─ $MFT                 1000
    /// ├─ beach-notes.txt         20
    /// └─ archive.tar.gz          7000
    fn sample_index_on(mount_point: &Path) -> VolumeSearchIndex {
        let mut builder = FileTreeBuilder::new(mount_point.to_path_buf(), DirectoryKey::new(0));
        builder.add_entry(&entry("Users", NodeKind::Directory, 0, Some(1), 0));
        builder.add_entry(&entry("me", NodeKind::Directory, 1, Some(2), 0));
        builder.add_entry(&entry("Photos", NodeKind::Directory, 2, Some(3), 0));
        builder.add_entry(&NewEntry {
            modified_time: Some(modified_at(300)),
            ..entry("Beach.JPG", NodeKind::File, 3, None, 4000)
        });
        builder.add_entry(&NewEntry {
            modified_time: Some(modified_at(100)),
            ..entry("city.png", NodeKind::File, 3, None, 3000)
        });
        builder.add_entry(&entry("holiday 2024", NodeKind::Directory, 3, Some(4), 0));
        builder.add_entry(&NewEntry {
            modified_time: Some(modified_at(200)),
            ..entry("dive.jpg", NodeKind::File, 4, None, 9000)
        });
        builder.add_entry(&entry("report.pdf", NodeKind::File, 2, None, 500));
        builder.add_synthetic_directory(
            "NTFS metadata",
            Some(DirectoryKey::new(0)),
            DirectoryKey::new(5),
        );
        builder.add_entry(&entry("$MFT", NodeKind::File, 5, None, 1000));
        builder.add_entry(&entry("beach-notes.txt", NodeKind::File, 0, None, 20));
        builder.add_entry(&entry("archive.tar.gz", NodeKind::File, 0, None, 7000));

        VolumeSearchIndex::from_file_tree(
            mount_point.to_path_buf(),
            builder.finish(),
            ScanMethod::FolderWalk,
        )
    }

    fn search(index: &VolumeSearchIndex, query: &str, sort: SearchSortOrder) -> Vec<String> {
        let parsed_query = ParsedQuery::parse(query).unwrap();
        search_volume_index(index, &parsed_query, sort, 100)
            .hits
            .into_iter()
            .map(|hit| hit.name)
            .collect()
    }

    fn names(query: &str) -> Vec<String> {
        search(
            &sample_index_on(&example_volume_root()),
            query,
            SearchSortOrder::NameAscending,
        )
    }

    #[test]
    fn finds_substrings_of_names_ignoring_case() {
        assert_eq!(names("beach"), ["beach-notes.txt", "Beach.JPG"]);
        assert_eq!(names("PHOTO"), ["Photos"]);
        assert_eq!(names("nothing-like-this"), Vec::<String>::new());
    }

    #[test]
    fn every_term_must_match() {
        assert_eq!(names("beach txt"), ["beach-notes.txt"]);
        assert_eq!(names("\"holiday 2024\""), ["holiday 2024"]);
        assert_eq!(names("holiday 2023"), Vec::<String>::new());
    }

    #[test]
    fn globs_match_whole_names() {
        assert_eq!(names("*.jpg"), ["Beach.JPG", "dive.jpg"]);
        assert_eq!(names("c?ty.*"), ["city.png"]);
        assert_eq!(names("*.jp"), Vec::<String>::new());
    }

    #[test]
    fn terms_with_a_separator_match_the_full_path() {
        assert_eq!(
            names(r"photos\"),
            ["Beach.JPG", "city.png", "dive.jpg", "holiday 2024"]
        );
        assert_eq!(names("me/photos/holiday"), ["dive.jpg", "holiday 2024"]);
        let root = example_volume_root_query();
        assert_eq!(names(&format!(r"{root}users\me\report")), ["report.pdf"]);
        assert_eq!(names(r"photos\ jpg"), ["Beach.JPG", "dive.jpg"]);
        assert_eq!(names(&format!("{root}*.pdf")), ["report.pdf"]);
    }

    #[test]
    fn synthetic_folders_are_not_part_of_paths_or_results() {
        let root = example_volume_root_query();
        assert_eq!(names(&format!("{root}$mft")), ["$MFT"]);
        assert_eq!(names("metadata"), Vec::<String>::new());
        assert_eq!(names(r"metadata\"), Vec::<String>::new());
    }

    #[test]
    fn filters_by_extension() {
        assert_eq!(names("ext:jpg"), ["Beach.JPG", "dive.jpg"]);
        assert_eq!(names("ext:png;pdf"), ["city.png", "report.pdf"]);
        assert_eq!(names("ext:tar.gz"), ["archive.tar.gz"]);
        assert_eq!(names("ext:gz"), ["archive.tar.gz"]);
        assert_eq!(names("ext:heic"), Vec::<String>::new());
        assert_eq!(names("ext:jpg ext:png"), Vec::<String>::new());
        assert_eq!(names("beach ext:jpg"), ["Beach.JPG"]);
    }

    #[test]
    fn filters_by_size_and_type() {
        assert_eq!(names("size:>5kb type:file"), ["archive.tar.gz", "dive.jpg"]);
        assert_eq!(names("size:1000..3000"), ["$MFT", "city.png"]);
        assert_eq!(
            names("type:folder"),
            ["holiday 2024", "me", "Photos", "Users"]
        );
        assert_eq!(names("folder: size:>15kb"), ["me", "Photos", "Users"]);
        assert_eq!(names("type:file type:folder"), Vec::<String>::new());
    }

    #[test]
    fn sorts_by_each_order() {
        let index = sample_index_on(&example_volume_root());

        assert_eq!(
            search(&index, "ext:jpg;png;pdf", SearchSortOrder::SizeDescending),
            ["dive.jpg", "Beach.JPG", "city.png", "report.pdf"]
        );
        assert_eq!(
            search(
                &index,
                "ext:jpg;png;pdf",
                SearchSortOrder::ModifiedDescending
            ),
            ["Beach.JPG", "dive.jpg", "city.png", "report.pdf"]
        );
        assert_eq!(
            search(&index, "ext:jpg;png;pdf", SearchSortOrder::PathAscending),
            ["Beach.JPG", "city.png", "dive.jpg", "report.pdf"]
        );
    }

    #[test]
    fn hits_carry_paths_and_details() {
        let index = sample_index_on(&example_volume_root());
        let query = ParsedQuery::parse("dive").unwrap();

        let hit = search_volume_index(&index, &query, SearchSortOrder::NameAscending, 10)
            .hits
            .remove(0);

        assert_eq!(
            hit.path,
            example_path("Users/me/Photos/holiday 2024/dive.jpg")
        );
        assert_eq!(hit.volume_mount_point, example_volume_root());
        assert!(!hit.is_directory);
        assert_eq!(hit.logical_size, 9000);
        assert_eq!(hit.modified_at_unix_milliseconds, Some(200_000));
        assert_eq!(hit.category, Some(FileCategory::Images));
        assert_eq!(
            index.find_searchable_node_by_path(&hit.path),
            Some(hit.node)
        );
    }

    #[test]
    fn limits_results_but_counts_every_match() {
        let index = sample_index_on(&example_volume_root());
        let query = ParsedQuery::parse("type:file").unwrap();

        let matches = search_volume_index(&index, &query, SearchSortOrder::SizeDescending, 2);

        let names: Vec<_> = matches.hits.iter().map(|hit| hit.name.as_str()).collect();
        assert_eq!(names, ["dive.jpg", "archive.tar.gz"]);
        assert_eq!(matches.total_match_count, 7);
    }

    #[test]
    fn an_empty_query_or_zero_limit_finds_nothing() {
        let index = sample_index_on(&example_volume_root());

        let empty = ParsedQuery::parse("  ").unwrap();
        let some = ParsedQuery::parse("beach").unwrap();

        assert_eq!(
            search_volume_index(&index, &empty, SearchSortOrder::NameAscending, 10),
            SearchMatches::default()
        );
        assert_eq!(
            search_volume_index(&index, &some, SearchSortOrder::NameAscending, 0),
            SearchMatches::default()
        );
    }

    #[test]
    fn merges_volumes_in_the_requested_order() {
        let c_volume = sample_index_on(&example_volume_root());
        let d_volume = sample_index_on(&second_example_volume_root());
        let query = ParsedQuery::parse("ext:jpg").unwrap();

        let by_size = search_volume_indexes(
            [&c_volume, &d_volume],
            &query,
            SearchSortOrder::SizeDescending,
            3,
        );
        let by_path = search_volume_indexes(
            [&c_volume, &d_volume],
            &query,
            SearchSortOrder::PathAscending,
            10,
        );

        let described = |matches: &SearchMatches| -> Vec<PathBuf> {
            matches.hits.iter().map(|hit| hit.path.clone()).collect()
        };
        let on_c = |relative_path| path_below(&example_volume_root(), relative_path);
        let on_d = |relative_path| path_below(&second_example_volume_root(), relative_path);
        assert_eq!(
            described(&by_size),
            [
                on_c("Users/me/Photos/holiday 2024/dive.jpg"),
                on_d("Users/me/Photos/holiday 2024/dive.jpg"),
                on_c("Users/me/Photos/Beach.JPG"),
            ]
        );
        assert_eq!(by_size.total_match_count, 4);
        assert_eq!(
            described(&by_path),
            [
                on_c("Users/me/Photos/Beach.JPG"),
                on_c("Users/me/Photos/holiday 2024/dive.jpg"),
                on_d("Users/me/Photos/Beach.JPG"),
                on_d("Users/me/Photos/holiday 2024/dive.jpg"),
            ]
        );
    }

    #[test]
    fn keep_best_keeps_the_smallest() {
        let mut positions = vec![9, 3, 7, 1, 8, 2];

        keep_best(&mut positions, 3, u32::cmp);
        positions.sort_unstable();

        assert_eq!(positions, [1, 2, 3]);
    }
}
