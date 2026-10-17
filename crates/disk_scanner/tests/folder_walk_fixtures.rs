//! Scans real directory structures that the tests build in a unique folder under the temp
//! directory, and removes them afterwards. Nothing outside that folder is touched.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use disk_scanner::{
    CancellationToken, ChildSortOrder, FileTree, FolderWalkError, NoProgressReporting, NodeFlags,
    NodeIndex, OnlineOnlyVisibility, ScanError, ScanMethod, ScanPhase, ScanProgressUpdate,
    ScanTarget, scan,
};

/// A uniquely named folder under the temp directory, deleted on drop.
struct TemporaryFixture {
    root: PathBuf,
    junction_target: PathBuf,
    junction: Option<PathBuf>,
}

impl TemporaryFixture {
    fn create(test_name: &str) -> Self {
        let unique_suffix = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let base =
            std::env::temp_dir().join(format!("vsm-disk-scanner-{test_name}-{unique_suffix}"));
        let root = base.join("scan-root");
        let junction_target = base.join("junction-target");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&junction_target).unwrap();

        Self {
            root,
            junction_target,
            junction: None,
        }
    }

    fn base(&self) -> &Path {
        self.root.parent().unwrap()
    }

    fn write_file(&self, relative_path: &str, size: usize, fill: u8) -> PathBuf {
        let path = self.root.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, vec![fill; size]).unwrap();
        path
    }

    /// Creates a junction with `mklink /J`. Returns false (and the test skips its junction
    /// checks) when that is not possible on this machine.
    fn try_create_junction(&mut self, relative_path: &str) -> bool {
        let junction = self.root.join(relative_path);
        let output = Command::new("cmd")
            .arg("/c")
            .arg("mklink")
            .arg("/J")
            .arg(&junction)
            .arg(&self.junction_target)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                self.junction = Some(junction);
                true
            }
            Ok(output) => {
                eprintln!(
                    "skipping junction checks: mklink /J failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                false
            }
            Err(error) => {
                eprintln!("skipping junction checks: could not run mklink: {error}");
                false
            }
        }
    }
}

impl Drop for TemporaryFixture {
    fn drop(&mut self) {
        // Remove the junction itself first, so cleanup can never reach into its target.
        if let Some(junction) = &self.junction {
            let _ = fs::remove_dir(junction);
        }
        let base = self.base().to_path_buf();
        assert!(
            base.starts_with(std::env::temp_dir()),
            "refusing to clean up outside the temp directory"
        );
        let _ = fs::remove_dir_all(&base);
    }
}

fn scan_folder(path: &Path) -> disk_scanner::ScanResult {
    scan(
        ScanTarget::Folder {
            path: path.to_path_buf(),
        },
        ScanMethod::FolderWalk,
        &NoProgressReporting,
        &CancellationToken::new(),
    )
    .unwrap()
}

fn find_by_path(tree: &FileTree, relative_path: &str) -> NodeIndex {
    let mut current = tree.root();
    for component in relative_path.split('/') {
        current = tree
            .children(current)
            .find(|&child| tree.name(child) == component)
            .unwrap_or_else(|| panic!("{relative_path}: {component} not found"));
    }
    current
}

fn set_hidden_attribute(path: &Path) -> bool {
    Command::new("attrib")
        .arg("+h")
        .arg(path)
        .status()
        .is_ok_and(|status| status.success())
}

