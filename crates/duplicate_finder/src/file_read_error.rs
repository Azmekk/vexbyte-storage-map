use std::io;

/// Why one file could not be hashed. The file is dropped from the comparison; the search goes on.
#[derive(Debug, thiserror::Error)]
pub(crate) enum FileReadError {
    #[error("the file could not be read")]
    ReadFailed {
        #[from]
        source: io::Error,
    },
    #[error("the file changed size while it was being compared")]
    SizeChanged,
    #[error("the search was cancelled")]
    Cancelled,
}
