use std::path::Path;

use crate::mount_entry::MountEntry;

/// The mount that holds the absolute `path`: the one with the longest mount point that is a
/// prefix of it. When several file systems are stacked on the same mount point, the last one
/// mounted is the visible one.
#[must_use]
pub fn mount_containing<'entries>(
    entries: &'entries [MountEntry],
    path: &Path,
) -> Option<&'entries MountEntry> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| path.starts_with(&entry.mount_point))
        .max_by_key(|(mount_order, entry)| (entry.mount_point.components().count(), *mount_order))
        .map(|(_, entry)| entry)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;

    use super::*;
    use crate::device_number::DeviceNumber;

    fn mount(mount_point: &str, file_system_type: &str) -> MountEntry {
        MountEntry {
            mount_point: PathBuf::from(mount_point),
            root: PathBuf::from("/"),
            file_system_type: file_system_type.to_owned(),
            source: "source".to_owned(),
            device: DeviceNumber { major: 0, minor: 0 },
        }
    }

    #[test]
    fn picks_the_deepest_mount_point() {
        let entries = [mount("/", "ext4"), mount("/home", "btrfs")];

        let found = mount_containing(&entries, Path::new("/home/martin/Videos")).unwrap();

        assert_eq!(found.mount_point, PathBuf::from("/home"));
    }

    #[test]
    fn matches_whole_components_only() {
        let entries = [mount("/", "ext4"), mount("/home", "btrfs")];

        let found = mount_containing(&entries, Path::new("/homework")).unwrap();

        assert_eq!(found.mount_point, PathBuf::from("/"));
    }

    #[test]
    fn the_last_mount_on_a_stacked_mount_point_wins() {
        let entries = [
            mount("/", "ext4"),
            mount("/mnt", "tmpfs"),
            mount("/mnt", "xfs"),
        ];

        let found = mount_containing(&entries, Path::new("/mnt/file")).unwrap();

        assert_eq!(found.file_system_type, "xfs");
    }
}