#[test]
#[allow(clippy::too_many_lines)]
fn walks_a_fixture_with_nested_folders_hard_links_junctions_and_long_paths() {
    let mut fixture = TemporaryFixture::create("walk");

    fixture.write_file("nested/a/b/c/deep.txt", 1000, b'd');
    fixture.write_file("nested/a/sibling.bin", 5000, b's');
    fs::create_dir_all(fixture.root.join("empty")).unwrap();
    fs::create_dir_all(fixture.root.join("empty_nested/inner")).unwrap();

    let original = fixture.write_file("links/original.dat", 10_000, b'o');
    fs::hard_link(&original, fixture.root.join("links/second-link.dat")).unwrap();
    fs::hard_link(&original, fixture.root.join("nested/third-link.dat")).unwrap();

    fixture.write_file("equal_size/one.txt", 4096, b'a');
    fixture.write_file("equal_size/two.txt", 4096, b'b');
    fixture.write_file("equal_size/three.txt", 4096, b'a');

    let long_folder_name = "a-folder-name-that-is-fifty-characters-long-xxxxxx";
    let long_relative_folder = [long_folder_name; 6].join("/");
    let long_file = fixture.write_file(&format!("{long_relative_folder}/long.txt"), 300, b'l');
    assert!(long_file.as_os_str().len() > 260);

    let hidden_file = fixture.write_file("hidden.txt", 10, b'h');
    let hidden_attribute_was_set = set_hidden_attribute(&hidden_file);

    fs::write(
        fixture.junction_target.join("outside.bin"),
        vec![0_u8; 50_000],
    )
    .unwrap();
    let has_junction = fixture.try_create_junction("junction");

    let result = scan_folder(&fixture.root);
    let tree = &result.tree;
    let totals = result.summary.totals;

    assert_eq!(
        totals.logical_size,
        1000 + 5000 + 10_000 + 3 * 4096 + 300 + 10,
        "hard links are counted once and the junction target is not followed"
    );
    assert_eq!(totals.file_count, 10);
    assert_eq!(totals.additional_hard_link_count, 2);
    assert_eq!(totals.inaccessible_directory_count, 0);
    assert_eq!(result.summary.method, ScanMethod::FolderWalk);
    assert_eq!(
        tree.root_path(),
        std::path::absolute(&fixture.root).unwrap()
    );

    let link_nodes = [
        find_by_path(tree, "links/original.dat"),
        find_by_path(tree, "links/second-link.dat"),
        find_by_path(tree, "nested/third-link.dat"),
    ];
    let counted_links = link_nodes
        .iter()
        .filter(|&&node| {
            !tree
                .node(node)
                .unwrap()
                .flags()
                .contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE)
        })
        .count();
    assert_eq!(counted_links, 1, "exactly one link is counted");
    for node in link_nodes {
        assert_eq!(tree.node(node).unwrap().logical_size(), 10_000);
    }

    let empty = find_by_path(tree, "empty");
    assert_eq!(tree.node(empty).unwrap().logical_size(), 0);
    assert_eq!(tree.children(empty).count(), 0);
    let empty_nested = find_by_path(tree, "empty_nested");
    assert_eq!(tree.node(empty_nested).unwrap().directory_count(), 1);

    let nested = find_by_path(tree, "nested");
    let nested_node = tree.node(nested).unwrap();
    assert_eq!(nested_node.file_count(), 3);
    assert_eq!(nested_node.directory_count(), 3);

    let long_node = find_by_path(tree, &format!("{long_relative_folder}/long.txt"));
    let long_path = tree.full_path(long_node).unwrap();
    assert!(long_path.as_os_str().len() > 260);
    assert_eq!(fs::metadata(&long_path).unwrap().len(), 300);

    if hidden_attribute_was_set {
        let hidden = find_by_path(tree, "hidden.txt");
        assert!(
            tree.node(hidden)
                .unwrap()
                .flags()
                .contains(NodeFlags::HIDDEN)
        );
    } else {
        eprintln!("skipping the hidden flag check: attrib +h failed");
    }

    if has_junction {
        let junction = find_by_path(tree, "junction");
        let junction_node = tree.node(junction).unwrap();
        assert!(junction_node.flags().contains(NodeFlags::JUNCTION));
        assert_eq!(
            tree.children(junction).count(),
            0,
            "junctions are never entered"
        );
        assert_eq!(totals.link_count, 1);
    }

    let largest = tree
        .largest_files(tree.root(), None, 3, OnlineOnlyVisibility::Hidden)
        .unwrap();
    assert!(largest.iter().all(|file| file.logical_size == 10_000));

    let search_results = tree.search_nodes("LINK", 10, OnlineOnlyVisibility::Hidden);
    let mut found_names: Vec<_> = search_results
        .iter()
        .map(|result| result.name.as_str())
        .collect();
    found_names.sort_unstable();
    assert_eq!(found_names, ["links", "second-link.dat", "third-link.dat"]);

    let page = tree
        .children_page(
            tree.root(),
            ChildSortOrder::LogicalSizeDescending,
            0,
            3,
            OnlineOnlyVisibility::Hidden,
        )
        .unwrap();
    assert_eq!(page.children.len(), 3);
    assert!(page.total_child_count >= 8);
    assert!(
        page.children
            .windows(2)
            .all(|pair| pair[0].logical_size >= pair[1].logical_size)
    );

    let breakdown = tree
        .file_type_breakdown(tree.root(), OnlineOnlyVisibility::Hidden)
        .unwrap();
    assert_eq!(breakdown.total_logical_size, totals.logical_size);
    let txt = breakdown
        .extensions
        .iter()
        .find(|share| share.extension.as_deref() == Some("txt"))
        .unwrap();
    assert_eq!(txt.file_count, 6);

    let visualization = tree
        .visualization_subtree(tree.root(), 3, 0.05, OnlineOnlyVisibility::Hidden)
        .unwrap();
    let visualized_total: u64 = visualization
        .children
        .iter()
        .map(|child| child.logical_size)
        .sum();
    assert_eq!(visualized_total, totals.logical_size);

    let details = tree
        .node_details(
            find_by_path(tree, "nested/a/b/c/deep.txt"),
            OnlineOnlyVisibility::Hidden,
        )
        .unwrap();
    assert!(details.modified_at_unix_milliseconds.is_some());
    assert_eq!(
        details.full_path.unwrap(),
        std::path::absolute(&fixture.root)
            .unwrap()
            .join("nested")
            .join("a")
            .join("b")
            .join("c")
            .join("deep.txt")
    );
}

