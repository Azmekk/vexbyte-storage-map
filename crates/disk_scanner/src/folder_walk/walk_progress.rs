use std::collections::HashMap;

use super::entry_batch::{BatchedEntry, EntryBatch};
use crate::file_tree::{DirectoryKey, NodeKind};
use crate::scan_progress::{
    ProgressThrottle, REPORTED_TOP_LEVEL_DIRECTORY_COUNT, ScanPhase, ScanProgressReporter,
    ScanProgressUpdate, TopLevelDirectoryProgress,
};

/// Running totals of a folder walk, fed from the batches the tree builder receives.
pub(crate) struct WalkProgress {
    files_found: u64,
    directories_found: u64,
    bytes_found: u64,
    current_path: String,
    top_level_directories: HashMap<DirectoryKey, TopLevelDirectoryProgress>,
    throttle: ProgressThrottle,
}

impl WalkProgress {
    pub(crate) fn starting_now() -> Self {
        Self {
            files_found: 0,
            directories_found: 0,
            bytes_found: 0,
            current_path: String::new(),
            top_level_directories: HashMap::new(),
            throttle: ProgressThrottle::starting_now(),
        }
    }

    pub(crate) fn record_batch(&mut self, batch: &EntryBatch) {
        for entry in batch.entries() {
            self.record_entry(entry, batch.name_of(entry));
        }
        if let Some(latest_directory_path) = batch.latest_directory_path() {
            self.current_path = latest_directory_path.to_string_lossy().into_owned();
        }
    }

    fn record_entry(&mut self, batched_entry: &BatchedEntry, name: &str) {
        let entry = &batched_entry.details;
        match entry.kind {
            NodeKind::File => {
                self.files_found += 1;
                self.bytes_found += entry.logical_size;
            }
            NodeKind::Directory => self.directories_found += 1,
        }

        let Some(top_level_directory) = entry.top_level_directory else {
            return;
        };
        let tally = self
            .top_level_directories
            .entry(top_level_directory)
            .or_insert_with(|| TopLevelDirectoryProgress {
                name: String::new(),
                bytes_found: 0,
            });
        if entry.directory_key == Some(top_level_directory) {
            name.clone_into(&mut tally.name);
        }
        if entry.kind == NodeKind::File {
            tally.bytes_found += entry.logical_size;
        }
    }

    pub(crate) fn report_if_due(&mut self, reporter: &dyn ScanProgressReporter) {
        if self.throttle.is_report_due() {
            reporter.report_progress(self.snapshot(ScanPhase::ListingDirectories));
        }
    }

    pub(crate) fn snapshot(&self, phase: ScanPhase) -> ScanProgressUpdate {
        let mut largest_top_level_directories: Vec<TopLevelDirectoryProgress> = self
            .top_level_directories
            .values()
            .filter(|tally| !tally.name.is_empty())
            .cloned()
            .collect();
        largest_top_level_directories.sort_by_key(|tally| std::cmp::Reverse(tally.bytes_found));
        largest_top_level_directories.truncate(REPORTED_TOP_LEVEL_DIRECTORY_COUNT);

        ScanProgressUpdate {
            phase,
            files_found: self.files_found,
            directories_found: self.directories_found,
            bytes_found: self.bytes_found,
            current_path: self.current_path.clone(),
            elapsed_milliseconds: self.throttle.elapsed_milliseconds(),
            largest_top_level_directories,
        }
    }
}
