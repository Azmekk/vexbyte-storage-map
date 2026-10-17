use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use file_actions::ThumbnailCache;

use crate::duplicate_search_registry::DuplicateSearchRegistry;
use crate::scan_registry::ScanRegistry;
use crate::search_index_registry::SearchIndexRegistry;
use crate::startup_arguments::StartupScanRequest;

/// The folder below the user's local app data folder (`%LOCALAPPDATA%` on Windows) that holds
/// the app's own data and caches. The asset protocol scope in `tauri.conf.json` names the same
/// folder through `$LOCALDATA`; keep both in sync.
pub(crate) const APP_DATA_FOLDER_NAMES: [&str; 2] = ["Vexbyte", "Storage Map"];
pub(crate) const THUMBNAIL_FOLDER_NAME: &str = "thumbnails";

/// State shared by every command, managed by Tauri. The file trees and duplicate groups stay in
/// Rust; the frontend only queries small views of them.
pub struct AppState {
    pub scans: ScanRegistry,
    pub duplicate_searches: DuplicateSearchRegistry,
    /// The whole-volume indexes of the global file search, built at app start.
    pub search_indexes: SearchIndexRegistry,
    pub thumbnail_cache: ThumbnailCache,
    pub startup_scan_request: Option<StartupScanRequest>,
    /// Held for the whole of every Recycle Bin action, from the last check to the shell call,
    /// so two actions can never each remove the copy the other one relies on being kept.
    pub recycle_bin_operations: Arc<Mutex<()>>,
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
            search_indexes: SearchIndexRegistry::default(),
            thumbnail_cache: ThumbnailCache::new(thumbnail_directory),
            startup_scan_request,
            recycle_bin_operations: Arc::new(Mutex::new(())),
        }
    }
}

/// `%LOCALAPPDATA%\Vexbyte\Storage Map\thumbnails` for a local data folder of `%LOCALAPPDATA%`.
pub(crate) fn thumbnail_directory_in(local_data_directory: &Path) -> PathBuf {
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
