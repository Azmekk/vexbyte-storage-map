//! Linux implementations of the platform functions, built on freedesktop conventions (xdg-open,
//! the `FileManager1` D-Bus interface, the shared thumbnail cache) and the mount table.

mod default_application;
mod file_manager_reveal;
mod file_stamp;
mod file_uri;
mod freedesktop_thumbnail_cache;
mod image_decoding;
mod recycle_bin_availability;
mod shell_image;
mod shell_name_rules;
mod spawned_process;

pub(crate) use default_application::open_with_default_application;
pub(crate) use file_manager_reveal::reveal_in_file_manager;
pub(crate) use file_stamp::{path_exists, read_file_stamp};
pub(crate) use recycle_bin_availability::volume_has_recycle_bin;
pub(crate) use shell_image::render_shell_image;
pub(crate) use shell_name_rules::has_name_the_shell_would_rewrite;
