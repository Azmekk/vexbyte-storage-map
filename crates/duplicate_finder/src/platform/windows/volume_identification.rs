use std::io;
use std::path::Path;

use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::Storage::FileSystem::{GetDriveTypeW, GetVolumeInformationW};
use windows::Win32::System::WindowsProgramming::DRIVE_FIXED;
use windows::core::PCWSTR;

use super::dos_device_name::query_dos_device_name;
use super::io_error::to_io_error;
use super::wide_path::to_null_terminated_wide_string;
use crate::cloud_drive_guard::VolumeIdentification;

/// Reads what is needed to tell whether the volume mounted at `mount_point` (for example `G:\`)
/// is a cloud virtual drive.
pub(crate) fn query_volume_identification(mount_point: &Path) -> io::Result<VolumeIdentification> {
    let wide_mount_point = to_null_terminated_wide_string(mount_point.as_os_str());

    // SAFETY: `wide_mount_point` is a null-terminated UTF-16 string that outlives the call.
    let drive_type = unsafe { GetDriveTypeW(PCWSTR(wide_mount_point.as_ptr())) };

    // The GetVolumeInformationW docs require MAX_PATH + 1 characters for the label buffer.
    let mut label_buffer = [0_u16; MAX_PATH as usize + 1];
    let mut serial_number = 0_u32;

    // SAFETY: `wide_mount_point` is null-terminated and outlives the call; the label buffer is a
    // writable slice whose length the wrapper passes to Win32; `serial_number` is a live local;
    // the unused out pointers are None.
    unsafe {
        GetVolumeInformationW(
            PCWSTR(wide_mount_point.as_ptr()),
            Some(&mut label_buffer),
            Some(&raw mut serial_number),
            None,
            None,
            None,
        )
    }
    .map_err(|error| to_io_error(&error))?;

    let label_length = label_buffer
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(label_buffer.len());

    Ok(VolumeIdentification {
        is_fixed_drive: drive_type == DRIVE_FIXED,
        label: String::from_utf16_lossy(&label_buffer[..label_length]),
        serial_number,
        dos_device_name: query_dos_device_name(mount_point).ok(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::super::volume_mount_point::query_volume_mount_point;
    use super::*;

    #[test]
    fn identifies_the_volume_of_the_temp_directory() {
        let mount_point = query_volume_mount_point(&std::env::temp_dir()).unwrap();

        let identification = query_volume_identification(&mount_point).unwrap();

        assert!(identification.dos_device_name.is_some());
    }
}
