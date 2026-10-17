use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::cancellation::{CancellationToken, SearchCancelled};
use crate::directory_entry::{DirectoryEntry, EntryFlags, EntryKind, FileIdentity};
use crate::platform::list_directory;
use crate::search_progress::ProgressReporter;
use crate::search_settings::ValidatedSearchSettings;

/// A file that may have duplicates: right category, large enough, stored locally.
#[derive(Debug, Clone)]
pub(crate) struct Candidate {
    pub(crate) path: PathBuf,
    pub(crate) size: u64,
    pub(crate) modified_time: SystemTime,
    pub(crate) created_time: SystemTime,
    /// Index into the validated search folders, used to pick the read strategy of its drive.
    pub(crate) search_folder_index: usize,
}

#[derive(Debug, Default)]
pub(crate) struct CandidateCollection {
    pub(crate) candidates: Vec<Candidate>,
    pub(crate) online_only_files_skipped: u64,
    pub(crate) hard_links_collapsed: u64,
    pub(crate) inaccessible_folder_count: u64,
}

/// Walks every search folder and keeps the files worth comparing. Links are never followed, cloud
/// placeholders are counted but never opened, system folders are skipped, and a second path to
/// an already collected file (a hard link) is dropped.
pub(crate) fn collect_candidates(
    settings: &ValidatedSearchSettings,
    reporter: &ProgressReporter<'_>,
    cancellation: &CancellationToken,
) -> Result<CandidateCollection, SearchCancelled> {
    let mut collector = CandidateCollector {
        settings,
        reporter,
        collection: CandidateCollection::default(),
        seen_identities: HashSet::new(),
    };

    for (search_folder_index, folder) in settings.folders.iter().enumerate() {
        collector.walk_search_folder(folder, search_folder_index, cancellation)?;
    }

    Ok(collector.collection)
}

struct CandidateCollector<'settings, 'reporter, 'callback> {
    settings: &'settings ValidatedSearchSettings,
    reporter: &'reporter ProgressReporter<'callback>,
    collection: CandidateCollection,
    seen_identities: HashSet<FileIdentity>,
}

#[derive(Default)]
struct DirectoryTally {
    files_collected: u64,
    bytes_collected: u64,
    online_only_files_skipped: u64,
}

impl CandidateCollector<'_, '_, '_> {
    fn walk_search_folder(
        &mut self,
        search_folder: &Path,
        search_folder_index: usize,
        cancellation: &CancellationToken,
    ) -> Result<(), SearchCancelled> {
        let mut pending_directories = vec![search_folder.to_path_buf()];

        while let Some(directory) = pending_directories.pop() {
            cancellation.return_if_cancelled()?;

            let entries = match list_directory(&directory) {
                Ok(entries) => entries,
                Err(error) => {
                    tracing::debug!(directory = %directory.display(), %error, "skipping unreadable folder");
                    self.collection.inaccessible_folder_count += 1;
                    continue;
                }
            };

            let mut tally = DirectoryTally::default();
            for entry in entries {
                let entry_path = directory.join(&entry.name);
                match entry.kind {
                    EntryKind::Directory => {
                        if self.should_descend_into(&entry, &entry_path) {
                            pending_directories.push(entry_path);
                        }
                    }
                    EntryKind::File => {
                        self.consider_file(&entry, entry_path, search_folder_index, &mut tally);
                    }
                }
            }

            self.reporter.update(|progress| {
                progress.folders_listed += 1;
                progress.files_collected += tally.files_collected;
                progress.bytes_collected += tally.bytes_collected;
                progress.online_only_files_skipped += tally.online_only_files_skipped;
            });
        }

        Ok(())
    }

    fn should_descend_into(&self, entry: &DirectoryEntry, entry_path: &Path) -> bool {
        let is_protected_system_folder = entry
            .flags
            .contains(EntryFlags::HIDDEN | EntryFlags::SYSTEM);

        !entry.flags.is_link()
            && !entry.flags.contains(EntryFlags::CLOUD_PLACEHOLDER)
            && !is_protected_system_folder
            && !self.settings.system_folder_guard.is_system_path(entry_path)
    }

    fn consider_file(
        &mut self,
        entry: &DirectoryEntry,
        entry_path: PathBuf,
        search_folder_index: usize,
        tally: &mut DirectoryTally,
    ) {
        if entry.flags.is_link() || !self.is_wanted_file(&entry_path, entry.logical_size) {
            return;
        }

        if entry.flags.contains(EntryFlags::CLOUD_PLACEHOLDER) {
            self.collection.online_only_files_skipped += 1;
            tally.online_only_files_skipped += 1;
            return;
        }

        if let Some(identity) = entry.identity
            && !self.seen_identities.insert(identity)
        {
            self.collection.hard_links_collapsed += 1;
            return;
        }

        tally.files_collected += 1;
        tally.bytes_collected += entry.logical_size;
        self.collection.candidates.push(Candidate {
            path: entry_path,
            size: entry.logical_size,
            modified_time: entry.modified_time,
            created_time: entry.created_time,
            search_folder_index,
        });
    }

    fn is_wanted_file(&self, path: &Path, size: u64) -> bool {
        // Empty files are all identical; they never waste space.
        if size == 0 || size < self.settings.minimum_file_size_bytes {
            return false;
        }

        path.extension()
            .map(|extension| extension.to_string_lossy().to_lowercase())
            .is_some_and(|extension| self.settings.extensions.contains(&extension))
    }
}
