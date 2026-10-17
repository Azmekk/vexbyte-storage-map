use std::io;
use std::mem::size_of;
use std::path::Path;

use windows::Win32::UI::Shell::{SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

use super::com_apartment::ComApartment;
use super::io_error::to_io_error;
use super::wide_path::to_shell_wide_path;

/// Opens `path` with the application registered for its type, exactly like a double-click in
/// Explorer. For programs that means running them.
pub(crate) fn open_with_default_application(path: &Path) -> io::Result<()> {
    let _com_apartment = ComApartment::enter_single_threaded()?;
    let wide_path = to_shell_wide_path(path)?;

    let mut execute_information = SHELLEXECUTEINFOW {
        cbSize: u32::try_from(size_of::<SHELLEXECUTEINFOW>()).unwrap_or(u32::MAX),
        // The calling thread may end right after this returns; NOASYNC makes the shell finish
        // handing the request off first.
        fMask: SEE_MASK_NOASYNC,
        lpVerb: w!("open"),
        lpFile: PCWSTR(wide_path.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..SHELLEXECUTEINFOW::default()
    };

    // SAFETY: the struct carries its own size, and every string it points to (`wide_path` and
    // the static verb) is null-terminated and outlives the call.
    unsafe { ShellExecuteExW(&raw mut execute_information) }.map_err(|error| to_io_error(&error))
}
