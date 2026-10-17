use std::io;

const FACILITY_WIN32: u32 = 7;

/// Converts a `windows` crate error into `io::Error`, keeping the exact Win32 error code so that
/// `io::Error::kind` recognizes codes such as access denied.
pub(super) fn to_io_error(error: &::windows::core::Error) -> io::Error {
    // The windows crate reports Win32 failures as HRESULTs of the form 0x8007xxxx.
    let hresult = error.code().0.cast_unsigned();
    let facility = (hresult >> 16) & 0x1FFF;
    let is_failure = hresult & 0x8000_0000 != 0;

    if is_failure && facility == FACILITY_WIN32 {
        let win32_error_code = (hresult & 0xFFFF).cast_signed();
        io::Error::from_raw_os_error(win32_error_code)
    } else {
        io::Error::from_raw_os_error(hresult.cast_signed())
    }
}

#[cfg(test)]
mod tests {
    use ::windows::Win32::Foundation::ERROR_ACCESS_DENIED;

    use super::*;

    #[test]
    fn keeps_the_win32_error_code_so_the_error_kind_is_recognized() {
        let windows_error = ::windows::core::Error::from(ERROR_ACCESS_DENIED.to_hresult());

        let io_error = to_io_error(&windows_error);

        assert_eq!(io_error.raw_os_error(), Some(5));
        assert_eq!(io_error.kind(), io::ErrorKind::PermissionDenied);
    }
}
