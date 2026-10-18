use std::io;
use std::path::Path;

use mount_table::{MountEntry, mount_containing, read_mount_table};

/// The visible mount whose mount point is exactly `mount_point`, for example `/home`.
pub(super) fn read_mount_entry_at(mount_point: &Path) -> io::Result<MountEntry> {
    let mount_table = read_mount_table()?;

    mount_containing(&mount_table, mount_point)
        .filter(|entry| entry.mount_point == mount_point)
        .cloned()
        .ok_or_else(|| not_a_mount_point_error(mount_point))
}

/// The visible mount holding the absolute, link-free `path`.
pub(super) fn read_mount_entry_containing(path: &Path) -> io::Result<MountEntry> {
    let mount_table = read_mount_table()?;

    mount_containing(&mount_table, path)
        .cloned()
        .ok_or_else(|| not_a_mount_point_error(path))
}

fn not_a_mount_point_error(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("no mounted file system found for {}", path.display()),
    )
}
