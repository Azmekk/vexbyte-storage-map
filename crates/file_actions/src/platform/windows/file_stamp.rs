use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use windows::Win32::Foundation::FILETIME;
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_OFFLINE, FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS,
    FILE_ATTRIBUTE_RECALL_ON_OPEN, GetFileAttributesExW, GetFileExInfoStandard,
    WIN32_FILE_ATTRIBUTE_DATA,
};
use windows::core::PCWSTR;

use super::io_error::to_io_error;
use super::wide_path::to_extended_length_wide_path;
use crate::file_stamp::FileStamp;

const CLOUD_PLACEHOLDER_ATTRIBUTES: u32 = FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS.0
    | FILE_ATTRIBUTE_RECALL_ON_OPEN.0
    | FILE_ATTRIBUTE_OFFLINE.0;

/// 100-nanosecond intervals between 1601-01-01 (the FILETIME epoch) and 1970-01-01.
const FILE_TIME_INTERVALS_BEFORE_UNIX_EPOCH: u64 = 116_444_736_000_000_000;

/// Reads size, modification time and attributes from file system metadata without opening the
/// file, so it never triggers a cloud download.
pub(crate) fn read_file_stamp(path: &Path) -> io::Result<FileStamp> {
    let wide_path = to_extended_length_wide_path(path)?;
    let mut attribute_data = WIN32_FILE_ATTRIBUTE_DATA::default();

    // SAFETY: `wide_path` is null-terminated and outlives the call; `attribute_data` is the
    // writable WIN32_FILE_ATTRIBUTE_DATA that GetFileExInfoStandard requires.
    unsafe {
        GetFileAttributesExW(
            PCWSTR(wide_path.as_ptr()),
            GetFileExInfoStandard,
            (&raw mut attribute_data).cast(),
        )
    }
    .map_err(|error| to_io_error(&error))?;

    let attributes = attribute_data.dwFileAttributes;
    Ok(FileStamp {
        size: (u64::from(attribute_data.nFileSizeHigh) << 32)
            | u64::from(attribute_data.nFileSizeLow),
        modified_time: system_time_from_file_time(attribute_data.ftLastWriteTime),
        is_directory: attributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0,
        is_cloud_placeholder: attributes & CLOUD_PLACEHOLDER_ATTRIBUTES != 0,
    })
}

/// True when anything exists at `path`, checked without opening it.
pub(crate) fn path_exists(path: &Path) -> bool {
    read_file_stamp(path).is_ok()
}

fn system_time_from_file_time(file_time: FILETIME) -> SystemTime {
    let intervals =
        (u64::from(file_time.dwHighDateTime) << 32) | u64::from(file_time.dwLowDateTime);

    match intervals.checked_sub(FILE_TIME_INTERVALS_BEFORE_UNIX_EPOCH) {
        Some(intervals_since_unix_epoch) => {
            UNIX_EPOCH + Duration::from_nanos(intervals_since_unix_epoch.saturating_mul(100))
        }
        None => UNIX_EPOCH,
    }
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
