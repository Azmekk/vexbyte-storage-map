use std::mem::size_of;
use std::path::Path;

use windows::Win32::Storage::FileSystem::{GetDriveTypeW, GetVolumePathNameW};
use windows::Win32::System::WindowsProgramming::DRIVE_FIXED;
use windows::Win32::UI::Shell::{SHQUERYRBINFO, SHQueryRecycleBinW};
use windows::core::PCWSTR;

use super::wide_path::to_extended_length_wide_path;

const MOUNT_POINT_BUFFER_CHARACTERS: usize = 32 * 1024;

/// True when files on the volume holding `path` can go to the Recycle Bin. Removable drives and
/// network shares have none; there the shell would delete permanently instead, so such paths are
/// refused before anything is attempted. Only queries are made.
pub(crate) fn volume_has_recycle_bin(path: &Path) -> bool {
    let Some(wide_root_path) = query_wide_volume_root(path) else {
        return false;
    };

    // SAFETY: `wide_root_path` is a null-terminated root path that outlives the call.
    let drive_type = unsafe { GetDriveTypeW(PCWSTR(wide_root_path.as_ptr())) };
    if drive_type != DRIVE_FIXED {
        return false;
    }

    let mut recycle_bin_information = SHQUERYRBINFO {
        cbSize: u32::try_from(size_of::<SHQUERYRBINFO>()).unwrap_or(u32::MAX),
        ..SHQUERYRBINFO::default()
    };
    // SAFETY: `wide_root_path` is null-terminated and outlives the call; the info struct is
    // writable and carries its own size, as SHQueryRecycleBinW requires.
    let query_result = unsafe {
        SHQueryRecycleBinW(
            PCWSTR(wide_root_path.as_ptr()),
            &raw mut recycle_bin_information,
        )
    };

    query_result.is_ok()
}

fn query_wide_volume_root(path: &Path) -> Option<Vec<u16>> {
    let wide_path = to_extended_length_wide_path(path).ok()?;
    let mut mount_point_buffer = vec![0_u16; MOUNT_POINT_BUFFER_CHARACTERS];

    // SAFETY: `wide_path` is null-terminated and outlives the call; the output buffer is a
    // writable slice whose length the wrapper passes to Win32.
    unsafe { GetVolumePathNameW(PCWSTR(wide_path.as_ptr()), &mut mount_point_buffer) }.ok()?;

    let length = mount_point_buffer
        .iter()
        .position(|&character| character == 0)?;
    mount_point_buffer.truncate(length + 1);
    Some(mount_point_buffer)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn the_system_drive_has_a_recycle_bin() {
        let windows_folder = std::env::var_os("SystemRoot").unwrap();

        assert!(volume_has_recycle_bin(Path::new(&windows_folder)));
    }
}
