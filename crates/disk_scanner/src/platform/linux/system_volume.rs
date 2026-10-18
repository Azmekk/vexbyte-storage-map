use std::path::PathBuf;

/// Linux always runs from the file system mounted at `/`.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn system_volume_mount_point() -> Option<PathBuf> {
    Some(PathBuf::from("/"))
}
