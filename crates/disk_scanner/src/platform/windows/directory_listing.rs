//! Lists a directory in bulk with `GetFileInformationByHandleEx(FileIdBothDirectoryInfo)`.
//!
//! One call returns tens of entries (names, sizes, attributes, timestamps and 64-bit file IDs)
//! without opening any of them, which is both fast and essential for cloud placeholders: they
//! must never be opened. `FileIdBothDirectoryInfo` is used instead of `FileIdExtdDirectoryInfo`
//! because it is supported by every local and network file system (NTFS, `ReFS`, FAT32,
//! `exFAT`, SMB), while the extended class fails on FAT and some network redirectors. Its 64-bit file
//! IDs are unique on NTFS and FAT; `ReFS` reports `-1` for IDs that do not fit, and those entries
//! simply get no identity for hard link tracking.

use std::io;
use std::mem::offset_of;
use std::path::Path;

use ::windows::Win32::Foundation::ERROR_NO_MORE_FILES;
use ::windows::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_ATTRIBUTE_DIRECTORY,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_ID_BOTH_DIR_INFO, FILE_LIST_DIRECTORY, FILE_SHARE_DELETE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, FileIdBothDirectoryInfo, GetFileInformationByHandle,
    GetFileInformationByHandleEx, OPEN_EXISTING,
};
use ::windows::core::PCWSTR;

use super::file_attributes::entry_flags_from_attributes;
use super::file_time::filetime_to_system_time;
use super::io_error::to_io_error;
use super::owned_handle::OwnedHandle;
use super::path_encoding::to_extended_length_wide_path;
use crate::file_identity::FileIdentity;
use crate::file_tree::NodeKind;
use crate::folder_walk::{DirectoryEntry, DirectoryListError};

/// 64 KiB holds several hundred entries per call; larger buffers do not reduce the call count
/// noticeably because the file system fills at most one internal directory block per call.
const LISTING_BUFFER_SIZE_IN_WORDS: usize = 8 * 1024;

/// Every directory listing entry starts on an 8-byte boundary.
type ListingBuffer = Vec<u64>;

/// Lists every entry of `directory_path` except `.` and `..`, calling `on_entry` for each.
///
/// The directory is opened with list access only and without following reparse points.
pub(crate) fn list_directory(
    directory_path: &Path,
    on_entry: &mut dyn FnMut(DirectoryEntry<'_>),
) -> Result<(), DirectoryListError> {
    let directory_handle = open_directory_for_listing(directory_path).map_err(|source| {
        DirectoryListError::OpenFailed {
            path: directory_path.to_path_buf(),
            source,
        }
    })?;
    let volume_id = query_volume_serial_number(&directory_handle);

    let mut listing_buffer: ListingBuffer = vec![0_u64; LISTING_BUFFER_SIZE_IN_WORDS];
    let mut decoded_name = String::new();

    loop {
        let has_more_entries = fill_listing_buffer(&directory_handle, &mut listing_buffer)
            .map_err(|source| DirectoryListError::ReadFailed {
                path: directory_path.to_path_buf(),
                source,
            })?;
        if !has_more_entries {
            return Ok(());
        }

        let listing_bytes = view_as_bytes(&listing_buffer);
        for raw_entry in RawDirectoryEntries::new(listing_bytes) {
            decode_utf16_le_into(raw_entry.name_utf16_le, &mut decoded_name);
            if decoded_name == "." || decoded_name == ".." {
                continue;
            }
            on_entry(raw_entry.to_directory_entry(&decoded_name, volume_id));
        }
    }
}

fn open_directory_for_listing(directory_path: &Path) -> io::Result<OwnedHandle> {
    let wide_path = to_extended_length_wide_path(directory_path);

    // SAFETY: `wide_path` is null-terminated and outlives the call. The handle is opened with
    // FILE_LIST_DIRECTORY only (CreateFileW adds SYNCHRONIZE and FILE_READ_ATTRIBUTES), never
    // with write access. FILE_FLAG_OPEN_REPARSE_POINT opens a junction or symlink itself instead
    // of its target; FILE_FLAG_BACKUP_SEMANTICS is required to open a directory at all.
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            None,
        )
    }
    .map_err(|error| to_io_error(&error))?;

    Ok(OwnedHandle::from_valid_handle(handle))
}

/// The serial number of the volume the directory is on; 0 when it cannot be read.
fn query_volume_serial_number(directory_handle: &OwnedHandle) -> u64 {
    let mut information = BY_HANDLE_FILE_INFORMATION::default();

    // SAFETY: the handle is an open directory handle and `information` is a writable struct of
    // the type the function expects.
    let result =
        unsafe { GetFileInformationByHandle(directory_handle.raw(), &raw mut information) };

    if result.is_ok() {
        u64::from(information.dwVolumeSerialNumber)
    } else {
        0
    }
}

