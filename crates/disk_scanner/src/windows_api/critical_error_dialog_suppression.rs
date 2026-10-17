use windows::Win32::System::Diagnostics::Debug::{
    SEM_FAILCRITICALERRORS, SetThreadErrorMode, THREAD_ERROR_MODE,
};

/// Stops Windows from showing "There is no disk in the drive" dialogs while drives are queried,
/// and restores the previous thread error mode when dropped.
pub(crate) struct CriticalErrorDialogSuppression {
    previous_error_mode: Option<THREAD_ERROR_MODE>,
}

impl CriticalErrorDialogSuppression {
    pub(crate) fn for_current_thread() -> Self {
        let mut previous_error_mode = THREAD_ERROR_MODE::default();

        // SAFETY: `previous_error_mode` is a valid, writable THREAD_ERROR_MODE for the duration
        // of the call.
        let result = unsafe {
            SetThreadErrorMode(SEM_FAILCRITICALERRORS, Some(&raw mut previous_error_mode))
        };

        Self {
            previous_error_mode: result.is_ok().then_some(previous_error_mode),
        }
    }
}

impl Drop for CriticalErrorDialogSuppression {
    fn drop(&mut self) {
        if let Some(previous_error_mode) = self.previous_error_mode {
            // SAFETY: restores a mode previously returned by SetThreadErrorMode; the optional out
            // pointer is not used.
            let _ = unsafe { SetThreadErrorMode(previous_error_mode, None) };
        }
    }
}
