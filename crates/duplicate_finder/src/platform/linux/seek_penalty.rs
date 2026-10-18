use std::io;
use std::path::Path;

use mount_table::block_device_facts;

use super::mount_lookup::read_mount_entry_at;

/// Asks `/sys` whether the disk behind the file system mounted at `mount_point` spins. Fails
/// with `NotFound` when no block device backs the mount (network shares, tmpfs, FUSE), which
/// the caller treats as an unknown drive.
pub(crate) fn query_volume_incurs_seek_penalty(mount_point: &Path) -> io::Result<bool> {
    let entry = read_mount_entry_at(mount_point)?;

    block_device_facts(&entry)
        .map(|facts| facts.is_rotational)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not backed by a block device", mount_point.display()),
            )
        })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::super::volume_mount_point::query_volume_mount_point;
    use super::*;

    #[test]
    fn answers_for_the_volume_holding_the_temp_directory() {
        let mount_point = query_volume_mount_point(&std::env::temp_dir()).unwrap();

        match query_volume_incurs_seek_penalty(&mount_point) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                eprintln!("no block device behind the temp directory: {error}");
            }
            Err(error) => panic!("unexpected seek penalty error: {error:?}"),
        }
    }
}
