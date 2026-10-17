//! Timed checks against the performance targets in PLAN.md section 12. Ignored by default
//! because timings depend on the machine; run them in release mode:
//!
//! ```text
//! cargo test -p disk_scanner --release --test performance -- --ignored --nocapture
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use disk_scanner::file_tree::{DirectoryKey, FileTreeBuilder, NewEntry};
use disk_scanner::{
    CancellationToken, ChildSortOrder, FileTree, NoProgressReporting, NodeFlags, NodeIndex,
    NodeKind, OnlineOnlyVisibility, ParsedQuery, ScanMethod, ScanTarget, SearchSortOrder,
    VolumeSearchIndex, build_volume_search_index, list_volumes, scan, search_volume_index,
};

const DIRECTORY_COUNT: u32 = 50_000;
const FILES_PER_DIRECTORY: u32 = 19;
const FILES_IN_LARGEST_DIRECTORY: u32 = 100_000;
const EXTENSIONS: [&str; 8] = ["jpg", "txt", "dll", "mp4", "json", "js", "pdf", "exe"];

/// About one million nodes: 50,000 directories in three levels with 19 files each, plus one
/// directory with 100,000 files.
fn synthetic_tree() -> FileTree {
    synthetic_tree_with_directories(DIRECTORY_COUNT)
}

/// `directory_count` directories in three levels with 19 files each, plus one directory with
/// 100,000 files: 20 nodes per directory.
fn synthetic_tree_with_directories(directory_count: u32) -> FileTree {
    let node_capacity = directory_count as usize * 20 + FILES_IN_LARGEST_DIRECTORY as usize;
    let mut builder =
        FileTreeBuilder::with_capacity(PathBuf::from("C:\\"), DirectoryKey::new(0), node_capacity);
    let mut name = String::new();

    for directory in 1..=directory_count {
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

        let file_count = if directory == 7 {
            FILES_IN_LARGEST_DIRECTORY
        } else {
            FILES_PER_DIRECTORY
        };
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
                modified_time: Some(
                    SystemTime::UNIX_EPOCH
                        + Duration::from_secs(u64::from(file * 13 + directory) * 60),
                ),
                flags: NodeFlags::empty(),
                hard_link_identity: None,
            });
        }
    }

    builder.finish()
}

fn time<T>(label: &str, target: Duration, operation: impl FnOnce() -> T) -> T {
    let started_at = Instant::now();
    let result = operation();
    let elapsed = started_at.elapsed();
    let verdict = if elapsed <= target {
        "ok"
    } else {
        "SLOWER THAN TARGET"
    };
    println!("{label:<40} {elapsed:>12.2?}  (target {target:?}) {verdict}");
    result
}

fn find_directory(tree: &FileTree, name: &str) -> NodeIndex {
    tree.search_nodes(name, 1_000, OnlineOnlyVisibility::Hidden)
        .into_iter()
        .find(|result| result.name == name)
        .unwrap()
        .node
}

#[test]
#[ignore = "timing test; run by hand in release mode"]
fn tree_queries_meet_the_targets_on_a_million_nodes() {
    let tree = time(
        "build 1.05 million node tree",
        Duration::from_secs(5),
        synthetic_tree,
    );
    let node_count = tree.node_count();
    let bytes_per_node = tree.estimated_memory_bytes() as f64 / node_count as f64;
    println!(
        "{node_count} nodes, about {bytes_per_node:.0} bytes per node including names (target 150)"
    );

    let largest_directory = find_directory(&tree, "folder-7");
    time(
        "children page of 100,000 children",
        Duration::from_millis(50),
        || {
            tree.children_page(
                largest_directory,
                ChildSortOrder::LogicalSizeDescending,
                0,
                200,
                OnlineOnlyVisibility::Hidden,
            )
            .unwrap()
        },
    );
    time(
        "children page sorted by name",
        Duration::from_millis(50),
        || {
            tree.children_page(
                largest_directory,
                ChildSortOrder::NameAscending,
                500,
                200,
                OnlineOnlyVisibility::Hidden,
            )
            .unwrap()
        },
    );
    time(
        "visualization subtree of the root",
        Duration::from_millis(100),
        || {
            tree.visualization_subtree(tree.root(), 6, 0.001, OnlineOnlyVisibility::Hidden)
                .unwrap()
        },
    );
    time(
        "largest files of the whole tree",
        Duration::from_millis(100),
        || {
            tree.largest_files(tree.root(), None, 1_000, OnlineOnlyVisibility::Hidden)
                .unwrap()
        },
    );
    time(
        "file type breakdown of the whole tree",
        Duration::from_millis(100),
        || {
            tree.file_type_breakdown(tree.root(), OnlineOnlyVisibility::Hidden)
                .unwrap()
        },
    );
    time("search for a substring", Duration::from_millis(200), || {
        tree.search_nodes("-4242", 100, OnlineOnlyVisibility::Hidden)
    });
}

#[test]
#[ignore = "timing test that scans a real folder read-only; run by hand in release mode"]
fn folder_walk_of_the_user_profile() {
    let folder = std::env::var_os("VSM_BENCHMARK_FOLDER")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap();

    let result = scan(
        ScanTarget::Folder { path: folder },
        ScanMethod::FolderWalk,
        &NoProgressReporting,
        &CancellationToken::new(),
    )
    .unwrap();

    let totals = result.summary.totals;
    let files_per_second = f64::from(totals.file_count)
        / (result.summary.duration_milliseconds.max(1) as f64 / 1000.0);
    println!(
        "{} files, {} folders, {} bytes in {} ms ({files_per_second:.0} files/s)",
        totals.file_count,
        totals.directory_count,
        totals.logical_size,
        result.summary.duration_milliseconds
    );
}

