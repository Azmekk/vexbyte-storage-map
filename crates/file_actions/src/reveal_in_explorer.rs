use std::io;
use std::path::{Path, PathBuf};

use crate::platform::reveal_in_file_manager;

#[derive(Debug, thiserror::Error)]
pub enum RevealInExplorerError {
    #[error("could not show {} in the file manager", path.display())]
    RevealFailed { path: PathBuf, source: io::Error },
}

/// Opens the folder containing `path` in Explorer (on Linux, the desktop's file manager) with the
/// item selected. Changes nothing on disk. Not exercised by automated tests, since it opens a
/// window.
///
/// # Errors
///
/// Fails when the shell cannot resolve the path or open the window.
pub fn reveal_in_explorer(path: &Path) -> Result<(), RevealInExplorerError> {
    reveal_in_file_manager(path).map_err(|source| RevealInExplorerError::RevealFailed {
        path: path.to_path_buf(),
        source,
    })
}
