//! Administrator rights: the app always starts unelevated (the manifest says `asInvoker`), and
//! the fast scan needs them, so the user can restart the app elevated for one scan.

use disk_scanner::ScanTarget;

use crate::platform;
use crate::startup_arguments::arguments_for_startup_scan;

#[derive(Debug, thiserror::Error)]
pub enum RelaunchError {
    #[error("the request for administrator rights was declined")]
    DeclinedByUser,
    #[error("could not start the app with administrator rights")]
    LaunchFailed { source: std::io::Error },
}

/// True when this process runs with administrator rights.
pub fn is_process_elevated() -> bool {
    disk_scanner::is_process_elevated()
}

/// Starts a new elevated instance that scans `target` with the fast method right away. The UAC
/// prompt blocks until the user answers it; call this from a background thread. The caller
/// exits the current instance afterwards.
///
/// # Errors
///
/// Fails when the user declines the prompt or Windows cannot start the new instance.
pub fn relaunch_elevated_for_scan(target: &ScanTarget) -> Result<(), RelaunchError> {
    let arguments = arguments_for_startup_scan(target);

    platform::launch_current_executable_elevated(&arguments).map_err(|source| {
        if platform::is_cancelled_by_user(&source) {
            RelaunchError::DeclinedByUser
        } else {
            RelaunchError::LaunchFailed { source }
        }
    })
}
