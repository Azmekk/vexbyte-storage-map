//! The search index as it was in v0.1.1, before it got its own compact node arena: it held the
//! whole `FileTree`, a lowercase copy of every name and a path order table. Kept only for tests,
//! which check that the compact index returns exactly the same results.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::cmp::Ordering;
use std::ops::Range;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use rayon::prelude::*;

use super::search_query::{ExtensionAlternatives, ParsedQuery, TermScope};
use super::search_results::{SearchHit, SearchMatches, SearchSortOrder};
use super::text_pattern::TextPattern;
use crate::file_tree::{
    ExtensionId, FileTree, NodeFlags, NodeIndex, NodeKind, is_plausible_extension,
};
use crate::scan_method::ScanMethod;

const SEARCH_CHUNK_SIZE: usize = 16 * 1024;
const PATH_SEPARATOR_BYTE: u8 = MAIN_SEPARATOR as u8;
const NOT_SEARCHABLE: u32 = u32::MAX;

/// Every node name in lowercase, stored once when the index is built so a query never
/// lowercases names again. Indexed by arena position, so a pass over the nodes also reads the
/// names front to back.
#[derive(Debug, Clone)]
pub(crate) struct LowercaseNameTable {
    text: Vec<u8>,
    /// `offsets[position]..offsets[position + 1]` is the byte range of the node's name.
    offsets: Vec<u32>,
}

impl LowercaseNameTable {
    pub(crate) fn of_tree(tree: &FileTree) -> Self {
        let node_count = tree.nodes.len();
        let mut text = Vec::with_capacity(tree.names.name_text_bytes());
        let mut offsets = Vec::with_capacity(node_count + 1);
        offsets.push(0);

        for node in &tree.nodes {
            let name = tree.names.name(node.name);
            if name.is_ascii() {
                text.extend(name.bytes().map(|byte| byte.to_ascii_lowercase()));
            } else {
                text.extend_from_slice(name.to_lowercase().as_bytes());
            }
            offsets.push(u32::try_from(text.len()).unwrap_or(u32::MAX));
        }

        text.shrink_to_fit();
        Self { text, offsets }
    }

    pub(crate) fn name(&self, position: usize) -> &[u8] {
        let (Some(&start), Some(&end)) =
            (self.offsets.get(position), self.offsets.get(position + 1))
        else {
            return &[];
        };
        self.text.get(start as usize..end as usize).unwrap_or(&[])
    }

    pub(crate) fn memory_bytes(&self) -> usize {
        self.text.capacity() + self.offsets.capacity() * size_of::<u32>()
    }
}

/// Everything on one volume, prepared for fast name searches.
///
/// It owns the volume's `FileTree` and adds a lowercase copy of every name plus the position
/// of every node in path order, so a query neither lowercases names nor builds paths to sort.
/// It is immutable: a refresh builds a new index and replaces the old one as a whole.
#[derive(Debug)]
pub(crate) struct LegacyVolumeSearchIndex {
    mount_point: PathBuf,
    method: ScanMethod,
    pub(crate) tree: FileTree,
    pub(crate) lowercase_names: LowercaseNameTable,
    /// The position of each node when all searchable nodes are sorted by full path
    /// (component by component, ignoring case), or `NOT_SEARCHABLE`.
    pub(crate) path_ranks: Vec<u32>,
}

impl LegacyVolumeSearchIndex {
    /// Prepares `tree`, the scan of the volume mounted at `mount_point`, for searching. Takes
    /// about a second for three million nodes.
    #[must_use]
    pub fn from_file_tree(mount_point: PathBuf, tree: FileTree, method: ScanMethod) -> Self {
        let lowercase_names = LowercaseNameTable::of_tree(&tree);
        let path_ranks = rank_nodes_in_path_order(&tree, &lowercase_names);

        Self {
            mount_point,
            method,
            tree,
            lowercase_names,
            path_ranks,
        }
    }

    /// Where the volume is mounted, for example `C:\`.
    #[must_use]
    pub fn mount_point(&self) -> &Path {
        &self.mount_point
    }

    /// How the volume was read.
    #[must_use]
    pub fn method(&self) -> ScanMethod {
        self.method
    }

    #[must_use]
    pub fn tree(&self) -> &FileTree {
        &self.tree
    }

    #[must_use]
    pub fn file_count(&self) -> u32 {
        self.tree.totals().file_count
    }

    #[must_use]
    pub fn directory_count(&self) -> u32 {
        self.tree.totals().directory_count
    }

    /// Approximate heap memory of the tree plus the search tables.
    #[must_use]
    pub fn estimated_memory_bytes(&self) -> usize {
        self.tree.estimated_memory_bytes()
            + self.lowercase_names.memory_bytes()
            + self.path_ranks.capacity() * size_of::<u32>()
    }

