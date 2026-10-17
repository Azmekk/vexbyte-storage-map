use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Lets another thread stop a running scan. Clones share the same flag.
///
/// The folder walk checks it between directories and the MFT scanner between chunks, so a
/// scan stops within a fraction of a second after `cancel`.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

/// Returned by [`CancellationToken::return_if_cancelled`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the scan was cancelled")]
pub struct ScanCancelled;

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// # Errors
    ///
    /// Returns `ScanCancelled` once `cancel` has been called on any clone.
    pub fn return_if_cancelled(&self) -> Result<(), ScanCancelled> {
        if self.is_cancelled() {
            Err(ScanCancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelling_a_clone_cancels_the_original() {
        let token = CancellationToken::new();
        let clone = token.clone();

        assert_eq!(token.return_if_cancelled(), Ok(()));
        clone.cancel();

        assert!(token.is_cancelled());
        assert_eq!(token.return_if_cancelled(), Err(ScanCancelled));
    }
}
