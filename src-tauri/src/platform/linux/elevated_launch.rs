use std::ffi::OsString;
use std::io;

/// The fast scan is never offered on Linux, so the app never asks to restart elevated.
pub(crate) fn launch_current_executable_elevated(_arguments: &[OsString]) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "restarting with elevated rights is not supported on Linux",
    ))
}

/// Linux never shows an elevation prompt the user could cancel.
pub(crate) fn is_cancelled_by_user(_error: &io::Error) -> bool {
    false
}
