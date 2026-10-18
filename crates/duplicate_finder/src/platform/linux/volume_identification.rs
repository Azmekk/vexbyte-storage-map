use std::io;
use std::path::Path;

use mount_table::{
    FileSystemCategory, block_device_facts, categorize_file_system, volume_label_of_source,
};

use super::mount_lookup::read_mount_entry_at;
use crate::cloud_drive_guard::VolumeIdentification;

/// Reads what the cloud drive check needs about the file system mounted at `mount_point`. Linux
/// has no DOS device names or volume serial numbers, so only the label rule can apply.
pub(crate) fn query_volume_identification(mount_point: &Path) -> io::Result<VolumeIdentification> {
    let entry = read_mount_entry_at(mount_point)?;

    let is_on_block_device =
        categorize_file_system(&entry.file_system_type) == FileSystemCategory::BlockDevice;
    let is_removable = block_device_facts(&entry).is_some_and(|facts| facts.is_removable);

    Ok(VolumeIdentification {
        is_fixed_drive: is_on_block_device && !is_removable,
        label: volume_label_of_source(&entry.source).unwrap_or_default(),
        serial_number: 0,
        dos_device_name: None,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::super::volume_mount_point::query_volume_mount_point;
    use super::*;

    #[test]
    fn identifies_the_volume_of_the_temp_directory() {
        let mount_point = query_volume_mount_point(&std::env::temp_dir()).unwrap();

        let identification = query_volume_identification(&mount_point).unwrap();

        assert_eq!(identification.dos_device_name, None);
        assert_eq!(identification.serial_number, 0);
    }
}
