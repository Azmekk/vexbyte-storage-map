use ::windows::Win32::Foundation::{CloseHandle, HANDLE};

/// A Win32 handle that is closed when dropped.
pub(super) struct OwnedHandle(HANDLE);

impl OwnedHandle {
    /// Takes ownership of a valid handle returned by a successful Win32 open call.
    pub(super) fn from_valid_handle(handle: HANDLE) -> Self {
        Self(handle)
    }

    pub(super) fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful Win32 open call, is owned exclusively by this
        // value and is closed exactly once here.
        let _ = unsafe { CloseHandle(self.0) };
    }
}
