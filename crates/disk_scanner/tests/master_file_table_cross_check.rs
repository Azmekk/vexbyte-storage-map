//! Cross-checks the MFT scanner against the folder walk on a real NTFS volume.
//!
//! Run by hand from an elevated terminal (both scans are read-only):
//!
//! ```text
//! cargo test -p disk_scanner --release --test master_file_table_cross_check -- --ignored --nocapture
//! ```
//!
//! The two methods are expected to differ slightly, which is why the comparison uses a
//! tolerance:
//!
//! - The MFT also sees NTFS metafiles (`$MFT`, `$LogFile`, `$Bitmap`, ...). They are grouped
//!   under "NTFS metadata" and excluded from the comparison.
//! - The folder walk cannot list some folders even with administrator rights (for example
//!   `System Volume Information` or parts of `WindowsApps`); the MFT sees their contents.
//! - Files created or deleted between the two scans (logs, browser caches, the page file).
//! - Entries the MFT cannot link to a live parent end up in "Unlinked items".
//! - Allocated sizes of WOF-compressed system files differ: the folder walk reports the
//!   compressed allocation, the MFT the size of the unnamed data stream.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::path::Path;

use disk_scanner::{
    CancellationToken, FileTree, NoProgressReporting, ScanMethod, ScanResult, ScanTarget,
    is_process_elevated, list_volumes, scan,
};

/// Allowed relative difference between the two methods.
const TOLERANCE: f64 = 0.02;
const SYNTHETIC_DIRECTORIES_ONLY_IN_MFT: [&str; 2] = ["NTFS metadata", "Unlinked items"];

fn scan_volume(mount_point: &Path, method: ScanMethod) -> ScanResult {
    scan(
        ScanTarget::Volume {
            mount_point: mount_point.to_path_buf(),
        },
        method,
        &NoProgressReporting,
        &CancellationToken::new(),
    )
    .unwrap()
}

/// Logical size and file count without the synthetic MFT-only directories.
fn comparable_totals(tree: &FileTree) -> (u64, u64) {
    let totals = tree.totals();
    let mut logical_size = totals.logical_size;
    let mut file_count = u64::from(totals.file_count);

    for child in tree.children(tree.root()) {
        if SYNTHETIC_DIRECTORIES_ONLY_IN_MFT.contains(&tree.name(child)) {
            let node = tree.node(child).unwrap();
            logical_size -= node.logical_size();
            file_count -= u64::from(node.file_count());
        }
    }
    (logical_size, file_count)
}

fn relative_difference(first: u64, second: u64) -> f64 {
    let larger = first.max(second).max(1) as f64;
    first.abs_diff(second) as f64 / larger
}

#[test]
#[ignore = "needs administrator rights and scans a whole real volume; run by hand"]
fn mft_scan_agrees_with_the_folder_walk() {
    if !is_process_elevated() {
        eprintln!("skipping: the MFT scan needs an elevated process");
        return;
    }
    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_owned());
    let volume = list_volumes()
        .unwrap()
        .into_iter()
        .find(|volume| {
            volume.supports_fast_scan
                && volume
                    .mount_point
                    .to_string_lossy()
                    .starts_with(&system_drive)
        })
        .expect("the system drive should be an NTFS volume");

    let mft_result = scan_volume(&volume.mount_point, ScanMethod::MasterFileTable);
    let walk_result = scan_volume(&volume.mount_point, ScanMethod::FolderWalk);

    let (mft_logical_size, mft_file_count) = comparable_totals(&mft_result.tree);
    let (walk_logical_size, walk_file_count) = comparable_totals(&walk_result.tree);
    println!(
        "MFT:         {mft_file_count} files, {mft_logical_size} bytes in {} ms",
        mft_result.summary.duration_milliseconds
    );
    println!(
        "Folder walk: {walk_file_count} files, {walk_logical_size} bytes in {} ms ({} folders inaccessible)",
        walk_result.summary.duration_milliseconds,
        walk_result.summary.totals.inaccessible_directory_count
    );

    assert!(
        relative_difference(mft_file_count, walk_file_count) < TOLERANCE,
        "file counts differ by more than {TOLERANCE}"
    );
    assert!(
        relative_difference(mft_logical_size, walk_logical_size) < TOLERANCE,
        "logical sizes differ by more than {TOLERANCE}"
    );
}