#[test]
fn removing_nodes_only_changes_the_tree_in_memory() {
    let fixture = TemporaryFixture::create("removal");
    let doomed = fixture.write_file("folder/doomed.bin", 2048, b'x');
    fixture.write_file("folder/kept.bin", 1024, b'y');

    let mut result = scan_folder(&fixture.root);
    let tree = &mut result.tree;
    let doomed_node = find_by_path(tree, "folder/doomed.bin");

    let removal = tree.remove_nodes_and_reaggregate(&[doomed_node]).unwrap();

    assert_eq!(removal.freed_logical_size, 2048);
    assert_eq!(tree.totals().logical_size, 1024);
    assert!(doomed.exists(), "the file system is never touched");
}

#[test]
fn reports_progress_and_finishes_with_a_final_update() {
    let fixture = TemporaryFixture::create("progress");
    for number in 0..50 {
        fixture.write_file(&format!("top-{}/file-{number}.txt", number % 5), 100, b'p');
    }
    let updates: RefCell<Vec<ScanProgressUpdate>> = RefCell::new(Vec::new());
    let reporter = |update: ScanProgressUpdate| updates.borrow_mut().push(update);

    scan(
        ScanTarget::Folder {
            path: fixture.root.clone(),
        },
        ScanMethod::FolderWalk,
        &reporter,
        &CancellationToken::new(),
    )
    .unwrap();

    let updates = updates.into_inner();
    let last_update = updates.last().unwrap();
    assert_eq!(last_update.phase, ScanPhase::Finished);
    assert_eq!(last_update.files_found, 50);
    assert_eq!(last_update.directories_found, 5);
    assert!(
        updates
            .iter()
            .any(|update| update.phase == ScanPhase::BuildingTree)
    );
}

#[test]
fn stops_when_cancelled_before_starting() {
    let fixture = TemporaryFixture::create("cancel");
    fixture.write_file("some/file.txt", 10, b'c');
    let cancellation = CancellationToken::new();
    cancellation.cancel();

    let result = scan(
        ScanTarget::Folder {
            path: fixture.root.clone(),
        },
        ScanMethod::FolderWalk,
        &NoProgressReporting,
        &cancellation,
    );

    assert!(matches!(result, Err(ScanError::Cancelled)));
}

#[test]
fn rejects_targets_that_are_missing_or_not_folders() {
    let fixture = TemporaryFixture::create("errors");
    let file = fixture.write_file("plain.txt", 1, b'f');

    let missing = scan(
        ScanTarget::Folder {
            path: fixture.root.join("does-not-exist"),
        },
        ScanMethod::FolderWalk,
        &NoProgressReporting,
        &CancellationToken::new(),
    );
    let not_a_folder = scan(
        ScanTarget::Folder { path: file },
        ScanMethod::FolderWalk,
        &NoProgressReporting,
        &CancellationToken::new(),
    );

    assert!(matches!(
        missing,
        Err(ScanError::FolderWalkFailed(
            FolderWalkError::RootNotFound { .. }
        ))
    ));
    assert!(matches!(
        not_a_folder,
        Err(ScanError::FolderWalkFailed(
            FolderWalkError::RootNotADirectory { .. }
        ))
    ));
}
