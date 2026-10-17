use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::time::Instant;

use serde::Serialize;
use specta::Type;

use crate::cancellation::{CancellationToken, SearchCancelled};
use crate::candidate_collector::{Candidate, collect_candidates};
use crate::confirmed_group_counter::ConfirmedGroupCounter;
use crate::drive_kind::{DriveKind, DriveKindCache, ReadStrategy};
use crate::duplicate_group::{DuplicateFile, DuplicateGroup, DuplicateGroupId};
use crate::file_read_error::FileReadError;
use crate::full_hashing::compute_full_hash;
use crate::keep_rules::{KeepRule, apply_keep_rule_to_all_groups};
use crate::partial_hashing::{compute_partial_hash, quick_check_covers_whole_file};
use crate::read_scheduling::read_files_with_drive_strategy;
use crate::search_progress::{DuplicateSearchProgress, DuplicateSearchStage, ProgressReporter};
use crate::search_settings::{
    DuplicateSearchSettings, SearchSettingsError, ValidatedSearchSettings,
};
use crate::size_grouping::group_by_exact_size;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FolderDriveKind {
    pub folder: PathBuf,
    pub drive_kind: DriveKind,
}

/// Everything a finished search found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSearchResult {
    /// Sorted by wasted bytes, largest first. Each group keeps its oldest copy by default.
    pub groups: Vec<DuplicateGroup>,
    pub duplicate_file_count: u64,
    pub total_wasted_bytes: u64,
    /// Files of the selected categories and sizes that were considered.
    pub files_examined: u64,
    /// Shown as "N files are online-only and were not checked".
    pub online_only_files_skipped: u64,
    /// Extra paths to an already examined file; they are the same file, not duplicates.
    pub hard_links_collapsed: u64,
    pub unreadable_file_count: u64,
    pub inaccessible_folder_count: u64,
    pub folder_drive_kinds: Vec<FolderDriveKind>,
    pub elapsed_milliseconds: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DuplicateSearchError {
    #[error(transparent)]
    InvalidSettings(#[from] SearchSettingsError),
    #[error("the duplicate search was cancelled")]
    Cancelled,
}

impl From<SearchCancelled> for DuplicateSearchError {
    fn from(_: SearchCancelled) -> Self {
        Self::Cancelled
    }
}

/// Finds exact duplicates in the folders and categories of `settings`.
///
/// Blocks until done; run it on a background thread. Hashing uses the global rayon pool for files
/// on SSDs. `progress_callback` is called from any of the worker threads, at most about ten times
/// per second plus once per stage change, and finally with [`DuplicateSearchStage::Done`].
///
/// Read-only: files are opened for reading only and cloud placeholders are never opened.
///
/// # Errors
///
/// Fails when the settings are invalid (for example a system folder was selected) or when
/// `cancellation` is triggered. Unreadable files and folders are counted, not fatal.
pub fn find_duplicates(
    settings: &DuplicateSearchSettings,
    progress_callback: impl Fn(&DuplicateSearchProgress) + Sync,
    cancellation: &CancellationToken,
) -> Result<DuplicateSearchResult, DuplicateSearchError> {
    let validated_settings = settings.validate()?;
    find_duplicates_with_validated_settings(&validated_settings, progress_callback, cancellation)
}

/// Like [`find_duplicates`], for settings the caller already validated, for example against a
/// custom [`SystemFolderGuard`](crate::SystemFolderGuard).
///
/// # Errors
///
/// Fails only when `cancellation` is triggered.
pub fn find_duplicates_with_validated_settings(
    validated_settings: &ValidatedSearchSettings,
    progress_callback: impl Fn(&DuplicateSearchProgress) + Sync,
    cancellation: &CancellationToken,
) -> Result<DuplicateSearchResult, DuplicateSearchError> {
    let started_at = Instant::now();
    let reporter = ProgressReporter::new(&progress_callback);
    reporter.enter_stage(DuplicateSearchStage::Collecting, |_| {});

    let drive_kind_cache = DriveKindCache::new();
    let folder_drive_kinds: Vec<FolderDriveKind> = validated_settings
        .folders()
        .iter()
        .map(|folder| FolderDriveKind {
            folder: folder.clone(),
            drive_kind: drive_kind_cache.drive_kind_of(folder),
        })
        .collect();
    let read_strategies: Vec<ReadStrategy> = folder_drive_kinds
        .iter()
        .map(|folder_drive_kind| folder_drive_kind.drive_kind.read_strategy())
        .collect();

    let collection = collect_candidates(validated_settings, &reporter, cancellation)?;
    let files_examined = collection.candidates.len() as u64;

    let mut pipeline = ComparisonPipeline {
        reporter: &reporter,
        cancellation,
        read_strategies: &read_strategies,
        unreadable_file_count: 0,
    };
    let duplicate_candidate_groups = pipeline.find_identical_files(collection.candidates)?;
    let groups = build_sorted_groups(duplicate_candidate_groups);

    let result = DuplicateSearchResult {
        duplicate_file_count: groups.iter().map(|group| group.files.len() as u64).sum(),
        total_wasted_bytes: groups.iter().map(|group| group.wasted_bytes).sum(),
        groups,
        files_examined,
        online_only_files_skipped: collection.online_only_files_skipped,
        hard_links_collapsed: collection.hard_links_collapsed,
        unreadable_file_count: pipeline.unreadable_file_count,
        inaccessible_folder_count: collection.inaccessible_folder_count,
        folder_drive_kinds,
        elapsed_milliseconds: u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX),
    };

    // The group count is not overwritten here: it was counted live as groups got confirmed, and
    // the tests check that it ends equal to the number of groups found.
    reporter.enter_stage(DuplicateSearchStage::Done, |_| {});
    Ok(result)
}

