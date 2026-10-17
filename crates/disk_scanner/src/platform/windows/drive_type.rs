use ::windows::Win32::Storage::FileSystem::GetDriveTypeW;
use ::windows::Win32::System::WindowsProgramming::{
    DRIVE_CDROM, DRIVE_FIXED, DRIVE_NO_ROOT_DIR, DRIVE_RAMDISK, DRIVE_REMOTE, DRIVE_REMOVABLE,
};
use ::windows::core::PCWSTR;

use super::wide_string::to_null_terminated_wide_string;
use crate::volumes::DriveType;

/// Returns `None` when no volume is mounted at `root_path`.
pub(super) fn query_drive_type(root_path: &str) -> Option<DriveType> {
    let wide_root_path = to_null_terminated_wide_string(root_path);

    // SAFETY: `wide_root_path` is a null-terminated UTF-16 string that outlives the call.
    let raw_drive_type = unsafe { GetDriveTypeW(PCWSTR(wide_root_path.as_ptr())) };

    match raw_drive_type {
        DRIVE_NO_ROOT_DIR => None,
        DRIVE_FIXED => Some(DriveType::Fixed),
        DRIVE_REMOVABLE => Some(DriveType::Removable),
        DRIVE_REMOTE => Some(DriveType::Network),
        DRIVE_CDROM => Some(DriveType::Optical),
        DRIVE_RAMDISK => Some(DriveType::Ram),
        _ => Some(DriveType::Unknown),
    }
}
