use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::cancellation::CancellationToken;
use crate::fast_scan_availability::is_process_elevated;
use crate::file_tree::{FileTree, TreeTotals};
use crate::folder_walk::{FolderWalkError, scan_folder_with_walk};
use crate::mft::{DEFAULT_MFT_CHUNK_SIZE_BYTES, MftScanError, scan_volume_with_mft};
use crate::platform;
use crate::scan_method::ScanMethod;
use crate::scan_progress::{ScanPhase, ScanProgressReporter, ScanProgressUpdate};
use crate::volume_kind::{VolumeKind, mount_point_and_kind_of_volume_holding};

/// What to scan.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ScanTarget {
    /// A whole volume, identified by its mount point (for example `C:\`).
    Volume { mount_point: PathBuf },
    /// One folder and everything below it.
    Folder { path: PathBuf },
}

impl ScanTarget {
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        match self {
            Self::Volume { mount_point } => mount_point,
            Self::Folder { path } => path,
        }
    }
}

/// Tuning knobs for a scan. `ScanOptions::default()` is right for normal use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanOptions {
    /// How many bytes the MFT scanner reads per request.
    pub mft_chunk_size_bytes: usize,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            mft_chunk_size_bytes: DEFAULT_MFT_CHUNK_SIZE_BYTES,
        }
    }
}

/// The headline numbers of a finished scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub target: ScanTarget,
    pub method: ScanMethod,
    /// The root of the returned tree.
    pub root_path: PathBuf,
    pub duration_milliseconds: u64,
    /// Totals at the end of the scan. After a recycle action, `FileTree::totals` gives
    /// up-to-date numbers.
    pub totals: TreeTotals,
}

pub struct ScanResult {
    pub tree: FileTree,
    pub summary: ScanSummary,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("the scan was cancelled")]
    Cancelled,
    #[error("the fast scan needs administrator rights")]
    FastScanRequiresElevation,
    #[error("the volume of {} does not support the fast scan", path.display())]
    FastScanUnsupported { path: PathBuf },
    #[error("could not find the volume that contains {}", path.display())]
    VolumeLookupFailed { path: PathBuf, source: io::Error },
    #[error(
        "{} is a streamed cloud drive and is not scanned, so nothing gets downloaded",
        mount_point.display()
    )]
    CloudVirtualVolumeNotScanned { mount_point: PathBuf },
    #[error("{} was not found on its volume", path.display())]
    FolderNotFoundOnVolume { path: PathBuf },
    #[error(transparent)]
    FolderWalkFailed(FolderWalkError),
    #[error(transparent)]
    MasterFileTableScanFailed(MftScanError),
}

impl From<FolderWalkError> for ScanError {
    fn from(error: FolderWalkError) -> Self {
        match error {
            FolderWalkError::Cancelled => Self::Cancelled,
            other => Self::FolderWalkFailed(other),
        }
    }
}

impl From<MftScanError> for ScanError {
    fn from(error: MftScanError) -> Self {
        match error {
            MftScanError::Cancelled => Self::Cancelled,
            other => Self::MasterFileTableScanFailed(other),
        }
    }
}

