use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use specta::Type;

/// How often progress is reported at most. Ten updates per second keep the counters lively
/// without flooding the user interface.
const MINIMUM_INTERVAL_BETWEEN_REPORTS: Duration = Duration::from_millis(100);
/// How many top-level directories the live "biggest folders" list shows.
pub(crate) const REPORTED_TOP_LEVEL_DIRECTORY_COUNT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ScanPhase {
    ListingDirectories,
    ReadingMasterFileTable,
    /// Linking parents and adding up sizes after everything was read.
    BuildingTree,
    Finished,
}

/// One directory directly under the scanned root and the bytes found in it so far.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TopLevelDirectoryProgress {
    pub name: String,
    pub bytes_found: u64,
}

/// A snapshot of a running scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressUpdate {
    pub phase: ScanPhase,
    pub files_found: u64,
    pub directories_found: u64,
    /// Logical bytes of the files found so far. Hard links may be counted more than once here;
    /// the finished tree counts them once.
    pub bytes_found: u64,
    /// The directory being listed most recently; empty while reading the MFT.
    pub current_path: String,
    pub elapsed_milliseconds: u64,
    /// The largest directories directly under the root, largest first (folder walk only).
    pub largest_top_level_directories: Vec<TopLevelDirectoryProgress>,
}

/// Receives progress updates of a scan.
///
/// `scan` calls it on the thread that called `scan`, at most about ten times per second plus
/// once when a phase changes, so an implementation may do a little work such as sending the
/// update over a channel. Any `Fn(ScanProgressUpdate)` closure is a reporter.
pub trait ScanProgressReporter {
    fn report_progress(&self, update: ScanProgressUpdate);
}

impl<Reporter: Fn(ScanProgressUpdate)> ScanProgressReporter for Reporter {
    fn report_progress(&self, update: ScanProgressUpdate) {
        self(update);
    }
}

/// A reporter that ignores every update.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoProgressReporting;

impl ScanProgressReporter for NoProgressReporting {
    fn report_progress(&self, _update: ScanProgressUpdate) {}
}

/// Limits how often progress is reported and measures the elapsed time.
pub(crate) struct ProgressThrottle {
    scan_started_at: Instant,
    last_report_at: Option<Instant>,
}

impl ProgressThrottle {
    pub(crate) fn starting_now() -> Self {
        Self {
            scan_started_at: Instant::now(),
            last_report_at: None,
        }
    }

    /// True when enough time has passed since the last report; remembers this moment if so.
    pub(crate) fn is_report_due(&mut self) -> bool {
        let now = Instant::now();
        let is_due = self
            .last_report_at
            .is_none_or(|last_report_at| now - last_report_at >= MINIMUM_INTERVAL_BETWEEN_REPORTS);
        if is_due {
            self.last_report_at = Some(now);
        }
        is_due
    }

    pub(crate) fn elapsed_milliseconds(&self) -> u64 {
        u64::try_from(self.scan_started_at.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[test]
    fn reports_at_most_ten_times_per_second() {
        let mut throttle = ProgressThrottle::starting_now();

        assert!(throttle.is_report_due(), "the first report is always due");
        assert!(!throttle.is_report_due());
    }

    #[test]
    fn closures_are_reporters() {
        let received = RefCell::new(Vec::new());
        let reporter = |update: ScanProgressUpdate| received.borrow_mut().push(update.files_found);

        reporter.report_progress(ScanProgressUpdate {
            phase: ScanPhase::ListingDirectories,
            files_found: 3,
            directories_found: 1,
            bytes_found: 10,
            current_path: String::new(),
            elapsed_milliseconds: 0,
            largest_top_level_directories: Vec::new(),
        });

        assert_eq!(received.into_inner(), vec![3]);
    }
}
