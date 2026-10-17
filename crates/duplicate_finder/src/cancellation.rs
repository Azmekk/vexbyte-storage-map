use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Lets another thread stop a running duplicate search. Clones share the same flag.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    is_cancelled: Arc<AtomicBool>,
}

/// Returned by [`CancellationToken::return_if_cancelled`] once cancellation was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the duplicate search was cancelled")]
pub struct SearchCancelled;

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::Relaxed);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::Relaxed)
    }

    /// # Errors
    ///
    /// Returns [`SearchCancelled`] once [`cancel`](Self::cancel) has been called on any clone.
    pub fn return_if_cancelled(&self) -> Result<(), SearchCancelled> {
        if self.is_cancelled() {
            Err(SearchCancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn clones_share_cancellation() {
        let token = CancellationToken::new();
        let clone = token.clone();

        assert_eq!(token.return_if_cancelled(), Ok(()));
        clone.cancel();

        assert!(token.is_cancelled());
        assert_eq!(token.return_if_cancelled(), Err(SearchCancelled));
    }
}
