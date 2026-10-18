//! Linux implementations of the platform functions.

mod elevated_launch;
mod operating_system;

pub(crate) use elevated_launch::{is_cancelled_by_user, launch_current_executable_elevated};
pub(crate) use operating_system::{current_operating_system, paths_are_case_sensitive};

/// The root of an example volume for tests, named by a drive letter so tests read the same on
/// every platform: `/mnt/c` for `'C'`. Roots keep the order of their letters.
#[cfg(test)]
pub(crate) fn example_volume_root(drive_letter: char) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("/mnt/{}", drive_letter.to_ascii_lowercase()))
}
