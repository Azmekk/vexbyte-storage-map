//! Win32 implementations of the platform functions. No other module of the app uses `unsafe`.

mod command_line;
mod elevated_launch;
mod io_error;
mod operating_system;
mod wide_string;

pub(crate) use elevated_launch::{is_cancelled_by_user, launch_current_executable_elevated};
pub(crate) use operating_system::{current_operating_system, paths_are_case_sensitive};

/// The root of an example volume for tests, named by its drive letter: `C:\` for `'C'`.
#[cfg(test)]
pub(crate) fn example_volume_root(drive_letter: char) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{drive_letter}:\\"))
}
