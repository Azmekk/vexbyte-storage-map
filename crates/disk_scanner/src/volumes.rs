use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::platform;

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
    /// The file system allows a fast scan. On Windows that is an NTFS volume, read through its
    /// Master File Table, which additionally needs administrator rights; see
    /// [`crate::fast_scan_availability`].
    pub supports_fast_scan: bool,
}

impl VolumeSummary {
    #[must_use]
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.free_bytes)
    }
}

/// What a platform backend reports about one mounted volume.
pub(crate) struct VolumeDetails {
    pub(crate) label: String,
    pub(crate) file_system_name: String,
    pub(crate) total_bytes: u64,
    pub(crate) free_bytes: u64,
    pub(crate) drive_type: DriveType,
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
    let supports_fast_scan = platform::file_system_supports_fast_scan(&details.file_system_name);

    VolumeSummary {
        mount_point,
        label: details.label,
        file_system_name: details.file_system_name,
        total_bytes: details.total_bytes,
        free_bytes: details.free_bytes,
        drive_type: details.drive_type,
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
