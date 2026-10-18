use std::io;
use std::path::Path;

/// The size of a mounted file system and the space still available to this user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskSpace {
    pub total_bytes: u64,
    /// Space a normal user can still write to. Excludes the blocks ext4 reserves for root.
    pub free_bytes: u64,
}

/// Reads the space of the file system mounted at `mount_point` with `statvfs`.
///
/// # Errors
///
/// Fails when the file system cannot be queried, for example a network share that stopped
/// responding.
pub fn query_disk_space(mount_point: &Path) -> io::Result<DiskSpace> {
    let statistics = rustix::fs::statvfs(mount_point)?;
    let block_size = statistics.f_frsize;

    Ok(DiskSpace {
        total_bytes: statistics.f_blocks.saturating_mul(block_size),
        free_bytes: statistics.f_bavail.saturating_mul(block_size),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn the_root_file_system_has_space() {
        let space = query_disk_space(Path::new("/")).unwrap();

        assert!(space.total_bytes > 0);
        assert!(space.free_bytes <= space.total_bytes);
    }
}
