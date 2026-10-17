//! Checks that the compact `VolumeSearchIndex` answers every query exactly like the v0.1.1
//! index in `legacy_search_index`: the same hits (paths, names, sizes, times, kinds, categories
//! and node indexes) in the same order and the same total match count, on hand-made trees with
//! tricky names and on randomized trees.
//!
//! The randomized run over 50 seeds of 200,000 nodes is ignored by default because it takes a
//! few minutes in debug builds; run it in release mode:
//!
//! ```text
//! cargo test -p disk_scanner --release equivalence -- --ignored --nocapture
//! ```

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::too_many_lines
)]

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::legacy_search_index::{LegacyVolumeSearchIndex, legacy_search_volume_index};
use super::search_query::ParsedQuery;
use super::search_results::{
    SearchMatches, SearchSortOrder, search_volume_index, search_volume_indexes,
};
use super::volume_search_index::VolumeSearchIndex;
use crate::file_identity::FileIdentity;
use crate::file_tree::{
    DirectoryKey, FileTree, FileTreeBuilder, NewEntry, NodeFlags, NodeIndex, NodeKind,
};
use crate::scan_method::ScanMethod;

const SORT_ORDERS: [SearchSortOrder; 4] = [
    SearchSortOrder::NameAscending,
    SearchSortOrder::SizeDescending,
    SearchSortOrder::ModifiedDescending,
    SearchSortOrder::PathAscending,
];
const LIMITS: [usize; 3] = [1, 5, 10_000];

/// Substrings, globs, phrases, path terms, every filter form, combinations, empty and invalid
/// queries.
const QUERIES: &[&str] = &[
    "report",
    "REPORT",
    "Report 2024",
    "photo",
    "img_",
    "IMG_????.JPG",
    "img_*.jpg",
    "*.txt",
    "*.TXT",
    "*.tar.gz",
    "*",
    "**",
    "?",
    "??",
    "*a*e*",
    "a?c*",
    "*.",
    ".",
    "..",
    "...",
    "ärger",
    "ÄRGER",
    "Ärger.TXT",
    "straße",
    "STRASSE",
    "strasse",
    "ß",
    "οδος",
    "ΟΔΟΣ",
    "σ",
    "istanbul",
    "i̇stanbul",
    "İSTANBUL",
    "\u{212A}",
    "k",
    "kelvin",
    "日本",
    "日本語.txt",
    "*語*",
    "?本語*",
    "\"annual report\"",
    "\"report \"",
    "\" \"",
    "\"ext:jpg\"",
    "note:2024",
    r"\",
    "/",
    r"c:\",
    r"C:\photos\",
    "photos/2024",
    r"photos\ jpg",
    r"c:\*\*.jpg",
    r"c:\*",
    r"*\report*",
    r"\report",
    r"ärger\",
    r"$mft",
    r"c:\$mft",
    "metadata",
    r"ntfs metadata\",
    "unlinked",
    "orphan",
    "ext:jpg",
    "ext:JPG;png",
    "ext:.txt,pdf",
    "ext:tar.gz",
    "ext:gz",
    "ext:verylongextension-name",
    "ext:",
    "ext:jpg ext:png",
    "size:0",
    "size:>1kb",
    "size:<1kb",
    "size:>=1mb",
    "size:<=10kb",
    "size:1kb..1mb",
    "size:=1.5kb",
    "size:4096",
    "size:<0",
    "type:file",
    "type:folder",
    "file:",
    "folder:",
    "folder:photo",
    "file:report ext:txt size:>0",
    "report size:<1mb type:file",
    "ärger ext:txt",
    "photo 2024 size:>=0",
    "",
    "   ",
    "\t \u{3000}",
    "dm:2024",
    "size:abc",
    "type:something",
    "size:",
    "type:",
];

/// A small xorshift generator, so every seed builds the same tree on every machine.
struct Xorshift(u64);

impl Xorshift {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }

    fn pick<'item, T>(&mut self, items: &'item [T]) -> &'item T {
        &items[self.below(items.len())]
    }
}

