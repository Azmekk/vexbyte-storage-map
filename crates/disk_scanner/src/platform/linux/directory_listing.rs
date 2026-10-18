//! Lists a directory with `readdir` and reads each entry with `statx` relative to the directory,
//! without following symlinks and without ever opening the entry itself.

use std::borrow::Cow;
use std::fs::{DirEntry, FileType, Metadata};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::file_identity::FileIdentity;
use crate::file_tree::NodeKind;
use crate::folder_walk::{DirectoryEntry, DirectoryListError, EntryFlags};

/// `st_blocks` always counts 512-byte units, whatever the block size of the file system.
const BLOCK_COUNT_UNIT_BYTES: u64 = 512;

/// Lists every entry of `directory_path` except `.` and `..`, calling `on_entry` for each.
///
/// A subdirectory on another device is a mount point; it is flagged so the walk never enters it,
/// which keeps a scan of `/` out of `/proc`, `/sys`, other disks and network shares.
pub(crate) fn list_directory(
    directory_path: &Path,
    on_entry: &mut dyn FnMut(DirectoryEntry<'_>),
) -> Result<(), DirectoryListError> {
    let open_failed = |source| DirectoryListError::OpenFailed {
        path: directory_path.to_path_buf(),
        source,
    };
    let directory_device = std::fs::metadata(directory_path)
        .map_err(open_failed)?
        .dev();
    let directory_entries = std::fs::read_dir(directory_path).map_err(open_failed)?;

    for directory_entry in directory_entries {
        let directory_entry = directory_entry.map_err(|source| DirectoryListError::ReadFailed {
            path: directory_path.to_path_buf(),
            source,
        })?;
        report_entry(&directory_entry, directory_device, on_entry);
    }

    Ok(())
}

fn report_entry(
    directory_entry: &DirEntry,
    directory_device: u64,
    on_entry: &mut dyn FnMut(DirectoryEntry<'_>),
) {
    let file_name = directory_entry.file_name();
    let (name, name_flags) = match file_name.to_string_lossy() {
        Cow::Borrowed(name) => (Cow::Borrowed(name), EntryFlags::empty()),
        Cow::Owned(lossy_name) => (Cow::Owned(lossy_name), EntryFlags::INACCESSIBLE),
    };
    let hidden_flag = if name.starts_with('.') {
        EntryFlags::HIDDEN
    } else {
        EntryFlags::empty()
    };

    let mut entry = match directory_entry.metadata() {
        Ok(metadata) => entry_from_metadata(&name, &metadata, directory_device),
        Err(_) => inaccessible_entry(&name, directory_entry.file_type().ok()),
    };
    entry.flags |= name_flags | hidden_flag;

    on_entry(entry);
}

fn entry_from_metadata<'name>(
    name: &'name str,
    metadata: &Metadata,
    directory_device: u64,
) -> DirectoryEntry<'name> {
    let file_type = metadata.file_type();
    let modified_time = metadata.modified().unwrap_or(UNIX_EPOCH);

    if file_type.is_dir() {
        let flags = if metadata.dev() == directory_device {
            EntryFlags::empty()
        } else {
            EntryFlags::JUNCTION
        };
        return sizeless_entry(name, NodeKind::Directory, modified_time, flags);
    }

    if file_type.is_symlink() {
        return sizeless_entry(name, NodeKind::File, modified_time, EntryFlags::SYMLINK);
    }

    // Only a file with more than one link can be a hard link, so files with a single link get no
    // identity and the hard link tracker's map stays small.
    let identity = (metadata.nlink() > 1).then(|| {
        FileIdentity::from_volume_and_file_ids(metadata.dev(), u128::from(metadata.ino()))
    });

    DirectoryEntry {
        name,
        kind: NodeKind::File,
        logical_size: metadata.size(),
        allocated_size: metadata.blocks().saturating_mul(BLOCK_COUNT_UNIT_BYTES),
        modified_time,
        identity,
        flags: EntryFlags::empty(),
    }
}

