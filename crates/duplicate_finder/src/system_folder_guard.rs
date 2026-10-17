use std::path::{Component, Path, PathBuf};

use crate::platform::{is_same_or_inside_folder, is_system_folder_name, system_folder_roots};

/// Knows which folders belong to the operating system or to installed programs, so the duplicate
/// finder neither accepts them as search folders nor walks into them.
#[derive(Debug, Clone)]
pub struct SystemFolderGuard {
    system_folders: Vec<PathBuf>,
    rejects_application_data_names: bool,
}

impl SystemFolderGuard {
    /// This computer's system folders, plus any folder named like `AppData` at any depth.
    #[must_use]
    pub fn for_this_computer() -> Self {
        Self {
            system_folders: system_folder_roots(),
            rejects_application_data_names: true,
        }
    }

    /// Only the given folders count as system folders. Meant for tests, whose fixtures live in
    /// the temp directory, which itself sits inside the user's application data.
    #[must_use]
    pub fn with_system_folders(system_folders: Vec<PathBuf>) -> Self {
        Self {
            system_folders,
            rejects_application_data_names: false,
        }
    }

    /// True when `path` is a system folder, lies inside one, or (for this computer's guard)
    /// passes through a folder such as `AppData` that only holds application data.
    #[must_use]
    pub fn is_system_path(&self, path: &Path) -> bool {
        let is_inside_system_folder = self
            .system_folders
            .iter()
            .any(|system_folder| is_same_or_inside_folder(path, system_folder));

        is_inside_system_folder
            || (self.rejects_application_data_names && has_system_folder_component(path))
    }
}

fn has_system_folder_component(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(name) => is_system_folder_name(name),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn guard_with_windows_folder() -> SystemFolderGuard {
        SystemFolderGuard::with_system_folders(vec![PathBuf::from(r"C:\Windows")])
    }

    #[test]
    fn rejects_system_folders_and_their_children() {
        let guard = guard_with_windows_folder();

        assert!(guard.is_system_path(Path::new(r"C:\Windows")));
        assert!(guard.is_system_path(Path::new(r"c:\windows\System32")));
    }

    #[test]
    fn rejects_application_data_at_any_depth() {
        let guard = SystemFolderGuard::for_this_computer();

        assert!(guard.is_system_path(Path::new(r"D:\Backup\Ana\AppData\Local")));
    }

    #[test]
    fn accepts_personal_folders() {
        let guard = guard_with_windows_folder();

        assert!(!guard.is_system_path(Path::new(r"C:\Users\Ana\Pictures")));
        assert!(!guard.is_system_path(Path::new(r"C:\Windows Photos Backup")));
    }

    #[test]
    fn knows_this_computers_program_files() {
        let guard = SystemFolderGuard::for_this_computer();
        let program_files = PathBuf::from(std::env::var_os("ProgramFiles").unwrap());

        assert!(guard.is_system_path(&program_files.join("Some App")));
    }
}
