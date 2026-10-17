use std::path::PathBuf;

use super::volume_details::volume_mount_point_for_path;

/// The mount point of the volume Windows runs from, found through `%SystemRoot%`
/// (normally `C:\Windows`). `None` when the variable is missing or its volume cannot be read.
pub(crate) fn system_volume_mount_point() -> Option<PathBuf> {
    let system_root = PathBuf::from(std::env::var_os("SystemRoot")?);
    volume_mount_point_for_path(&system_root).ok()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn the_system_volume_is_a_mount_point() {
        let mount_point = system_volume_mount_point().unwrap();

        assert!(mount_point.is_absolute());
        assert!(mount_point.parent().is_none(), "{}", mount_point.display());
    }
}
