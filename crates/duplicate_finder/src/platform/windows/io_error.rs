use std::io;

/// Facility code Windows uses when it wraps a plain Win32 error code in an HRESULT.
const FACILITY_WIN32: u32 = 7;

/// Converts a Win32 error into `io::Error`, keeping the original error code so `kind()` and the
/// system message stay accurate.
pub(super) fn to_io_error(error: &windows::core::Error) -> io::Error {
    let hresult = error.code().0.cast_unsigned();
    let facility = (hresult >> 16) & 0x1FFF;

    if facility == FACILITY_WIN32 {
        let win32_error_code = hresult & 0xFFFF;
        io::Error::from_raw_os_error(win32_error_code.cast_signed())
    } else {
        io::Error::from_raw_os_error(hresult.cast_signed())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;

    use super::*;

    #[test]
    fn keeps_the_win32_error_code() {
        let error = windows::core::Error::from(ERROR_FILE_NOT_FOUND.to_hresult());

        let io_error = to_io_error(&error);

        assert_eq!(io_error.kind(), io::ErrorKind::NotFound);
        assert_eq!(io_error.raw_os_error(), Some(2));
    }
}
