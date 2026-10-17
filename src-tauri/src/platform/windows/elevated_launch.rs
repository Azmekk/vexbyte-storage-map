use std::ffi::OsString;
use std::io;

use windows::Win32::Foundation::ERROR_CANCELLED;
use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
};
use windows::Win32::UI::Shell::{SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

use super::command_line::join_command_line_arguments;
use super::io_error::to_io_error;
use super::wide_string::to_null_terminated_wide_string;

/// Starts a new instance of this executable with administrator rights. Windows shows the UAC
/// prompt; this blocks until the user answered it and the new process was created.
pub(crate) fn launch_current_executable_elevated(arguments: &[OsString]) -> io::Result<()> {
    let executable_path = std::env::current_exe()?;
    let executable_wide = to_null_terminated_wide_string(executable_path.as_os_str());
    let parameters_wide = to_null_terminated_wide_string(&join_command_line_arguments(arguments));

    // ShellExecuteEx may use COM for the verb and wants a single-threaded apartment, which a
    // short-lived thread of our own provides regardless of the caller's COM state.
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("vsm-elevated-launch".to_owned())
            .spawn_scoped(scope, || {
                shell_execute_run_as_administrator(&executable_wide, &parameters_wide)
            })?
            .join()
            .map_err(|_| io::Error::other("the elevated launch thread stopped unexpectedly"))?
    })
}

/// True when the user dismissed the UAC prompt.
pub(crate) fn is_cancelled_by_user(error: &io::Error) -> bool {
    error.raw_os_error() == Some(ERROR_CANCELLED.0.cast_signed())
}

fn shell_execute_run_as_administrator(
    executable_wide: &[u16],
    parameters_wide: &[u16],
) -> io::Result<()> {
    // SAFETY: the first COM call on this fresh thread; balanced by CoUninitialize below when it
    // succeeds.
    let com_initialization =
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };

    let mut execute_info = SHELLEXECUTEINFOW {
        cbSize: u32::try_from(size_of::<SHELLEXECUTEINFOW>()).unwrap_or(u32::MAX),
        // This process exits right after the launch, so the shell must finish its work before
        // the call returns.
        fMask: SEE_MASK_NOASYNC,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(executable_wide.as_ptr()),
        lpParameters: PCWSTR(parameters_wide.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    // SAFETY: `execute_info` is initialized with a correct `cbSize`, and the strings it points
    // to are null-terminated and outlive the call.
    let launch_result = unsafe { ShellExecuteExW(&raw mut execute_info) };

    if com_initialization.is_ok() {
        // SAFETY: balances the successful CoInitializeEx above on the same thread.
        unsafe { CoUninitialize() };
    }

    launch_result.map_err(|error| to_io_error(&error))
}
