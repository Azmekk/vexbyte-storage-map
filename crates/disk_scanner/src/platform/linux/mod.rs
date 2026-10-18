//! The Linux backend: directory listing through `std::fs` and volume facts from the mount table.
//! It needs no `unsafe`; the few raw system calls go through `rustix` and `mount_table`.

mod directory_listing;
mod drive_type;
mod media_error_prompt_suppression;
mod process_elevation;
#[cfg(test)]
mod sample_mount_entries;
mod system_volume;
mod volume_details;
mod volume_mount_points;

pub(crate) use directory_listing::list_directory;
pub(crate) use media_error_prompt_suppression::MediaErrorPromptSuppression;
pub(crate) use process_elevation::is_process_elevated;
pub(crate) use system_volume::system_volume_mount_point;
pub(crate) use volume_details::{
    file_system_supports_fast_scan, is_volume_not_ready_error, query_volume_details,
    volume_mount_point_for_path,
};
pub(crate) use volume_mount_points::list_volume_mount_points;

/// A volume root that tests can use to build absolute paths, `/` on Linux.
#[cfg(test)]
pub(crate) fn example_volume_root() -> std::path::PathBuf {
    std::path::PathBuf::from("/")
}

/// The root of a second example volume for tests, `/mnt/second` on Linux.
#[cfg(test)]
pub(crate) fn second_example_volume_root() -> std::path::PathBuf {
    std::path::PathBuf::from("/mnt/second")
}
