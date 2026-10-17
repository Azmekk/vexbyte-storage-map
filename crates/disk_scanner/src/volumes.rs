use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::platform;
use crate::volume_kind::{VolumeClassificationFacts, VolumeKind, classify_volume};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DriveType {
    Fixed,
    Removable,
    Network,
    Optical,
    Ram,
    Unknown,
}

/// A mounted volume with its space usage, as shown on the home screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VolumeSummary {
    /// Where the volume is mounted, for example `C:\`. This identifies the volume.
    pub mount_point: PathBuf,
    /// The user-visible volume label. Empty when the volume has none.
    pub label: String,
    /// For example `NTFS`, `exFAT` or `FAT32`.
    pub file_system_name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub drive_type: DriveType,
    /// What the volume really is. A [`VolumeKind::CloudVirtual`] volume is never scanned.
    pub kind: VolumeKind,
    /// The file system allows a fast scan. On Windows that is an NTFS volume, read through its
    /// Master File Table, which additionally needs administrator rights; see
    /// [`crate::fast_scan_availability`]. Always false for a cloud virtual volume.
    pub supports_fast_scan: bool,
}

impl VolumeSummary {
    #[must_use]
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.free_bytes)
    }

    #[must_use]
    pub fn is_cloud_virtual(&self) -> bool {
        self.kind.is_cloud_virtual()
    }
}

/// What a platform backend reports about one mounted volume.
pub(crate) struct VolumeDetails {
    pub(crate) label: String,
    pub(crate) file_system_name: String,
    pub(crate) total_bytes: u64,
    pub(crate) free_bytes: u64,
    pub(crate) drive_type: DriveType,
    pub(crate) serial_number: u32,
    /// The NT device behind the drive letter, when it could be read.
    pub(crate) dos_device_name: Option<String>,
}

impl VolumeDetails {
    pub(crate) fn kind(&self) -> VolumeKind {
        classify_volume(&VolumeClassificationFacts {
            drive_type: self.drive_type,
            label: &self.label,
            serial_number: self.serial_number,
            dos_device_name: self.dos_device_name.as_deref(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VolumeListError {
    #[error("could not list the mounted volumes of this computer")]
    MountPointListFailed { source: io::Error },
}

/// Lists every mounted volume that is ready to be read.
///
/// Volumes that are not ready, such as an empty optical drive or a disconnected network share,
/// are skipped instead of failing the whole list.
///
/// # Errors
///
/// Fails only when the operating system cannot report which volumes are mounted.
pub fn list_volumes() -> Result<Vec<VolumeSummary>, VolumeListError> {
    let _prompt_suppression = platform::MediaErrorPromptSuppression::for_current_thread();

    let mount_points = platform::list_volume_mount_points()
        .map_err(|source| VolumeListError::MountPointListFailed { source })?;

    let volumes = mount_points
        .into_iter()
        .filter_map(read_volume_summary_or_skip)
        .collect();

    Ok(volumes)
}

/// The mount point of the volume the operating system runs from, for example `C:\`.
#[must_use]
pub fn system_volume_mount_point() -> Option<PathBuf> {
    platform::system_volume_mount_point()
}

fn read_volume_summary_or_skip(mount_point: PathBuf) -> Option<VolumeSummary> {
    match platform::query_volume_details(&mount_point) {
        Ok(Some(details)) => Some(volume_summary_from_details(mount_point, details)),
        Ok(None) => None,
        Err(error) if platform::is_volume_not_ready_error(&error) => {
            tracing::debug!(mount_point = %mount_point.display(), "skipping volume that is not ready");
            None
        }
        Err(error) => {
            tracing::warn!(
                mount_point = %mount_point.display(),
                %error,
                "skipping volume that could not be read"
            );
            None
        }
    }
}

fn volume_summary_from_details(mount_point: PathBuf, details: VolumeDetails) -> VolumeSummary {
    let kind = details.kind();
    let supports_fast_scan = !kind.is_cloud_virtual()
        && platform::file_system_supports_fast_scan(&details.file_system_name);

    VolumeSummary {
        mount_point,
        label: details.label,
        file_system_name: details.file_system_name,
        total_bytes: details.total_bytes,
        free_bytes: details.free_bytes,
        drive_type: details.drive_type,
        kind,
        supports_fast_scan,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn lists_at_least_one_volume_with_consistent_space() {
        let volumes = list_volumes().expect("listing volumes should succeed");

        assert!(!volumes.is_empty(), "expected at least one mounted volume");
        for volume in &volumes {
            assert!(
                volume.total_bytes >= volume.free_bytes,
                "{} reports more free than total space",
                volume.mount_point.display()
            );
            assert!(volume.mount_point.is_absolute());
        }
    }
}