/// Scans `target` with `method` and returns the finished tree with its summary.
///
/// Blocks until the scan is done: call it on a background thread. Progress is reported on
/// the calling thread through `progress_reporter`, at most about ten times per second.
/// Cancel from any thread through a clone of `cancellation`.
///
/// # Errors
///
/// Fails when the target lies on a cloud virtual volume such as Google Drive for desktop (walking
/// it would download files), when the target cannot be read, when the fast scan is not available
/// for it, or when the scan is cancelled. Folders that cannot be listed do not fail the scan; they are
/// flagged inaccessible in the tree.
pub fn scan(
    target: ScanTarget,
    method: ScanMethod,
    progress_reporter: &dyn ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<ScanResult, ScanError> {
    scan_with_options(
        target,
        method,
        &ScanOptions::default(),
        progress_reporter,
        cancellation,
    )
}

/// Like [`scan`], with explicit tuning options.
///
/// # Errors
///
/// See [`scan`].
pub fn scan_with_options(
    target: ScanTarget,
    method: ScanMethod,
    options: &ScanOptions,
    progress_reporter: &dyn ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<ScanResult, ScanError> {
    let started_at = Instant::now();

    refuse_cloud_virtual_target(&target)?;

    let tree = match method {
        ScanMethod::FolderWalk => {
            scan_folder_with_walk(target.path(), progress_reporter, cancellation)?
        }
        ScanMethod::MasterFileTable => {
            scan_with_master_file_table(&target, *options, progress_reporter, cancellation)?
        }
    };

    let duration_milliseconds = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
    let totals = tree.totals();
    progress_reporter.report_progress(ScanProgressUpdate {
        phase: ScanPhase::Finished,
        files_found: u64::from(totals.file_count),
        directories_found: u64::from(totals.directory_count),
        bytes_found: totals.logical_size,
        current_path: String::new(),
        elapsed_milliseconds: duration_milliseconds,
        largest_top_level_directories: Vec::new(),
    });

    let summary = ScanSummary {
        target,
        method,
        root_path: tree.root_path().to_path_buf(),
        duration_milliseconds,
        totals,
    };
    Ok(ScanResult { tree, summary })
}

/// Refuses a target that lies on a cloud virtual volume such as Google Drive for desktop.
/// [`scan`] checks this itself; callers may check earlier to report the problem right away.
///
/// A failed volume lookup is not an error here: the scan itself then reports why the target
/// cannot be read.
///
/// # Errors
///
/// [`ScanError::CloudVirtualVolumeNotScanned`] when the target is on a cloud virtual volume.
pub fn refuse_cloud_virtual_target(target: &ScanTarget) -> Result<(), ScanError> {
    refuse_cloud_virtual_volume(target.path())
}

fn refuse_cloud_virtual_volume(target_path: &Path) -> Result<(), ScanError> {
    match mount_point_and_kind_of_volume_holding(target_path) {
        Ok((mount_point, kind)) => ensure_volume_kind_may_be_scanned(mount_point, kind),
        Err(error) => {
            tracing::debug!(path = %target_path.display(), %error, "volume kind unknown");
            Ok(())
        }
    }
}

fn ensure_volume_kind_may_be_scanned(
    mount_point: PathBuf,
    kind: VolumeKind,
) -> Result<(), ScanError> {
    if kind.is_cloud_virtual() {
        return Err(ScanError::CloudVirtualVolumeNotScanned { mount_point });
    }
    Ok(())
}

/// The MFT always covers the whole volume; for a folder target the volume is scanned and the
/// folder's subtree is presented.
fn scan_with_master_file_table(
    target: &ScanTarget,
    options: ScanOptions,
    progress_reporter: &dyn ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<FileTree, ScanError> {
    if !is_process_elevated() {
        return Err(ScanError::FastScanRequiresElevation);
    }

    let target_path =
        std::path::absolute(target.path()).map_err(|source| ScanError::VolumeLookupFailed {
            path: target.path().to_path_buf(),
            source,
        })?;
    let mount_point = platform::volume_mount_point_for_path(&target_path).map_err(|source| {
        ScanError::VolumeLookupFailed {
            path: target_path.clone(),
            source,
        }
    })?;
    let volume_supports_fast_scan = platform::query_volume_details(&mount_point)
        .ok()
        .flatten()
        .is_some_and(|details| platform::file_system_supports_fast_scan(&details.file_system_name));
    if !volume_supports_fast_scan {
        return Err(ScanError::FastScanUnsupported { path: target_path });
    }

    let volume_tree = scan_volume_with_mft(
        &mount_point,
        options.mft_chunk_size_bytes,
        progress_reporter,
        cancellation,
    )?;

    let components_below_mount_point: Vec<_> = target_path
        .components()
        .skip(mount_point.components().count())
        .map(|component| component.as_os_str().to_os_string())
        .collect();
    if components_below_mount_point.is_empty() {
        return Ok(volume_tree);
    }

    let folder_node = volume_tree
        .find_descendant_by_names(components_below_mount_point.iter().map(AsRef::as_ref))
        .ok_or_else(|| ScanError::FolderNotFoundOnVolume {
            path: target_path.clone(),
        })?;
    Ok(volume_tree.extract_subtree(folder_node, target_path))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::volume_kind::{VolumeClassificationFacts, classify_volume};
    use crate::volumes::DriveType;

    #[test]
    fn refuses_a_google_drive_volume() {
        let google_drive_kind = classify_volume(&VolumeClassificationFacts {
            drive_type: DriveType::Fixed,
            label: "Google Drive",
            serial_number: 0x1983_1116,
            dos_device_name: Some(r"\Device\Volume{f5ae2bcb-da01-3bf2-8935-408102040811}"),
        });

        let result = ensure_volume_kind_may_be_scanned(PathBuf::from(r"G:\"), google_drive_kind);

        assert!(matches!(
            result,
            Err(ScanError::CloudVirtualVolumeNotScanned { mount_point }) if mount_point == Path::new(r"G:\")
        ));
    }

    #[test]
    fn allows_local_and_removable_volumes() {
        assert!(
            ensure_volume_kind_may_be_scanned(PathBuf::from(r"C:\"), VolumeKind::LocalDisk).is_ok()
        );
        assert!(
            ensure_volume_kind_may_be_scanned(PathBuf::from(r"E:\"), VolumeKind::Removable).is_ok()
        );
    }

    #[test]
    fn the_temp_directory_is_not_refused() {
        assert!(refuse_cloud_virtual_volume(&std::env::temp_dir()).is_ok());
    }
}
