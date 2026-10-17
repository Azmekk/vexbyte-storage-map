use std::ffi::OsString;
use std::io;
use std::mem::{offset_of, size_of};
use std::os::windows::ffi::OsStringExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;

use windows::Win32::Foundation::{
    ERROR_INVALID_FUNCTION, ERROR_INVALID_LEVEL, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_FILES,
    ERROR_NOT_SUPPORTED, HANDLE,
};
use windows::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_ATTRIBUTE_COMPRESSED, FILE_ATTRIBUTE_DIRECTORY,
    FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_OFFLINE, FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS,
    FILE_ATTRIBUTE_RECALL_ON_OPEN, FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_SPARSE_FILE,
    FILE_ATTRIBUTE_SYSTEM, FILE_FLAG_BACKUP_SEMANTICS, FILE_ID_BOTH_DIR_INFO,
    FILE_ID_EXTD_DIR_INFO, FILE_INFO_BY_HANDLE_CLASS, FILE_LIST_DIRECTORY, FILE_SHARE_DELETE,
    FILE_SHARE_READ, FILE_SHARE_WRITE, FileIdBothDirectoryInfo, FileIdBothDirectoryRestartInfo,
    FileIdExtdDirectoryInfo, GetFileInformationByHandle, GetFileInformationByHandleEx,
    OPEN_EXISTING,
};
use windows::core::PCWSTR;

use super::file_time::system_time_from_file_time;
use super::io_error::to_io_error;
use super::wide_path::to_extended_length_wide_path;
use crate::directory_entry::{DirectoryEntry, EntryFlags, EntryKind, FileIdentity};

/// 64 KiB is the largest buffer the SMB redirector accepts for directory queries.
const LISTING_BUFFER_BYTES: usize = 64 * 1024;

const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
/// Bit 29 of a reparse tag marks name surrogates: symlinks, junctions and similar links that
/// stand for another path. Other tags (deduplication, compression, hydrated cloud files) keep
/// their data locally and read like ordinary files.
const REPARSE_TAG_NAME_SURROGATE_BIT: u32 = 0x2000_0000;

const CLOUD_PLACEHOLDER_ATTRIBUTES: u32 = FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS.0
    | FILE_ATTRIBUTE_RECALL_ON_OPEN.0
    | FILE_ATTRIBUTE_OFFLINE.0;

/// Lists one directory in bulk without opening any of the entries, so cloud placeholders are
/// never touched. `.` and `..` are left out.
pub(crate) fn list_directory(directory: &Path) -> io::Result<Vec<DirectoryEntry>> {
    let directory_handle = open_directory_for_listing(directory)?;
    let volume_serial_number = query_volume_serial_number(&directory_handle)?;

    match read_all_entries(
        &directory_handle,
        ListingFormat::Extended,
        volume_serial_number,
    ) {
        Err(error) if is_unsupported_information_class(&error) => read_all_entries(
            &directory_handle,
            ListingFormat::Basic,
            volume_serial_number,
        ),
        result => result,
    }
}

#[derive(Clone, Copy)]
enum ListingFormat {
    /// `FILE_ID_EXTD_DIR_INFO`, with 128-bit file ids (required for `ReFS`) and the reparse tag.
    Extended,
    /// `FILE_ID_BOTH_DIR_INFO`, supported by older and network file systems.
    Basic,
}

fn open_directory_for_listing(directory: &Path) -> io::Result<OwnedHandle> {
    let wide_path = to_extended_length_wide_path(directory)?;

    // SAFETY: `wide_path` is null-terminated and outlives the call. The handle is opened with
    // list-only access (never write) and full sharing; ownership passes to `OwnedHandle`.
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }
    .map_err(|error| to_io_error(&error))?;

    // SAFETY: `handle` was just returned by CreateFileW, is valid and is owned by nobody else.
    Ok(unsafe { OwnedHandle::from_raw_handle(handle.0) })
}

fn query_volume_serial_number(directory_handle: &OwnedHandle) -> io::Result<u64> {
    let mut information = BY_HANDLE_FILE_INFORMATION::default();

    // SAFETY: the handle is open for the duration of the call and `information` is a writable
    // BY_HANDLE_FILE_INFORMATION.
    unsafe {
        GetFileInformationByHandle(
            HANDLE(directory_handle.as_raw_handle()),
            &raw mut information,
        )
    }
    .map_err(|error| to_io_error(&error))?;

    Ok(u64::from(information.dwVolumeSerialNumber))
}

fn read_all_entries(
    directory_handle: &OwnedHandle,
    format: ListingFormat,
    volume_serial_number: u64,
) -> io::Result<Vec<DirectoryEntry>> {
    // u64 elements keep the buffer 8-byte aligned, as the directory information structs require.
    let mut buffer = vec![0_u64; LISTING_BUFFER_BYTES / size_of::<u64>()];
    let mut entries = Vec::new();
    let mut is_first_call = true;

    loop {
        let information_class = information_class_for(format, is_first_call);
        is_first_call = false;

        // SAFETY: the handle is open, `buffer` is writable for its full byte length and stays
        // alive for the call.
        let result = unsafe {
            GetFileInformationByHandleEx(
                HANDLE(directory_handle.as_raw_handle()),
                information_class,
                buffer.as_mut_ptr().cast(),
                u32::try_from(LISTING_BUFFER_BYTES).unwrap_or(u32::MAX),
            )
        };

        match result {
            Ok(()) => {}
            Err(error) if error.code() == ERROR_NO_MORE_FILES.to_hresult() => break,
            Err(error) => return Err(to_io_error(&error)),
        }

        let filled_bytes = bytes_of(&buffer);
        parse_entries_from_buffer(filled_bytes, format, volume_serial_number, &mut entries);
    }

    Ok(entries)
}

