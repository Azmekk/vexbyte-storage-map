use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::platform;
use crate::volumes::DriveType;

/// What a volume really is, which decides whether the app may scan it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum VolumeKind {
    LocalDisk,
    Removable,
    Optical,
    Network,
    RamDisk,
    /// A virtual drive of a cloud or user-mode file system, such as Google Drive for desktop.
    /// Reading it is slow and downloads files, so it is never scanned.
    CloudVirtual,
    Unknown,
}

impl VolumeKind {
    #[must_use]
    pub fn is_cloud_virtual(self) -> bool {
        self == Self::CloudVirtual
    }
}

/// Google Drive for desktop gives every one of its volumes this serial number.
pub const GOOGLE_DRIVE_VOLUME_SERIAL_NUMBER: u32 = 0x1983_1116;

const GOOGLE_DRIVE_VOLUME_LABEL: &str = "Google Drive";
/// Every partition and virtual hard disk Windows mounts itself is a `HarddiskVolume` device.
const LOCAL_DISK_DEVICE_PREFIX: &str = r"\Device\HarddiskVolume";
/// A `subst` drive letter points at a folder (`\??\C:\folder`), not at a device. Its serial
/// number and label are those of the volume holding that folder, so those rules still apply.
const SUBSTITUTED_PATH_PREFIX: &str = r"\??\";

/// What the operating system reports about a volume, as far as classification needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeClassificationFacts<'facts> {
    pub drive_type: DriveType,
    pub label: &'facts str,
    pub serial_number: u32,
    /// The NT device the drive letter maps to, for example `\Device\HarddiskVolume3`. `None`
    /// when it could not be read, for example for a folder mount point.
    pub dos_device_name: Option<&'facts str>,
}

/// Decides what kind of volume the facts describe.
///
/// Cloud and user-mode file systems (Google Drive, Dokan, `WinFsp`, CBFS) report themselves as
/// fixed disks, so a fixed disk only counts as local when it is backed by a real disk volume.
#[must_use]
pub fn classify_volume(facts: &VolumeClassificationFacts<'_>) -> VolumeKind {
    match facts.drive_type {
        DriveType::Network => VolumeKind::Network,
        DriveType::Optical => VolumeKind::Optical,
        DriveType::Removable => VolumeKind::Removable,
        DriveType::Ram => VolumeKind::RamDisk,
        DriveType::Unknown => VolumeKind::Unknown,
        DriveType::Fixed if is_cloud_virtual_fixed_volume(facts) => VolumeKind::CloudVirtual,
        DriveType::Fixed => VolumeKind::LocalDisk,
    }
}

fn is_cloud_virtual_fixed_volume(facts: &VolumeClassificationFacts<'_>) -> bool {
    let has_google_drive_serial = facts.serial_number == GOOGLE_DRIVE_VOLUME_SERIAL_NUMBER;
    let has_google_drive_label = facts.label.eq_ignore_ascii_case(GOOGLE_DRIVE_VOLUME_LABEL);
    let is_backed_by_other_device = facts.dos_device_name.is_some_and(|device_name| {
        !device_name.starts_with(LOCAL_DISK_DEVICE_PREFIX)
            && !device_name.starts_with(SUBSTITUTED_PATH_PREFIX)
    });

    has_google_drive_serial || has_google_drive_label || is_backed_by_other_device
}

/// Finds the volume that holds `path` and classifies it.
///
/// # Errors
///
/// Fails when the volume of `path` cannot be found or read.
pub fn volume_kind_for_path(path: &Path) -> io::Result<VolumeKind> {
    let (_mount_point, kind) = mount_point_and_kind_of_volume_holding(path)?;
    Ok(kind)
}

