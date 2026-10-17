use windows::Win32::Storage::FileSystem::GetLogicalDrives;

use crate::volumes::DriveLetter;

pub(crate) fn query_logical_drive_letters() -> Result<Vec<DriveLetter>, windows::core::Error> {
    // SAFETY: GetLogicalDrives takes no arguments and only returns a bitmask.
    let drive_bitmask = unsafe { GetLogicalDrives() };

    // GetLogicalDrives returns 0 on failure; a running Windows system always has a drive.
    if drive_bitmask == 0 {
        return Err(windows::core::Error::from_thread());
    }

    let drive_letters = ('A'..='Z')
        .enumerate()
        .filter(|(bit_position, _)| drive_bitmask & (1 << bit_position) != 0)
        .filter_map(|(_, letter)| DriveLetter::new(letter))
        .collect();

    Ok(drive_letters)
}
