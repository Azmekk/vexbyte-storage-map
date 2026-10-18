//! Win32 implementations of the platform functions. No other module uses `unsafe`.

mod directory_listing;
mod dos_device_name;
mod file_reading;
mod file_stamp;
mod file_time;
mod io_error;
mod path_comparison;
mod seek_penalty;
mod system_folders;
#[cfg(test)]
mod test_examples;
mod volume_identification;
mod volume_mount_point;
mod wide_path;

pub(crate) use directory_listing::list_directory;
pub(crate) use file_reading::open_file_for_sequential_reading;
pub(crate) use file_stamp::read_file_stamp;
pub(crate) use path_comparison::{is_same_or_inside_folder, normalize_folder_path};
pub(crate) use seek_penalty::query_volume_incurs_seek_penalty;
pub(crate) use system_folders::{is_system_folder_name, system_folder_roots};
#[cfg(test)]
pub(crate) use test_examples::{
    example_system_folder, example_system_folder_name, example_volume_root,
    folder_names_ignore_case,
};
pub(crate) use volume_identification::query_volume_identification;
pub(crate) use volume_mount_point::query_volume_mount_point;
