//! A manual, read-only smoke run over the current user's real Pictures and Downloads folders.
//! It only lists and reads files; nothing is changed. Run it with:
//!
//! `cargo test --release -p duplicate_finder --test smoke_search_of_personal_folders -- --ignored --nocapture`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

use std::time::Instant;

use duplicate_finder::{
    CancellationToken, DuplicateSearchSettings, DuplicateSearchStage, FileCategory, find_duplicates,
};

#[test]
#[ignore = "reads the real Pictures and Downloads folders; run by hand"]
fn search_pictures_and_downloads_for_duplicates() {
    let folders: Vec<_> = [dirs::picture_dir(), dirs::download_dir()]
        .into_iter()
        .flatten()
        .filter(|folder| folder.is_dir())
        .collect();
    let settings = DuplicateSearchSettings::new(
        folders,
        vec![
            FileCategory::Images,
            FileCategory::Videos,
            FileCategory::Pdfs,
        ],
    );
    let started_at = Instant::now();

    let result = find_duplicates(
        &settings,
        |progress| {
            if progress.stage != DuplicateSearchStage::Collecting {
                eprintln!("{:>8} ms  {:?}", started_at.elapsed().as_millis(), progress);
            }
        },
        &CancellationToken::new(),
    )
    .unwrap();

    println!("folders and drive kinds: {:?}", result.folder_drive_kinds);
    println!("files examined:          {}", result.files_examined);
    println!("duplicate groups:        {}", result.groups.len());
    println!("duplicate files:         {}", result.duplicate_file_count);
    println!(
        "wasted bytes:            {} ({:.1} MiB)",
        result.total_wasted_bytes,
        result.total_wasted_bytes as f64 / (1024.0 * 1024.0)
    );
    println!(
        "online-only skipped:     {}",
        result.online_only_files_skipped
    );
    println!("hard links collapsed:    {}", result.hard_links_collapsed);
    println!("unreadable files:        {}", result.unreadable_file_count);
    println!(
        "inaccessible folders:    {}",
        result.inaccessible_folder_count
    );
    println!(
        "elapsed:                 {} ms",
        result.elapsed_milliseconds
    );
    for group in result.groups.iter().take(5) {
        println!(
            "  {} copies x {} bytes, first {}",
            group.files.len(),
            group.file_size,
            group.files[0].path.display()
        );
    }
}
