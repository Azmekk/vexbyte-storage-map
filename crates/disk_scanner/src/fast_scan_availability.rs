use serde::{Deserialize, Serialize};
use specta::Type;

use crate::platform;
use crate::volumes::VolumeSummary;

/// Whether the fast scan method can be used for a volume right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FastScanAvailability {
    Available,
    /// The file system supports it, but the process must run with administrator rights.
    RequiresElevation,
    /// The file system has no structure that can be read directly (for example exFAT).
    UnsupportedFileSystem,
}

/// Decides whether `volume` can be scanned with the fast method by this process.
#[must_use]
pub fn fast_scan_availability(volume: &VolumeSummary) -> FastScanAvailability {
    if !volume.supports_fast_scan {
        FastScanAvailability::UnsupportedFileSystem
    } else if is_process_elevated() {
        FastScanAvailability::Available
    } else {
        FastScanAvailability::RequiresElevation
    }
}

/// True when the process runs with administrator rights.
#[must_use]
pub fn is_process_elevated() -> bool {
    platform::is_process_elevated()
}
