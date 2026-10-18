use std::fs::{DirEntry, Metadata};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::UNIX_EPOCH;

use super::hidden_name::is_hidden_name;
use crate::directory_entry::{DirectoryEntry, EntryFlags, EntryKind, FileIdentity};

/// Lists one directory without opening any of the entries: every entry is described by
/// `lstat`, so symlinks are reported, never followed. `.` and `..` are left out.
///
/// A subdirectory on another device is a mount point and is flagged as a junction, so the
/// search never crosses into another file system. Device nodes, pipes and sockets are left out:
/// opening a pipe blocks, and none of them hold file contents.
pub(crate) fn list_directory(directory: &Path) -> io::Result<Vec<DirectoryEntry>> {
    let directory_device = std::fs::metadata(directory)?.dev();
    let mut entries = Vec::new();

    for listed_entry in std::fs::read_dir(directory)? {
        let listed_entry = listed_entry?;
        match listed_entry.metadata() {
            Ok(metadata) => {
                if let Some(entry) =
                    directory_entry_from_metadata(&listed_entry, &metadata, directory_device)
                {
                    entries.push(entry);
                }
            }
            Err(error) => {
                tracing::debug!(path = %listed_entry.path().display(), %error, "skipping unreadable entry");
            }
        }
    }

    Ok(entries)
}

fn directory_entry_from_metadata(
    listed_entry: &DirEntry,
    metadata: &Metadata,
    directory_device: u64,
) -> Option<DirectoryEntry> {
    let file_type = metadata.file_type();
    let name = listed_entry.file_name();
    let mut flags = EntryFlags::empty();

    let kind = if file_type.is_symlink() {
        flags |= EntryFlags::SYMLINK;
        EntryKind::File
    } else if file_type.is_dir() {
        if metadata.dev() != directory_device {
            flags |= EntryFlags::JUNCTION;
        }
        EntryKind::Directory
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        return None;
    };

    if is_hidden_name(&name) {
        flags |= EntryFlags::HIDDEN;
    }

    let modified_time = metadata.modified().unwrap_or(UNIX_EPOCH);
    let created_time = metadata.created().unwrap_or(modified_time);
    let identity = (metadata.ino() != 0)
        .then(|| FileIdentity::new(metadata.dev(), u128::from(metadata.ino())));

    Some(DirectoryEntry {
        name,
        kind,
        logical_size: metadata.len(),
        modified_time,
        created_time,
        identity,
        flags,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    struct TemporaryDirectory(std::path::PathBuf);

    impl TemporaryDirectory {
        fn create(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("vsm-{name}-{}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn sorted_entries(directory: &Path) -> Vec<DirectoryEntry> {
        let mut entries = list_directory(directory).unwrap();
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        entries
    }

    #[test]
    fn lists_a_temporary_directory_with_identities() {
        let directory = TemporaryDirectory::create("directory-listing-test");
        std::fs::create_dir(directory.0.join("nested")).unwrap();
        std::fs::write(directory.0.join("file.txt"), b"hello").unwrap();
        std::fs::write(directory.0.join(".hidden"), b"x").unwrap();

        let entries = sorted_entries(&directory.0);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name, ".hidden");
        assert_eq!(entries[0].flags, EntryFlags::HIDDEN);
        assert_eq!(entries[1].name, "file.txt");
        assert_eq!(entries[1].kind, EntryKind::File);
        assert_eq!(entries[1].logical_size, 5);
        assert!(entries[1].identity.is_some());
        assert_eq!(entries[1].flags, EntryFlags::empty());
        assert_eq!(entries[2].name, "nested");
        assert_eq!(entries[2].kind, EntryKind::Directory);
        assert_eq!(entries[2].flags, EntryFlags::empty());
    }

    #[test]
    fn reports_symlinks_without_following_them() {
        let directory = TemporaryDirectory::create("directory-listing-symlink-test");
        std::fs::create_dir(directory.0.join("target")).unwrap();
        std::os::unix::fs::symlink(directory.0.join("target"), directory.0.join("link")).unwrap();

        let entries = sorted_entries(&directory.0);

        assert_eq!(entries[0].name, "link");
        assert!(entries[0].flags.contains(EntryFlags::SYMLINK));
        assert_eq!(entries[1].name, "target");
        assert!(!entries[1].flags.is_link());
    }

    #[test]
    fn hard_links_share_an_identity() {
        let directory = TemporaryDirectory::create("directory-listing-hard-link-test");
        std::fs::write(directory.0.join("a"), b"same").unwrap();
        std::fs::hard_link(directory.0.join("a"), directory.0.join("b")).unwrap();

        let entries = sorted_entries(&directory.0);

        assert_eq!(entries[0].identity, entries[1].identity);
    }
}