/// Fills the buffer with the next block of entries. Returns false once the listing is complete.
fn fill_listing_buffer(
    directory_handle: &OwnedHandle,
    listing_buffer: &mut ListingBuffer,
) -> io::Result<bool> {
    let buffer_size_in_bytes = u32::try_from(listing_buffer.len() * size_of::<u64>())
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;

    // SAFETY: the buffer is a writable, 8-byte aligned allocation of exactly
    // `buffer_size_in_bytes` bytes that lives for the whole call; the handle is an open directory
    // handle with list access.
    let result = unsafe {
        GetFileInformationByHandleEx(
            directory_handle.raw(),
            FileIdBothDirectoryInfo,
            listing_buffer.as_mut_ptr().cast(),
            buffer_size_in_bytes,
        )
    };

    match result {
        Ok(()) => Ok(true),
        Err(error) if error.code() == ERROR_NO_MORE_FILES.to_hresult() => Ok(false),
        Err(error) => Err(to_io_error(&error)),
    }
}

fn view_as_bytes(listing_buffer: &ListingBuffer) -> &[u8] {
    // SAFETY: a u64 slice is valid to read as bytes: the pointer is non-null and aligned, every
    // byte is initialized, the length covers exactly the allocation and the borrow keeps the
    // buffer alive and unmodified for the returned lifetime.
    unsafe {
        std::slice::from_raw_parts(
            listing_buffer.as_ptr().cast::<u8>(),
            listing_buffer.len() * size_of::<u64>(),
        )
    }
}

fn decode_utf16_le_into(name_utf16_le: &[u8], decoded_name: &mut String) {
    decoded_name.clear();
    let (code_unit_bytes, _) = name_utf16_le.as_chunks::<2>();
    let code_units = code_unit_bytes.iter().map(|pair| u16::from_le_bytes(*pair));

    // NTFS allows unpaired surrogates in names; they are shown with a replacement character.
    decoded_name.extend(
        char::decode_utf16(code_units)
            .map(|character| character.unwrap_or(char::REPLACEMENT_CHARACTER)),
    );
}

/// One `FILE_ID_BOTH_DIR_INFO` record, read field by field from the byte buffer.
struct RawDirectoryEntry<'buffer> {
    last_write_time: i64,
    end_of_file: i64,
    allocation_size: i64,
    file_attributes: u32,
    /// For reparse points, the `EaSize` field holds the reparse tag instead.
    extended_attributes_size_or_reparse_tag: u32,
    file_id: i64,
    name_utf16_le: &'buffer [u8],
}

impl RawDirectoryEntry<'_> {
    fn to_directory_entry<'name>(
        &self,
        decoded_name: &'name str,
        volume_id: u64,
    ) -> DirectoryEntry<'name> {
        let is_directory = self.file_attributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0;
        let reparse_tag = if self.file_attributes & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
            self.extended_attributes_size_or_reparse_tag
        } else {
            0
        };
        let flags = entry_flags_from_attributes(self.file_attributes, reparse_tag, is_directory);

        let (kind, logical_size, allocated_size) = if is_directory {
            (NodeKind::Directory, 0, 0)
        } else {
            (
                NodeKind::File,
                self.end_of_file.max(0).cast_unsigned(),
                self.allocation_size.max(0).cast_unsigned(),
            )
        };

        let file_id = self.file_id.cast_unsigned();
        let identity = (file_id != 0 && file_id != u64::MAX)
            .then(|| FileIdentity::from_volume_and_file_ids(volume_id, u128::from(file_id)));

        DirectoryEntry {
            name: decoded_name,
            kind,
            logical_size,
            allocated_size,
            modified_time: filetime_to_system_time(self.last_write_time),
            identity,
            flags,
        }
    }
}

/// Iterates the records of a filled listing buffer by following `NextEntryOffset`.
struct RawDirectoryEntries<'buffer> {
    remaining_bytes: Option<&'buffer [u8]>,
}

impl<'buffer> RawDirectoryEntries<'buffer> {
    fn new(listing_bytes: &'buffer [u8]) -> Self {
        Self {
            remaining_bytes: Some(listing_bytes),
        }
    }
}

impl<'buffer> Iterator for RawDirectoryEntries<'buffer> {
    type Item = RawDirectoryEntry<'buffer>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry_bytes = self.remaining_bytes?;
        let entry = parse_raw_entry(entry_bytes);

