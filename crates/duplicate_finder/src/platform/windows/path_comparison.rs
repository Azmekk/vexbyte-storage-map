use std::io;
use std::path::{Component, Path, PathBuf};

use super::wide_path::remove_extended_length_prefix;

/// True when `path` is `folder` itself or lies somewhere below it. Windows file names are
/// case-insensitive, so components are compared without regard to case.
pub(crate) fn is_same_or_inside_folder(path: &Path, folder: &Path) -> bool {
    let mut path_components = path.components().filter(is_meaningful_component);

    folder
        .components()
        .filter(is_meaningful_component)
        .all(|folder_component| {
            path_components.next().is_some_and(|path_component| {
                components_are_equal_ignoring_case(path_component, folder_component)
            })
        })
}

fn is_meaningful_component(component: &Component<'_>) -> bool {
    !matches!(component, Component::CurDir)
}

fn components_are_equal_ignoring_case(left: Component<'_>, right: Component<'_>) -> bool {
    let left_text = left.as_os_str().to_string_lossy();
    let right_text = right.as_os_str().to_string_lossy();

    left_text == right_text || left_text.to_lowercase() == right_text.to_lowercase()
}

/// Makes `path` absolute and resolves `.` and `..`, without following links and without the
/// `\?\` prefix, so it can be shown to users and compared with other folders.
pub(crate) fn normalize_folder_path(path: &Path) -> io::Result<PathBuf> {
    let absolute_path = std::path::absolute(path)?;
    Ok(remove_extended_length_prefix(&absolute_path))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn matches_the_same_folder_ignoring_case() {
        assert!(is_same_or_inside_folder(
            Path::new(r"C:\Users\Ana\Pictures"),
            Path::new(r"c:\users\ana\pictures")
        ));
    }

    #[test]
    fn matches_nested_paths() {
        assert!(is_same_or_inside_folder(
            Path::new(r"C:\Users\Ana\Pictures\2024\a.jpg"),
            Path::new(r"C:\Users\Ana\Pictures")
        ));
    }

    #[test]
    fn rejects_siblings_with_a_common_name_prefix() {
        assert!(!is_same_or_inside_folder(
            Path::new(r"C:\Users\Ana\Pictures Old\a.jpg"),
            Path::new(r"C:\Users\Ana\Pictures")
        ));
    }

    #[test]
    fn rejects_parents() {
        assert!(!is_same_or_inside_folder(
            Path::new(r"C:\Users"),
            Path::new(r"C:\Users\Ana")
        ));
    }

    #[test]
    fn normalizes_parent_components() {
        assert_eq!(
            normalize_folder_path(Path::new(r"C:\Users\Ana\..\Bo")).unwrap(),
            PathBuf::from(r"C:\Users\Bo")
        );
    }
}