struct ComparisonPipeline<'search, 'callback> {
    reporter: &'search ProgressReporter<'callback>,
    cancellation: &'search CancellationToken,
    read_strategies: &'search [ReadStrategy],
    unreadable_file_count: u64,
}

impl ComparisonPipeline<'_, '_> {
    fn find_identical_files(
        &mut self,
        candidates: Vec<Candidate>,
    ) -> Result<Vec<Vec<Candidate>>, DuplicateSearchError> {
        self.reporter
            .enter_stage(DuplicateSearchStage::ComparingSizes, |_| {});
        let size_groups = group_by_exact_size(candidates);
        let files_with_matching_size = count_files(&size_groups);
        self.reporter.update(|progress| {
            progress.files_with_matching_size = files_with_matching_size;
        });
        self.cancellation.return_if_cancelled()?;

        let quick_check_groups = self.run_quick_check(size_groups)?;
        let (final_groups, groups_needing_full_check): (Vec<_>, Vec<_>) = quick_check_groups
            .into_iter()
            .partition(|group| quick_check_covers_whole_file(group[0].size));

        let mut identical_groups = final_groups;
        identical_groups.extend(self.run_full_check(groups_needing_full_check)?);
        Ok(identical_groups)
    }

    fn run_quick_check(
        &mut self,
        size_groups: Vec<Vec<Candidate>>,
    ) -> Result<Vec<Vec<Candidate>>, DuplicateSearchError> {
        let files_to_check = size_groups.into_iter().flatten().collect::<Vec<_>>();
        let total_files = files_to_check.len() as u64;
        self.reporter
            .enter_stage(DuplicateSearchStage::QuickCheck, |progress| {
                progress.quick_check_files_total = total_files;
            });

        let reporter = self.reporter;
        let cancellation = self.cancellation;
        let groups_confirmed_by_quick_check = ConfirmedGroupCounter::new();
        let hashed_files = self.read_with_drive_strategy(files_to_check, |candidate| {
            if cancellation.is_cancelled() {
                return (candidate, Err(FileReadError::Cancelled));
            }
            let partial_hash = compute_partial_hash(&candidate.path, candidate.size);
            let confirms_new_group = match &partial_hash {
                Ok(hash) if quick_check_covers_whole_file(candidate.size) => {
                    groups_confirmed_by_quick_check
                        .record_file_and_check_new_group((candidate.size, *hash))
                }
                _ => false,
            };
            reporter.update(|progress| {
                progress.quick_check_files_done += 1;
                if confirms_new_group {
                    progress.duplicate_groups_found += 1;
                }
            });
            (candidate, partial_hash)
        });

        self.cancellation.return_if_cancelled()?;
        Ok(self.regroup_by_size_and_hash(hashed_files))
    }

