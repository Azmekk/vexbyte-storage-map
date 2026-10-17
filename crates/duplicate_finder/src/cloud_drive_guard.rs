//! Keeps the duplicate finder off cloud virtual drives such as Google Drive for desktop, where
//! hashing a file downloads it.
//!
//! The same rules live in `disk_scanner::volume_kind`. They are duplicated (about 30 lines) so
//! this crate stays independent of the scanner; keep both in step.

use std::path::Path;

use crate::platform::{query_volume_identification, query_volume_mount_point};

/// Google Drive for desktop gives every one of its volumes this serial number.
const GOOGLE_DRIVE_VOLUME_SERIAL_NUMBER: u32 = 0x1983_1116;
const GOOGLE_DRIVE_VOLUME_LABEL: &str = "Google Drive";
/// Every partition and virtual hard disk Windows mounts itself is a `HarddiskVolume` device.
const LOCAL_DISK_DEVICE_PREFIX: &str = r"\Device\HarddiskVolume";
/// A `subst` drive letter points at a folder (`\??\C:\folder`), not at a device. Its serial
/// number and label are those of the volume holding that folder, so those rules still apply.
const SUBSTITUTED_PATH_PREFIX: &str = r"\??\";

/// What the operating system reports about a volume, as far as the cloud check needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VolumeIdentification {
    pub(crate) is_fixed_drive: bool,
    pub(crate) label: String,
    pub(crate) serial_number: u32,
    /// The NT device the drive letter maps to, for example `\Device\HarddiskVolume3`. `None`
    /// when it could not be read, for example for a folder mount point.
    pub(crate) dos_device_name: Option<String>,
}

/// Cloud and user-mode file systems (Google Drive, Dokan, `WinFsp`, CBFS) report themselves as
/// fixed disks, so a fixed disk only counts as local when it is backed by a real disk volume.
/// Network, removable and optical drives are never cloud virtual drives here.
pub(crate) fn is_cloud_virtual_volume(identification: &VolumeIdentification) -> bool {
    if !identification.is_fixed_drive {
        return false;
    }

    let has_google_drive_serial = identification.serial_number == GOOGLE_DRIVE_VOLUME_SERIAL_NUMBER;
    let has_google_drive_label = identification
        .label
        .eq_ignore_ascii_case(GOOGLE_DRIVE_VOLUME_LABEL);
    let is_backed_by_other_device =
        identification
            .dos_device_name
            .as_deref()
            .is_some_and(|device_name| {
                !device_name.starts_with(LOCAL_DISK_DEVICE_PREFIX)
                    && !device_name.starts_with(SUBSTITUTED_PATH_PREFIX)
            });

    has_google_drive_serial || has_google_drive_label || is_backed_by_other_device
}

/// True when `path` lies on a cloud virtual drive. A volume that cannot be identified counts as
/// local: the folder checks that follow report why it cannot be read.
pub(crate) fn is_on_cloud_virtual_volume(path: &Path) -> bool {
    let identification = query_volume_mount_point(path)
        .and_then(|mount_point| query_volume_identification(&mount_point));

    match identification {
        Ok(identification) => is_cloud_virtual_volume(&identification),
        Err(error) => {
            tracing::debug!(path = %path.display(), %error, "volume could not be identified");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn local_disk() -> VolumeIdentification {
        VolumeIdentification {
            is_fixed_drive: true,
            label: "Data".to_owned(),
            serial_number: 0x1234_5678,
            dos_device_name: Some(r"\Device\HarddiskVolume6".to_owned()),
        }
    }

    #[test]
    fn a_local_disk_is_not_a_cloud_drive() {
        assert!(!is_cloud_virtual_volume(&local_disk()));
    }

    #[test]
    fn google_drive_is_recognized_by_device_serial_or_label() {
        let by_device = VolumeIdentification {
            dos_device_name: Some(
                r"\Device\Volume{f5ae2bcb-da01-3bf2-8935-408102040811}".to_owned(),
            ),
            ..local_disk()
        };
        let by_serial = VolumeIdentification {
            serial_number: GOOGLE_DRIVE_VOLUME_SERIAL_NUMBER,
            ..local_disk()
        };
        let by_label = VolumeIdentification {
            label: "GOOGLE DRIVE".to_owned(),
            ..local_disk()
        };

        assert!(is_cloud_virtual_volume(&by_device));
        assert!(is_cloud_virtual_volume(&by_serial));
        assert!(is_cloud_virtual_volume(&by_label));
    }

    #[test]
    fn an_unreadable_or_substituted_device_name_does_not_count() {
        let unreadable = VolumeIdentification {
            dos_device_name: None,
            ..local_disk()
        };
        let substituted = VolumeIdentification {
            dos_device_name: Some(r"\??\C:\Projects".to_owned()),
            ..local_disk()
        };

        assert!(!is_cloud_virtual_volume(&unreadable));
        assert!(!is_cloud_virtual_volume(&substituted));
    }

    #[test]
    fn drives_that_are_not_fixed_are_not_cloud_drives() {
        let removable = VolumeIdentification {
            is_fixed_drive: false,
            dos_device_name: Some(r"\Device\CdRom0".to_owned()),
            ..local_disk()
        };

        assert!(!is_cloud_virtual_volume(&removable));
    }

    #[test]
    fn the_temp_directory_is_not_on_a_cloud_drive() {
        assert!(!is_on_cloud_virtual_volume(&std::env::temp_dir()));
    }
}
