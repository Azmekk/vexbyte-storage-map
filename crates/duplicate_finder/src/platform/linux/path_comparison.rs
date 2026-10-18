use std::io;
use std::path::{Component, Path, PathBuf};

/// True when `path` is `folder` itself or lies somewhere below it. Linux file names are
/// case-sensitive, so components must match exactly.
pub(crate) fn is_same_or_inside_folder(path: &Path, folder: &Path) -> bool {
    let mut path_components = path.components().filter(is_meaningful_component);

    folder
        .components()
        .filter(is_meaningful_component)
        .all(|folder_component| path_components.next() == Some(folder_component))
}

fn is_meaningful_component(component: &Component<'_>) -> bool {
    !matches!(component, Component::CurDir)
}

/// Makes `path` absolute and resolves `.` and `..` by name, without following links, so it can
/// be shown to users and compared with other folders. `std::path::absolute` keeps `..` on Unix,
/// which would let `/home/ana/../../usr` slip past the system folder check.
pub(crate) fn normalize_folder_path(path: &Path) -> io::Result<PathBuf> {
    let absolute_path = std::path::absolute(path)?;
    let mut normalized_path = PathBuf::new();

    for component in absolute_path.components() {
        match component {
            Component::ParentDir => {
                normalized_path.pop();
            }
            Component::CurDir => {}
            other => normalized_path.push(other),
        }
    }

    Ok(normalized_path)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn matches_the_same_folder_only_with_the_same_case() {
        assert!(is_same_or_inside_folder(
            Path::new("/home/ana/Pictures"),
            Path::new("/home/ana/Pictures")
        ));
        assert!(!is_same_or_inside_folder(
            Path::new("/home/ana/Pictures"),
            Path::new("/home/ana/pictures")
        ));
    }

    #[test]
    fn matches_nested_paths() {
        assert!(is_same_or_inside_folder(
            Path::new("/home/ana/Pictures/2024/a.jpg"),
            Path::new("/home/ana/Pictures")
        ));
    }

    #[test]
    fn rejects_siblings_with_a_common_name_prefix() {
        assert!(!is_same_or_inside_folder(
            Path::new("/home/ana/Pictures Old/a.jpg"),
            Path::new("/home/ana/Pictures")
        ));
    }

    #[test]
    fn rejects_parents() {
        assert!(!is_same_or_inside_folder(
            Path::new("/home"),
            Path::new("/home/ana")
        ));
    }

    #[test]
    fn normalizes_parent_and_current_components() {
        assert_eq!(
            normalize_folder_path(Path::new("/home/ana/../bo/./Pictures")).unwrap(),
            PathBuf::from("/home/bo/Pictures")
        );
        assert_eq!(
            normalize_folder_path(Path::new("/../..")).unwrap(),
            PathBuf::from("/")
        );
    }
}