/// The mount point of the volume that holds `path`, for example `G:\`, and its kind.
pub(crate) fn mount_point_and_kind_of_volume_holding(
    path: &Path,
) -> io::Result<(PathBuf, VolumeKind)> {
    let absolute_path = std::path::absolute(path)?;
    let mount_point = platform::volume_mount_point_for_path(&absolute_path)?;

    let kind = match platform::query_volume_details(&mount_point)? {
        Some(details) => details.kind(),
        None => VolumeKind::Unknown,
    };

    Ok((mount_point, kind))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn fixed_disk_facts() -> VolumeClassificationFacts<'static> {
        VolumeClassificationFacts {
            drive_type: DriveType::Fixed,
            label: "Windows",
            serial_number: 0x1234_5678,
            dos_device_name: Some(r"\Device\HarddiskVolume3"),
        }
    }

    #[test]
    fn a_fixed_disk_volume_is_local() {
        assert_eq!(classify_volume(&fixed_disk_facts()), VolumeKind::LocalDisk);
    }

    #[test]
    fn google_drive_is_recognized_by_device_serial_or_label() {
        let by_device = VolumeClassificationFacts {
            dos_device_name: Some(r"\Device\Volume{f5ae2bcb-da01-3bf2-8935-408102040811}"),
            ..fixed_disk_facts()
        };
        let by_serial = VolumeClassificationFacts {
            serial_number: GOOGLE_DRIVE_VOLUME_SERIAL_NUMBER,
            ..fixed_disk_facts()
        };
        let by_label = VolumeClassificationFacts {
            label: "google drive",
            ..fixed_disk_facts()
        };

        assert_eq!(classify_volume(&by_device), VolumeKind::CloudVirtual);
        assert_eq!(classify_volume(&by_serial), VolumeKind::CloudVirtual);
        assert_eq!(classify_volume(&by_label), VolumeKind::CloudVirtual);
    }

    #[test]
    fn other_user_mode_file_systems_are_cloud_virtual() {
        let dokan_volume = VolumeClassificationFacts {
            label: "Dokan",
            dos_device_name: Some(r"\Device\Volume{d6cc17c5-1734-4085-bce7-964f1e9f5de9}"),
            ..fixed_disk_facts()
        };

        assert_eq!(classify_volume(&dokan_volume), VolumeKind::CloudVirtual);
    }

    #[test]
    fn an_unreadable_device_name_does_not_make_a_volume_cloud_virtual() {
        let facts = VolumeClassificationFacts {
            dos_device_name: None,
            ..fixed_disk_facts()
        };

        assert_eq!(classify_volume(&facts), VolumeKind::LocalDisk);
    }

    #[test]
    fn a_substituted_drive_keeps_the_kind_of_its_target_volume() {
        let substituted_local_folder = VolumeClassificationFacts {
            dos_device_name: Some(r"\??\C:\Projects"),
            ..fixed_disk_facts()
        };
        let substituted_google_drive_folder = VolumeClassificationFacts {
            dos_device_name: Some(r"\??\G:\My Drive"),
            serial_number: GOOGLE_DRIVE_VOLUME_SERIAL_NUMBER,
            ..fixed_disk_facts()
        };

        assert_eq!(
            classify_volume(&substituted_local_folder),
            VolumeKind::LocalDisk
        );
        assert_eq!(
            classify_volume(&substituted_google_drive_folder),
            VolumeKind::CloudVirtual
        );
    }

    #[test]
    fn other_drive_types_follow_the_drive_type() {
        let kind_of = |drive_type| {
            classify_volume(&VolumeClassificationFacts {
                drive_type,
                dos_device_name: Some(r"\Device\CdRom0"),
                serial_number: GOOGLE_DRIVE_VOLUME_SERIAL_NUMBER,
                ..fixed_disk_facts()
            })
        };

        assert_eq!(kind_of(DriveType::Network), VolumeKind::Network);
        assert_eq!(kind_of(DriveType::Optical), VolumeKind::Optical);
        assert_eq!(kind_of(DriveType::Removable), VolumeKind::Removable);
        assert_eq!(kind_of(DriveType::Ram), VolumeKind::RamDisk);
        assert_eq!(kind_of(DriveType::Unknown), VolumeKind::Unknown);
    }

    #[test]
    fn the_temp_directory_is_not_on_a_cloud_drive() {
        let kind = volume_kind_for_path(&std::env::temp_dir()).unwrap();

        assert!(!kind.is_cloud_virtual(), "{kind:?}");
    }
}
