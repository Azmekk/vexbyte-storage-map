use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use mount_table::{FileSystemCategory, MountEntry, categorize_file_system, read_mount_table};

/// System locations whose mounts are plumbing, not drives a user would want to see.
const SYSTEM_MOUNT_LOCATIONS: &[&str] = &[
    "/proc",
    "/sys",
    "/dev",
    "/run",
    "/snap",
    "/var/lib/docker",
    "/var/lib/containers",
    "/boot/efi",
    "/efi",
    // WSL mounts the Windows GPU driver libraries here.
    "/usr/lib",
    // WSL's own plumbing (WSLg, other distributions) and Docker Desktop's integration mounts.
    "/mnt/wsl",
    "/mnt/wslg",
    "/Docker",
];

/// The xdg document portal: a FUSE view of files that sandboxed apps were given, not a drive.
const DOCUMENT_PORTAL_FILE_SYSTEM: &str = "fuse.portal";

/// Fedora, Arch and other distributions that use udisks2 mount USB sticks here, inside `/run`.
const REMOVABLE_MEDIA_LOCATION: &str = "/run/media";

/// Lists the mount point of every drive, partition, optical disc and network share a user would
/// recognize, for example `/`, `/home` or `/media/user/USB STICK`.
pub(crate) fn list_volume_mount_points() -> io::Result<Vec<PathBuf>> {
    let mount_entries = read_mount_table()?;
    // A bind mount of a single file is not a drive; only folders can be scanned.
    Ok(user_visible_mount_points(&mount_entries)
        .into_iter()
        .filter(|mount_point| mount_point.is_dir())
        .collect())
}

pub(super) fn user_visible_mount_points(mount_entries: &[MountEntry]) -> Vec<PathBuf> {
    let mut listed_entries: Vec<&MountEntry> = Vec::new();

    for entry in visible_mount_entries(mount_entries) {
        let is_user_volume = is_user_volume_file_system(&entry.file_system_type)
            && !is_system_mount_location(&entry.mount_point);
        if is_user_volume && !is_already_listed(entry, &listed_entries) {
            listed_entries.push(entry);
        }
    }

    listed_entries
        .into_iter()
        .map(|entry| entry.mount_point.clone())
        .collect()
}

/// The entries that are not hidden by a later mount on the same mount point, in mount order.
fn visible_mount_entries(mount_entries: &[MountEntry]) -> impl Iterator<Item = &MountEntry> {
    let last_index_by_mount_point: HashMap<&Path, usize> = mount_entries
        .iter()
        .enumerate()
        .map(|(index, entry)| (entry.mount_point.as_path(), index))
        .collect();

    mount_entries
        .iter()
        .enumerate()
        .filter(move |(index, entry)| {
            last_index_by_mount_point.get(entry.mount_point.as_path()) == Some(index)
        })
        .map(|(_, entry)| entry)
}

fn is_user_volume_file_system(file_system_type: &str) -> bool {
    file_system_type != DOCUMENT_PORTAL_FILE_SYSTEM
        && matches!(
            categorize_file_system(file_system_type),
            FileSystemCategory::BlockDevice
                | FileSystemCategory::Optical
                | FileSystemCategory::Remote
        )
}

fn is_system_mount_location(mount_point: &Path) -> bool {
    if mount_point.starts_with(REMOVABLE_MEDIA_LOCATION) {
        return false;
    }
    SYSTEM_MOUNT_LOCATIONS
        .iter()
        .any(|location| mount_point.starts_with(location))
}