const NAME_PARTS: [&str; 40] = [
    "report",
    "Report",
    "REPORT",
    "annual report",
    "photo",
    "Photos",
    "IMG_",
    "img_",
    "2024",
    "0001",
    "Ärger",
    "ärger",
    "straße",
    "STRASSE",
    "ΟΔΟΣ",
    "οδοσ",
    "İstanbul",
    "\u{212A}elvin",
    "kelvin",
    "日本語",
    "a",
    "A",
    "b",
    "abc",
    "ABC",
    "aXe",
    " ",
    "  ",
    ".",
    "..",
    "-",
    "_",
    "$",
    "~",
    "notes",
    "Zebra",
    "z",
    "\u{1F600}",
    "e\u{301}",
    "é",
];
const EXTENSIONS: [&str; 16] = [
    "",
    ".txt",
    ".TXT",
    ".jpg",
    ".JPG",
    ".png",
    ".pdf",
    ".tar.gz",
    ".gz",
    ".",
    ". ",
    ".verylongextension-name",
    ".Jpg ",
    ".dll",
    ".日本",
    ".ÄRG",
];

fn random_name(random: &mut Xorshift) -> String {
    let mut name = String::new();
    if random.chance(1) {
        for _ in 0..random.below(60) + 60 {
            name.push_str(random.pick(&NAME_PARTS));
        }
        return name;
    }
    for _ in 0..=random.below(3) {
        name.push_str(random.pick(&NAME_PARTS));
    }
    name.push_str(random.pick(&EXTENSIONS));
    // "." and ".." cannot be file names.
    if name.is_empty() || name == "." || name == ".." {
        name.push('x');
    }
    name
}

fn time_at(seconds: u64, milliseconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds) + Duration::from_millis(milliseconds)
}

