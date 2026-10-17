use std::io;
use std::path::Path;

use windows::Win32::Storage::FileSystem::QueryDosDeviceW;
use windows::core::PCWSTR;

use super::wide_path::to_null_terminated_wide_string;

/// Room for the longest device name Windows reports; `\Device\Volume{guid}` is about 60.
const DEVICE_NAME_BUFFER_CHARACTERS: usize = 1024;

/// Reads the NT device a drive letter points to, for example `\Device\HarddiskVolume3` for
/// `C:\`. Only drive letter mount points have a DOS device name; folder mount points fail.
///
/// The same wrapper exists in `disk_scanner`; it is duplicated so this crate stays independent.
pub(super) fn query_dos_device_name(mount_point: &Path) -> io::Result<String> {
    let mount_point_text = mount_point.to_string_lossy();
    // QueryDosDeviceW wants the bare `C:` without the trailing backslash.
    let drive_name = mount_point_text.trim_end_matches('\\');
    let wide_drive_name = to_null_terminated_wide_string(drive_name.as_ref());
    let mut device_name_buffer = vec![0_u16; DEVICE_NAME_BUFFER_CHARACTERS];

    // SAFETY: `wide_drive_name` is null-terminated and outlives the call; the output buffer is a
    // writable slice whose length the wrapper passes to Win32.
    let written_characters = unsafe {
        QueryDosDeviceW(
            PCWSTR(wide_drive_name.as_ptr()),
            Some(&mut device_name_buffer),
        )
    };

    // QueryDosDeviceW returns 0 on failure and sets the last error.
    if written_characters == 0 {
        return Err(io::Error::last_os_error());
    }

    // The buffer holds a list of null-terminated strings; the first is the current mapping.
    let first_name_length = device_name_buffer
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(device_name_buffer.len());
    Ok(String::from_utf16_lossy(
        &device_name_buffer[..first_name_length],
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::super::volume_mount_point::query_volume_mount_point;
    use super::*;

    #[test]
    fn the_system_drive_has_a_device_name() {
        let mount_point = query_volume_mount_point(&std::env::temp_dir()).unwrap();

        let device_name = query_dos_device_name(&mount_point).unwrap();

        assert!(device_name.starts_with(r"\Device\"), "{device_name}");
    }
}
