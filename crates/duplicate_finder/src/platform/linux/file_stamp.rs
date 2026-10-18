use std::io;
use std::path::Path;
use std::time::UNIX_EPOCH;

use super::hidden_name::is_hidden_name;
use crate::directory_entry::{EntryFlags, EntryKind, FileStamp};

/// Reads size, modification time and kind of `path` with `lstat`, without opening the file and
/// without following a symlink.
pub(crate) fn read_file_stamp(path: &Path) -> io::Result<FileStamp> {
    let metadata = std::fs::symlink_metadata(path)?;
    let file_type = metadata.file_type();

    let mut flags = EntryFlags::empty();
    if file_type.is_symlink() {
        flags |= EntryFlags::SYMLINK;
    }
    if path.file_name().is_some_and(is_hidden_name) {
        flags |= EntryFlags::HIDDEN;
    }

    let kind = if file_type.is_dir() {
        EntryKind::Directory
    } else {
        EntryKind::File
    };

    Ok(FileStamp {
        size: metadata.len(),
        modified_time: metadata.modified().unwrap_or(UNIX_EPOCH),
        kind,
        flags,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn stamps_a_temporary_file_and_folder() {
        let directory =
            std::env::temp_dir().join(format!("vsm-file-stamp-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("file.bin"), b"12345").unwrap();

        let file_stamp = read_file_stamp(&directory.join("file.bin"));
        let folder_stamp = read_file_stamp(&directory);
        std::fs::remove_dir_all(&directory).unwrap();

        let file_stamp = file_stamp.unwrap();
        assert_eq!(file_stamp.size, 5);
        assert_eq!(file_stamp.kind, EntryKind::File);
        assert_eq!(file_stamp.flags, EntryFlags::empty());
        assert_eq!(folder_stamp.unwrap().kind, EntryKind::Directory);
    }
}
