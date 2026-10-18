/// What kind of storage a file system type stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileSystemCategory {
    /// A file system stored on a block device: a disk, partition, USB stick or memory card.
    BlockDevice,
    /// A CD, DVD or Blu-ray image format.
    Optical,
    /// Files served from somewhere else: a network share, a cloud drive (rclone, gvfs) or the
    /// Windows drives inside WSL.
    Remote,
    /// Files that live only in memory.
    Memory,
    /// A kernel interface or a container layer, not something a person stores files on.
    Virtual,
}

const BLOCK_DEVICE_FILE_SYSTEMS: &[&str] = &[
    "ext2", "ext3", "ext4", "xfs", "btrfs", "bcachefs", "f2fs", "jfs", "reiserfs", "nilfs2", "zfs",
    "vfat", "msdos", "exfat", "ntfs", "ntfs3", "hfs", "hfsplus", "apfs", "ufs", "fuseblk",
];
const OPTICAL_FILE_SYSTEMS: &[&str] = &["iso9660", "udf"];
const REMOTE_FILE_SYSTEMS: &[&str] = &[
    "nfs",
    "nfs4",
    "cifs",
    "smb3",
    "smbfs",
    "9p",
    "virtiofs",
    "drvfs",
    "ceph",
    "glusterfs",
    "afs",
    "davfs",
];
const MEMORY_FILE_SYSTEMS: &[&str] = &["tmpfs", "ramfs"];
/// FUSE file systems are named `fuse.<program>`. Apart from `fuseblk` (NTFS and exFAT through
/// FUSE, listed above) they serve files from a program, which is almost always a network or
/// cloud service such as sshfs, rclone or gvfs.
const USER_SPACE_FILE_SYSTEM_PREFIX: &str = "fuse.";

/// Sorts a file system type from `/proc/self/mountinfo` into a [`FileSystemCategory`].
#[must_use]
pub fn categorize_file_system(file_system_type: &str) -> FileSystemCategory {
    if BLOCK_DEVICE_FILE_SYSTEMS.contains(&file_system_type) {
        FileSystemCategory::BlockDevice
    } else if OPTICAL_FILE_SYSTEMS.contains(&file_system_type) {
        FileSystemCategory::Optical
    } else if REMOTE_FILE_SYSTEMS.contains(&file_system_type)
        || file_system_type.starts_with(USER_SPACE_FILE_SYSTEM_PREFIX)
    {
        FileSystemCategory::Remote
    } else if MEMORY_FILE_SYSTEMS.contains(&file_system_type) {
        FileSystemCategory::Memory
    } else {
        FileSystemCategory::Virtual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_common_file_systems() {
        assert_eq!(
            categorize_file_system("ext4"),
            FileSystemCategory::BlockDevice
        );
        assert_eq!(
            categorize_file_system("btrfs"),
            FileSystemCategory::BlockDevice
        );
        assert_eq!(
            categorize_file_system("fuseblk"),
            FileSystemCategory::BlockDevice
        );
        assert_eq!(
            categorize_file_system("iso9660"),
            FileSystemCategory::Optical
        );
        assert_eq!(categorize_file_system("nfs4"), FileSystemCategory::Remote);
        assert_eq!(
            categorize_file_system("fuse.sshfs"),
            FileSystemCategory::Remote
        );
        assert_eq!(
            categorize_file_system("fuse.rclone"),
            FileSystemCategory::Remote
        );
        assert_eq!(categorize_file_system("9p"), FileSystemCategory::Remote);
        assert_eq!(categorize_file_system("tmpfs"), FileSystemCategory::Memory);
        assert_eq!(categorize_file_system("proc"), FileSystemCategory::Virtual);
        assert_eq!(
            categorize_file_system("overlay"),
            FileSystemCategory::Virtual
        );
        assert_eq!(
            categorize_file_system("squashfs"),
            FileSystemCategory::Virtual
        );
    }
}
