use std::io;
use std::path::{Path, PathBuf};

use ::windows::Win32::Foundation::{ERROR_NOT_READY, MAX_PATH};
use ::windows::Win32::Storage::FileSystem::GetVolumePathNameW;
use ::windows::core::PCWSTR;

use super::disk_space::{DiskSpace, query_disk_space};
use super::drive_type::query_drive_type;
use super::io_error::to_io_error;
use super::path_encoding::to_null_terminated_wide_path;
use super::volume_information::{VolumeInformation, query_volume_information};
use super::wide_string::from_null_terminated_wide_buffer;
use crate::volumes::VolumeDetails;

/// Reads label, file system, space and drive type of the volume mounted at `mount_point`.
///
/// Returns `Ok(None)` when no volume is mounted there.
pub(crate) fn query_volume_details(mount_point: &Path) -> io::Result<Option<VolumeDetails>> {
    let root_path = mount_point.to_string_lossy();

    let Some(drive_type) = query_drive_type(&root_path) else {
        return Ok(None);
    };

    let VolumeInformation {
        label,
        file_system_name,
    } = query_volume_information(&root_path).map_err(|error| to_io_error(&error))?;

    let DiskSpace {
        total_bytes,
        free_bytes,
    } = query_disk_space(&root_path).map_err(|error| to_io_error(&error))?;

    Ok(Some(VolumeDetails {
        label,
        file_system_name,
        total_bytes,
        free_bytes,
        drive_type,
    }))
}

/// True for the error Windows reports for an empty optical drive or card reader.
pub(crate) fn is_volume_not_ready_error(error: &io::Error) -> bool {
    error.raw_os_error() == i32::try_from(ERROR_NOT_READY.0).ok()
}

/// Only NTFS has a Master File Table that can be read directly.
pub(crate) fn file_system_supports_fast_scan(file_system_name: &str) -> bool {
    file_system_name.eq_ignore_ascii_case("NTFS")
}

/// Finds the mount point of the volume that contains `path`, for example `C:\`.
pub(crate) fn volume_mount_point_for_path(path: &Path) -> io::Result<PathBuf> {
    let wide_path = to_null_terminated_wide_path(path);
    let mut mount_point_buffer = vec![0_u16; MAX_PATH as usize + 1];

    // SAFETY: `wide_path` is null-terminated and outlives the call; the output buffer is a
    // writable slice whose length the wrapper passes to Win32.
    unsafe { GetVolumePathNameW(PCWSTR(wide_path.as_ptr()), &mut mount_point_buffer) }
        .map_err(|error| to_io_error(&error))?;

    Ok(PathBuf::from(from_null_terminated_wide_buffer(
        &mount_point_buffer,
    )))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn finds_the_volume_of_the_temp_directory() {
        let temp_directory = std::env::temp_dir();

        let mount_point = volume_mount_point_for_path(&temp_directory).unwrap();

        assert!(temp_directory.starts_with(&mount_point));
    }

    #[test]
    fn recognizes_the_not_ready_error() {
        assert!(is_volume_not_ready_error(&io::Error::from_raw_os_error(21)));
        assert!(!is_volume_not_ready_error(&io::Error::from_raw_os_error(5)));
    }
}
