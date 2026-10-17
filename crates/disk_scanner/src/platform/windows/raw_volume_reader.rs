use std::io;
use std::path::Path;

use ::windows::Win32::Foundation::GENERIC_READ;
use ::windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE,
    GetVolumeNameForVolumeMountPointW, OPEN_EXISTING, ReadFile,
};
use ::windows::Win32::System::IO::{DeviceIoControl, OVERLAPPED, OVERLAPPED_0, OVERLAPPED_0_0};
use ::windows::Win32::System::Ioctl::{FSCTL_GET_NTFS_VOLUME_DATA, NTFS_VOLUME_DATA_BUFFER};
use ::windows::core::PCWSTR;

use super::io_error::to_io_error;
use super::owned_handle::OwnedHandle;
use super::path_encoding::to_null_terminated_wide_path;
use super::wide_string::{from_null_terminated_wide_buffer, to_null_terminated_wide_string};
use crate::mft::NtfsVolumeLayout;

/// The volume GUID path `\\?\Volume{...}\` is at most 49 characters plus the terminator.
const VOLUME_GUID_PATH_CAPACITY: usize = 64;

/// A read-only handle to a whole volume, used to read the Master File Table directly.
///
/// Opening a volume needs administrator rights. The handle is opened with `GENERIC_READ` only
/// and shares read and write access, so the running system keeps using the volume normally.
/// Nothing in this type can write to the volume.
pub(crate) struct RawVolumeReader {
    volume_handle: OwnedHandle,
}

impl RawVolumeReader {
    pub(crate) fn open_read_only(mount_point: &Path) -> io::Result<Self> {
        let volume_device_path = volume_device_path_for_mount_point(mount_point)?;
        let wide_device_path = to_null_terminated_wide_string(&volume_device_path);

        // SAFETY: `wide_device_path` is null-terminated and outlives the call. The access mask is
        // GENERIC_READ only; no write access is ever requested.
        let handle = unsafe {
            CreateFileW(
                PCWSTR(wide_device_path.as_ptr()),
                GENERIC_READ.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
        }
        .map_err(|error| to_io_error(&error))?;

        Ok(Self {
            volume_handle: OwnedHandle::from_valid_handle(handle),
        })
    }

    /// Asks NTFS for its geometry and the location of the Master File Table.
    pub(crate) fn read_ntfs_volume_layout(&self) -> io::Result<NtfsVolumeLayout> {
        let mut volume_data = NTFS_VOLUME_DATA_BUFFER::default();
        let mut bytes_returned = 0_u32;
        let output_size = u32::try_from(size_of::<NTFS_VOLUME_DATA_BUFFER>())
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;

        // SAFETY: FSCTL_GET_NTFS_VOLUME_DATA takes no input and writes one
        // NTFS_VOLUME_DATA_BUFFER; the output pointer and size describe exactly that writable
        // struct, and the call is synchronous (no OVERLAPPED).
        unsafe {
            DeviceIoControl(
                self.volume_handle.raw(),
                FSCTL_GET_NTFS_VOLUME_DATA,
                None,
                0,
                Some((&raw mut volume_data).cast()),
                output_size,
                Some(&raw mut bytes_returned),
                None,
            )
        }
        .map_err(|error| to_io_error(&error))?;

        Ok(NtfsVolumeLayout {
            volume_serial_number: volume_data.VolumeSerialNumber.cast_unsigned(),
            bytes_per_sector: volume_data.BytesPerSector,
            bytes_per_cluster: volume_data.BytesPerCluster,
            bytes_per_file_record: volume_data.BytesPerFileRecordSegment,
            total_clusters: volume_data.TotalClusters.max(0).cast_unsigned(),
            mft_start_cluster: volume_data.MftStartLcn.max(0).cast_unsigned(),
            mft_valid_data_length: volume_data.MftValidDataLength.max(0).cast_unsigned(),
        })
    }

    /// Reads up to `buffer.len()` bytes starting at byte `offset` of the volume.
    ///
    /// Volume handles only accept reads whose offset and length are multiples of the sector size.
    pub(crate) fn read_at(&self, offset: u64, buffer: &mut [u8]) -> io::Result<usize> {
        let mut overlapped = OVERLAPPED {
            Anonymous: OVERLAPPED_0 {
                Anonymous: OVERLAPPED_0_0 {
                    Offset: (offset & 0xFFFF_FFFF) as u32,
                    OffsetHigh: (offset >> 32) as u32,
                },
            },
            ..OVERLAPPED::default()
        };
        let mut bytes_read = 0_u32;

        // SAFETY: the buffer is a writable slice whose length the wrapper passes; the handle is
        // synchronous, so the OVERLAPPED only carries the read offset and does not need to
        // outlive the call; `bytes_read` is a writable u32.
        unsafe {
            ReadFile(
                self.volume_handle.raw(),
                Some(buffer),
                Some(&raw mut bytes_read),
                Some(&raw mut overlapped),
            )
        }
        .map_err(|error| to_io_error(&error))?;

        Ok(bytes_read as usize)
    }
}

/// Turns `C:\` into the device path of its volume (`\\?\Volume{GUID}` without the trailing
/// backslash), which also works for volumes mounted in a folder instead of a drive letter.
fn volume_device_path_for_mount_point(mount_point: &Path) -> io::Result<String> {
    let mut mount_point_text = mount_point.to_string_lossy().into_owned();
    if !mount_point_text.ends_with('\\') {
        mount_point_text.push('\\');
    }
    let wide_mount_point = to_null_terminated_wide_path(Path::new(&mount_point_text));
    let mut volume_name_buffer = [0_u16; VOLUME_GUID_PATH_CAPACITY];

    // SAFETY: `wide_mount_point` is null-terminated with a trailing backslash as the function
    // requires and outlives the call; the output buffer is a writable slice whose length the
    // wrapper passes.
    unsafe {
        GetVolumeNameForVolumeMountPointW(
            PCWSTR(wide_mount_point.as_ptr()),
            &mut volume_name_buffer,
        )
    }
    .map_err(|error| to_io_error(&error))?;

    let volume_name = from_null_terminated_wide_buffer(&volume_name_buffer);
    // With the trailing backslash the path names the root directory; without it, the volume.
    Ok(volume_name.trim_end_matches('\\').to_owned())
}
