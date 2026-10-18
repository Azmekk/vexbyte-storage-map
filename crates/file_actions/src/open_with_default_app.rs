use std::io;
use std::path::{Path, PathBuf};

use crate::platform::open_with_default_application;

#[derive(Debug, thiserror::Error)]
pub enum OpenWithDefaultAppError {
    #[error("could not open {}", path.display())]
    OpenFailed { path: PathBuf, source: io::Error },
}

/// Opens `path` with the application registered for its file type, like a double-click in
/// the file manager. Programs and scripts are started, so callers should only pass what the user chose to
/// open. Not exercised by automated tests, since it starts other applications.
///
/// # Errors
///
/// Fails when no application is associated or the shell refuses to open the file.
pub fn open_with_default_app(path: &Path) -> Result<(), OpenWithDefaultAppError> {
    open_with_default_application(path).map_err(|source| OpenWithDefaultAppError::OpenFailed {
        path: path.to_path_buf(),
        source,
    })
}
