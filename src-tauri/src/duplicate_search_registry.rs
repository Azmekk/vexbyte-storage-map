//! Every duplicate search of this session, running or done, keyed by its `DuplicateSearchId`.
//! Finished searches keep their groups here; the review screen pages through them.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};

use duplicate_finder::{
    CancellationToken, DuplicateGroup, DuplicateSearchError, DuplicateSearchResult, FolderDriveKind,
};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Identifies one duplicate search for the lifetime of the app. Sent to the frontend as a
/// number.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type,
)]
#[serde(transparent)]
pub struct DuplicateSearchId(u32);

impl DuplicateSearchId {
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// The groups of a finished search, shared between page queries (read lock) and keep choices or
/// recycle updates (write lock).
pub type SharedDuplicateGroups = Arc<RwLock<Vec<DuplicateGroup>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateSearchStatus {
    Running,
    Finished,
    Cancelled,
    Failed,
}

/// The headline numbers of a finished search, as it ended. Current group totals come with each
/// page of groups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSearchSummary {
    pub duplicate_group_count: u64,
    pub duplicate_file_count: u64,
    pub total_wasted_bytes: u64,
    pub files_examined: u64,
    pub online_only_files_skipped: u64,
    pub hard_links_collapsed: u64,
    pub unreadable_file_count: u64,
    pub inaccessible_folder_count: u64,
    pub folder_drive_kinds: Vec<FolderDriveKind>,
    pub elapsed_milliseconds: u64,
}

