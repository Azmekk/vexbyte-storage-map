//! Timed checks against the performance targets in PLAN.md section 12. Ignored by default
//! because timings depend on the machine; run them in release mode:
//!
//! ```text
//! cargo test -p disk_scanner --release --test performance -- --ignored --nocapture
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use disk_scanner::file_tree::{DirectoryKey, FileTreeBuilder, NewEntry};
use disk_scanner::{
    CancellationToken, ChildSortOrder, FileTree, NoProgressReporting, NodeFlags, NodeIndex,
    NodeKind, ScanMethod, ScanTarget, scan,
};

const DIRECTORY_COUNT: u32 = 50_000;
const FILES_PER_DIRECTORY: u32 = 19;
const FILES_IN_LARGEST_DIRECTORY: u32 = 100_000;
const EXTENSIONS: [&str; 8] = ["jpg", "txt", "dll", "mp4", "json", "js", "pdf", "exe"];

/// About one million nodes: 50,000 directories in three levels with 19 files each, plus one
/// directory with 100,000 files.
fn synthetic_tree() -> FileTree {
    let mut builder =
        FileTreeBuilder::with_capacity(PathBuf::from("C:\\"), DirectoryKey::new(0), 1_100_000);
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
                modified_time: None,
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
    tree.search_nodes(name, 1_000)
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
            )
            .unwrap()
        },
    );
    time(
        "children page sorted by name",
        Duration::from_millis(50),
        || {
            tree.children_page(largest_directory, ChildSortOrder::NameAscending, 500, 200)
                .unwrap()
        },
    );
    time(
        "visualization subtree of the root",
        Duration::from_millis(100),
        || tree.visualization_subtree(tree.root(), 6, 0.001).unwrap(),
    );
    time(
        "largest files of the whole tree",
        Duration::from_millis(100),
        || tree.largest_files(tree.root(), None, 1_000).unwrap(),
    );
    time(
        "file type breakdown of the whole tree",
        Duration::from_millis(100),
        || tree.file_type_breakdown(tree.root()).unwrap(),
    );
    time("search for a substring", Duration::from_millis(200), || {
        tree.search_nodes("-4242", 100)
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
