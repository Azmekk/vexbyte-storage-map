use windows::Win32::Foundation::ERROR_NOT_READY;

/// True for the error Windows reports for an empty optical drive or card reader.
pub(crate) fn is_drive_not_ready_error(error: &windows::core::Error) -> bool {
    error.code() == ERROR_NOT_READY.to_hresult()
}
