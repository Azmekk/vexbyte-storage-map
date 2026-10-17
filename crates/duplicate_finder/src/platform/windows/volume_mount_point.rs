use std::ffi::OsString;
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use windows::Win32::Storage::FileSystem::GetVolumePathNameW;
use windows::core::PCWSTR;

use super::io_error::to_io_error;
use super::wide_path::{remove_extended_length_prefix, to_extended_length_wide_path};

/// Longest mount point path accepted; folder mount points can be deep.
const MOUNT_POINT_BUFFER_CHARACTERS: usize = 32 * 1024;

/// Finds the mount point of the volume holding `path`, for example `C:\` or a folder that has a
/// volume mounted on it.
pub(crate) fn query_volume_mount_point(path: &Path) -> io::Result<PathBuf> {
    let wide_path = to_extended_length_wide_path(path)?;
    let mut mount_point_buffer = vec![0_u16; MOUNT_POINT_BUFFER_CHARACTERS];

    // SAFETY: `wide_path` is null-terminated and outlives the call; the output buffer is a
    // writable slice whose length the wrapper passes to Win32.
    unsafe { GetVolumePathNameW(PCWSTR(wide_path.as_ptr()), &mut mount_point_buffer) }
        .map_err(|error| to_io_error(&error))?;

    let length = mount_point_buffer
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(mount_point_buffer.len());
    let mount_point = PathBuf::from(OsString::from_wide(&mount_point_buffer[..length]));

    Ok(remove_extended_length_prefix(&mount_point))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn finds_the_mount_point_of_the_temp_directory() {
        let temp_directory = std::env::temp_dir();

        let mount_point = query_volume_mount_point(&temp_directory).unwrap();

        assert!(
            temp_directory.starts_with(&mount_point),
            "{}",
            mount_point.display()
        );
    }
}
