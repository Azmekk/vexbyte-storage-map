use ::windows::Win32::Foundation::HANDLE;
use ::windows::Win32::Security::{
    GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
};
use ::windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use super::owned_handle::OwnedHandle;

/// True when the process runs with full administrator rights (UAC elevated).
///
/// Reports false when the token cannot be queried, so callers fall back to the folder walk.
pub(crate) fn is_process_elevated() -> bool {
    let mut token_handle = HANDLE::default();

    // SAFETY: GetCurrentProcess returns a pseudo handle that needs no closing; `token_handle` is a
    // writable HANDLE on this stack frame.
    let open_result =
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token_handle) };
    if open_result.is_err() {
        return false;
    }
    let token = OwnedHandle::from_valid_handle(token_handle);

    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned_length = 0_u32;
    let elevation_size = u32::try_from(size_of::<TOKEN_ELEVATION>()).unwrap_or(u32::MAX);

    // SAFETY: `elevation` is a writable TOKEN_ELEVATION whose exact size is passed, and
    // `returned_length` is a writable u32; the token is open with TOKEN_QUERY access.
    let query_result = unsafe {
        GetTokenInformation(
            token.raw(),
            TokenElevation,
            Some((&raw mut elevation).cast()),
            elevation_size,
            &raw mut returned_length,
        )
    };

    query_result.is_ok() && elevation.TokenIsElevated != 0
}
