use std::ffi::OsStr;
use std::path::PathBuf;

/// Folders that hold the operating system, installed programs, kernel interfaces or temporary
/// data, never personal files. `/home`, `/media`, `/mnt` and `/srv` hold user data and stay open.
const OPERATING_SYSTEM_FOLDERS: &[&str] = &[
    "/proc", "/sys", "/dev", "/run", "/boot", "/usr", "/etc", "/var", "/tmp", "/bin", "/sbin",
    "/lib", "/lib32", "/lib64", "/libx32", "/opt", "/snap",
];

/// Folder names that never hold personal files, wherever they appear.
const SYSTEM_FOLDER_NAMES: &[&str] = &["lost+found", ".Trash"];
/// Per-user trash folders on removable drives are named `.Trash-<uid>`.
const PER_USER_TRASH_PREFIX: &str = ".Trash-";

/// Folders the duplicate finder must never search: the operating system folders and the
/// per-user application data, configuration and cache folders (the `AppData` of Linux).
pub(crate) fn system_folder_roots() -> Vec<PathBuf> {
    let operating_system_folders = OPERATING_SYSTEM_FOLDERS.iter().map(PathBuf::from);

    let application_data_folders = [
        dirs::data_dir(),
        dirs::data_local_dir(),
        dirs::config_dir(),
        dirs::cache_dir(),
    ]
    .into_iter()
    .flatten();

    let mut roots: Vec<PathBuf> = operating_system_folders
        .chain(application_data_folders)
        .collect();
    roots.dedup();
    roots
}

/// True for folder names such as `lost+found` and trash folders, at any depth.
pub(crate) fn is_system_folder_name(folder_name: &OsStr) -> bool {
    let folder_name = folder_name.to_string_lossy();

    SYSTEM_FOLDER_NAMES.contains(&folder_name.as_ref())
        || folder_name.starts_with(PER_USER_TRASH_PREFIX)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn includes_the_operating_system_and_application_data_folders() {
        let roots = system_folder_roots();

        assert!(roots.contains(&PathBuf::from("/usr")));
        assert!(roots.contains(&PathBuf::from("/proc")));
        assert!(roots.contains(&dirs::data_local_dir().unwrap()));
        assert!(!roots.contains(&PathBuf::from("/home")));
    }

    #[test]
    fn recognizes_trash_and_recovery_folder_names() {
        assert!(is_system_folder_name(OsStr::new("lost+found")));
        assert!(is_system_folder_name(OsStr::new(".Trash")));
        assert!(is_system_folder_name(OsStr::new(".Trash-1000")));
        assert!(!is_system_folder_name(OsStr::new("Trash")));
        assert!(!is_system_folder_name(OsStr::new("Pictures")));
    }
}
