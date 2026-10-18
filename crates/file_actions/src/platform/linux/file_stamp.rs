use std::io;
use std::path::Path;

use crate::file_stamp::FileStamp;

/// Reads size, modification time and kind from metadata without opening the file. Symbolic links
/// are described themselves, not their targets. Linux has no cloud placeholders.
pub(crate) fn read_file_stamp(path: &Path) -> io::Result<FileStamp> {
    let metadata = std::fs::symlink_metadata(path)?;

    Ok(FileStamp {
        size: metadata.len(),
        modified_time: metadata.modified()?,
        is_directory: metadata.is_dir(),
        is_cloud_placeholder: false,
    })
}

/// True when anything exists at `path`, a dangling symbolic link included.
pub(crate) fn path_exists(path: &Path) -> bool {
    read_file_stamp(path).is_ok()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn reads_size_and_time_of_a_temporary_file() {
        let path = std::env::temp_dir().join(format!("vsm-file-stamp-{}.txt", std::process::id()));
        std::fs::write(&path, b"twelve bytes").unwrap();
        let expected_modified_time = std::fs::metadata(&path).unwrap().modified().unwrap();

        let stamp = read_file_stamp(&path);
        let exists = path_exists(&path);
        std::fs::remove_file(&path).unwrap();
        let stamp = stamp.unwrap();

        assert!(exists);
        assert!(!path_exists(&path));
        assert_eq!(stamp.size, 12);
        assert_eq!(stamp.modified_time, expected_modified_time);
        assert!(!stamp.is_directory);
        assert!(!stamp.is_cloud_placeholder);
    }
}
