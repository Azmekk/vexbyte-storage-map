use ::windows::Win32::Foundation::MAX_PATH;
use ::windows::Win32::Storage::FileSystem::GetVolumeInformationW;
use ::windows::core::PCWSTR;

use super::wide_string::{from_null_terminated_wide_buffer, to_null_terminated_wide_string};

pub(super) struct VolumeInformation {
    pub(super) label: String,
    pub(super) file_system_name: String,
}

pub(super) fn query_volume_information(
    root_path: &str,
) -> Result<VolumeInformation, ::windows::core::Error> {
    let wide_root_path = to_null_terminated_wide_string(root_path);

    // The GetVolumeInformationW docs require MAX_PATH + 1 characters for both buffers.
    let mut label_buffer = [0_u16; MAX_PATH as usize + 1];
    let mut file_system_name_buffer = [0_u16; MAX_PATH as usize + 1];

    // SAFETY: `wide_root_path` is null-terminated and outlives the call; both output buffers are
    // writable slices whose lengths the wrapper passes to Win32; the unused out pointers are None.
    unsafe {
        GetVolumeInformationW(
            PCWSTR(wide_root_path.as_ptr()),
            Some(&mut label_buffer),
            None,
            None,
            None,
            Some(&mut file_system_name_buffer),
        )?;
    }

    Ok(VolumeInformation {
        label: from_null_terminated_wide_buffer(&label_buffer),
        file_system_name: from_null_terminated_wide_buffer(&file_system_name_buffer),
    })
}
