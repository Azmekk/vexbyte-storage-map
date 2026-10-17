use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::{Arc, PoisonError};

use duplicate_finder::{
    ChangedFile, DuplicateGroup, DuplicateGroupId, DuplicateSearchProgress,
    DuplicateSearchSettings, KeepRule, RemovalPlanError, SearchSettingsError, SuggestedFolder,
};
use file_actions::RecycleFailure;
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::app_state::AppState;
use crate::duplicate_group_store::{
    self, DuplicateGroupSortOrder, DuplicateGroupsPage, DuplicateSelectionSummary, KeepChangeError,
};
use crate::duplicate_search_registry::{
    DuplicateSearchId, DuplicateSearchLookupError, DuplicateSearchOutcome, DuplicateSearchOverview,
    DuplicateSearchStatus,
};
use crate::error_details::{describe_error_with_sources, describe_panic};
use crate::events::DuplicateSearchFinished;

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DuplicateCommandError {
    #[error("the duplicate search settings are not valid: {details}")]
    InvalidSettings { details: String },
    #[error("{folder} is on a streamed cloud drive and cannot be searched")]
    CloudVirtualFolderNotAllowed { folder: String },
    #[error("there is no duplicate search {id}")]
    SearchNotFound { id: u32 },
    #[error("duplicate search {id} has no result ({status:?})")]
    SearchNotFinished {
        id: u32,
        status: DuplicateSearchStatus,
    },
    #[error("there is no duplicate group {group_id}")]
    GroupNotFound { group_id: u32 },
    #[error("{path} is not part of duplicate group {group_id}")]
    FileNotInGroup { group_id: u32, path: String },
    #[error("{path} is not part of any duplicate group")]
    FileNotInAnyGroup { path: String },
    #[error("duplicate group {group_id} must keep at least one file")]
    GroupWouldHaveNoKeptFile { group_id: u32 },
    #[error("the Recycle Bin operation failed: {details}")]
    RecycleFailed { details: String },
    #[error("the duplicate search could not be started: {details}")]
    SearchStartFailed { details: String },
    #[error("the operation stopped unexpectedly: {details}")]
    BackgroundTaskFailed { details: String },
}

impl From<DuplicateSearchLookupError> for DuplicateCommandError {
    fn from(error: DuplicateSearchLookupError) -> Self {
        match error {
            DuplicateSearchLookupError::SearchNotFound { id } => {
                Self::SearchNotFound { id: id.get() }
            }
            DuplicateSearchLookupError::SearchNotFinished { id, status } => {
                Self::SearchNotFinished {
                    id: id.get(),
                    status,
                }
            }
        }
    }
}

impl From<SearchSettingsError> for DuplicateCommandError {
    fn from(error: SearchSettingsError) -> Self {
        match error {
            SearchSettingsError::CloudVirtualFolderNotAllowed { folder } => {
                Self::CloudVirtualFolderNotAllowed {
                    folder: folder.display().to_string(),
                }
            }
            other => Self::InvalidSettings {
                details: describe_error_with_sources(&other),
            },
        }
    }
}

impl From<KeepChangeError> for DuplicateCommandError {
    fn from(error: KeepChangeError) -> Self {
        match error {
            KeepChangeError::GroupNotFound { group_id } => Self::GroupNotFound {
                group_id: group_id.0,
            },
            KeepChangeError::FileNotInGroup { group_id, path } => Self::FileNotInGroup {
                group_id: group_id.0,
                path: path.display().to_string(),
            },
            KeepChangeError::GroupWouldHaveNoKeptFile { group_id } => {
                Self::GroupWouldHaveNoKeptFile {
                    group_id: group_id.0,
                }
            }
        }
    }
}

impl From<RemovalPlanError> for DuplicateCommandError {
    fn from(error: RemovalPlanError) -> Self {
        match error {
            RemovalPlanError::GroupWouldHaveNoKeptFile { group_id } => {
                Self::GroupWouldHaveNoKeptFile {
                    group_id: group_id.0,
                }
            }
            RemovalPlanError::FileNotInAnyGroup { path } => Self::FileNotInAnyGroup {
                path: path.display().to_string(),
            },
        }
    }
}

