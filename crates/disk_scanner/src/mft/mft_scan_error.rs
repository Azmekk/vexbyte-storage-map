use std::io;
use std::path::PathBuf;

use super::data_runs::DataRunError;
use super::mft_chunk_reader::{MftChunkReadError, MftExtentError};

#[derive(Debug, thiserror::Error)]
pub enum MftScanError {
    #[error("reading the Master File Table is only supported on Windows")]
    UnsupportedPlatform,
    #[error("could not open the volume {} for reading (administrator rights are required)", mount_point.display())]
    VolumeOpenFailed {
        mount_point: PathBuf,
        source: io::Error,
    },
    #[error("could not read the NTFS layout of {} (is it an NTFS volume?)", mount_point.display())]
    VolumeLayoutQueryFailed {
        mount_point: PathBuf,
        source: io::Error,
    },
    #[error("the volume reports an unusable NTFS layout: {reason}")]
    InvalidVolumeLayout { reason: &'static str },
    #[error("MFT record {record_number} could not be read: {reason}")]
    MftRecordUnreadable { record_number: u64, reason: String },
    #[error("the location of the MFT could not be decoded")]
    MftDataRunsInvalid(#[from] DataRunError),
    #[error("the $MFT record has no data attribute")]
    MftDataMissing,
    #[error("the MFT is too fragmented to be located")]
    MftTooFragmented,
    #[error("the MFT location is inconsistent")]
    MftExtentsInvalid(#[from] MftExtentError),
    #[error("reading the MFT failed")]
    MftChunkReadFailed(#[from] MftChunkReadError),
    #[error("reading the volume at offset {offset} failed")]
    VolumeReadFailed { offset: u64, source: io::Error },
    #[error("the scan was cancelled")]
    Cancelled,
}