fn information_class_for(format: ListingFormat, is_first_call: bool) -> FILE_INFO_BY_HANDLE_CLASS {
    match (format, is_first_call) {
        (ListingFormat::Extended, _) => FileIdExtdDirectoryInfo,
        // The basic listing runs after a failed extended attempt, so it restarts the scan.
        (ListingFormat::Basic, true) => FileIdBothDirectoryRestartInfo,
        (ListingFormat::Basic, false) => FileIdBothDirectoryInfo,
    }
}

fn bytes_of(buffer: &[u64]) -> &[u8] {
    // SAFETY: any u64 slice can be viewed as bytes; the length covers exactly the same memory.
    unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), size_of_val(buffer)) }
}

fn is_unsupported_information_class(error: &io::Error) -> bool {
    let unsupported_codes = [
        ERROR_INVALID_PARAMETER.0,
        ERROR_INVALID_FUNCTION.0,
        ERROR_NOT_SUPPORTED.0,
        ERROR_INVALID_LEVEL.0,
    ];

    error.raw_os_error().is_some_and(|code| {
        unsupported_codes
            .iter()
            .any(|&unsupported| unsupported.cast_signed() == code)
    })
}

/// The fields both directory information layouts share, read out of the raw buffer.
struct RawEntry {
    next_entry_offset: usize,
    attributes: u32,
    creation_time: i64,
    last_write_time: i64,
    end_of_file: i64,
    reparse_tag: u32,
    file_id: u128,
    name: OsString,
}

fn parse_entries_from_buffer(
    buffer: &[u8],
    format: ListingFormat,
    volume_serial_number: u64,
    entries: &mut Vec<DirectoryEntry>,
) {
    let mut entry_offset = 0_usize;

    while let Some(raw_entry) = read_raw_entry(buffer, entry_offset, format) {
        if raw_entry.name != "." && raw_entry.name != ".." {
            entries.push(directory_entry_from_raw(&raw_entry, volume_serial_number));
        }

        if raw_entry.next_entry_offset == 0 {
            break;
        }
        entry_offset += raw_entry.next_entry_offset;
    }
}

fn read_raw_entry(buffer: &[u8], entry_offset: usize, format: ListingFormat) -> Option<RawEntry> {
    match format {
        ListingFormat::Extended => read_extended_entry(buffer, entry_offset),
        ListingFormat::Basic => read_basic_entry(buffer, entry_offset),
    }
}

fn read_extended_entry(buffer: &[u8], entry_offset: usize) -> Option<RawEntry> {
    let name_offset = offset_of!(FILE_ID_EXTD_DIR_INFO, FileName);
    if entry_offset.checked_add(name_offset)? > buffer.len() {
        return None;
    }

    // SAFETY: the bounds check above guarantees the fixed-size header lies inside `buffer`;
    // `read_unaligned` copies it out without requiring alignment.
    let information: FILE_ID_EXTD_DIR_INFO = unsafe {
        std::ptr::read_unaligned(
            buffer
                .as_ptr()
                .add(entry_offset)
                .cast::<FILE_ID_EXTD_DIR_INFO>(),
        )
    };

    let name = read_entry_name(
        buffer,
        entry_offset + name_offset,
        information.FileNameLength,
    )?;

    Some(RawEntry {
        next_entry_offset: information.NextEntryOffset as usize,
        attributes: information.FileAttributes,
        creation_time: information.CreationTime,
        last_write_time: information.LastWriteTime,
        end_of_file: information.EndOfFile,
        reparse_tag: information.ReparsePointTag,
        file_id: u128::from_le_bytes(information.FileId.Identifier),
        name,
    })
}

fn read_basic_entry(buffer: &[u8], entry_offset: usize) -> Option<RawEntry> {
    let name_offset = offset_of!(FILE_ID_BOTH_DIR_INFO, FileName);
    if entry_offset.checked_add(name_offset)? > buffer.len() {
        return None;
    }

    // SAFETY: the bounds check above guarantees the fixed-size header lies inside `buffer`;
    // `read_unaligned` copies it out without requiring alignment.
    let information: FILE_ID_BOTH_DIR_INFO = unsafe {
        std::ptr::read_unaligned(
            buffer
                .as_ptr()
                .add(entry_offset)
                .cast::<FILE_ID_BOTH_DIR_INFO>(),
        )
    };

    let name = read_entry_name(
        buffer,
        entry_offset + name_offset,
        information.FileNameLength,
    )?;

    Some(RawEntry {
        next_entry_offset: information.NextEntryOffset as usize,
        attributes: information.FileAttributes,
        creation_time: information.CreationTime,
        last_write_time: information.LastWriteTime,
        end_of_file: information.EndOfFile,
        // For reparse points, FILE_ID_BOTH_DIR_INFO reports the reparse tag in the EaSize field.
        reparse_tag: information.EaSize,
        file_id: u128::from(information.FileId.cast_unsigned()),
        name,
    })
}

