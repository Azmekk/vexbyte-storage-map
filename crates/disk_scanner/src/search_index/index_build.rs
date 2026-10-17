use std::path::PathBuf;

use super::volume_search_index::VolumeSearchIndex;
use crate::cancellation::CancellationToken;
use crate::fast_scan_availability::{FastScanAvailability, fast_scan_availability};
use crate::scan::{ScanError, ScanTarget, scan};
use crate::scan_method::ScanMethod;
use crate::scan_progress::ScanProgressReporter;
use crate::volume_kind::VolumeKind;
use crate::volumes::VolumeSummary;

#[derive(Debug, thiserror::Error)]
pub enum SearchIndexBuildError {
    #[error("{} is not a local disk ({kind:?}) and is not indexed", mount_point.display())]
    VolumeNotIndexable {
        mount_point: PathBuf,
        kind: VolumeKind,
    },
    #[error("could not read {}", mount_point.display())]
    ScanFailed {
        mount_point: PathBuf,
        source: ScanError,
    },
}

/// The scan method an index build uses: the fast MFT read when this process may use it on the
/// volume, otherwise the folder walk.
#[must_use]
pub fn search_index_scan_method(volume: &VolumeSummary) -> ScanMethod {
    match fast_scan_availability(volume) {
        FastScanAvailability::Available => ScanMethod::MasterFileTable,
        FastScanAvailability::RequiresElevation | FastScanAvailability::UnsupportedFileSystem => {
            ScanMethod::FolderWalk
        }
    }
}

/// Reads a whole local volume read-only and prepares it for searching.
///
/// Blocks for seconds (MFT) to minutes (folder walk on a large drive); call it on a background
/// thread. When the MFT read fails, the volume is read once more with the folder walk. Only
/// [`VolumeKind::LocalDisk`] volumes are indexed, so a cloud drive is never read.
///
/// # Errors
///
/// Fails for volumes that are not local disks, when the volume cannot be read and when the build
/// is cancelled.
pub fn build_volume_search_index(
    volume: &VolumeSummary,
    progress_reporter: &dyn ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<VolumeSearchIndex, SearchIndexBuildError> {
    if volume.kind != VolumeKind::LocalDisk {
        return Err(SearchIndexBuildError::VolumeNotIndexable {
            mount_point: volume.mount_point.clone(),
            kind: volume.kind,
        });
    }

    let target = ScanTarget::Volume {
        mount_point: volume.mount_point.clone(),
    };
    let preferred_method = search_index_scan_method(volume);

    let scan_result = match scan(
        target.clone(),
        preferred_method,
        progress_reporter,
        cancellation,
    ) {
        Err(error)
            if preferred_method == ScanMethod::MasterFileTable
                && !matches!(error, ScanError::Cancelled) =>
        {
            tracing::warn!(
                mount_point = %volume.mount_point.display(),
                %error,
                "the MFT read for the search index failed; reading the volume with the folder walk"
            );
            scan(
                target,
                ScanMethod::FolderWalk,
                progress_reporter,
                cancellation,
            )
        }
        other => other,
    };

    let scan_result = scan_result.map_err(|source| SearchIndexBuildError::ScanFailed {
        mount_point: volume.mount_point.clone(),
        source,
    })?;

    Ok(VolumeSearchIndex::from_file_tree(
        volume.mount_point.clone(),
        scan_result.tree,
        scan_result.summary.method,
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::scan_progress::NoProgressReporting;
    use crate::volumes::DriveType;

    fn volume_of_kind(kind: VolumeKind) -> VolumeSummary {
        VolumeSummary {
            mount_point: PathBuf::from(r"G:\"),
            label: "Google Drive".to_owned(),
            file_system_name: "FAT32".to_owned(),
            total_bytes: 0,
            free_bytes: 0,
            drive_type: DriveType::Fixed,
            kind,
            supports_fast_scan: false,
        }
    }

    #[test]
    fn refuses_every_volume_that_is_not_a_local_disk() {
        for kind in [
            VolumeKind::CloudVirtual,
            VolumeKind::Network,
            VolumeKind::Removable,
            VolumeKind::Optical,
            VolumeKind::RamDisk,
            VolumeKind::Unknown,
        ] {
            let result = build_volume_search_index(
                &volume_of_kind(kind),
                &NoProgressReporting,
                &CancellationToken::new(),
            );

            assert!(
                matches!(result, Err(SearchIndexBuildError::VolumeNotIndexable { kind: refused, .. }) if refused == kind),
                "{kind:?} must not be indexed"
            );
        }
    }

    #[test]
    fn a_volume_without_fast_scan_support_uses_the_folder_walk() {
        assert_eq!(
            search_index_scan_method(&volume_of_kind(VolumeKind::LocalDisk)),
            ScanMethod::FolderWalk
        );
    }
}