    /// The node at exactly `path`, if `path` lies on this volume and appears in search results.
    /// Used to check that a path the user interface sends back really came from a search.
    #[must_use]
    pub fn find_searchable_node_by_path(&self, path: &Path) -> Option<NodeIndex> {
        let path_below_root = path.strip_prefix(self.tree.root_path()).ok()?;
        let node = self.tree.find_descendant_by_names(
            path_below_root
                .components()
                .map(std::path::Component::as_os_str),
        )?;

        let is_searchable = self.path_ranks.get(node.as_usize()) != Some(&NOT_SEARCHABLE);
        let has_exactly_this_path = self.tree.full_path(node).as_deref() == Some(path);
        (is_searchable && has_exactly_this_path).then_some(node)
    }

    pub(crate) fn is_searchable(&self, position: usize) -> bool {
        self.path_ranks[position] != NOT_SEARCHABLE
    }
}

/// Numbers the nodes in a depth-first walk that visits the children of every folder in
/// lowercase name order, which is the order of their full paths. Sorting results by path is
/// then a comparison of two integers.
fn rank_nodes_in_path_order(tree: &FileTree, lowercase_names: &LowercaseNameTable) -> Vec<u32> {
    let nodes = &tree.nodes;
    let mut path_ranks = vec![NOT_SEARCHABLE; nodes.len()];
    if nodes.is_empty() {
        return path_ranks;
    }

    let mut next_rank: u32 = 0;
    // Each entry is an arena position and whether the node has a known location on disk.
    let mut pending: Vec<(usize, bool)> = vec![(0, true)];

    while let Some((position, has_known_location)) = pending.pop() {
        let node = &nodes[position];
        let has_known_location =
            has_known_location && !node.flags.contains(NodeFlags::LOCATION_UNKNOWN);
        let is_searchable = position != 0
            && has_known_location
            && !node
                .flags
                .intersects(NodeFlags::SYNTHETIC | NodeFlags::REMOVED);
        if is_searchable {
            path_ranks[position] = next_rank;
            next_rank = next_rank.saturating_add(1);
        }

        let first_pending_child = pending.len();
        pending.extend(
            node.child_index_range()
                .filter(|&child| !nodes[child].is_removed())
                .map(|child| (child, has_known_location)),
        );
        // Pushed in descending order, so the smallest name is taken next.
        pending[first_pending_child..].sort_unstable_by(|(first, _), (second, _)| {
            lowercase_names
                .name(*second)
                .cmp(lowercase_names.name(*first))
                .then(second.cmp(first))
        });
    }

    path_ranks
}

