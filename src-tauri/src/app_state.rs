use std::path::{Path, PathBuf};

use file_actions::ThumbnailCache;

use crate::duplicate_search_registry::DuplicateSearchRegistry;
use crate::scan_registry::ScanRegistry;
use crate::startup_arguments::StartupScanRequest;

/// The folder below the user's local app data folder (`%LOCALAPPDATA%` on Windows) that holds
/// the app's own data and caches. The asset protocol scope in `tauri.conf.json` names the same
/// folder through `$LOCALDATA`; keep both in sync.
const APP_DATA_FOLDER_NAMES: [&str; 2] = ["Vexbyte", "Storage Map"];
const THUMBNAIL_FOLDER_NAME: &str = "thumbnails";

/// State shared by every command, managed by Tauri. The file trees and duplicate groups stay in
/// Rust; the frontend only queries small views of them.
pub struct AppState {
    pub scans: ScanRegistry,
    pub duplicate_searches: DuplicateSearchRegistry,
    pub thumbnail_cache: ThumbnailCache,
    pub startup_scan_request: Option<StartupScanRequest>,
}

impl AppState {
    pub fn new(
        local_data_directory: &Path,
        startup_scan_request: Option<StartupScanRequest>,
    ) -> Self {
        let thumbnail_directory = thumbnail_directory_in(local_data_directory);
        if let Err(error) = std::fs::create_dir_all(&thumbnail_directory) {
            tracing::warn!(
                %error,
                directory = %thumbnail_directory.display(),
                "could not create the thumbnail cache folder; it is retried on first use"
            );
        }

        Self {
            scans: ScanRegistry::default(),
            duplicate_searches: DuplicateSearchRegistry::default(),
            thumbnail_cache: ThumbnailCache::new(thumbnail_directory),
            startup_scan_request,
        }
    }
}

/// `%LOCALAPPDATA%\Vexbyte\Storage Map\thumbnails` for a local data folder of `%LOCALAPPDATA%`.
fn thumbnail_directory_in(local_data_directory: &Path) -> PathBuf {
    let mut directory = local_data_directory.to_path_buf();
    directory.extend(APP_DATA_FOLDER_NAMES);
    directory.push(THUMBNAIL_FOLDER_NAME);
    directory
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn thumbnails_live_in_the_vexbyte_storage_map_folder() {
        let local_data = std::env::temp_dir();

        assert_eq!(
            thumbnail_directory_in(&local_data),
            local_data
                .join("Vexbyte")
                .join("Storage Map")
                .join("thumbnails")
        );
    }
}
