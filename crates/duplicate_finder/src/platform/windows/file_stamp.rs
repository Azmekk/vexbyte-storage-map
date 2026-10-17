use std::io;
use std::path::Path;

use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DIRECTORY, GetFileAttributesExW, GetFileExInfoStandard,
    WIN32_FILE_ATTRIBUTE_DATA,
};
use windows::core::PCWSTR;

use super::directory_listing::entry_flags_from_attributes;
use super::file_time::system_time_from_file_time_parts;
use super::io_error::to_io_error;
use super::wide_path::to_extended_length_wide_path;
use crate::directory_entry::{EntryKind, FileStamp};

/// Reads size, modification time and attributes of `path` from the file system metadata, without
/// opening the file, so it is safe to call on cloud placeholders.
pub(crate) fn read_file_stamp(path: &Path) -> io::Result<FileStamp> {
    let wide_path = to_extended_length_wide_path(path)?;
    let mut attribute_data = WIN32_FILE_ATTRIBUTE_DATA::default();

    // SAFETY: `wide_path` is null-terminated and outlives the call; `attribute_data` is the
    // WIN32_FILE_ATTRIBUTE_DATA that GetFileExInfoStandard requires and is writable.
    unsafe {
        GetFileAttributesExW(
            PCWSTR(wide_path.as_ptr()),
            GetFileExInfoStandard,
            (&raw mut attribute_data).cast(),
        )
    }
    .map_err(|error| to_io_error(&error))?;

    let size =
        (u64::from(attribute_data.nFileSizeHigh) << 32) | u64::from(attribute_data.nFileSizeLow);
    let kind = if attribute_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 == 0 {
        EntryKind::File
    } else {
        EntryKind::Directory
    };
    let modified_time = system_time_from_file_time_parts(
        attribute_data.ftLastWriteTime.dwHighDateTime,
        attribute_data.ftLastWriteTime.dwLowDateTime,
    );

    Ok(FileStamp {
        size,
        modified_time,
        kind,
        // GetFileAttributesExW does not report the reparse tag, so links are not flagged here.
        flags: entry_flags_from_attributes(attribute_data.dwFileAttributes, 0),
    })
}