/// A bind mount shows a folder of a file system that is already listed, so it adds nothing.
/// Its root (the folder of the file system it shows) lies inside the root of the earlier mount.
/// Btrfs subvolumes such as `/` and `/home` share one device number but have separate roots,
/// so both stay listed.
fn is_already_listed(entry: &MountEntry, listed_entries: &[&MountEntry]) -> bool {
    listed_entries.iter().any(|listed_entry| {
        listed_entry.device == entry.device && entry.root.starts_with(&listed_entry.root)
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::super::sample_mount_entries::sample_mount_entry;
    use super::*;

    fn listed_mount_points(mount_entries: &[MountEntry]) -> Vec<String> {
        user_visible_mount_points(mount_entries)
            .into_iter()
            .map(|mount_point| mount_point.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn lists_disks_optical_discs_and_network_shares() {
        let entries = [
            sample_mount_entry("/", "ext4", "/dev/nvme0n1p2", (259, 2)),
            sample_mount_entry("/mnt/dvd", "iso9660", "/dev/sr0", (11, 0)),
            sample_mount_entry("/mnt/nas", "nfs4", "nas:/export", (0, 52)),
            sample_mount_entry("/media/user/USB", "vfat", "/dev/sdb1", (8, 17)),
        ];

        assert_eq!(
            listed_mount_points(&entries),
            ["/", "/mnt/dvd", "/mnt/nas", "/media/user/USB"]
        );
    }

    #[test]
    fn skips_memory_and_virtual_file_systems() {
        let entries = [
            sample_mount_entry("/", "ext4", "/dev/sda1", (8, 1)),
            sample_mount_entry("/tmp", "tmpfs", "tmpfs", (0, 30)),
            sample_mount_entry("/mnt/overlay", "overlay", "overlay", (0, 31)),
            sample_mount_entry("/sys/fs/cgroup", "cgroup2", "cgroup2", (0, 26)),
        ];

        assert_eq!(listed_mount_points(&entries), ["/"]);
    }

    #[test]
    fn skips_system_locations_but_keeps_removable_media_under_run() {
        let entries = [
            sample_mount_entry("/", "ext4", "/dev/sda2", (8, 2)),
            sample_mount_entry("/boot/efi", "vfat", "/dev/sda1", (8, 1)),
            sample_mount_entry("/snap/core22/1380", "squashfs", "/dev/loop0", (7, 0)),
            sample_mount_entry("/snap/firefox/1", "ext4", "/dev/loop1", (7, 1)),
            sample_mount_entry("/usr/lib/wsl/drivers", "9p", "drivers", (0, 60)),
            sample_mount_entry("/run/user/1000/doc", "fuse.portal", "portal", (0, 61)),
            sample_mount_entry("/var/lib/docker/overlay2/x", "ext4", "/dev/sda2", (8, 2)),
            sample_mount_entry("/run/media/user/STICK", "exfat", "/dev/sdc1", (8, 33)),
            sample_mount_entry(
                "/mnt/wslg/run/user/1000/doc",
                "fuse.portal",
                "portal",
                (0, 62),
            ),
            sample_mount_entry(
                "/mnt/wsl/docker-desktop/cli-tools",
                "iso9660",
                "/dev/sdb",
                (8, 16),
            ),
            sample_mount_entry("/Docker/host", "9p", "drvfs", (0, 63)),
        ];

        assert_eq!(
            listed_mount_points(&entries),
            ["/", "/run/media/user/STICK"]
        );
    }

    #[test]
    fn a_later_mount_on_the_same_mount_point_hides_the_earlier_one() {
        let entries = [
            sample_mount_entry("/", "ext4", "/dev/sda1", (8, 1)),
            sample_mount_entry("/mnt/data", "ext4", "/dev/sdb1", (8, 17)),
            sample_mount_entry("/mnt/data", "tmpfs", "tmpfs", (0, 40)),
        ];

        assert_eq!(listed_mount_points(&entries), ["/"]);
    }

    #[test]
    fn skips_bind_mounts_of_listed_file_systems() {
        let mut bind_mount = sample_mount_entry("/srv/share", "ext4", "/dev/sdb1", (8, 17));
        bind_mount.root = PathBuf::from("/projects/share");
        let entries = [
            sample_mount_entry("/", "ext4", "/dev/sda1", (8, 1)),
            sample_mount_entry("/mnt/data", "ext4", "/dev/sdb1", (8, 17)),
            bind_mount,
        ];

        assert_eq!(listed_mount_points(&entries), ["/", "/mnt/data"]);
    }

    #[test]
    fn keeps_btrfs_subvolumes_of_one_device() {
        let mut root_subvolume = sample_mount_entry("/", "btrfs", "/dev/nvme0n1p3", (0, 35));
        root_subvolume.root = PathBuf::from("/root");
        let mut home_subvolume = sample_mount_entry("/home", "btrfs", "/dev/nvme0n1p3", (0, 35));
        home_subvolume.root = PathBuf::from("/home");

        assert_eq!(
            listed_mount_points(&[root_subvolume, home_subvolume]),
            ["/", "/home"]
        );
    }

    #[test]
    fn lists_the_mounts_of_this_computer() {
        let mount_points = list_volume_mount_points().unwrap();

        assert!(
            mount_points
                .iter()
                .all(|mount_point| mount_point.is_absolute())
        );
    }
}
