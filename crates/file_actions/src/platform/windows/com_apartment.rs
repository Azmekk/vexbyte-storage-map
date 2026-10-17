use std::io;
use std::marker::PhantomData;

use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
};

use super::io_error::to_io_error;

/// Keeps COM initialized as a single-threaded apartment on the current thread, as the shell APIs
/// expect, and uninitializes it again when dropped. Not `Send`: COM state is per thread.
pub(super) struct ComApartment {
    must_uninitialize: bool,
    _not_send: PhantomData<*const ()>,
}

impl ComApartment {
    pub(super) fn enter_single_threaded() -> io::Result<Self> {
        // SAFETY: the reserved pointer must be None; every successful call (S_OK or S_FALSE) is
        // balanced by CoUninitialize in Drop on this same thread, which `!Send` guarantees.
        let result =
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };

        if result == RPC_E_CHANGED_MODE {
            // The thread already joined the multithreaded apartment. The shell calls used here
            // work there too, and that initialization is not ours to undo.
            return Ok(Self {
                must_uninitialize: false,
                _not_send: PhantomData,
            });
        }
        result.ok().map_err(|error| to_io_error(&error))?;

        Ok(Self {
            must_uninitialize: true,
            _not_send: PhantomData,
        })
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.must_uninitialize {
            // SAFETY: balances the successful CoInitializeEx made on this thread in
            // `enter_single_threaded`.
            unsafe { CoUninitialize() };
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn can_be_entered_twice_on_one_thread() {
        let outer = ComApartment::enter_single_threaded().unwrap();
        let inner = ComApartment::enter_single_threaded().unwrap();

        drop(inner);
        drop(outer);
    }
}
