//! Typed events sent to the frontend. They tell the UI that background work ended even when
//! the progress channel it passed in is gone, for example after a page reload.

use serde::Serialize;
use specta::Type;
use tauri_specta::Event;

use crate::duplicate_search_registry::{DuplicateSearchId, DuplicateSearchOutcome};
use crate::scan_registry::{ScanId, ScanOutcome};

/// Emitted once per scan, after its result (or failure) is stored, so queries work as soon as
/// the frontend receives it.
#[derive(Debug, Clone, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ScanFinished {
    pub scan_id: ScanId,
    pub outcome: ScanOutcome,
}

/// Emitted once per duplicate search, after its groups (or failure) are stored.
#[derive(Debug, Clone, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSearchFinished {
    pub id: DuplicateSearchId,
    pub outcome: DuplicateSearchOutcome,
}
