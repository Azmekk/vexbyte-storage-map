use std::path::PathBuf;

use mount_table::{DeviceNumber, MountEntry};

pub(super) fn sample_mount_entry(
    mount_point: &str,
    file_system_type: &str,
    source: &str,
    (major, minor): (u32, u32),
) -> MountEntry {
    MountEntry {
        mount_point: PathBuf::from(mount_point),
        root: PathBuf::from("/"),
        file_system_type: file_system_type.to_owned(),
        source: source.to_owned(),
        device: DeviceNumber { major, minor },
    }
}
