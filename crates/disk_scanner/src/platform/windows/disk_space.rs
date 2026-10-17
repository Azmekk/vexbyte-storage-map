use ::windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use ::windows::core::PCWSTR;

use super::wide_string::to_null_terminated_wide_string;

pub(super) struct DiskSpace {
    pub(super) total_bytes: u64,
    /// Free space on the whole volume, ignoring per-user disk quotas.
    pub(super) free_bytes: u64,
}

pub(super) fn query_disk_space(root_path: &str) -> Result<DiskSpace, ::windows::core::Error> {
    let wide_root_path = to_null_terminated_wide_string(root_path);
    let mut total_bytes = 0_u64;
    let mut free_bytes = 0_u64;

    // SAFETY: `wide_root_path` is null-terminated and outlives the call; both out pointers point
    // to writable u64 values on this stack frame.
    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(wide_root_path.as_ptr()),
            None,
            Some(&raw mut total_bytes),
            Some(&raw mut free_bytes),
        )?;
    }

    Ok(DiskSpace {
        total_bytes,
        free_bytes,
    })
}
