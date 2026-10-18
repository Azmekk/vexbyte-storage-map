use std::path::{Path, PathBuf};

use mount_table::{FileSystemCategory, categorize_file_system, mount_containing, read_mount_table};

/// True when files at `path` can go to the freedesktop Trash, which the `trash` crate implements
/// (the home Trash, or `.Trash-<uid>` at the top of another mount). Network shares, memory file
/// systems and kernel interfaces are refused, as are paths whose mount cannot be determined.
/// Only the mount table and metadata are read.
pub(crate) fn volume_has_recycle_bin(path: &Path) -> bool {
    let Some(resolved_path) = resolve_location_of(path) else {
        return false;
    };
    let Ok(mount_entries) = read_mount_table() else {
        return false;
    };

    mount_containing(&mount_entries, &resolved_path).is_some_and(|mount_entry| {
        file_system_category_has_trash(categorize_file_system(&mount_entry.file_system_type))
    })
}

/// Resolves symbolic links in the folders above `path` but not in its last component: the Trash
/// moves a symbolic link itself, which lives where its parent folder does.
fn resolve_location_of(path: &Path) -> Option<PathBuf> {
    let file_name = path.file_name()?;
    let parent = path.parent()?;
    let resolved_parent = std::fs::canonicalize(parent).ok()?;
    Some(resolved_parent.join(file_name))
}

fn file_system_category_has_trash(category: FileSystemCategory) -> bool {
    match category {
        FileSystemCategory::BlockDevice | FileSystemCategory::Optical => true,
        FileSystemCategory::Remote | FileSystemCategory::Memory | FileSystemCategory::Virtual => {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn only_local_storage_has_a_trash() {
        assert!(file_system_category_has_trash(
            FileSystemCategory::BlockDevice
        ));
        assert!(!file_system_category_has_trash(FileSystemCategory::Remote));
        assert!(!file_system_category_has_trash(FileSystemCategory::Memory));
        assert!(!file_system_category_has_trash(FileSystemCategory::Virtual));
    }

    #[test]
    fn keeps_the_last_component_unresolved() {
        let temporary_folder = std::fs::canonicalize(std::env::temp_dir()).unwrap();

        let resolved = resolve_location_of(&std::env::temp_dir().join("not-there.txt")).unwrap();

        assert_eq!(resolved, temporary_folder.join("not-there.txt"));
        assert_eq!(resolve_location_of(Path::new("/")), None);
    }
}
