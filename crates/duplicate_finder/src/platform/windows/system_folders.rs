use std::ffi::OsStr;
use std::path::PathBuf;

/// Environment variables that point at folders holding Windows itself or installed programs.
const SYSTEM_FOLDER_VARIABLES: &[&str] = &[
    "SystemRoot",
    "windir",
    "ProgramFiles",
    "ProgramFiles(x86)",
    "ProgramW6432",
    "ProgramData",
];

/// Folder names that never hold personal files, wherever they appear.
const SYSTEM_FOLDER_NAMES: &[&str] = &["AppData", "$Recycle.Bin", "System Volume Information"];

/// Folders the duplicate finder must never search: the Windows folder, both Program Files
/// folders, `ProgramData` and the per-user application data folders.
pub(crate) fn system_folder_roots() -> Vec<PathBuf> {
    let from_environment = SYSTEM_FOLDER_VARIABLES
        .iter()
        .filter_map(std::env::var_os)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    let application_data_folders = [dirs::data_dir(), dirs::data_local_dir()]
        .into_iter()
        .flatten();

    from_environment.chain(application_data_folders).collect()
}

/// True for folder names such as `AppData` that mark application data at any depth.
pub(crate) fn is_system_folder_name(folder_name: &OsStr) -> bool {
    let folder_name = folder_name.to_string_lossy();

    SYSTEM_FOLDER_NAMES
        .iter()
        .any(|system_name| system_name.eq_ignore_ascii_case(&folder_name))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn includes_the_windows_and_program_files_folders() {
        let roots = system_folder_roots();
        let windows_folder = PathBuf::from(std::env::var_os("SystemRoot").unwrap());

        assert!(roots.contains(&windows_folder));
        assert!(roots.iter().any(|root| root.ends_with("Program Files")));
    }

    #[test]
    fn recognizes_application_data_names_ignoring_case() {
        assert!(is_system_folder_name(OsStr::new("appdata")));
        assert!(is_system_folder_name(OsStr::new("$RECYCLE.BIN")));
        assert!(!is_system_folder_name(OsStr::new("Pictures")));
    }
}
