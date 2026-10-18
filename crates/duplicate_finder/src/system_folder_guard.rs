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
    use crate::platform::{
        example_system_folder, example_system_folder_name, example_volume_root,
        folder_names_ignore_case,
    };

    fn guard_with_example_system_folder() -> SystemFolderGuard {
        SystemFolderGuard::with_system_folders(vec![example_system_folder()])
    }

    fn with_swapped_case(path: &Path) -> PathBuf {
        let text = path.to_string_lossy();
        let swapped: String = text
            .chars()
            .map(|character| {
                if character.is_uppercase() {
                    character.to_lowercase().collect::<String>()
                } else {
                    character.to_uppercase().collect::<String>()
                }
            })
            .collect();
        PathBuf::from(swapped)
    }

    #[test]
    fn rejects_system_folders_and_their_children() {
        let guard = guard_with_example_system_folder();
        let system_folder = example_system_folder();

        assert!(guard.is_system_path(&system_folder));
        assert!(guard.is_system_path(&system_folder.join("Some App")));
    }

    #[test]
    fn compares_case_as_the_platform_does() {
        let guard = guard_with_example_system_folder();
        let swapped_case_child = with_swapped_case(&example_system_folder().join("Some App"));

        assert_eq!(
            guard.is_system_path(&swapped_case_child),
            folder_names_ignore_case()
        );
    }

    #[test]
    fn rejects_application_data_at_any_depth() {
        let guard = SystemFolderGuard::for_this_computer();
        let path = example_volume_root()
            .join("Backup")
            .join("Ana")
            .join(example_system_folder_name())
            .join("Local");

        assert!(guard.is_system_path(&path));
    }

    #[test]
    fn accepts_personal_folders() {
        let guard = guard_with_example_system_folder();
        let system_folder = example_system_folder();
        let mut sibling_name = system_folder.file_name().unwrap().to_os_string();
        sibling_name.push(" Photos Backup");

        let pictures = example_volume_root()
            .join("Users")
            .join("Ana")
            .join("Pictures");
        assert!(!guard.is_system_path(&pictures));
        assert!(!guard.is_system_path(&system_folder.with_file_name(sibling_name)));
    }

    #[test]
    fn knows_this_computers_system_folders() {
        let guard = SystemFolderGuard::for_this_computer();

        assert!(guard.is_system_path(&example_system_folder().join("Some App")));
    }
}
