//! Example paths for platform-neutral unit tests, so they can be written once for every backend.

use std::path::PathBuf;

/// The root of a volume, as users would pick it.
pub(crate) fn example_volume_root() -> PathBuf {
    PathBuf::from("/")
}

/// A folder of this computer that the system folder guard always rejects.
pub(crate) fn example_system_folder() -> PathBuf {
    PathBuf::from("/usr")
}

/// A folder name that marks system data at any depth.
pub(crate) fn example_system_folder_name() -> &'static str {
    "lost+found"
}

/// Whether two folder names that differ only in case name the same folder.
pub(crate) fn folder_names_ignore_case() -> bool {
    false
}
