use std::io;
use std::path::{Path, PathBuf};

use super::mount_lookup::read_mount_entry_containing;

/// Finds the mount point of the file system holding `path`, for example `/` or `/home`. Links
/// are resolved first, because the mount table lists real paths.
pub(crate) fn query_volume_mount_point(path: &Path) -> io::Result<PathBuf> {
    let real_path = std::fs::canonicalize(path)?;
    let entry = read_mount_entry_containing(&real_path)?;

    Ok(entry.mount_point)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn finds_the_mount_point_of_the_temp_directory() {
        let temp_directory = std::fs::canonicalize(std::env::temp_dir()).unwrap();

        let mount_point = query_volume_mount_point(&temp_directory).unwrap();

        assert!(
            temp_directory.starts_with(&mount_point),
            "{}",
            mount_point.display()
        );
    }
}
