use std::path::PathBuf;

use super::mft_chunk_reader::{MftChunkReader, VolumeBytes};
use super::mft_location::locate_mft_extents;
use super::mft_record_table::MftRecordTable;
use super::mft_record_to_tree::build_tree_from_records;
use super::mft_scan_error::MftScanError;
use super::volume_layout::NtfsVolumeLayout;
use crate::cancellation::CancellationToken;
use crate::file_tree::FileTree;
use crate::scan_progress::{ProgressThrottle, ScanPhase, ScanProgressReporter, ScanProgressUpdate};

/// 8 MiB reads keep an SSD at full sequential speed while the buffer stays small.
pub const DEFAULT_MFT_CHUNK_SIZE_BYTES: usize = 8 * 1024 * 1024;

/// Reads the whole MFT from `volume` and builds the tree of the volume rooted at
/// `volume_root_path`.
pub(crate) fn scan_mft_from_volume_bytes<Volume: VolumeBytes>(
    volume: &Volume,
    layout: &NtfsVolumeLayout,
    volume_root_path: PathBuf,
    chunk_size_bytes: usize,
    progress_reporter: &dyn ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<FileTree, MftScanError> {
    layout
        .validate()
        .map_err(|reason| MftScanError::InvalidVolumeLayout { reason })?;
    let mut throttle = ProgressThrottle::starting_now();

    let mft_extents = locate_mft_extents(volume, layout)?;
    let record_size = layout.record_size();
    let chunk_size = chunk_size_bytes.max(record_size);
    let mut chunk_reader = MftChunkReader::new(volume, mft_extents, chunk_size, record_size);
    let mut record_table = MftRecordTable::with_record_count(layout.record_count());

    let mut chunk = Vec::with_capacity(chunk_size);
    while let Some(first_record_number) = chunk_reader.read_next_chunk(&mut chunk)? {
        if cancellation.is_cancelled() {
            return Err(MftScanError::Cancelled);
        }

        for (position, record) in chunk.chunks_exact_mut(record_size).enumerate() {
            record_table.add_record(first_record_number + position as u64, record);
        }

        if throttle.is_report_due() {
            progress_reporter.report_progress(progress_update(
                &record_table,
                &throttle,
                ScanPhase::ReadingMasterFileTable,
            ));
        }
    }

    if record_table.skipped_record_count() > 0 {
        tracing::debug!(
            skipped_record_count = record_table.skipped_record_count(),
            "MFT records without valid signature or fixups were skipped"
        );
    }

    progress_reporter.report_progress(progress_update(
        &record_table,
        &throttle,
        ScanPhase::BuildingTree,
    ));
    record_table.merge_extension_records();
    Ok(build_tree_from_records(
        &record_table,
        volume_root_path,
        layout.volume_serial_number,
    ))
}

fn progress_update(
    record_table: &MftRecordTable,
    throttle: &ProgressThrottle,
    phase: ScanPhase,
) -> ScanProgressUpdate {
    let totals = record_table.totals();
    ScanProgressUpdate {
        phase,
        files_found: totals.files,
        directories_found: totals.directories,
        bytes_found: totals.logical_bytes,
        current_path: String::new(),
        elapsed_milliseconds: throttle.elapsed_milliseconds(),
        largest_top_level_directories: Vec::new(),
    }
}
