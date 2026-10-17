//! Scan method B: reads the NTFS Master File Table directly.
//!
//! A focused parser for the handful of attributes the tree needs, fed by large sequential
//! reads. The parsing is plain byte handling and builds everywhere; only opening a volume is
//! Windows-specific, so this module root is the one place that gates on the platform.

mod attributes;
mod data_runs;
mod file_record;
mod file_reference;
mod little_endian_fields;
mod mft_chunk_reader;
mod mft_location;
mod mft_record_table;
mod mft_record_to_tree;
mod mft_scan;
mod mft_scan_error;
#[cfg(test)]
mod mft_scan_tests;
mod ntfs_file_attributes;
mod record_fixups;
#[cfg(test)]
mod test_fixtures;
mod volume_layout;
#[cfg(windows)]
mod volume_scan;

use std::path::Path;

pub use data_runs::DataRunError;
pub use mft_chunk_reader::{MftChunkReadError, MftExtentError};
pub use mft_scan::DEFAULT_MFT_CHUNK_SIZE_BYTES;
pub use mft_scan_error::MftScanError;
pub(crate) use volume_layout::NtfsVolumeLayout;

use crate::cancellation::CancellationToken;
use crate::file_tree::FileTree;
use crate::scan_progress::ScanProgressReporter;

/// Opens the volume mounted at `mount_point` read-only and builds its tree from the MFT.
#[cfg(windows)]
pub(crate) fn scan_volume_with_mft(
    mount_point: &Path,
    chunk_size_bytes: usize,
    progress_reporter: &dyn ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<FileTree, MftScanError> {
    volume_scan::scan_volume_with_mft(
        mount_point,
        chunk_size_bytes,
        progress_reporter,
        cancellation,
    )
}

#[cfg(not(windows))]
pub(crate) fn scan_volume_with_mft(
    _mount_point: &Path,
    _chunk_size_bytes: usize,
    _progress_reporter: &dyn ScanProgressReporter,
    _cancellation: &CancellationToken,
) -> Result<FileTree, MftScanError> {
    Err(MftScanError::UnsupportedPlatform)
}
