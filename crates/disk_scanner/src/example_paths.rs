//! Absolute example paths for tests, so that tests about paths run unchanged on every platform:
//! below `C:\` on Windows and below `/` on Linux.

use std::path::{Path, PathBuf};

use crate::platform;

/// The root of the example volume, `C:\` on Windows.
pub(crate) fn example_volume_root() -> PathBuf {
    platform::example_volume_root()
}

/// The root of a second example volume, `D:\` on Windows.
pub(crate) fn second_example_volume_root() -> PathBuf {
    platform::second_example_volume_root()
}

/// `relative_path` below the example volume root, with names separated by `/`, for example
/// `Users/me/notes.txt`.
pub(crate) fn example_path(relative_path: &str) -> PathBuf {
    path_below(&example_volume_root(), relative_path)
}

/// `relative_path` below `root`, with names separated by `/`.
pub(crate) fn path_below(root: &Path, relative_path: &str) -> PathBuf {
    relative_path
        .split('/')
        .fold(root.to_path_buf(), |path, name| path.join(name))
}

/// The example volume root as the user would type it in a search, for example `c:\`.
pub(crate) fn example_volume_root_query() -> String {
    example_volume_root().to_string_lossy().to_lowercase()
}
