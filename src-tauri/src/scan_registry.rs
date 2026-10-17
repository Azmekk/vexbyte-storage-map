//! Every scan of this session, running or done, keyed by its `ScanId`. Finished scans keep their
//! file tree here; the frontend only ever asks for small views of it.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};

use disk_scanner::{
    CancellationToken, FileTree, NodeIndex, ScanError, ScanMethod, ScanResult, ScanSummary,
    ScanTarget, TreeTotals,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error_details::describe_error_with_sources;

/// Identifies one scan for the lifetime of the app. Sent to the frontend as a number.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type,
)]
#[serde(transparent)]
pub struct ScanId(u32);

impl ScanId {
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A finished tree, shared between concurrent queries (read lock) and recycle updates (write
/// lock).
pub type SharedFileTree = Arc<RwLock<FileTree>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ScanStatus {
    Running,
    Finished,
    Cancelled,
    Failed,
}

/// Why a scan stopped without a result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ScanFailure {
    FastScanRequiresElevation,
    FastScanUnsupported { path: String },
    VolumeLookupFailed { path: String, details: String },
    FolderNotFoundOnVolume { path: String },
    CloudVirtualVolumeNotScanned { mount_point: String },
    FolderWalkFailed { details: String },
    MasterFileTableScanFailed { details: String },
    ScanThreadFailed { details: String },
}

impl ScanFailure {
    /// `None` for a cancelled scan, which is not a failure.
    pub fn from_scan_error(error: &ScanError) -> Option<Self> {
        let failure = match error {
            ScanError::Cancelled => return None,
            ScanError::FastScanRequiresElevation => Self::FastScanRequiresElevation,
            ScanError::FastScanUnsupported { path } => Self::FastScanUnsupported {
                path: path.display().to_string(),
            },
            ScanError::VolumeLookupFailed { path, .. } => Self::VolumeLookupFailed {
                path: path.display().to_string(),
                details: describe_error_with_sources(error),
            },
            ScanError::FolderNotFoundOnVolume { path } => Self::FolderNotFoundOnVolume {
                path: path.display().to_string(),
            },
            ScanError::CloudVirtualVolumeNotScanned { mount_point } => {
                Self::CloudVirtualVolumeNotScanned {
                    mount_point: mount_point.display().to_string(),
                }
            }
            ScanError::FolderWalkFailed(_) => Self::FolderWalkFailed {
                details: describe_error_with_sources(error),
            },
            ScanError::MasterFileTableScanFailed(_) => Self::MasterFileTableScanFailed {
                details: describe_error_with_sources(error),
            },
        };
        Some(failure)
    }
}

/// How a scan ended, as sent with the `ScanFinished` event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ScanOutcome {
    Finished { summary: ScanSummary },
    Cancelled,
    Failed { failure: ScanFailure },
}

