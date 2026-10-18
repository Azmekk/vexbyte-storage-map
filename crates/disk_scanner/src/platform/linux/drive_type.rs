use mount_table::{FileSystemCategory, MountEntry, block_device_facts, categorize_file_system};

use crate::volumes::DriveType;

/// What kind of drive the file system of `entry` lives on. Reading `/sys` for the removable flag
/// is only needed for file systems on block devices.
pub(super) fn drive_type_of_mount(entry: &MountEntry) -> DriveType {
    match categorize_file_system(&entry.file_system_type) {
        FileSystemCategory::BlockDevice => {
            let is_removable = block_device_facts(entry).is_some_and(|facts| facts.is_removable);
            if is_removable {
                DriveType::Removable
            } else {
                DriveType::Fixed
            }
        }
        FileSystemCategory::Optical => DriveType::Optical,
        FileSystemCategory::Remote => DriveType::Network,
        FileSystemCategory::Memory => DriveType::Ram,
        FileSystemCategory::Virtual => DriveType::Unknown,
    }
}