        let next_entry_offset = read_u32(
            entry_bytes,
            offset_of!(FILE_ID_BOTH_DIR_INFO, NextEntryOffset),
        )
        .map_or(0, |offset| offset as usize);
        self.remaining_bytes = if next_entry_offset == 0 || entry.is_none() {
            None
        } else {
            entry_bytes.get(next_entry_offset..)
        };

        entry
    }
}

fn parse_raw_entry(entry_bytes: &[u8]) -> Option<RawDirectoryEntry<'_>> {
    let name_length_in_bytes = read_u32(
        entry_bytes,
        offset_of!(FILE_ID_BOTH_DIR_INFO, FileNameLength),
    )? as usize;
    let name_start = offset_of!(FILE_ID_BOTH_DIR_INFO, FileName);
    let name_utf16_le = entry_bytes.get(name_start..name_start + name_length_in_bytes)?;

    Some(RawDirectoryEntry {
        last_write_time: read_i64(
            entry_bytes,
            offset_of!(FILE_ID_BOTH_DIR_INFO, LastWriteTime),
        )?,
        end_of_file: read_i64(entry_bytes, offset_of!(FILE_ID_BOTH_DIR_INFO, EndOfFile))?,
        allocation_size: read_i64(
            entry_bytes,
            offset_of!(FILE_ID_BOTH_DIR_INFO, AllocationSize),
        )?,
        file_attributes: read_u32(
            entry_bytes,
            offset_of!(FILE_ID_BOTH_DIR_INFO, FileAttributes),
        )?,
        extended_attributes_size_or_reparse_tag: read_u32(
            entry_bytes,
            offset_of!(FILE_ID_BOTH_DIR_INFO, EaSize),
        )?,
        file_id: read_i64(entry_bytes, offset_of!(FILE_ID_BOTH_DIR_INFO, FileId))?,
        name_utf16_le,
    })
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let field_bytes = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(field_bytes.try_into().ok()?))
}

fn read_i64(bytes: &[u8], offset: usize) -> Option<i64> {
    let field_bytes = bytes.get(offset..offset + 8)?;
    Some(i64::from_le_bytes(field_bytes.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn write_entry(buffer: &mut [u8], next_entry_offset: u32, name: &str, end_of_file: i64) {
        let name_units: Vec<u16> = name.encode_utf16().collect();
        let name_start = offset_of!(FILE_ID_BOTH_DIR_INFO, FileName);

        buffer[0..4].copy_from_slice(&next_entry_offset.to_le_bytes());
        let size_offset = offset_of!(FILE_ID_BOTH_DIR_INFO, EndOfFile);
        buffer[size_offset..size_offset + 8].copy_from_slice(&end_of_file.to_le_bytes());
        let name_length_offset = offset_of!(FILE_ID_BOTH_DIR_INFO, FileNameLength);
        let name_length = u32::try_from(name_units.len() * 2).unwrap();
        buffer[name_length_offset..name_length_offset + 4]
            .copy_from_slice(&name_length.to_le_bytes());
        for (index, unit) in name_units.iter().enumerate() {
            buffer[name_start + index * 2..name_start + index * 2 + 2]
                .copy_from_slice(&unit.to_le_bytes());
        }
    }

    #[test]
    fn walks_entries_by_next_entry_offset() {
        let mut buffer = vec![0_u8; 512];
        write_entry(&mut buffer[..256], 256, "first.txt", 10);
        write_entry(&mut buffer[256..], 0, "zweite-Datei.bin", 20);

        let entries: Vec<_> = RawDirectoryEntries::new(&buffer).collect();
        let mut names = Vec::new();
        for entry in &entries {
            let mut decoded_name = String::new();
            decode_utf16_le_into(entry.name_utf16_le, &mut decoded_name);
            names.push(decoded_name);
        }

        assert_eq!(names, ["first.txt", "zweite-Datei.bin"]);
        assert_eq!(entries[0].end_of_file, 10);
        assert_eq!(entries[1].end_of_file, 20);
    }

    #[test]
    fn stops_on_a_truncated_entry() {
        let mut buffer = vec![0_u8; 200];
        write_entry(&mut buffer, 0, "name", 1);
        let name_length_offset = offset_of!(FILE_ID_BOTH_DIR_INFO, FileNameLength);
        buffer[name_length_offset..name_length_offset + 4].copy_from_slice(&5000_u32.to_le_bytes());

        assert_eq!(RawDirectoryEntries::new(&buffer).count(), 0);
    }
}
