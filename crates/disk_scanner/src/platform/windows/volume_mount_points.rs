use std::io;
use std::path::PathBuf;

use ::windows::Win32::Storage::FileSystem::GetLogicalDrives;

/// Lists the root directory of every drive letter in use, for example `C:\`.
pub(crate) fn list_volume_mount_points() -> io::Result<Vec<PathBuf>> {
    // SAFETY: GetLogicalDrives takes no arguments and only returns a bitmask.
    let drive_bitmask = unsafe { GetLogicalDrives() };

    // GetLogicalDrives returns 0 on failure; a running Windows system always has a drive.
    if drive_bitmask == 0 {
        return Err(io::Error::last_os_error());
    }

    let mount_points = ('A'..='Z')
        .enumerate()
        .filter(|(bit_position, _)| drive_bitmask & (1 << bit_position) != 0)
        .map(|(_, letter)| PathBuf::from(format!("{letter}:\\")))
        .collect();

    Ok(mount_points)
}