impl DuplicateSearchSummary {
    fn of(result: &DuplicateSearchResult) -> Self {
        Self {
            duplicate_group_count: result.groups.len() as u64,
            duplicate_file_count: result.duplicate_file_count,
            total_wasted_bytes: result.total_wasted_bytes,
            files_examined: result.files_examined,
            online_only_files_skipped: result.online_only_files_skipped,
            hard_links_collapsed: result.hard_links_collapsed,
            unreadable_file_count: result.unreadable_file_count,
            inaccessible_folder_count: result.inaccessible_folder_count,
            folder_drive_kinds: result.folder_drive_kinds.clone(),
            elapsed_milliseconds: result.elapsed_milliseconds,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DuplicateSearchFailure {
    InvalidSettings { details: String },
    SearchThreadFailed { details: String },
}

/// How a search ended, as sent with the `DuplicateSearchFinished` event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DuplicateSearchOutcome {
    Finished { summary: DuplicateSearchSummary },
    Cancelled,
    Failed { failure: DuplicateSearchFailure },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSearchOverview {
    pub id: DuplicateSearchId,
    pub status: DuplicateSearchStatus,
    /// Present once the search has ended.
    pub outcome: Option<DuplicateSearchOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DuplicateSearchLookupError {
    #[error("there is no duplicate search {}", id.get())]
    SearchNotFound { id: DuplicateSearchId },
    #[error("duplicate search {} has no result ({status:?})", id.get())]
    SearchNotFinished {
        id: DuplicateSearchId,
        status: DuplicateSearchStatus,
    },
}

enum SearchEntryState {
    Running,
    Ended {
        outcome: DuplicateSearchOutcome,
        groups: Option<SharedDuplicateGroups>,
    },
}

struct SearchEntry {
    cancellation: CancellationToken,
    state: SearchEntryState,
}

impl SearchEntry {
    fn status(&self) -> DuplicateSearchStatus {
        match &self.state {
            SearchEntryState::Running => DuplicateSearchStatus::Running,
            SearchEntryState::Ended { outcome, .. } => match outcome {
                DuplicateSearchOutcome::Finished { .. } => DuplicateSearchStatus::Finished,
                DuplicateSearchOutcome::Cancelled => DuplicateSearchStatus::Cancelled,
                DuplicateSearchOutcome::Failed { .. } => DuplicateSearchStatus::Failed,
            },
        }
    }
}

#[derive(Default)]
struct SearchRegistryContents {
    next_search_number: u32,
    entries: BTreeMap<DuplicateSearchId, SearchEntry>,
}

/// Thread-safe registry of duplicate searches.
#[derive(Default)]
pub struct DuplicateSearchRegistry {
    contents: Mutex<SearchRegistryContents>,
}

impl DuplicateSearchRegistry {
    fn lock(&self) -> MutexGuard<'_, SearchRegistryContents> {
        self.contents.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn register_running_search(&self) -> (DuplicateSearchId, CancellationToken) {
        let mut contents = self.lock();
        contents.next_search_number = contents.next_search_number.saturating_add(1);
        let id = DuplicateSearchId(contents.next_search_number);
        let cancellation = CancellationToken::new();

        contents.entries.insert(
            id,
            SearchEntry {
                cancellation: cancellation.clone(),
                state: SearchEntryState::Running,
            },
        );
        (id, cancellation)
    }

    /// Stores the result of a search that ended and returns how it ended.
    pub fn record_search_end(
        &self,
        id: DuplicateSearchId,
        result: Result<DuplicateSearchResult, DuplicateSearchError>,
    ) -> DuplicateSearchOutcome {
        let (outcome, groups) = match result {
            Ok(result) => {
                let summary = DuplicateSearchSummary::of(&result);
                (
                    DuplicateSearchOutcome::Finished { summary },
                    Some(Arc::new(RwLock::new(result.groups))),
                )
            }
            Err(DuplicateSearchError::Cancelled) => (DuplicateSearchOutcome::Cancelled, None),
            Err(DuplicateSearchError::InvalidSettings(error)) => {
                let failure = DuplicateSearchFailure::InvalidSettings {
                    details: crate::error_details::describe_error_with_sources(&error),
                };
                (DuplicateSearchOutcome::Failed { failure }, None)
            }
        };

        self.record_state(
            id,
            SearchEntryState::Ended {
                outcome: outcome.clone(),
                groups,
            },
        );
        outcome
    }

    /// Marks a search whose thread ended without returning, for example by panicking.
    pub fn record_search_thread_failure(
        &self,
        id: DuplicateSearchId,
        details: String,
    ) -> DuplicateSearchOutcome {
        let outcome = DuplicateSearchOutcome::Failed {
            failure: DuplicateSearchFailure::SearchThreadFailed { details },
        };
        self.record_state(
            id,
            SearchEntryState::Ended {
                outcome: outcome.clone(),
                groups: None,
            },
        );
        outcome
    }

    fn record_state(&self, id: DuplicateSearchId, state: SearchEntryState) {
        if let Some(entry) = self.lock().entries.get_mut(&id) {
            entry.state = state;
        }
    }

    /// # Errors
    ///
    /// Fails when there is no such search.
    pub fn cancel_search(&self, id: DuplicateSearchId) -> Result<(), DuplicateSearchLookupError> {
        let contents = self.lock();
        let entry = contents
            .entries
            .get(&id)
            .ok_or(DuplicateSearchLookupError::SearchNotFound { id })?;
        entry.cancellation.cancel();
        Ok(())
    }

    /// Forgets a search and its groups. A running search is cancelled first.
    ///
    /// # Errors
    ///
    /// Fails when there is no such search.
    pub fn discard_search(&self, id: DuplicateSearchId) -> Result<(), DuplicateSearchLookupError> {
        let entry = self
            .lock()
            .entries
            .remove(&id)
            .ok_or(DuplicateSearchLookupError::SearchNotFound { id })?;
        entry.cancellation.cancel();
        Ok(())
    }

    /// The groups of a finished search.
    ///
    /// # Errors
    ///
    /// Fails when there is no such search, or it is still running, was cancelled or failed.
    pub fn finished_groups(
        &self,
        id: DuplicateSearchId,
    ) -> Result<SharedDuplicateGroups, DuplicateSearchLookupError> {
        let contents = self.lock();
        let entry = contents
            .entries
            .get(&id)
            .ok_or(DuplicateSearchLookupError::SearchNotFound { id })?;

        match &entry.state {
            SearchEntryState::Ended {
                groups: Some(groups),
                ..
            } => Ok(Arc::clone(groups)),
            _ => Err(DuplicateSearchLookupError::SearchNotFinished {
                id,
                status: entry.status(),
            }),
        }
    }

    /// # Errors
    ///
    /// Fails when there is no such search.
    pub fn search_overview(
        &self,
        id: DuplicateSearchId,
    ) -> Result<DuplicateSearchOverview, DuplicateSearchLookupError> {
        let contents = self.lock();
        let entry = contents
            .entries
            .get(&id)
            .ok_or(DuplicateSearchLookupError::SearchNotFound { id })?;

        let outcome = match &entry.state {
            SearchEntryState::Running => None,
            SearchEntryState::Ended { outcome, .. } => Some(outcome.clone()),
        };
        Ok(DuplicateSearchOverview {
            id,
            status: entry.status(),
            outcome,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn empty_result() -> DuplicateSearchResult {
        DuplicateSearchResult {
            groups: Vec::new(),
            duplicate_file_count: 0,
            total_wasted_bytes: 0,
            files_examined: 12,
            online_only_files_skipped: 0,
            hard_links_collapsed: 0,
            unreadable_file_count: 0,
            inaccessible_folder_count: 0,
            folder_drive_kinds: Vec::new(),
            elapsed_milliseconds: 5,
        }
    }

    #[test]
    fn allocates_increasing_ids() {
        let registry = DuplicateSearchRegistry::default();

        let (first, _) = registry.register_running_search();
        let (second, _) = registry.register_running_search();

        assert_eq!((first.get(), second.get()), (1, 2));
    }

    #[test]
    fn a_finished_search_keeps_its_groups() {
        let registry = DuplicateSearchRegistry::default();
        let (id, _) = registry.register_running_search();
        assert_eq!(
            registry.finished_groups(id).err(),
            Some(DuplicateSearchLookupError::SearchNotFinished {
                id,
                status: DuplicateSearchStatus::Running
            })
        );

        let outcome = registry.record_search_end(id, Ok(empty_result()));

        assert!(matches!(
            outcome,
            DuplicateSearchOutcome::Finished { ref summary } if summary.files_examined == 12
        ));
        assert!(registry.finished_groups(id).is_ok());
        assert_eq!(
            registry.search_overview(id).unwrap().status,
            DuplicateSearchStatus::Finished
        );
    }

    #[test]
    fn cancelled_searches_have_no_groups() {
        let registry = DuplicateSearchRegistry::default();
        let (id, token) = registry.register_running_search();

        registry.cancel_search(id).unwrap();
        assert!(token.is_cancelled());
        registry.record_search_end(id, Err(DuplicateSearchError::Cancelled));

        assert_eq!(
            registry.finished_groups(id).err(),
            Some(DuplicateSearchLookupError::SearchNotFinished {
                id,
                status: DuplicateSearchStatus::Cancelled
            })
        );
    }

    #[test]
    fn discarded_searches_are_gone() {
        let registry = DuplicateSearchRegistry::default();
        let (id, token) = registry.register_running_search();

        registry.discard_search(id).unwrap();

        assert!(token.is_cancelled());
        assert_eq!(
            registry.search_overview(id).err(),
            Some(DuplicateSearchLookupError::SearchNotFound { id })
        );
    }
}
