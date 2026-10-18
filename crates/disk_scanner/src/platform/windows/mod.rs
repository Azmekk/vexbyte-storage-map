//! The Windows backend: safe wrappers around every Win32 call this crate makes. No other module
//! uses `unsafe`, and no `windows::` type leaves this module.

mod directory_listing;
mod disk_space;
mod dos_device_name;
mod drive_type;
mod file_attributes;
mod file_time;
mod io_error;
mod media_error_prompt_suppression;
mod owned_handle;
mod path_encoding;
mod process_elevation;
mod raw_volume_reader;
mod system_volume;
mod volume_details;
mod volume_information;
mod volume_mount_points;
mod wide_string;

pub(crate) use directory_listing::list_directory;
pub(crate) use media_error_prompt_suppression::MediaErrorPromptSuppression;
pub(crate) use process_elevation::is_process_elevated;
pub(crate) use raw_volume_reader::RawVolumeReader;
pub(crate) use system_volume::system_volume_mount_point;
pub(crate) use volume_details::{
    file_system_supports_fast_scan, is_volume_not_ready_error, query_volume_details,
    volume_mount_point_for_path,
};
pub(crate) use volume_mount_points::list_volume_mount_points;

/// A volume root that tests can use to build absolute paths, `C:\` on Windows.
#[cfg(test)]
pub(crate) fn example_volume_root() -> std::path::PathBuf {
    std::path::PathBuf::from(r"C:\")
}

/// The root of a second example volume for tests, `D:\` on Windows.
#[cfg(test)]
pub(crate) fn second_example_volume_root() -> std::path::PathBuf {
    std::path::PathBuf::from(r"D:\")
}
