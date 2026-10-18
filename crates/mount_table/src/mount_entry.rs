use std::path::PathBuf;

use crate::device_number::DeviceNumber;

/// One line of `/proc/self/mountinfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    /// Where the file system is mounted, for example `/home`.
    pub mount_point: PathBuf,
    /// The folder of the file system that appears at the mount point: `/` for a whole file
    /// system, something else for a bind mount or a btrfs subvolume.
    pub root: PathBuf,
    /// The kernel's file system name, for example `ext4`, `btrfs` or `fuse.sshfs`.
    pub file_system_type: String,
    /// What was mounted, for example `/dev/nvme0n1p2`, `server:/export` or `tmpfs`.
    pub source: String,
    /// The device number every file on this mount reports as `st_dev`.
    pub device: DeviceNumber,
}
