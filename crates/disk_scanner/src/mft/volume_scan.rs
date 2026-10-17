use std::io;
use std::path::Path;

use super::mft_chunk_reader::VolumeBytes;
use super::mft_scan::scan_mft_from_volume_bytes;
use super::mft_scan_error::MftScanError;
use crate::cancellation::CancellationToken;
use crate::file_tree::FileTree;
use crate::platform::RawVolumeReader;
use crate::scan_progress::ScanProgressReporter;

impl VolumeBytes for RawVolumeReader {
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> io::Result<usize> {
        RawVolumeReader::read_at(self, offset, buffer)
    }
}

/// Opens the volume mounted at `mount_point` read-only and scans its MFT.
pub(crate) fn scan_volume_with_mft(
    mount_point: &Path,
    chunk_size_bytes: usize,
    progress_reporter: &dyn ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<FileTree, MftScanError> {
    let volume = RawVolumeReader::open_read_only(mount_point).map_err(|source| {
        MftScanError::VolumeOpenFailed {
            mount_point: mount_point.to_path_buf(),
            source,
        }
    })?;
    let layout = volume.read_ntfs_volume_layout().map_err(|source| {
        MftScanError::VolumeLayoutQueryFailed {
            mount_point: mount_point.to_path_buf(),
            source,
        }
    })?;

    scan_mft_from_volume_bytes(
        &volume,
        &layout,
        mount_point.to_path_buf(),
        chunk_size_bytes,
        progress_reporter,
        cancellation,
    )
}