/// Searches one volume and returns its `limit` best matches in `sort` order.
///
/// Every node is checked, in parallel on the rayon thread pool, so the results are the true
/// first `limit` items in that order and `total_match_count` is exact. Takes a few tens of
/// milliseconds for three million nodes; call it from a background thread.
#[must_use]
pub(crate) fn legacy_search_volume_index(
    index: &LegacyVolumeSearchIndex,
    query: &ParsedQuery,
    sort: SearchSortOrder,
    limit: usize,
) -> SearchMatches {
    if query.is_empty() || limit == 0 {
        return SearchMatches::default();
    }

    let plan = VolumeQueryPlan::new(index, query);
    let node_count = index.tree.nodes.len();
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

/// A total order: every searchable node has its own path rank, which breaks all ties.
fn compare_positions(
    index: &LegacyVolumeSearchIndex,
    sort: SearchSortOrder,
    first: u32,
    second: u32,
) -> Ordering {
    let (first, second) = (first as usize, second as usize);
    let by_path = || index.path_ranks[first].cmp(&index.path_ranks[second]);
    let first_node = &index.tree.nodes[first];
    let second_node = &index.tree.nodes[second];

    match sort {
        SearchSortOrder::NameAscending => index
            .lowercase_names
            .name(first)
            .cmp(index.lowercase_names.name(second))
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

fn search_hit_for(index: &LegacyVolumeSearchIndex, position: usize) -> Option<SearchHit> {
    let node_index = NodeIndex::from_usize(position);
    let node = index.tree.node(node_index)?;
    let path = index.tree.full_path(node_index)?;

    Some(SearchHit {
        volume_mount_point: index.mount_point().to_path_buf(),
        path,
        name: index.tree.name(node_index).to_owned(),
        is_directory: node.is_directory(),
        logical_size: node.logical_size,
        modified_at_unix_milliseconds: node.modified_time.unix_milliseconds(),
        category: index.tree.category(node_index),
        node: node_index,
    })
}

/// One `ext:` filter resolved against a volume's interned extensions.
struct ResolvedExtensionFilter {
    extension_ids: Vec<ExtensionId>,
    /// `.tar.gz` for `ext:tar.gz`: extensions that are never interned are matched by name.
    lowercase_name_suffixes: Vec<Vec<u8>>,
}

impl ResolvedExtensionFilter {
    fn resolve(filter: &ExtensionAlternatives, index: &LegacyVolumeSearchIndex) -> Self {
        let mut extension_ids = Vec::new();
        let mut lowercase_name_suffixes = Vec::new();

        for extension in &filter.lowercase_extensions {
            if is_plausible_extension(extension) {
                // An extension no file on the volume has is simply not found.
                extension_ids.extend(index.tree.names.interned_extension_id(extension));
            } else {
                lowercase_name_suffixes.push(format!(".{extension}").into_bytes());
            }
        }

        Self {
            extension_ids,
            lowercase_name_suffixes,
        }
    }

    fn matches(&self, extension: ExtensionId, lowercase_name: &[u8]) -> bool {
        self.extension_ids.contains(&extension)
            || self
                .lowercase_name_suffixes
                .iter()
                .any(|suffix| lowercase_name.ends_with(suffix))
    }
}

/// A query prepared for one volume.
struct VolumeQueryPlan<'search> {
    index: &'search LegacyVolumeSearchIndex,
    query: &'search ParsedQuery,
    extension_filters: Vec<ResolvedExtensionFilter>,
    name_patterns: Vec<&'search TextPattern>,
    path_patterns: Vec<&'search TextPattern>,
}

impl<'search> VolumeQueryPlan<'search> {
    fn new(index: &'search LegacyVolumeSearchIndex, query: &'search ParsedQuery) -> Self {
        let patterns_in = |scope: TermScope| {
            query
                .terms
                .iter()
                .filter(|term| term.scope == scope)
                .map(|term| &term.pattern)
                .collect::<Vec<_>>()
        };

        Self {
            index,
            query,
            extension_filters: query
                .extension_filters
                .iter()
                .map(|filter| ResolvedExtensionFilter::resolve(filter, index))
                .collect(),
            name_patterns: patterns_in(TermScope::Name),
            path_patterns: patterns_in(TermScope::FullPath),
        }
    }

    fn matching_positions(&self, positions: Range<usize>) -> ChunkMatches {
        let mut chunk_matches = ChunkMatches::default();
        let mut path_builder = LowercasePathBuilder::default();

        for position in positions {
            if self.matches_without_path(position)
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
    fn matches_without_path(&self, position: usize) -> bool {
        if !self.index.is_searchable(position) {
            return false;
        }
        let node = &self.index.tree.nodes[position];

        if !self
            .query
            .kind_filters
            .iter()
            .all(|&kind| kind == node.kind)
        {
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

        let lowercase_name = self.index.lowercase_names.name(position);
        if !self.extension_filters.is_empty() {
            let is_file = node.kind == NodeKind::File;
            let matches_every_filter = self
                .extension_filters
                .iter()
                .all(|filter| filter.matches(node.extension, lowercase_name));
            if !is_file || !matches_every_filter {
                return false;
            }
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
}

impl Default for LowercasePathBuilder {
    fn default() -> Self {
        Self {
            cached_parent: usize::MAX,
            cached_parent_path: Vec::new(),
            full_path: Vec::new(),
            ancestors: Vec::new(),
        }
    }
}

impl LowercasePathBuilder {
    fn lowercase_path_of(&mut self, index: &LegacyVolumeSearchIndex, position: usize) -> &[u8] {
        let parent = index.tree.nodes[position]
            .parent()
            .map_or(usize::MAX, NodeIndex::as_usize);
        if parent != self.cached_parent {
            self.cached_parent = parent;
            self.rebuild_parent_path(index, parent);
        }

        self.full_path.clear();
        self.full_path.extend_from_slice(&self.cached_parent_path);
        push_path_component(&mut self.full_path, index.lowercase_names.name(position));
        &self.full_path
    }

    /// Synthetic grouping folders are not part of a path, as in `FileTree::full_path`.
    fn rebuild_parent_path(&mut self, index: &LegacyVolumeSearchIndex, parent: usize) {
        self.cached_parent_path.clear();
        if parent == usize::MAX {
            return;
        }

        self.ancestors.clear();
        let mut current = Some(NodeIndex::from_usize(parent));
        while let Some(ancestor) = current {
            let ancestor_node = &index.tree.nodes[ancestor.as_usize()];
            let is_root = ancestor_node.parent().is_none();
            if is_root || !ancestor_node.flags.contains(NodeFlags::SYNTHETIC) {
                self.ancestors.push(ancestor.as_usize());
            }
            current = ancestor_node.parent();
        }

        for &ancestor in self.ancestors.iter().rev() {
            let name = index.lowercase_names.name(ancestor);
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
