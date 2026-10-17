use std::fmt;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::windows_api::{
    CriticalErrorDialogSuppression, DiskSpace, VolumeInformation, is_drive_not_ready_error,
    query_disk_space, query_drive_type, query_logical_drive_letters, query_volume_information,
};

/// The letter a volume is mounted at, always uppercase, for example `C`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type,
)]
#[serde(transparent)]
pub struct DriveLetter(char);

impl DriveLetter {
    /// Returns `None` unless `letter` is an ASCII letter.
    #[must_use]
    pub fn new(letter: char) -> Option<Self> {
        letter
            .is_ascii_alphabetic()
            .then(|| Self(letter.to_ascii_uppercase()))
    }

    #[must_use]
    pub fn as_char(self) -> char {
        self.0
    }

    /// The root directory of the volume, for example `C:\`.
    #[must_use]
    pub fn root_path(self) -> String {
        format!("{}:\\", self.0)
    }
}

impl fmt::Display for DriveLetter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:", self.0)
    }
}

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
    pub drive_letter: DriveLetter,
    /// The user-visible volume label. Empty when the volume has none.
    pub label: String,
    /// For example `NTFS`, `exFAT` or `FAT32`.
    pub file_system_name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub drive_type: DriveType,
    pub supports_master_file_table_scan: bool,
}

impl VolumeSummary {
    #[must_use]
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.free_bytes)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VolumeListError {
    #[error("could not query the logical drives of this computer")]
    LogicalDrivesQueryFailed { source: windows::core::Error },
}

#[derive(Debug, thiserror::Error)]
enum VolumeQueryError {
    #[error("could not read the volume information of {drive_letter}")]
    VolumeInformationQueryFailed {
        drive_letter: DriveLetter,
        source: windows::core::Error,
    },
    #[error("could not read the free space of {drive_letter}")]
    DiskSpaceQueryFailed {
        drive_letter: DriveLetter,
        source: windows::core::Error,
    },
}

impl VolumeQueryError {
    fn is_drive_not_ready(&self) -> bool {
        match self {
            Self::VolumeInformationQueryFailed { source, .. }
            | Self::DiskSpaceQueryFailed { source, .. } => is_drive_not_ready_error(source),
        }
    }
}

/// Lists every mounted volume that is ready to be read.
///
/// Drives that are not ready, such as an empty optical drive or a disconnected network share,
/// are skipped instead of failing the whole list.
///
/// # Errors
///
/// Fails only when Windows cannot report which drive letters are in use.
pub fn list_volumes() -> Result<Vec<VolumeSummary>, VolumeListError> {
    let _dialog_suppression = CriticalErrorDialogSuppression::for_current_thread();

    let drive_letters = query_logical_drive_letters()
        .map_err(|source| VolumeListError::LogicalDrivesQueryFailed { source })?;

    let volumes = drive_letters
        .into_iter()
        .filter_map(read_volume_summary_or_skip)
        .collect();

    Ok(volumes)
}

fn read_volume_summary_or_skip(drive_letter: DriveLetter) -> Option<VolumeSummary> {
    match read_volume_summary(drive_letter) {
        Ok(volume_summary) => volume_summary,
        Err(error) if error.is_drive_not_ready() => {
            tracing::debug!(%drive_letter, "skipping drive that is not ready");
            None
        }
        Err(error) => {
            tracing::warn!(%drive_letter, %error, "skipping drive that could not be read");
            None
        }
    }
}

fn read_volume_summary(
    drive_letter: DriveLetter,
) -> Result<Option<VolumeSummary>, VolumeQueryError> {
    let root_path = drive_letter.root_path();

    let Some(drive_type) = query_drive_type(&root_path) else {
        return Ok(None);
    };

    let VolumeInformation {
        label,
        file_system_name,
    } = query_volume_information(&root_path).map_err(|source| {
        VolumeQueryError::VolumeInformationQueryFailed {
            drive_letter,
            source,
        }
    })?;

    let DiskSpace {
        total_bytes,
        free_bytes,
    } = query_disk_space(&root_path).map_err(|source| VolumeQueryError::DiskSpaceQueryFailed {
        drive_letter,
        source,
    })?;

    let supports_master_file_table_scan = file_system_name.eq_ignore_ascii_case("NTFS");

    Ok(Some(VolumeSummary {
        drive_letter,
        label,
        file_system_name,
        total_bytes,
        free_bytes,
        drive_type,
        supports_master_file_table_scan,
    }))
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
                volume.drive_letter
            );
        }
    }

    #[test]
    fn drive_letter_is_uppercased_and_formats_root_path() {
        let drive_letter = DriveLetter::new('d').unwrap();

        assert_eq!(drive_letter.as_char(), 'D');
        assert_eq!(drive_letter.root_path(), "D:\\");
        assert_eq!(drive_letter.to_string(), "D:");
    }

    #[test]
    fn drive_letter_rejects_non_letters() {
        assert_eq!(DriveLetter::new('1'), None);
        assert_eq!(DriveLetter::new('é'), None);
    }
}
