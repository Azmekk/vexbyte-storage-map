use std::io;
use std::path::{Path, PathBuf};

use mount_table::{
    DiskSpace, MountEntry, mount_containing, query_disk_space, read_mount_table,
    volume_label_of_source,
};
use rustix::io::Errno;

use super::drive_type::drive_type_of_mount;
use crate::volumes::VolumeDetails;

/// Reads label, file system, space and drive type of the file system mounted at `mount_point`.
///
/// Returns `Ok(None)` when nothing is mounted there.
pub(crate) fn query_volume_details(mount_point: &Path) -> io::Result<Option<VolumeDetails>> {
    let mount_entries = read_mount_table()?;
    let Some(entry) = mount_entry_at(&mount_entries, mount_point) else {
        return Ok(None);
    };

    let DiskSpace {
        total_bytes,
        free_bytes,
    } = query_disk_space(mount_point)?;

    Ok(Some(VolumeDetails {
        label: volume_label_of_source(&entry.source).unwrap_or_default(),
        file_system_name: entry.file_system_type.clone(),
        total_bytes,
        free_bytes,
        drive_type: drive_type_of_mount(entry),
        serial_number: 0,
        dos_device_name: None,
    }))
}

/// The entry mounted at exactly `mount_point`. When several file systems are stacked on the
/// same mount point, the last one mounted is the one that is visible.
pub(super) fn mount_entry_at<'entries>(
    mount_entries: &'entries [MountEntry],
    mount_point: &Path,
) -> Option<&'entries MountEntry> {
    mount_entries
        .iter()
        .rev()
        .find(|entry| entry.mount_point == mount_point)
}

/// True for the errors Linux reports for a drive without media or an unreachable network mount.
pub(crate) fn is_volume_not_ready_error(error: &io::Error) -> bool {
    const NOT_READY_ERRORS: [Errno; 4] = [
        Errno::NOMEDIUM,
        Errno::NOTCONN,
        Errno::HOSTDOWN,
        Errno::STALE,
    ];

    Errno::from_io_error(error).is_some_and(|errno| NOT_READY_ERRORS.contains(&errno))
}

/// No Linux file system has a table that can be read faster than walking its folders.
pub(crate) fn file_system_supports_fast_scan(_file_system_name: &str) -> bool {
    false
}

/// Finds the mount point of the file system that contains `path`, for example `/` or
/// `/media/user/USB STICK`.
pub(crate) fn volume_mount_point_for_path(path: &Path) -> io::Result<PathBuf> {
    let resolved_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mount_entries = read_mount_table()?;

    mount_containing(&mount_entries, &resolved_path)
        .map(|entry| entry.mount_point.clone())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "no mounted file system contains {}",
                    resolved_path.display()
                ),
            )
        })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::super::sample_mount_entries::sample_mount_entry;
    use super::*;

    #[test]
    fn the_last_entry_on_a_mount_point_wins() {
        let entries = [
            sample_mount_entry("/mnt/data", "ext4", "/dev/sdb1", (8, 17)),
            sample_mount_entry("/", "ext4", "/dev/sda1", (8, 1)),
            sample_mount_entry("/mnt/data", "tmpfs", "tmpfs", (0, 40)),
        ];

        let entry = mount_entry_at(&entries, Path::new("/mnt/data")).unwrap();

        assert_eq!(entry.file_system_type, "tmpfs");
        assert!(mount_entry_at(&entries, Path::new("/mnt")).is_none());
    }

    #[test]
    fn recognizes_the_not_ready_errors() {
        let no_medium = io::Error::from_raw_os_error(Errno::NOMEDIUM.raw_os_error());
        let stale = io::Error::from_raw_os_error(Errno::STALE.raw_os_error());
        let access_denied = io::Error::from_raw_os_error(Errno::ACCESS.raw_os_error());

        assert!(is_volume_not_ready_error(&no_medium));
        assert!(is_volume_not_ready_error(&stale));
        assert!(!is_volume_not_ready_error(&access_denied));
        assert!(!is_volume_not_ready_error(&io::Error::other(
            "not an os error"
        )));
    }

    #[test]
    fn finds_the_volume_of_the_temp_directory() {
        let temp_directory = std::fs::canonicalize(std::env::temp_dir()).unwrap();

        let mount_point = volume_mount_point_for_path(&temp_directory).unwrap();

        assert!(temp_directory.starts_with(&mount_point));
    }

    #[test]
    fn reads_the_details_of_the_root_file_system() {
        let details = query_volume_details(Path::new("/")).unwrap().unwrap();

        assert_ne!(details.file_system_name, "");
        assert!(details.total_bytes >= details.free_bytes);
    }
}