fn read_entry_name(buffer: &[u8], name_start: usize, name_length_bytes: u32) -> Option<OsString> {
    let name_end = name_start.checked_add(name_length_bytes as usize)?;
    let name_bytes = buffer.get(name_start..name_end)?;

    let name_units: Vec<u16> = name_bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| u16::from_le_bytes(pair))
        .collect();

    Some(OsString::from_wide(&name_units))
}

fn directory_entry_from_raw(raw_entry: &RawEntry, volume_serial_number: u64) -> DirectoryEntry {
    let kind = if raw_entry.attributes & FILE_ATTRIBUTE_DIRECTORY.0 == 0 {
        EntryKind::File
    } else {
        EntryKind::Directory
    };

    let identity = (raw_entry.file_id != 0)
        .then(|| FileIdentity::new(volume_serial_number, raw_entry.file_id));

    DirectoryEntry {
        name: raw_entry.name.clone(),
        kind,
        logical_size: raw_entry.end_of_file.max(0).cast_unsigned(),
        modified_time: system_time_from_file_time(raw_entry.last_write_time),
        created_time: system_time_from_file_time(raw_entry.creation_time),
        identity,
        flags: entry_flags_from_attributes(raw_entry.attributes, raw_entry.reparse_tag),
    }
}

pub(super) fn entry_flags_from_attributes(attributes: u32, reparse_tag: u32) -> EntryFlags {
    let mut flags = EntryFlags::empty();

    let is_reparse_point = attributes & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0;
    if is_reparse_point && reparse_tag & REPARSE_TAG_NAME_SURROGATE_BIT != 0 {
        if reparse_tag == IO_REPARSE_TAG_MOUNT_POINT {
            flags |= EntryFlags::JUNCTION;
        } else {
            flags |= EntryFlags::SYMLINK;
        }
    }

    let attribute_flags = [
        (CLOUD_PLACEHOLDER_ATTRIBUTES, EntryFlags::CLOUD_PLACEHOLDER),
        (FILE_ATTRIBUTE_HIDDEN.0, EntryFlags::HIDDEN),
        (FILE_ATTRIBUTE_SYSTEM.0, EntryFlags::SYSTEM),
        (FILE_ATTRIBUTE_COMPRESSED.0, EntryFlags::COMPRESSED),
        (FILE_ATTRIBUTE_SPARSE_FILE.0, EntryFlags::SPARSE),
    ];
    for (attribute_mask, flag) in attribute_flags {
        if attributes & attribute_mask != 0 {
            flags |= flag;
        }
    }

    flags
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn recognizes_cloud_placeholders_by_any_recall_attribute() {
        for attribute in [
            FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS.0,
            FILE_ATTRIBUTE_RECALL_ON_OPEN.0,
            FILE_ATTRIBUTE_OFFLINE.0,
        ] {
            assert!(
                entry_flags_from_attributes(attribute, 0).contains(EntryFlags::CLOUD_PLACEHOLDER)
            );
        }
    }

    #[test]
    fn treats_name_surrogate_reparse_points_as_links() {
        let reparse = FILE_ATTRIBUTE_REPARSE_POINT.0;

        assert_eq!(
            entry_flags_from_attributes(reparse, IO_REPARSE_TAG_MOUNT_POINT),
            EntryFlags::JUNCTION
        );
        assert_eq!(
            entry_flags_from_attributes(reparse, 0xA000_000C),
            EntryFlags::SYMLINK
        );
    }

    #[test]
    fn treats_hydrated_cloud_files_as_ordinary_files() {
        let hydrated_onedrive_file_tag = 0x9000_601A;

        let flags =
            entry_flags_from_attributes(FILE_ATTRIBUTE_REPARSE_POINT.0, hydrated_onedrive_file_tag);

        assert_eq!(flags, EntryFlags::empty());
    }

    #[test]
    fn lists_a_temporary_directory_with_identities() {
        let directory =
            std::env::temp_dir().join(format!("vsm-directory-listing-test-{}", std::process::id()));
        std::fs::create_dir_all(directory.join("nested")).unwrap();
        std::fs::write(directory.join("file.txt"), b"hello").unwrap();

        let entries = list_directory(&directory);
        std::fs::remove_dir_all(&directory).unwrap();
        let mut entries = entries.unwrap();
        entries.sort_by(|left, right| left.name.cmp(&right.name));

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "file.txt");
        assert_eq!(entries[0].kind, EntryKind::File);
        assert_eq!(entries[0].logical_size, 5);
        assert!(entries[0].identity.is_some());
        assert_eq!(entries[1].name, "nested");
        assert_eq!(entries[1].kind, EntryKind::Directory);
    }
}