impl From<tauri::Error> for DuplicateCommandError {
    fn from(error: tauri::Error) -> Self {
        Self::BackgroundTaskFailed {
            details: error.to_string(),
        }
    }
}

/// What `recycle_duplicates` did.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateRecycleResult {
    pub moved: Vec<PathBuf>,
    pub failed: Vec<RecycleFailure>,
    /// Files skipped because they (or every kept copy of their group) changed since the search.
    pub changed: Vec<ChangedFile>,
    pub bytes_freed: u64,
    /// The selection over the groups that remain after the moved files were taken out.
    pub selection: DuplicateSelectionSummary,
}

/// The known personal folders (Pictures, Videos, Music, Documents, Downloads, Desktop) that
/// exist on this computer.
#[tauri::command]
#[specta::specta]
pub async fn get_default_duplicate_folders() -> Result<Vec<SuggestedFolder>, DuplicateCommandError>
{
    let folders =
        tauri::async_runtime::spawn_blocking(duplicate_finder::suggested_duplicate_folders).await?;
    Ok(folders)
}

/// Checks one folder the user wants to add to the search, so the setup step can refuse a system
/// folder or a folder on a cloud drive right away. Rejects with `invalidSettings` or
/// `cloudVirtualFolderNotAllowed`.
#[tauri::command]
#[specta::specta]
pub async fn check_duplicate_search_folder(folder: PathBuf) -> Result<(), DuplicateCommandError> {
    tauri::async_runtime::spawn_blocking(move || duplicate_finder::validate_search_folder(&folder))
        .await??;
    Ok(())
}

/// Checks `settings` and starts the search on a background thread; returns its id right away.
///
/// Progress arrives on `progress`. When the search ends, the `DuplicateSearchFinished` event is
/// emitted and `get_duplicate_groups` works.
#[tauri::command]
#[specta::specta]
pub async fn start_duplicate_search(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: DuplicateSearchSettings,
    progress: Channel<DuplicateSearchProgress>,
) -> Result<DuplicateSearchId, DuplicateCommandError> {
    let validated_settings =
        tauri::async_runtime::spawn_blocking(move || settings.validate()).await??;

    let (id, cancellation) = state.duplicate_searches.register_running_search();

    let thread_app = app.clone();
    let spawn_result = std::thread::Builder::new()
        .name(format!("vsm-duplicate-search-{}", id.get()))
        .spawn(move || {
            let report_progress = |update: &DuplicateSearchProgress| {
                let _ = progress.send(update.clone());
            };
            let search_result = catch_unwind(AssertUnwindSafe(|| {
                duplicate_finder::find_duplicates_with_validated_settings(
                    &validated_settings,
                    report_progress,
                    &cancellation,
                )
            }));

            let registry = &thread_app.state::<AppState>().duplicate_searches;
            let outcome = match search_result {
                Ok(result) => registry.record_search_end(id, result),
                Err(panic_payload) => registry
                    .record_search_thread_failure(id, describe_panic(panic_payload.as_ref())),
            };
            emit_duplicate_search_finished(&thread_app, id, outcome);
        });

    if let Err(error) = spawn_result {
        state
            .duplicate_searches
            .record_search_thread_failure(id, error.to_string());
        return Err(DuplicateCommandError::SearchStartFailed {
            details: error.to_string(),
        });
    }

    Ok(id)
}

fn emit_duplicate_search_finished(
    app: &AppHandle,
    id: DuplicateSearchId,
    outcome: DuplicateSearchOutcome,
) {
    if let Err(error) = (DuplicateSearchFinished { id, outcome }).emit(app) {
        tracing::warn!(%error, id = id.get(), "could not emit the duplicate search finished event");
    }
}

/// Asks a running search to stop. Cancelling a search that already ended does nothing.
#[tauri::command]
#[specta::specta]
pub async fn cancel_duplicate_search(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
) -> Result<(), DuplicateCommandError> {
    state.duplicate_searches.cancel_search(id)?;
    Ok(())
}