/// A tree of about `node_count` nodes with random names, sizes, times and flags, deep and wide
/// folders, identical names in one folder and in different folders, hard links, cloud
/// placeholders, inaccessible folders, synthetic folders and unlinked items.
fn random_tree(seed: u64, node_count: usize) -> FileTree {
    let mut random = Xorshift::new(seed);
    let mut builder =
        FileTreeBuilder::with_capacity(PathBuf::from(r"C:\"), DirectoryKey::new(0), node_count);
    let mut directory_keys: Vec<u32> = vec![0];
    let mut next_key = 1_u32;

    builder.add_synthetic_directory(
        "NTFS metadata",
        Some(DirectoryKey::new(0)),
        DirectoryKey::new(next_key),
    );
    let metadata_key = next_key;
    next_key += 1;
    for name in ["$MFT", "$LogFile", "$Bitmap", "report.txt"] {
        builder.add_entry(&NewEntry {
            name,
            kind: NodeKind::File,
            parent: Some(DirectoryKey::new(metadata_key)),
            directory_key: None,
            logical_size: 4096,
            allocated_size: 4096,
            modified_time: None,
            flags: NodeFlags::SYSTEM,
            hard_link_identity: None,
        });
    }

    let sizes = [
        0_u64,
        0,
        1,
        1023,
        1024,
        1536,
        4096,
        10_240,
        1_048_576,
        5_000_000,
        123_456_789,
    ];
    let mut name = String::new();
    while builder.entry_count() < node_count {
        let recent_directories = directory_keys.len().min(50);
        let parent_key = if random.chance(60) {
            directory_keys[directory_keys.len() - 1 - random.below(recent_directories)]
        } else {
            *random.pick(&directory_keys)
        };
        let parent = match random.below(1000) {
            0 => None,
            1 => Some(DirectoryKey::new(u32::MAX - 7)),
            2 => Some(DirectoryKey::new(metadata_key)),
            _ => Some(DirectoryKey::new(parent_key)),
        };

        name.clear();
        if random.chance(5) {
            // The same name again, often in another case: a case-sensitive folder or another
            // folder.
            name.push_str(random.pick(&["report.txt", "Report.TXT", "photo", "PHOTO", "a", "A"]));
        } else {
            name.push_str(&random_name(&mut random));
        }

        let is_directory = random.chance(15);
        let modified_time = if random.chance(20) {
            None
        } else {
            Some(time_at(
                1_000_000_000 + random.below(500) as u64 * 3600,
                random.below(3) as u64 * 7,
            ))
        };

        if is_directory {
            let key = next_key;
            next_key += 1;
            if random.chance(2) {
                builder.add_synthetic_directory(&name, parent, DirectoryKey::new(key));
            } else {
                let mut flags = NodeFlags::empty();
                if random.chance(3) {
                    flags |= NodeFlags::JUNCTION;
                }
                builder.add_entry(&NewEntry {
                    name: &name,
                    kind: NodeKind::Directory,
                    parent,
                    directory_key: Some(DirectoryKey::new(key)),
                    logical_size: 0,
                    allocated_size: 0,
                    modified_time,
                    flags,
                    hard_link_identity: None,
                });
                if random.chance(3) {
                    builder.mark_directory_inaccessible(DirectoryKey::new(key));
                }
            }
            directory_keys.push(key);
        } else {
            let size = *random.pick(&sizes)
                + if random.chance(30) {
                    random.below(5000) as u64
                } else {
                    0
                };
            let mut flags = NodeFlags::empty();
            if random.chance(5) {
                flags |= NodeFlags::CLOUD_PLACEHOLDER;
            }
            if random.chance(3) {
                flags |= NodeFlags::HIDDEN | NodeFlags::SYMLINK;
            }
            let hard_link_identity = random
                .chance(4)
                .then(|| FileIdentity::from_volume_and_file_ids(1, random.below(200) as u128));
            builder.add_entry(&NewEntry {
                name: &name,
                kind: NodeKind::File,
                parent,
                directory_key: None,
                logical_size: size,
                allocated_size: size.next_multiple_of(4096),
                modified_time,
                flags,
                hard_link_identity,
            });
        }
    }

    let mut tree = builder.finish();
    if random.chance(50) {
        let removable: Vec<NodeIndex> = (1..tree.node_count())
            .map(NodeIndex::from_usize)
            .filter(|&node| {
                let flags = tree.node(node).unwrap().flags();
                !flags.intersects(NodeFlags::SYNTHETIC | NodeFlags::LOCATION_UNKNOWN)
            })
            .take(2000)
            .collect();
        let chosen = [*random.pick(&removable), *random.pick(&removable)];
        if chosen[0] != chosen[1] {
            let _ = tree.remove_nodes_and_reaggregate(&chosen);
        }
    }
    tree
}

/// C:\
/// ├─ Users/
/// │  ├─ me/
/// │  │  ├─ Photos/
/// │  │  │  ├─ Beach.JPG, beach.jpg, IMG_0001.JPG, img_0002.jpg
/// │  │  │  └─ holiday 2024/  dive.jpg, Ärger.TXT
/// │  │  ├─ report.pdf, Report.PDF, annual report 2024.pdf, "report " (trailing space)
/// │  │  └─ Photos (a file with a folder's name)
/// │  └─ you/ straße, STRASSE, ΟΔΟΣ, İstanbul.txt, Kelvin (U+212A), 日本語.txt, long name
/// ├─ NTFS metadata/ (synthetic)  $MFT
/// ├─ Unlinked items/ (location unknown)  orphan.txt
/// ├─ cloud.docx (placeholder), link.jpg (hard link), Locked/ (inaccessible)
/// └─ archive.tar.gz
fn tricky_tree(mount_point: &str) -> FileTree {
    let mut builder = FileTreeBuilder::new(PathBuf::from(mount_point), DirectoryKey::new(0));
    let mut add = |name: &str,
                   kind: NodeKind,
                   parent: Option<u32>,
                   key: Option<u32>,
                   size: u64,
                   seconds: Option<u64>,
                   flags: NodeFlags,
                   link: Option<u128>| {
        builder.add_entry(&NewEntry {
            name,
            kind,
            parent: parent.map(DirectoryKey::new),
            directory_key: key.map(DirectoryKey::new),
            logical_size: size,
            allocated_size: size,
            modified_time: seconds.map(|seconds| time_at(seconds, 0)),
            flags,
            hard_link_identity: link.map(|id| FileIdentity::from_volume_and_file_ids(9, id)),
        });
    };
    let none = NodeFlags::empty();
    let file = NodeKind::File;
    let folder = NodeKind::Directory;
    let long_name = format!("{}.txt", "Long Name ".repeat(24));

    add("Users", folder, Some(0), Some(1), 0, Some(10), none, None);
    add("me", folder, Some(1), Some(2), 0, None, none, None);
    add("Photos", folder, Some(2), Some(3), 0, Some(20), none, None);
    add(
        "Beach.JPG",
        file,
        Some(3),
        None,
        4000,
        Some(300),
        none,
        None,
    );
    add(
        "beach.jpg",
        file,
        Some(3),
        None,
        4000,
        Some(300),
        none,
        None,
    );
    add("IMG_0001.JPG", file, Some(3), None, 3000, None, none, None);
    add("img_0002.jpg", file, Some(3), None, 3000, None, none, None);
    add(
        "holiday 2024",
        folder,
        Some(3),
        Some(4),
        0,
        Some(5),
        none,
        None,
    );
    add(
        "dive.jpg",
        file,
        Some(4),
        None,
        9000,
        Some(200),
        none,
        Some(1),
    );
    add("Ärger.TXT", file, Some(4), None, 0, Some(200), none, None);
    add(
        "report.pdf",
        file,
        Some(2),
        None,
        500,
        Some(100),
        none,
        None,
    );
    add(
        "Report.PDF",
        file,
        Some(2),
        None,
        500,
        Some(100),
        none,
        None,
    );
    add(
        "annual report 2024.pdf",
        file,
        Some(2),
        None,
        1536,
        Some(100),
        none,
        None,
    );
    add("report ", file, Some(2), None, 0, None, none, None);
    add("Photos", file, Some(2), None, 4096, Some(1), none, None);
    add("you", folder, Some(1), Some(5), 0, None, none, None);
    add("straße", file, Some(5), None, 10, Some(7), none, None);
    add("STRASSE", file, Some(5), None, 10, Some(7), none, None);
    add("ΟΔΟΣ", folder, Some(5), Some(6), 0, None, none, None);
    add("οδοσ.txt", file, Some(6), None, 1024, None, none, None);
    add("İstanbul.txt", file, Some(5), None, 1024, None, none, None);
    add("\u{212A}elvin", file, Some(5), None, 1, None, none, None);
    add("日本語.txt", file, Some(5), None, 2048, Some(9), none, None);
    add(&long_name, file, Some(5), None, 2048, Some(9), none, None);
    add(
        "link.jpg",
        file,
        Some(0),
        None,
        9000,
        Some(200),
        none,
        Some(1),
    );
    add(
        "cloud.docx",
        file,
        Some(0),
        None,
        1_048_576,
        Some(50),
        NodeFlags::CLOUD_PLACEHOLDER,
        None,
    );
    add("Locked", folder, Some(0), Some(7), 0, None, none, None);
    add(
        "archive.tar.gz",
        file,
        Some(0),
        None,
        7000,
        Some(60),
        none,
        None,
    );
    add("orphan.txt", file, None, None, 5, None, none, None);
    add("$MFT", file, Some(8), None, 1000, None, none, None);
    builder.add_synthetic_directory(
        "NTFS metadata",
        Some(DirectoryKey::new(0)),
        DirectoryKey::new(8),
    );
    builder.mark_directory_inaccessible(DirectoryKey::new(7));
    builder.finish()
}

fn both_indexes(mount_point: &str, tree: FileTree) -> (LegacyVolumeSearchIndex, VolumeSearchIndex) {
    let legacy = LegacyVolumeSearchIndex::from_file_tree(
        PathBuf::from(mount_point),
        tree.clone(),
        ScanMethod::FolderWalk,
    );
    let compact =
        VolumeSearchIndex::from_file_tree(PathBuf::from(mount_point), tree, ScanMethod::FolderWalk);
    (legacy, compact)
}

fn describe_difference(legacy: &SearchMatches, compact: &SearchMatches) -> String {
    let mut description = format!(
        "total {} vs {}, {} vs {} hits",
        legacy.total_match_count,
        compact.total_match_count,
        legacy.hits.len(),
        compact.hits.len()
    );
    for (position, (old_hit, new_hit)) in legacy.hits.iter().zip(&compact.hits).enumerate() {
        if old_hit != new_hit {
            let _ = write!(
                description,
                "; first difference at {position}: {old_hit:?} vs {new_hit:?}"
            );
            break;
        }
    }
    description
}

/// Runs every query in every order and with every limit on both indexes; returns how many
/// searches were compared.
fn assert_same_results(
    legacy: &LegacyVolumeSearchIndex,
    compact: &VolumeSearchIndex,
    label: &str,
) -> usize {
    assert_eq!(legacy.file_count(), compact.file_count(), "{label}");
    assert_eq!(
        legacy.directory_count(),
        compact.directory_count(),
        "{label}"
    );
    assert_eq!(legacy.tree().node_count(), compact.node_count(), "{label}");

    let mut comparison_count = 0;
    for &query_text in QUERIES {
        let Ok(query) = ParsedQuery::parse(query_text) else {
            continue;
        };
        for sort in SORT_ORDERS {
            for limit in LIMITS {
                let old_matches = legacy_search_volume_index(legacy, &query, sort, limit);
                let new_matches = search_volume_index(compact, &query, sort, limit);
                assert!(
                    old_matches == new_matches,
                    "{label}: {query_text:?} {sort:?} limit {limit}: {}",
                    describe_difference(&old_matches, &new_matches)
                );
                comparison_count += 1;
            }
        }
    }
    comparison_count
}

/// `find_searchable_node_by_path(full_path(n))` finds `n` (or the first node with the same
/// path ignoring case) for the searchable nodes at `positions`, also with the path in another
/// case, and agrees with the legacy lookup wherever that one found a node.
fn assert_paths_resolve(
    legacy: &LegacyVolumeSearchIndex,
    compact: &VolumeSearchIndex,
    positions: impl Iterator<Item = usize>,
) -> usize {
    let mut first_node_by_lowercase_path: HashMap<String, usize> = HashMap::new();
    for position in 0..compact.node_count() {
        if compact.nodes[position].is_searchable() && !has_synthetic_ancestor(compact, position) {
            let path = compact.full_path(position).unwrap();
            first_node_by_lowercase_path
                .entry(path.to_string_lossy().to_lowercase())
                .or_insert(position);
        }
    }

    let mut checked = 0;
    for position in positions {
        let node = NodeIndex::from_usize(position);
        if !compact.nodes[position].is_searchable() {
            if let Some(path) = compact.full_path(position) {
                let found = compact.find_searchable_node_by_path(&path);
                assert!(found.is_none_or(|found| found != node), "{path:?}");
            }
            continue;
        }

        let path = compact.full_path(position).unwrap();
        assert_eq!(Some(&path), legacy.tree().full_path(node).as_ref());
        let lowercase_path = path.to_string_lossy().to_lowercase();
        let expected = first_node_by_lowercase_path
            .get(&lowercase_path)
            .copied()
            .map(NodeIndex::from_usize);
        if !has_synthetic_ancestor(compact, position) {
            assert!(expected.is_some(), "{path:?}");
        }

        assert_eq!(
            compact.find_searchable_node_by_path(&path),
            expected,
            "{path:?}"
        );
        let uppercase_path = PathBuf::from(path.to_string_lossy().to_uppercase());
        if uppercase_path.to_string_lossy().to_lowercase() == lowercase_path {
            assert_eq!(
                compact.find_searchable_node_by_path(&uppercase_path),
                expected,
                "{uppercase_path:?}"
            );
        }
        if let Some(legacy_found) = legacy.find_searchable_node_by_path(&path) {
            assert_eq!(Some(legacy_found), expected, "{path:?}");
        }
        checked += 1;
    }
    checked
}

fn has_synthetic_ancestor(index: &VolumeSearchIndex, position: usize) -> bool {
    let mut current = index.nodes[position].parent();
    while let Some(ancestor) = current {
        let ancestor_node = &index.nodes[ancestor.as_usize()];
        if ancestor_node.parent().is_some()
            && ancestor_node
                .flags
                .contains(super::search_index_node::SearchNodeFlags::SYNTHETIC)
        {
            return true;
        }
        current = ancestor_node.parent();
    }
    false
}

#[test]
fn the_tricky_tree_gives_the_same_results() {
    let (legacy, compact) = both_indexes(r"C:\", tricky_tree(r"C:\"));

    let comparisons = assert_same_results(&legacy, &compact, "tricky tree");
    let checked_paths = assert_paths_resolve(&legacy, &compact, 0..compact.node_count());

    assert!(comparisons > 800);
    assert!(checked_paths >= 28);
    assert_eq!(
        compact.find_searchable_node_by_path(Path::new(r"C:\Unlinked items\orphan.txt")),
        None
    );
    assert_eq!(
        compact.find_searchable_node_by_path(Path::new(r"C:\orphan.txt")),
        None
    );
    assert_eq!(
        compact.find_searchable_node_by_path(Path::new(r"C:\NTFS metadata\$MFT")),
        None
    );
    assert_eq!(
        compact.find_searchable_node_by_path(Path::new(r"C:\$mft")),
        None
    );
    assert!(
        compact
            .find_searchable_node_by_path(Path::new(r"c:\users\ME\photos\BEACH.jpg"))
            .is_some()
    );
}

#[test]
fn several_volumes_give_the_same_merged_results() {
    let (legacy_c, compact_c) = both_indexes(r"C:\", tricky_tree(r"C:\"));
    let (legacy_d, compact_d) = both_indexes(r"D:\", random_tree(3, 3_000));

    for &query_text in QUERIES {
        let Ok(query) = ParsedQuery::parse(query_text) else {
            continue;
        };
        for sort in SORT_ORDERS {
            for limit in LIMITS {
                let mut old_matches = SearchMatches::default();
                for legacy in [&legacy_c, &legacy_d] {
                    let volume_matches = legacy_search_volume_index(legacy, &query, sort, limit);
                    old_matches.total_match_count += volume_matches.total_match_count;
                    old_matches.hits.extend(volume_matches.hits);
                }
                super::search_results::sort_hits_of_several_volumes(&mut old_matches.hits, sort);
                old_matches.hits.truncate(limit);

                let new_matches =
                    search_volume_indexes([&compact_c, &compact_d], &query, sort, limit);
                assert!(
                    old_matches == new_matches,
                    "{query_text:?} {sort:?} limit {limit}: {}",
                    describe_difference(&old_matches, &new_matches)
                );
            }
        }
    }
}

#[test]
fn small_random_trees_give_the_same_results() {
    for seed in 1..=3 {
        let (legacy, compact) = both_indexes(r"C:\", random_tree(seed, 8_000));
        assert_same_results(&legacy, &compact, &format!("seed {seed}"));
    }
}

#[test]
fn every_path_of_a_random_tree_resolves_to_its_node() {
    let (legacy, compact) = both_indexes(r"C:\", random_tree(77, 4_000));

    let checked = assert_paths_resolve(&legacy, &compact, 0..compact.node_count());

    assert!(checked > 3_000);
}

#[test]
#[ignore = "50 seeds of 200,000 nodes; run by hand in release mode"]
fn large_random_trees_give_the_same_results() {
    let mut total_comparisons = 0;
    let mut total_paths = 0;
    for seed in 1..=50 {
        let tree = random_tree(seed, 200_000);
        let (legacy, compact) = both_indexes(r"C:\", tree);
        let comparisons = assert_same_results(&legacy, &compact, &format!("seed {seed}"));

        let mut random = Xorshift::new(seed + 1000);
        let sample = (0..2_000).map(|_| random.below(compact.node_count()));
        let paths = assert_paths_resolve(&legacy, &compact, sample);

        println!(
            "seed {seed:>2}: {} nodes, {comparisons} searches identical, {paths} paths resolved, \
             memory {} -> {} bytes per node",
            compact.node_count(),
            legacy.estimated_memory_bytes() / compact.node_count(),
            compact.estimated_memory_bytes() / compact.node_count()
        );
        total_comparisons += comparisons;
        total_paths += paths;
    }
    println!("{total_comparisons} searches compared, {total_paths} paths resolved, no differences");
}

/// The 3.1 million node tree of `tests/performance.rs`: 150,000 folders in three levels with 19
/// files each, plus one folder with 100,000 files.
fn performance_test_tree() -> FileTree {
    const DIRECTORY_COUNT: u32 = 150_000;
    const EXTENSIONS: [&str; 8] = ["jpg", "txt", "dll", "mp4", "json", "js", "pdf", "exe"];
    let mut builder = FileTreeBuilder::with_capacity(
        PathBuf::from(r"C:\"),
        DirectoryKey::new(0),
        DIRECTORY_COUNT as usize * 20 + 100_000,
    );
    let mut name = String::new();
    for directory in 1..=DIRECTORY_COUNT {
        let parent = if directory <= 100 { 0 } else { directory / 100 };
        name.clear();
        let _ = write!(name, "folder-{directory}");
        builder.add_entry(&NewEntry {
            name: &name,
            kind: NodeKind::Directory,
            parent: Some(DirectoryKey::new(parent)),
            directory_key: Some(DirectoryKey::new(directory)),
            logical_size: 0,
            allocated_size: 0,
            modified_time: None,
            flags: NodeFlags::empty(),
            hard_link_identity: None,
        });
        let file_count = if directory == 7 { 100_000 } else { 19 };
        for file in 0..file_count {
            name.clear();
            let extension = EXTENSIONS[(file as usize + directory as usize) % EXTENSIONS.len()];
            let _ = write!(name, "file-{directory}-{file}.{extension}");
            let size = u64::from((file * 7919 + directory * 104_729) % 5_000_000);
            builder.add_entry(&NewEntry {
                name: &name,
                kind: NodeKind::File,
                parent: Some(DirectoryKey::new(directory)),
                directory_key: None,
                logical_size: size,
                allocated_size: size.next_multiple_of(4096),
                modified_time: Some(time_at(u64::from(file * 13 + directory) * 60, 0)),
                flags: NodeFlags::empty(),
                hard_link_identity: None,
            });
        }
    }
    builder.finish()
}

#[test]
#[ignore = "builds two indexes of 3.1 million nodes; run by hand in release mode"]
fn the_compact_index_takes_under_60_bytes_per_node() {
    let tree = performance_test_tree();
    let tree_bytes = tree.estimated_memory_bytes();
    let (legacy, compact) = both_indexes(r"C:\", tree);
    let node_count = compact.node_count() as f64;

    let (legacy_bytes, compact_bytes) = (
        legacy.estimated_memory_bytes() as f64,
        compact.estimated_memory_bytes() as f64,
    );
    println!(
        "{node_count} nodes; scan tree {} MB; index before {:.0} MB ({:.1} bytes per node), after {:.0} MB ({:.1} bytes per node); SearchIndexNode {} bytes",
        tree_bytes / 1_000_000,
        legacy_bytes / 1e6,
        legacy_bytes / node_count,
        compact_bytes / 1e6,
        compact_bytes / node_count,
        size_of::<super::search_index_node::SearchIndexNode>()
    );
    assert!(compact_bytes / node_count < 60.0);
}
