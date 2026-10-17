//! Win32 implementations of the platform functions. No other module of the app uses `unsafe`.

mod command_line;
mod elevated_launch;
mod io_error;
mod wide_string;

pub(crate) use elevated_launch::{is_cancelled_by_user, launch_current_executable_elevated};
