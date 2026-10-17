use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use specta::Type;

/// The pipeline stages, in the order they run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateSearchStage {
    #[default]
    Collecting,
    ComparingSizes,
    QuickCheck,
    FullCheck,
    Done,
}

/// A snapshot of the running search. Counts of earlier stages stay filled in when later stages
/// run, so the UI can show every step of the stepper at once.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSearchProgress {
    pub stage: DuplicateSearchStage,

    pub folders_listed: u64,
    pub files_collected: u64,
    pub bytes_collected: u64,
    pub online_only_files_skipped: u64,

    /// Files that share their exact size with at least one other file.
    pub files_with_matching_size: u64,

    pub quick_check_files_done: u64,
    pub quick_check_files_total: u64,

    pub full_check_files_done: u64,
    pub full_check_files_total: u64,
    pub full_check_bytes_done: u64,
    pub full_check_bytes_total: u64,

    pub duplicate_groups_found: u64,
}

/// Most progress callbacks end up as IPC messages; ten updates a second look live without
/// flooding the channel.
const MINIMUM_INTERVAL_BETWEEN_REPORTS: Duration = Duration::from_millis(100);

struct ReporterState {
    progress: DuplicateSearchProgress,
    last_report: Option<Instant>,
}

/// Collects progress from any thread and forwards it to the caller's callback at most about ten
/// times per second. Stage changes are always forwarded immediately.
pub(crate) struct ProgressReporter<'callback> {
    callback: &'callback (dyn Fn(&DuplicateSearchProgress) + Sync),
    state: Mutex<ReporterState>,
}

impl<'callback> ProgressReporter<'callback> {
    pub(crate) fn new(callback: &'callback (dyn Fn(&DuplicateSearchProgress) + Sync)) -> Self {
        Self {
            callback,
            state: Mutex::new(ReporterState {
                progress: DuplicateSearchProgress::default(),
                last_report: None,
            }),
        }
    }

    /// Applies `change` and reports if enough time has passed since the last report.
    pub(crate) fn update(&self, change: impl FnOnce(&mut DuplicateSearchProgress)) {
        self.update_and_maybe_report(change, false);
    }

    /// Moves to `stage`, applies `change` and reports right away.
    pub(crate) fn enter_stage(
        &self,
        stage: DuplicateSearchStage,
        change: impl FnOnce(&mut DuplicateSearchProgress),
    ) {
        self.update_and_maybe_report(
            |progress| {
                progress.stage = stage;
                change(progress);
            },
            true,
        );
    }

    fn update_and_maybe_report(
        &self,
        change: impl FnOnce(&mut DuplicateSearchProgress),
        force_report: bool,
    ) {
        let snapshot = {
            let mut state = match self.state.lock() {
                Ok(state) => state,
                Err(poisoned) => poisoned.into_inner(),
            };
            change(&mut state.progress);

            let now = Instant::now();
            let is_due = state.last_report.is_none_or(|last_report| {
                now.duration_since(last_report) >= MINIMUM_INTERVAL_BETWEEN_REPORTS
            });
            if !(force_report || is_due) {
                return;
            }
            state.last_report = Some(now);
            state.progress.clone()
        };

        // The callback runs outside the lock so a slow consumer never blocks hashing threads.
        (self.callback)(&snapshot);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn throttles_updates_but_always_reports_stage_changes() {
        let reports = Mutex::new(Vec::new());
        let callback = |progress: &DuplicateSearchProgress| {
            reports.lock().unwrap().push(progress.clone());
        };
        let reporter = ProgressReporter::new(&callback);

        for _ in 0..1000 {
            reporter.update(|progress| progress.files_collected += 1);
        }
        reporter.enter_stage(DuplicateSearchStage::ComparingSizes, |_| {});

        let reports = reports.into_inner().unwrap();
        assert!(reports.len() < 1000, "updates were not throttled");
        let last_report = reports.last().unwrap();
        assert_eq!(last_report.stage, DuplicateSearchStage::ComparingSizes);
        assert_eq!(last_report.files_collected, 1000);
    }
}