    fn run_full_check(
        &mut self,
        groups: Vec<Vec<Candidate>>,
    ) -> Result<Vec<Vec<Candidate>>, DuplicateSearchError> {
        let files_to_check = groups.into_iter().flatten().collect::<Vec<_>>();
        let total_files = files_to_check.len() as u64;
        let total_bytes = files_to_check.iter().map(|candidate| candidate.size).sum();
        self.reporter
            .enter_stage(DuplicateSearchStage::FullCheck, |progress| {
                progress.full_check_files_total = total_files;
                progress.full_check_bytes_total = total_bytes;
            });

        let reporter = self.reporter;
        let cancellation = self.cancellation;
        let groups_confirmed_by_full_check = ConfirmedGroupCounter::new();
        let hashed_files = self.read_with_drive_strategy(files_to_check, |candidate| {
            let content_hash =
                compute_full_hash(&candidate.path, candidate.size, cancellation, |bytes| {
                    reporter.update(|progress| progress.full_check_bytes_done += bytes);
                });
            let confirms_new_group = content_hash.as_ref().is_ok_and(|hash| {
                groups_confirmed_by_full_check
                    .record_file_and_check_new_group((candidate.size, *hash))
            });
            reporter.update(|progress| {
                progress.full_check_files_done += 1;
                if confirms_new_group {
                    progress.duplicate_groups_found += 1;
                }
            });
            (candidate, content_hash)
        });

        self.cancellation.return_if_cancelled()?;
        Ok(self.regroup_by_size_and_hash(hashed_files))
    }

    fn read_with_drive_strategy<Output: Send>(
        &self,
        candidates: Vec<Candidate>,
        read_file: impl Fn(Candidate) -> Output + Sync,
    ) -> Vec<Output> {
        let read_strategies = self.read_strategies;
        read_files_with_drive_strategy(
            candidates,
            |candidate| {
                read_strategies
                    .get(candidate.search_folder_index)
                    .copied()
                    .unwrap_or(ReadStrategy::SequentialInPathOrder)
            },
            |candidate| candidate.path.as_path(),
            read_file,
        )
    }

    /// Groups files whose size and hash match, dropping unreadable files and groups that end up
    /// with a single file.
    fn regroup_by_size_and_hash<HashValue: Eq + Hash>(
        &mut self,
        hashed_files: Vec<(Candidate, Result<HashValue, FileReadError>)>,
    ) -> Vec<Vec<Candidate>> {
        let mut groups: HashMap<(u64, HashValue), Vec<Candidate>> = HashMap::new();

        for (candidate, hash_result) in hashed_files {
            match hash_result {
                Ok(hash_value) => groups
                    .entry((candidate.size, hash_value))
                    .or_default()
                    .push(candidate),
                Err(FileReadError::Cancelled) => {}
                Err(error) => {
                    tracing::debug!(path = %candidate.path.display(), %error, "skipping unreadable file");
                    self.unreadable_file_count += 1;
                }
            }
        }

        groups
            .into_values()
            .filter(|group| group.len() > 1)
            .collect()
    }
}

fn count_files(groups: &[Vec<Candidate>]) -> u64 {
    groups.iter().map(|group| group.len() as u64).sum()
}

fn build_sorted_groups(identical_groups: Vec<Vec<Candidate>>) -> Vec<DuplicateGroup> {
    let mut groups: Vec<DuplicateGroup> = identical_groups
        .into_iter()
        .map(|candidates| {
            let file_size = candidates[0].size;
            let files = candidates
                .into_iter()
                .map(|candidate| {
                    DuplicateFile::new(
                        candidate.path,
                        candidate.modified_time,
                        candidate.created_time,
                    )
                })
                .collect();
            DuplicateGroup::new(DuplicateGroupId(0), file_size, files)
        })
        .collect();

    groups.sort_by(|left, right| {
        right
            .wasted_bytes
            .cmp(&left.wasted_bytes)
            .then_with(|| right.file_size.cmp(&left.file_size))
            .then_with(|| left.files[0].path.cmp(&right.files[0].path))
    });
    for (index, group) in groups.iter_mut().enumerate() {
        group.id = DuplicateGroupId(u32::try_from(index).unwrap_or(u32::MAX));
    }

    apply_keep_rule_to_all_groups(&mut groups, &KeepRule::default());
    groups
}
