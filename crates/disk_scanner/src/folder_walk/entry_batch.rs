use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use crate::file_identity::FileIdentity;
use crate::file_tree::{DirectoryKey, NodeFlags, NodeKind};

/// Workers hand entries to the tree builder in batches of this size, so the channel and the
/// builder see a few hundred messages instead of a million.
const ENTRIES_PER_BATCH: usize = 4096;
/// A partly filled batch is still sent after this long, so progress keeps moving on slow
/// network drives.
const MAXIMUM_BATCH_AGE: Duration = Duration::from_millis(100);

/// One listed entry waiting to be added to the tree. Its name lives in the batch's shared
/// text buffer instead of its own allocation.
pub(crate) struct BatchedEntry {
    name_start: u32,
    name_length: u32,
    pub(crate) details: BatchedEntryDetails,
}

/// Everything about a `BatchedEntry` except its name.
pub(crate) struct BatchedEntryDetails {
    pub(crate) kind: NodeKind,
    pub(crate) parent: DirectoryKey,
    pub(crate) directory_key: Option<DirectoryKey>,
    /// The directory directly under the scan root this entry is in, for the live
    /// "biggest folders" list. `None` for entries directly in the root.
    pub(crate) top_level_directory: Option<DirectoryKey>,
    pub(crate) logical_size: u64,
    pub(crate) allocated_size: u64,
    pub(crate) modified_time: SystemTime,
    pub(crate) flags: NodeFlags,
    pub(crate) identity: Option<FileIdentity>,
}

#[derive(Default)]
pub(crate) struct EntryBatch {
    names: String,
    entries: Vec<BatchedEntry>,
    inaccessible_directories: Vec<DirectoryKey>,
    latest_directory_path: Option<PathBuf>,
    first_entry_added_at: Option<Instant>,
}

impl EntryBatch {
    pub(crate) fn add_entry(&mut self, name: &str, details: BatchedEntryDetails) {
        if self.first_entry_added_at.is_none() {
            self.first_entry_added_at = Some(Instant::now());
        }
        let name_start = u32::try_from(self.names.len()).unwrap_or(u32::MAX);
        let name_length = u32::try_from(name.len()).unwrap_or(0);
        self.names.push_str(name);

        self.entries.push(BatchedEntry {
            name_start,
            name_length,
            details,
        });
    }

    pub(crate) fn mark_directory_inaccessible(&mut self, directory_key: DirectoryKey) {
        if self.first_entry_added_at.is_none() {
            self.first_entry_added_at = Some(Instant::now());
        }
        self.inaccessible_directories.push(directory_key);
    }

    pub(crate) fn set_latest_directory_path(&mut self, path: PathBuf) {
        self.latest_directory_path = Some(path);
    }

    pub(crate) fn is_ready_to_send(&self) -> bool {
        self.entries.len() >= ENTRIES_PER_BATCH
            || self
                .first_entry_added_at
                .is_some_and(|added_at| added_at.elapsed() >= MAXIMUM_BATCH_AGE)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.inaccessible_directories.is_empty()
    }

    pub(crate) fn entries(&self) -> &[BatchedEntry] {
        &self.entries
    }

    pub(crate) fn name_of(&self, entry: &BatchedEntry) -> &str {
        let start = entry.name_start as usize;
        let end = start + entry.name_length as usize;
        self.names.get(start..end).unwrap_or("")
    }

    pub(crate) fn inaccessible_directories(&self) -> &[DirectoryKey] {
        &self.inaccessible_directories
    }

    pub(crate) fn latest_directory_path(&self) -> Option<&PathBuf> {
        self.latest_directory_path.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn details() -> BatchedEntryDetails {
        BatchedEntryDetails {
            kind: NodeKind::File,
            parent: DirectoryKey::new(0),
            directory_key: None,
            top_level_directory: None,
            logical_size: 1,
            allocated_size: 1,
            modified_time: SystemTime::UNIX_EPOCH,
            flags: NodeFlags::empty(),
            identity: None,
        }
    }

    #[test]
    fn stores_names_in_one_buffer() {
        let mut batch = EntryBatch::default();
        batch.add_entry("first", details());
        batch.add_entry("zweite", details());

        let names: Vec<_> = batch
            .entries()
            .iter()
            .map(|entry| batch.name_of(entry))
            .collect();

        assert_eq!(names, ["first", "zweite"]);
        assert!(!batch.is_ready_to_send());
    }

    #[test]
    fn is_ready_once_full() {
        let mut batch = EntryBatch::default();
        for _ in 0..ENTRIES_PER_BATCH {
            batch.add_entry("x", details());
        }

        assert!(batch.is_ready_to_send());
    }
}