/// Runs `query` once to warm up the thread pool and caches, then measures the median of five
/// runs and checks it against `target`.
fn assert_search_time(
    index: &VolumeSearchIndex,
    query: &str,
    sort: SearchSortOrder,
    target: Duration,
) {
    let parsed_query = ParsedQuery::parse(query).unwrap();
    let _warm_up = search_volume_index(index, &parsed_query, sort, 1_000);

    let mut durations = Vec::new();
    let mut match_count = 0;
    for _ in 0..5 {
        let started_at = Instant::now();
        match_count = search_volume_index(index, &parsed_query, sort, 1_000).total_match_count;
        durations.push(started_at.elapsed());
    }
    durations.sort_unstable();
    let median = durations[durations.len() / 2];

    println!(
        "search {query:<32} {sort:<20?} {median:>10.2?}  {match_count:>9} matches (target {target:?})"
    );
    assert!(
        median <= target,
        "searching {query:?} took {median:?}, target {target:?}"
    );
}

#[test]
#[ignore = "timing test; run by hand in release mode"]
fn search_index_meets_the_targets_on_three_million_nodes() {
    let tree = time(
        "build 3.1 million node tree",
        Duration::from_secs(15),
        || synthetic_tree_with_directories(150_000),
    );
    let tree_bytes = tree.estimated_memory_bytes();

    let index = time("prepare the search index", Duration::from_secs(3), || {
        VolumeSearchIndex::from_file_tree(PathBuf::from("C:\\"), tree, ScanMethod::FolderWalk)
    });
    let node_count = index.node_count();
    let index_bytes = index.estimated_memory_bytes();
    let index_bytes_per_node = index_bytes as f64 / node_count as f64;
    println!(
        "{node_count} nodes; tree {} MB, index {} MB ({index_bytes_per_node:.0} bytes per node)",
        tree_bytes / 1_000_000,
        index_bytes / 1_000_000,
    );
    assert!(
        index_bytes_per_node < 60.0,
        "the index takes {index_bytes_per_node:.1} bytes per node, target under 60"
    );

    let deep_file = std::path::Path::new(r"C:\folder-1\folder-123\folder-12345\file-12345-7.jpg");
    for _ in 0..3 {
        let found = time(
            "find a search hit by its path",
            Duration::from_millis(10),
            || index.find_searchable_node_by_path(deep_file),
        );
        assert!(found.is_some());
    }

    let substring_target = Duration::from_millis(50);
    let glob_target = Duration::from_millis(200);
    assert_search_time(
        &index,
        "-4242",
        SearchSortOrder::NameAscending,
        substring_target,
    );
    assert_search_time(
        &index,
        "file-12",
        SearchSortOrder::SizeDescending,
        substring_target,
    );
    assert_search_time(
        &index,
        "jpg",
        SearchSortOrder::ModifiedDescending,
        substring_target,
    );
    assert_search_time(
        &index,
        "file 77 jpg",
        SearchSortOrder::PathAscending,
        substring_target,
    );
    assert_search_time(
        &index,
        "ext:mp4 size:>4mb",
        SearchSortOrder::SizeDescending,
        substring_target,
    );
    assert_search_time(
        &index,
        "*-42?.jpg",
        SearchSortOrder::NameAscending,
        glob_target,
    );
    assert_search_time(
        &index,
        "file-1*",
        SearchSortOrder::NameAscending,
        glob_target,
    );
    assert_search_time(
        &index,
        "folder-12\\",
        SearchSortOrder::PathAscending,
        glob_target,
    );
    assert_search_time(
        &index,
        r"c:\*\folder-9*\*.json",
        SearchSortOrder::PathAscending,
        glob_target,
    );
}

#[test]
#[ignore = "builds a search index of every local disk read-only; run by hand in release mode"]
fn search_index_of_the_local_disks() {
    for volume in list_volumes().unwrap() {
        if volume.kind != disk_scanner::VolumeKind::LocalDisk {
            println!(
                "{}: skipped ({:?})",
                volume.mount_point.display(),
                volume.kind
            );
            continue;
        }

        let started_at = Instant::now();
        let index =
            build_volume_search_index(&volume, &NoProgressReporting, &CancellationToken::new())
                .unwrap();
        println!(
            "{}: {} files, {} folders, {:?} with {:?}, about {} MB",
            volume.mount_point.display(),
            index.file_count(),
            index.directory_count(),
            started_at.elapsed(),
            index.method(),
            index.estimated_memory_bytes() / 1_000_000
        );

        for query in [
            "dll",
            "*.jpg",
            "ext:mp4 size:>100mb",
            r"windows\system32\ drivers",
        ] {
            let parsed_query = ParsedQuery::parse(query).unwrap();
            let _warm_up =
                search_volume_index(&index, &parsed_query, SearchSortOrder::NameAscending, 1_000);
            let started_at = Instant::now();
            let matches =
                search_volume_index(&index, &parsed_query, SearchSortOrder::NameAscending, 1_000);
            println!(
                "    {query:<32} {:>10.2?}  {} matches",
                started_at.elapsed(),
                matches.total_match_count
            );
        }
    }
}