/// An entry whose metadata could not be read. Its kind still comes from the directory listing.
fn inaccessible_entry(name: &str, file_type: Option<FileType>) -> DirectoryEntry<'_> {
    let kind = if file_type.is_some_and(|file_type| file_type.is_dir()) {
        NodeKind::Directory
    } else {
        NodeKind::File
    };
    sizeless_entry(name, kind, UNIX_EPOCH, EntryFlags::INACCESSIBLE)
}

fn sizeless_entry(
    name: &str,
    kind: NodeKind,
    modified_time: SystemTime,
    flags: EntryFlags,
) -> DirectoryEntry<'_> {
    DirectoryEntry {
        name,
        kind,
        logical_size: 0,
        allocated_size: 0,
        modified_time,
        identity: None,
        flags,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::*;

    struct TemporaryDirectory {
        path: PathBuf,
    }

    impl TemporaryDirectory {
        fn create(test_name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "disk_scanner_linux_listing_{test_name}_{}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    struct ListedEntry {
        kind: NodeKind,
        logical_size: u64,
        allocated_size: u64,
        has_identity: bool,
        flags: EntryFlags,
    }

    fn list_into_map(directory_path: &Path) -> HashMap<String, ListedEntry> {
        let mut listed_entries = HashMap::new();
        list_directory(directory_path, &mut |entry| {
            listed_entries.insert(
                entry.name.to_owned(),
                ListedEntry {
                    kind: entry.kind,
                    logical_size: entry.logical_size,
                    allocated_size: entry.allocated_size,
                    has_identity: entry.identity.is_some(),
                    flags: entry.flags,
                },
            );
        })
        .unwrap();
        listed_entries
    }

    #[test]
    fn lists_files_directories_symlinks_and_hidden_entries() {
        let fixture = TemporaryDirectory::create("kinds");
        std::fs::write(fixture.path.join("report.txt"), vec![7_u8; 5000]).unwrap();
        std::fs::write(fixture.path.join(".settings"), b"hidden").unwrap();
        std::fs::create_dir(fixture.path.join("photos")).unwrap();
        std::os::unix::fs::symlink("report.txt", fixture.path.join("link-to-report")).unwrap();

        let listed_entries = list_into_map(&fixture.path);

        assert_eq!(listed_entries.len(), 4);
        let report = &listed_entries["report.txt"];
        assert_eq!(report.kind, NodeKind::File);
        assert_eq!(report.logical_size, 5000);
        assert!(report.allocated_size > 0);
        assert!(!report.has_identity);
        assert!(report.flags.is_empty());

        assert!(
            listed_entries[".settings"]
                .flags
                .contains(EntryFlags::HIDDEN)
        );

        let photos = &listed_entries["photos"];
        assert_eq!(photos.kind, NodeKind::Directory);
        assert_eq!((photos.logical_size, photos.allocated_size), (0, 0));
        assert!(!photos.flags.must_not_be_entered());

        let link = &listed_entries["link-to-report"];
        assert_eq!(link.kind, NodeKind::File);
        assert_eq!((link.logical_size, link.allocated_size), (0, 0));
        assert_eq!(link.flags, EntryFlags::SYMLINK);
    }

    #[test]
    fn hard_linked_files_get_an_identity() {
        let fixture = TemporaryDirectory::create("hard_links");
        std::fs::write(fixture.path.join("original.bin"), b"shared content").unwrap();
        std::fs::hard_link(
            fixture.path.join("original.bin"),
            fixture.path.join("second-name.bin"),
        )
        .unwrap();

        let listed_entries = list_into_map(&fixture.path);

        assert!(listed_entries["original.bin"].has_identity);
        assert!(listed_entries["second-name.bin"].has_identity);
    }

    #[test]
    fn a_missing_directory_fails_to_open() {
        let fixture = TemporaryDirectory::create("missing");

        let result = list_directory(&fixture.path.join("does-not-exist"), &mut |_| {});

        assert!(matches!(result, Err(DirectoryListError::OpenFailed { .. })));
    }
}