/// Where a scan stands right now, for `get_scan_summary` and the list of recent scans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ScanState {
    Running,
    Finished {
        /// The numbers at the end of the scan.
        summary: ScanSummary,
        /// Up-to-date totals; they shrink after items are moved to the Recycle Bin.
        current_totals: TreeTotals,
        root_node: NodeIndex,
    },
    Cancelled,
    Failed {
        failure: ScanFailure,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ScanOverview {
    pub scan_id: ScanId,
    pub target: ScanTarget,
    pub method: ScanMethod,
    pub status: ScanStatus,
    pub state: ScanState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ScanLookupError {
    #[error("there is no scan {}", scan_id.get())]
    ScanNotFound { scan_id: ScanId },
    #[error("scan {} has no result ({status:?})", scan_id.get())]
    ScanNotFinished { scan_id: ScanId, status: ScanStatus },
}

enum ScanEntryState {
    Running,
    Finished {
        tree: SharedFileTree,
        summary: Box<ScanSummary>,
    },
    Cancelled,
    Failed(ScanFailure),
}

struct ScanEntry {
    target: ScanTarget,
    method: ScanMethod,
    cancellation: CancellationToken,
    state: ScanEntryState,
}

impl ScanEntry {
    fn status(&self) -> ScanStatus {
        match self.state {
            ScanEntryState::Running => ScanStatus::Running,
            ScanEntryState::Finished { .. } => ScanStatus::Finished,
            ScanEntryState::Cancelled => ScanStatus::Cancelled,
            ScanEntryState::Failed(_) => ScanStatus::Failed,
        }
    }
}

#[derive(Default)]
struct ScanRegistryContents {
    next_scan_number: u32,
    entries: BTreeMap<ScanId, ScanEntry>,
}

/// Thread-safe registry of scans. The lock is held only to look up or update an entry, never
/// while a scan runs or a tree is queried.
#[derive(Default)]
pub struct ScanRegistry {
    contents: Mutex<ScanRegistryContents>,
}

impl ScanRegistry {
    fn lock(&self) -> MutexGuard<'_, ScanRegistryContents> {
        self.contents.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Records a new running scan and returns its id and the token that cancels it.
    pub fn register_running_scan(
        &self,
        target: ScanTarget,
        method: ScanMethod,
    ) -> (ScanId, CancellationToken) {
        let mut contents = self.lock();
        contents.next_scan_number = contents.next_scan_number.saturating_add(1);
        let scan_id = ScanId(contents.next_scan_number);
        let cancellation = CancellationToken::new();

        contents.entries.insert(
            scan_id,
            ScanEntry {
                target,
                method,
                cancellation: cancellation.clone(),
                state: ScanEntryState::Running,
            },
        );
        (scan_id, cancellation)
    }

    /// Stores the result of a scan that ended and returns how it ended. A scan discarded while
    /// it ran keeps no result.
    pub fn record_scan_end(
        &self,
        scan_id: ScanId,
        result: Result<ScanResult, ScanError>,
    ) -> ScanOutcome {
        let (state, outcome) = match result {
            Ok(ScanResult { tree, summary }) => {
                let outcome = ScanOutcome::Finished {
                    summary: summary.clone(),
                };
                let state = ScanEntryState::Finished {
                    tree: Arc::new(RwLock::new(tree)),
                    summary: Box::new(summary),
                };
                (state, outcome)
            }
            Err(error) => match ScanFailure::from_scan_error(&error) {
                None => (ScanEntryState::Cancelled, ScanOutcome::Cancelled),
                Some(failure) => (
                    ScanEntryState::Failed(failure.clone()),
                    ScanOutcome::Failed { failure },
                ),
            },
        };

        self.record_state(scan_id, state);
        outcome
    }

    /// Marks a scan whose thread ended without returning, for example by panicking.
    pub fn record_scan_thread_failure(&self, scan_id: ScanId, details: String) -> ScanOutcome {
        let failure = ScanFailure::ScanThreadFailed { details };
        self.record_state(scan_id, ScanEntryState::Failed(failure.clone()));
        ScanOutcome::Failed { failure }
    }

    fn record_state(&self, scan_id: ScanId, state: ScanEntryState) {
        let mut contents = self.lock();
        if let Some(entry) = contents.entries.get_mut(&scan_id) {
            entry.state = state;
        }
    }

    /// Asks a running scan to stop. Stopping a scan that already ended does nothing.
    ///
    /// # Errors
    ///
    /// Fails when there is no such scan.
    pub fn cancel_scan(&self, scan_id: ScanId) -> Result<(), ScanLookupError> {
        let contents = self.lock();
        let entry = contents
            .entries
            .get(&scan_id)
            .ok_or(ScanLookupError::ScanNotFound { scan_id })?;
        entry.cancellation.cancel();
        Ok(())
    }

    /// Forgets a scan and frees its tree once no query uses it any more. A running scan is
    /// cancelled first.
    ///
    /// # Errors
    ///
    /// Fails when there is no such scan.
    pub fn discard_scan(&self, scan_id: ScanId) -> Result<(), ScanLookupError> {
        let entry = self
            .lock()
            .entries
            .remove(&scan_id)
            .ok_or(ScanLookupError::ScanNotFound { scan_id })?;
        entry.cancellation.cancel();
        Ok(())
    }

    /// The tree of a finished scan.
    ///
    /// # Errors
    ///
    /// Fails when there is no such scan, or it is still running, was cancelled or failed.
    pub fn finished_tree(&self, scan_id: ScanId) -> Result<SharedFileTree, ScanLookupError> {
        let contents = self.lock();
        let entry = contents
            .entries
            .get(&scan_id)
            .ok_or(ScanLookupError::ScanNotFound { scan_id })?;

        match &entry.state {
            ScanEntryState::Finished { tree, .. } => Ok(Arc::clone(tree)),
            _ => Err(ScanLookupError::ScanNotFinished {
                scan_id,
                status: entry.status(),
            }),
        }
    }

    /// # Errors
    ///
    /// Fails when there is no such scan.
    pub fn scan_overview(&self, scan_id: ScanId) -> Result<ScanOverview, ScanLookupError> {
        let snapshot = {
            let contents = self.lock();
            let entry = contents
                .entries
                .get(&scan_id)
                .ok_or(ScanLookupError::ScanNotFound { scan_id })?;
            ScanEntrySnapshot::of(scan_id, entry)
        };

        Ok(snapshot.into_overview())
    }

    /// Every scan of this session, oldest first.
    pub fn all_scan_overviews(&self) -> Vec<ScanOverview> {
        let snapshots: Vec<ScanEntrySnapshot> = self
            .lock()
            .entries
            .iter()
            .map(|(&scan_id, entry)| ScanEntrySnapshot::of(scan_id, entry))
            .collect();

        snapshots
            .into_iter()
            .map(ScanEntrySnapshot::into_overview)
            .collect()
    }
}

/// A copy of an entry taken under the registry lock, so the tree's own lock is only taken
/// after the registry lock is released.
struct ScanEntrySnapshot {
    scan_id: ScanId,
    target: ScanTarget,
    method: ScanMethod,
    status: ScanStatus,
    finished: Option<(SharedFileTree, ScanSummary)>,
    failure: Option<ScanFailure>,
}

impl ScanEntrySnapshot {
    fn of(scan_id: ScanId, entry: &ScanEntry) -> Self {
        let (finished, failure) = match &entry.state {
            ScanEntryState::Finished { tree, summary } => {
                (Some((Arc::clone(tree), summary.as_ref().clone())), None)
            }
            ScanEntryState::Failed(failure) => (None, Some(failure.clone())),
            ScanEntryState::Running | ScanEntryState::Cancelled => (None, None),
        };

        Self {
            scan_id,
            target: entry.target.clone(),
            method: entry.method,
            status: entry.status(),
            finished,
            failure,
        }
    }

    fn into_overview(self) -> ScanOverview {
        let state = match (self.status, self.finished, self.failure) {
            (ScanStatus::Finished, Some((tree, summary)), _) => {
                let tree = tree.read().unwrap_or_else(PoisonError::into_inner);
                ScanState::Finished {
                    summary,
                    current_totals: tree.totals(),
                    root_node: tree.root(),
                }
            }
            (ScanStatus::Failed, _, Some(failure)) => ScanState::Failed { failure },
            (ScanStatus::Cancelled, _, _) => ScanState::Cancelled,
            _ => ScanState::Running,
        };

        ScanOverview {
            scan_id: self.scan_id,
            target: self.target,
            method: self.method,
            status: self.status,
            state,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;

    use super::*;

    fn folder_target() -> ScanTarget {
        ScanTarget::Folder {
            path: std::env::temp_dir(),
        }
    }

    fn register(registry: &ScanRegistry) -> (ScanId, CancellationToken) {
        registry.register_running_scan(folder_target(), ScanMethod::FolderWalk)
    }

    #[test]
    fn allocates_increasing_ids_starting_at_one() {
        let registry = ScanRegistry::default();

        let (first, _) = register(&registry);
        let (second, _) = register(&registry);

        assert_eq!(first.get(), 1);
        assert_eq!(second.get(), 2);
        assert_eq!(registry.all_scan_overviews().len(), 2);
    }

    #[test]
    fn a_running_scan_has_no_tree_yet() {
        let registry = ScanRegistry::default();
        let (scan_id, _) = register(&registry);

        assert_eq!(
            registry.finished_tree(scan_id).err(),
            Some(ScanLookupError::ScanNotFinished {
                scan_id,
                status: ScanStatus::Running
            })
        );
        assert_eq!(
            registry.scan_overview(scan_id).unwrap().state,
            ScanState::Running
        );
    }

    #[test]
    fn cancelling_sets_the_token_and_a_cancelled_end_is_recorded() {
        let registry = ScanRegistry::default();
        let (scan_id, token) = register(&registry);

        registry.cancel_scan(scan_id).unwrap();
        assert!(token.is_cancelled());

        let outcome = registry.record_scan_end(scan_id, Err(ScanError::Cancelled));
        assert_eq!(outcome, ScanOutcome::Cancelled);
        assert_eq!(
            registry.scan_overview(scan_id).unwrap().status,
            ScanStatus::Cancelled
        );
    }

    #[test]
    fn failures_keep_their_reason() {
        let registry = ScanRegistry::default();
        let (scan_id, _) = register(&registry);

        let outcome = registry.record_scan_end(
            scan_id,
            Err(ScanError::FolderNotFoundOnVolume {
                path: PathBuf::from("missing"),
            }),
        );

        let expected_failure = ScanFailure::FolderNotFoundOnVolume {
            path: "missing".to_owned(),
        };
        assert_eq!(
            outcome,
            ScanOutcome::Failed {
                failure: expected_failure.clone()
            }
        );
        assert_eq!(
            registry.scan_overview(scan_id).unwrap().state,
            ScanState::Failed {
                failure: expected_failure
            }
        );
    }

    #[test]
    fn a_thread_failure_marks_the_scan_failed() {
        let registry = ScanRegistry::default();
        let (scan_id, _) = register(&registry);

        registry.record_scan_thread_failure(scan_id, "panicked".to_owned());

        assert_eq!(
            registry.scan_overview(scan_id).unwrap().status,
            ScanStatus::Failed
        );
    }

    #[test]
    fn unknown_and_discarded_scans_are_not_found() {
        let registry = ScanRegistry::default();
        let unknown = ScanId(42);
        assert_eq!(
            registry.cancel_scan(unknown),
            Err(ScanLookupError::ScanNotFound { scan_id: unknown })
        );

        let (scan_id, token) = register(&registry);
        registry.discard_scan(scan_id).unwrap();

        assert!(token.is_cancelled());
        assert!(registry.scan_overview(scan_id).is_err());
        assert_eq!(
            registry.record_scan_end(scan_id, Err(ScanError::Cancelled)),
            ScanOutcome::Cancelled
        );
        assert_eq!(registry.all_scan_overviews(), Vec::new());
    }
}
