//! Scans a folder read-only with the folder walk and prints totals and timings.
//!
//! ```text
//! cargo run -p disk_scanner --release --example scan_folder -- [FOLDER]
//! ```
//!
//! Without an argument it scans `%USERPROFILE%`. Nothing is ever written or changed.

#![allow(clippy::cast_precision_loss)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use disk_scanner::{
    CancellationToken, ChildSortOrder, OnlineOnlyVisibility, ScanMethod, ScanProgressUpdate,
    ScanTarget, scan,
};

fn main() -> ExitCode {
    let Some(folder) = std::env::args_os()
        .nth(1)
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
    else {
        eprintln!("usage: scan_folder FOLDER");
        return ExitCode::FAILURE;
    };

    let report_progress = |update: ScanProgressUpdate| {
        eprint!(
            "\r{:?}: {} files, {} folders, {:.1} GiB, {} ms      ",
            update.phase,
            update.files_found,
            update.directories_found,
            update.bytes_found as f64 / 1_073_741_824.0,
            update.elapsed_milliseconds
        );
    };

    let result = match scan(
        ScanTarget::Folder {
            path: folder.clone(),
        },
        ScanMethod::FolderWalk,
        &report_progress,
        &CancellationToken::new(),
    ) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("\nscan of {} failed: {error}", folder.display());
            return ExitCode::FAILURE;
        }
    };
    eprintln!();

    let tree = &result.tree;
    let totals = result.summary.totals;
    let duration_seconds = result.summary.duration_milliseconds as f64 / 1000.0;
    println!("scanned            {}", tree.root_path().display());
    println!("duration           {duration_seconds:.2} s");
    println!(
        "files / folders    {} / {} ({:.0} files per second)",
        totals.file_count,
        totals.directory_count,
        f64::from(totals.file_count) / duration_seconds.max(0.001)
    );
    println!(
        "logical size       {:.2} GiB",
        totals.logical_size as f64 / 1_073_741_824.0
    );
    println!(
        "allocated size     {:.2} GiB",
        totals.allocated_size as f64 / 1_073_741_824.0
    );
    println!(
        "inaccessible       {} folders",
        totals.inaccessible_directory_count
    );
    println!(
        "cloud only         {} files, {:.2} GiB",
        totals.cloud_placeholder_file_count,
        totals.cloud_placeholder_logical_size as f64 / 1_073_741_824.0
    );
    println!("extra hard links   {}", totals.additional_hard_link_count);
    println!("symlinks/junctions {}", totals.link_count);
    println!(
        "tree memory        {:.1} MiB ({:.0} bytes per node)",
        tree.estimated_memory_bytes() as f64 / 1_048_576.0,
        tree.estimated_memory_bytes() as f64 / tree.node_count().max(1) as f64
    );

    let started_at = Instant::now();
    let page = tree.children_page(
        tree.root(),
        ChildSortOrder::LogicalSizeDescending,
        0,
        10,
        OnlineOnlyVisibility::Hidden,
    );
    let children_elapsed = started_at.elapsed();
    let started_at = Instant::now();
    let visualization =
        tree.visualization_subtree(tree.root(), 6, 0.002, OnlineOnlyVisibility::Hidden);
    let visualization_elapsed = started_at.elapsed();
    let started_at = Instant::now();
    let search_results = tree.search_nodes("readme", 100, OnlineOnlyVisibility::Hidden);
    let search_elapsed = started_at.elapsed();
    println!(
        "query timings      children {children_elapsed:.2?}, visualization {visualization_elapsed:.2?}, search {search_elapsed:.2?} ({} hits)",
        search_results.len()
    );
    let _ = visualization;

    if let Ok(page) = page {
        println!("largest entries:");
        for child in page.children {
            println!(
                "  {:>10.2} MiB  {}",
                child.logical_size as f64 / 1_048_576.0,
                child.name
            );
        }
    }

    ExitCode::SUCCESS
}