/// Status of a search plus, once it ended, how it ended.
#[tauri::command]
#[specta::specta]
pub async fn get_duplicate_search_summary(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
) -> Result<DuplicateSearchOverview, DuplicateCommandError> {
    Ok(state.duplicate_searches.search_overview(id)?)
}

/// Forgets a search and its groups. A running search is cancelled first.
#[tauri::command]
#[specta::specta]
pub async fn discard_duplicate_search(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
) -> Result<(), DuplicateCommandError> {
    state.duplicate_searches.discard_search(id)?;
    Ok(())
}

/// One page of the groups of a finished search, with totals and the current selection.
#[tauri::command]
#[specta::specta]
pub async fn get_duplicate_groups(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
    sort: DuplicateGroupSortOrder,
    offset: u32,
    limit: u32,
) -> Result<DuplicateGroupsPage, DuplicateCommandError> {
    let groups = state.duplicate_searches.finished_groups(id)?;
    let groups = groups.read().unwrap_or_else(PoisonError::into_inner);

    Ok(duplicate_group_store::page_of_groups(
        &groups, sort, offset, limit,
    ))
}

/// Applies a keep rule to every group, replacing choices made by hand.
#[tauri::command]
#[specta::specta]
pub async fn set_keep_rule(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
    rule: KeepRule,
) -> Result<DuplicateSelectionSummary, DuplicateCommandError> {
    let groups = state.duplicate_searches.finished_groups(id)?;
    let mut groups = groups.write().unwrap_or_else(PoisonError::into_inner);

    Ok(duplicate_group_store::apply_keep_rule(&mut groups, &rule))
}

/// Keeps or removes one file by hand. Refuses to remove the last kept file of a group with the
/// `groupWouldHaveNoKeptFile` error.
#[tauri::command]
#[specta::specta]
pub async fn set_file_keep(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
    group_id: DuplicateGroupId,
    path: PathBuf,
    keep: bool,
) -> Result<DuplicateGroup, DuplicateCommandError> {
    let groups = state.duplicate_searches.finished_groups(id)?;
    let mut groups = groups.write().unwrap_or_else(PoisonError::into_inner);

    Ok(duplicate_group_store::set_file_keep(
        &mut groups,
        group_id,
        &path,
        keep,
    )?)
}

/// Moves `files_to_remove` to the Recycle Bin after checking that every group keeps a copy and
/// that nothing changed since the search. Moved files leave their groups.
#[tauri::command]
#[specta::specta]
pub async fn recycle_duplicates(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
    files_to_remove: Vec<PathBuf>,
) -> Result<DuplicateRecycleResult, DuplicateCommandError> {
    let shared_groups = state.duplicate_searches.finished_groups(id)?;
    let plan = {
        let groups = shared_groups.read().unwrap_or_else(PoisonError::into_inner);
        duplicate_group_store::plan_removal(&groups, &files_to_remove)?
    };
    if plan.is_empty() {
        return Ok(DuplicateRecycleResult::default());
    }

    let recycle_bin_operations = Arc::clone(&state.recycle_bin_operations);
    let (revalidation, recycle_result) = tauri::async_runtime::spawn_blocking(move || {
        let _exclusive_recycle = recycle_bin_operations
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let revalidation = duplicate_finder::revalidate_before_removal(&plan);
        let paths_to_remove = revalidation.paths_to_remove();
        let recycle_result = if paths_to_remove.is_empty() {
            Ok(file_actions::RecycleResult::default())
        } else {
            file_actions::move_to_recycle_bin(&paths_to_remove)
        };
        (revalidation, recycle_result)
    })
    .await?;
    let recycle_result = recycle_result.map_err(|error| DuplicateCommandError::RecycleFailed {
        details: describe_error_with_sources(&error),
    })?;

    let selection = {
        let mut groups = shared_groups
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        duplicate_group_store::remove_recycled_files(&mut groups, &recycle_result.moved);
        duplicate_group_store::selection_summary(&groups)
    };

    Ok(DuplicateRecycleResult {
        moved: recycle_result.moved,
        failed: recycle_result.failed,
        changed: revalidation.changed,
        bytes_freed: recycle_result.bytes_freed,
        selection,
    })
}
