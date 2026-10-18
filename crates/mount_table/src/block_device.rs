use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::device_number::DeviceNumber;
use crate::mount_entry::MountEntry;

const SYSFS_BLOCK_DEVICES: &str = "/sys/dev/block";
const DEVICE_NODE_PREFIX: &str = "/dev/";

/// What `/sys` reports about the disk behind a mount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDeviceFacts {
    /// A spinning hard disk, where reading many files at once makes the head seek back and forth.
    pub is_rotational: bool,
    /// A USB drive, memory card or other disk the kernel marks as removable.
    pub is_removable: bool,
}

/// Looks up the disk behind `entry`. `None` when the mount is not backed by a block device the
/// kernel knows, for example a network share.
#[must_use]
pub fn block_device_facts(entry: &MountEntry) -> Option<BlockDeviceFacts> {
    let device_directory = sysfs_device_directory(entry)?;
    let disk_directory = disk_directory_of(&device_directory);

    let is_rotational = read_sysfs_flag(&disk_directory.join("queue/rotational"));
    let device_path_text = device_directory.to_string_lossy();
    let is_removable = read_sysfs_flag(&disk_directory.join("removable"))
        || device_path_text.contains("/usb")
        || device_path_text.contains("/mmc");

    Some(BlockDeviceFacts {
        is_rotational,
        is_removable,
    })
}

/// The `/sys/devices/...` folder of the block device behind the mount.
///
/// Btrfs and some other file systems report an anonymous device number in mountinfo, so the
/// device node named as the source is asked for its real number when the first lookup fails.
fn sysfs_device_directory(entry: &MountEntry) -> Option<PathBuf> {
    sysfs_directory_of(entry.device).or_else(|| {
        let source_path = entry.source.strip_prefix(DEVICE_NODE_PREFIX)?;
        let device_node = Path::new(DEVICE_NODE_PREFIX).join(source_path);
        let device_number = DeviceNumber::from_raw(std::fs::metadata(device_node).ok()?.rdev());
        sysfs_directory_of(device_number)
    })
}

fn sysfs_directory_of(device: DeviceNumber) -> Option<PathBuf> {
    std::fs::canonicalize(Path::new(SYSFS_BLOCK_DEVICES).join(device.to_string())).ok()
}

/// A partition's folder sits inside its disk's folder; the queue and removable flags belong to
/// the disk.
fn disk_directory_of(device_directory: &Path) -> PathBuf {
    let is_partition = device_directory.join("partition").exists();
    match device_directory.parent() {
        Some(disk_directory) if is_partition => disk_directory.to_path_buf(),
        _ => device_directory.to_path_buf(),
    }
}

fn read_sysfs_flag(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|content| content.trim() == "1")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn a_remote_mount_has_no_block_device() {
        let entry = MountEntry {
            mount_point: PathBuf::from("/mnt/share"),
            root: PathBuf::from("/"),
            file_system_type: "nfs4".to_owned(),
            source: "server:/export".to_owned(),
            device: DeviceNumber {
                major: 0,
                minor: 4_000_000,
            },
        };

        assert_eq!(block_device_facts(&entry), None);
    }
}
