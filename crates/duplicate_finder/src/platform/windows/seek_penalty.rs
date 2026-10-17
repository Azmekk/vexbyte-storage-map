use std::ffi::{OsStr, OsString};
use std::io;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
    GetVolumeNameForVolumeMountPointW, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;
use windows::Win32::System::Ioctl::{
    DEVICE_SEEK_PENALTY_DESCRIPTOR, IOCTL_STORAGE_QUERY_PROPERTY, PropertyStandardQuery,
    STORAGE_PROPERTY_QUERY, StorageDeviceSeekPenaltyProperty,
};
use windows::core::PCWSTR;

use super::io_error::to_io_error;
use super::wide_path::to_null_terminated_wide_string;

/// `\\?\Volume{GUID}\` plus the terminating null, with room to spare.
const VOLUME_NAME_BUFFER_CHARACTERS: usize = 64;
const BACKSLASH: u16 = b'\\' as u16;

/// Asks the storage driver whether the disk behind the volume mounted at `mount_point` pays a
/// seek penalty, which is true for spinning hard disks and false for SSDs.
pub(crate) fn query_volume_incurs_seek_penalty(mount_point: &Path) -> io::Result<bool> {
    let volume_device_path = query_volume_device_path(mount_point)?;
    let volume_handle = open_volume_for_queries(&volume_device_path)?;

    let query = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceSeekPenaltyProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0],
    };
    let mut descriptor = DEVICE_SEEK_PENALTY_DESCRIPTOR::default();
    let mut bytes_returned = 0_u32;

    // SAFETY: the handle is open; the input and output pointers reference correctly sized,
    // live structs whose sizes are passed alongside them; no OVERLAPPED is used.
    unsafe {
        DeviceIoControl(
            HANDLE(volume_handle.as_raw_handle()),
            IOCTL_STORAGE_QUERY_PROPERTY,
            Some((&raw const query).cast()),
            struct_size::<STORAGE_PROPERTY_QUERY>(),
            Some((&raw mut descriptor).cast()),
            struct_size::<DEVICE_SEEK_PENALTY_DESCRIPTOR>(),
            Some(&raw mut bytes_returned),
            None,
        )
    }
    .map_err(|error| to_io_error(&error))?;

    if (bytes_returned as usize) < size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the storage driver returned an incomplete seek penalty descriptor",
        ));
    }

    Ok(descriptor.IncursSeekPenalty)
}

fn struct_size<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

/// Resolves a mount point such as `C:\` to its volume device path `\\?\Volume{GUID}`.
fn query_volume_device_path(mount_point: &Path) -> io::Result<OsString> {
    let mut wide_mount_point = to_null_terminated_wide_string(mount_point.as_os_str());
    wide_mount_point.pop();
    // GetVolumeNameForVolumeMountPointW requires the trailing backslash.
    if wide_mount_point.last() != Some(&BACKSLASH) {
        wide_mount_point.push(BACKSLASH);
    }
    wide_mount_point.push(0);

    let mut volume_name_buffer = [0_u16; VOLUME_NAME_BUFFER_CHARACTERS];

    // SAFETY: `wide_mount_point` is null-terminated and outlives the call; the output buffer is
    // a writable slice whose length the wrapper passes to Win32.
    unsafe {
        GetVolumeNameForVolumeMountPointW(
            PCWSTR(wide_mount_point.as_ptr()),
            &mut volume_name_buffer,
        )
    }
    .map_err(|error| to_io_error(&error))?;

    let length = volume_name_buffer
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(volume_name_buffer.len());
    let mut volume_name = &volume_name_buffer[..length];
    // The volume device is opened without the trailing backslash; with it, the root directory
    // of the file system would be opened instead.
    if volume_name.last() == Some(&BACKSLASH) {
        volume_name = &volume_name[..volume_name.len() - 1];
    }

    Ok(OsString::from_wide(volume_name))
}

fn open_volume_for_queries(volume_device_path: &OsStr) -> io::Result<OwnedHandle> {
    let wide_device_path = to_null_terminated_wide_string(volume_device_path);

    // SAFETY: `wide_device_path` is null-terminated and outlives the call. Desired access 0
    // grants neither read nor write: only metadata queries such as this IOCTL, which also work
    // without administrator rights. Ownership passes to `OwnedHandle`.
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_device_path.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }
    .map_err(|error| to_io_error(&error))?;

    // SAFETY: `handle` was just returned by CreateFileW, is valid and is owned by nobody else.
    Ok(unsafe { OwnedHandle::from_raw_handle(handle.0) })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::super::volume_mount_point::query_volume_mount_point;
    use super::*;

    #[test]
    fn answers_for_the_volume_holding_the_temp_directory() {
        let mount_point = query_volume_mount_point(&std::env::temp_dir()).unwrap();

        let incurs_seek_penalty = query_volume_incurs_seek_penalty(&mount_point);

        // Virtual disks (for example on hosted CI runners) refuse the storage property query
        // with ERROR_GEN_FAILURE (31). The caller treats any error as an unknown drive kind, so
        // only a crash would be a failure here.
        match incurs_seek_penalty {
            Ok(_) => {}
            Err(error) if error.raw_os_error() == Some(31) => {
                eprintln!("seek penalty query unsupported on this disk: {error}");
            }
            Err(error) => panic!("unexpected seek penalty error: {error:?}"),
        }
    }
}
