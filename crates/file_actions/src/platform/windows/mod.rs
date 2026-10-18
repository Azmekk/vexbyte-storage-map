//! Win32 and shell implementations of the platform functions. No other module uses `unsafe`.

mod bitmap_pixels;
mod com_apartment;
mod default_application;
mod file_stamp;
mod io_error;
mod recycle_bin_availability;
mod shell_folder_selection;
mod shell_image_factory;
mod shell_name_rules;
mod wide_path;

pub(crate) use default_application::open_with_default_application;
pub(crate) use file_stamp::{path_exists, read_file_stamp};
pub(crate) use recycle_bin_availability::volume_has_recycle_bin;
pub(crate) use shell_folder_selection::reveal_in_file_manager;
pub(crate) use shell_image_factory::render_shell_image;
pub(crate) use shell_name_rules::has_name_the_shell_would_rewrite;
