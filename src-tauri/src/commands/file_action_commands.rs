use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, PoisonError};

use disk_scanner::{FileTree, NodeIndex, RemovalSummary, TreeQueryError, TreeTotals};
use file_actions::RecycleResult;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};

use crate::app_state::AppState;
use crate::duplicate_search_registry::{DuplicateSearchId, DuplicateSearchLookupError};
use crate::error_details::describe_error_with_sources;
use crate::scan_registry::{ScanId, ScanLookupError, ScanStatus};
use crate::thumbnail_asset_scope::allow_thumbnail_folder;

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum FileActionCommandError {
    #[error("there is no scan {scan_id}")]
    ScanNotFound { scan_id: u32 },
    #[error("scan {scan_id} has no file tree ({status:?})")]
    ScanNotFinished { scan_id: u32, status: ScanStatus },
    #[error("item {node} does not exist in this scan")]
    NodeNotFound { node: u32 },
    #[error("item {node} was moved to the Recycle Bin")]
    NodeRemoved { node: u32 },
    #[error("item {node} has no path on disk")]
    NodeHasNoPath { node: u32 },
    #[error("the scanned folder itself cannot be moved to the Recycle Bin")]
    CannotRecycleScanRoot,
    #[error("there is no duplicate search {id}")]
    SearchNotFound { id: u32 },
    #[error("{path} is not part of this duplicate search")]
    FileNotInSearch { path: String },
    #[error("could not show {path} in Explorer: {details}")]
    RevealFailed { path: String, details: String },
    #[error("could not open {path}: {details}")]
    OpenFailed { path: String, details: String },
    #[error("the Recycle Bin operation failed: {details}")]
    RecycleFailed { details: String },
    #[error("could not create a thumbnail of {path}: {details}")]
    ThumbnailFailed { path: String, details: String },
    #[error("the operation stopped unexpectedly: {details}")]
    BackgroundTaskFailed { details: String },
}

impl From<ScanLookupError> for FileActionCommandError {
    fn from(error: ScanLookupError) -> Self {
        match error {
            ScanLookupError::ScanNotFound { scan_id } => Self::ScanNotFound {
                scan_id: scan_id.get(),
            },
            ScanLookupError::ScanNotFinished { scan_id, status } => Self::ScanNotFinished {
                scan_id: scan_id.get(),
                status,
            },
        }
    }
}

impl From<TreeQueryError> for FileActionCommandError {
    fn from(error: TreeQueryError) -> Self {
        match error {
            TreeQueryError::NodeNotFound { node } => Self::NodeNotFound { node: node.get() },
            TreeQueryError::NodeRemoved { node } => Self::NodeRemoved { node: node.get() },
        }
    }
}

impl From<DuplicateSearchLookupError> for FileActionCommandError {
    fn from(error: DuplicateSearchLookupError) -> Self {
        match error {
            DuplicateSearchLookupError::SearchNotFound { id }
            | DuplicateSearchLookupError::SearchNotFinished { id, .. } => {
                Self::SearchNotFound { id: id.get() }
            }
        }
    }
}

impl From<tauri::Error> for FileActionCommandError {
    fn from(error: tauri::Error) -> Self {
        Self::BackgroundTaskFailed {
            details: error.to_string(),
        }
    }
}

/// What `move_to_recycle_bin` did to the disk and to the tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TreeRecycleResult {
    /// Per path: moved, or why not. Its `bytesFreed` counts files only; `removal` has the
    /// space freed including folders.
    pub recycle_result: RecycleResult,
    /// The moved items taken out of the tree.
    pub removal: RemovalSummary,
    /// The tree's totals after the removal.
    pub totals: TreeTotals,
}

/// A cached thumbnail PNG for the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailHandle {
    /// Load it with `convertFileSrc`; it lies inside the asset protocol scope.
    pub png_path: PathBuf,
    /// True when no content preview existed and the file type icon is shown instead.
    pub is_file_type_icon: bool,
}

fn live_node_path(tree: &FileTree, node: NodeIndex) -> Result<PathBuf, FileActionCommandError> {
    let details = tree.node_details(node)?;
    details
        .full_path
        .ok_or(FileActionCommandError::NodeHasNoPath { node: node.get() })
}

fn scan_node_path(
    state: &AppState,
    scan_id: ScanId,
    node: NodeIndex,
) -> Result<PathBuf, FileActionCommandError> {
    let tree = state.scans.finished_tree(scan_id)?;
    let tree = tree.read().unwrap_or_else(PoisonError::into_inner);
    live_node_path(&tree, node)
}

/// Only paths that appear in the search's groups may be shown or opened through it.
fn duplicate_file_path(
    state: &AppState,
    id: DuplicateSearchId,
    path: PathBuf,
) -> Result<PathBuf, FileActionCommandError> {
    let groups = state.duplicate_searches.finished_groups(id)?;
    let groups = groups.read().unwrap_or_else(PoisonError::into_inner);
    let is_in_search = groups
        .iter()
        .flat_map(|group| &group.files)
        .any(|file| file.path == path);

    if is_in_search {
        Ok(path)
    } else {
        Err(FileActionCommandError::FileNotInSearch {
            path: path.display().to_string(),
        })
    }
}

async fn reveal_path(path: PathBuf) -> Result<(), FileActionCommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        file_actions::reveal_in_explorer(&path).map_err(|error| {
            FileActionCommandError::RevealFailed {
                path: path.display().to_string(),
                details: describe_error_with_sources(&error),
            }
        })
    })
    .await?
}

async fn open_path(path: PathBuf) -> Result<(), FileActionCommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        file_actions::open_with_default_app(&path).map_err(|error| {
            FileActionCommandError::OpenFailed {
                path: path.display().to_string(),
                details: describe_error_with_sources(&error),
            }
        })
    })
    .await?
}

/// Opens an Explorer window with the item selected.
#[tauri::command]
#[specta::specta]
pub async fn reveal_in_explorer(
    state: State<'_, AppState>,
    scan_id: ScanId,
    node: NodeIndex,
) -> Result<(), FileActionCommandError> {
    let path = scan_node_path(&state, scan_id, node)?;
    reveal_path(path).await
}

/// Opens the item with its default application, or a folder in Explorer.
#[tauri::command]
#[specta::specta]
pub async fn open_with_default_app(
    state: State<'_, AppState>,
    scan_id: ScanId,
    node: NodeIndex,
) -> Result<(), FileActionCommandError> {
    let path = scan_node_path(&state, scan_id, node)?;
    open_path(path).await
}

/// Opens an Explorer window with one copy of a duplicate selected.
#[tauri::command]
#[specta::specta]
pub async fn reveal_duplicate_in_explorer(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
    path: PathBuf,
) -> Result<(), FileActionCommandError> {
    let path = duplicate_file_path(&state, id, path)?;
    reveal_path(path).await
}

/// Opens one copy of a duplicate with its default application.
#[tauri::command]
#[specta::specta]
pub async fn open_duplicate_with_default_app(
    state: State<'_, AppState>,
    id: DuplicateSearchId,
    path: PathBuf,
) -> Result<(), FileActionCommandError> {
    let path = duplicate_file_path(&state, id, path)?;
    open_path(path).await
}

/// Moves items to the Recycle Bin, then takes the moved ones out of the tree so sizes update
/// without a rescan. Items the shell did not move stay in the tree and are listed as failed.
#[tauri::command]
#[specta::specta]
pub async fn move_to_recycle_bin(
    state: State<'_, AppState>,
    scan_id: ScanId,
    nodes: Vec<NodeIndex>,
) -> Result<TreeRecycleResult, FileActionCommandError> {
    let shared_tree = state.scans.finished_tree(scan_id)?;
    let node_by_path = {
        let tree = shared_tree.read().unwrap_or_else(PoisonError::into_inner);
        resolve_recyclable_paths(&tree, &nodes)?
    };
    let paths: Vec<PathBuf> = node_by_path.keys().cloned().collect();

    let recycle_bin_operations = Arc::clone(&state.recycle_bin_operations);
    let recycle_result = tauri::async_runtime::spawn_blocking(move || {
        let _exclusive_recycle = recycle_bin_operations
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        file_actions::move_to_recycle_bin(&paths)
    })
    .await?
    .map_err(|error| FileActionCommandError::RecycleFailed {
        details: describe_error_with_sources(&error),
    })?;

    let moved_nodes: Vec<NodeIndex> = recycle_result
        .moved
        .iter()
        .filter_map(|path| node_by_path.get(path).copied())
        .collect();

    let mut tree = shared_tree.write().unwrap_or_else(PoisonError::into_inner);
    let removal = match tree.remove_nodes_and_reaggregate(&moved_nodes) {
        Ok(removal) => removal,
        Err(error) => {
            // Cannot happen for nodes checked above; the files are in the Recycle Bin either way,
            // so report what the shell did instead of failing.
            tracing::error!(%error, "could not take recycled items out of the tree");
            RemovalSummary::default()
        }
    };

    Ok(TreeRecycleResult {
        recycle_result,
        removal,
        totals: tree.totals(),
    })
}

/// Full paths of the requested items, refusing the scan root and items without a path.
fn resolve_recyclable_paths(
    tree: &FileTree,
    nodes: &[NodeIndex],
) -> Result<HashMap<PathBuf, NodeIndex>, FileActionCommandError> {
    let mut node_by_path = HashMap::with_capacity(nodes.len());
    for &node in nodes {
        if node == tree.root() {
            return Err(FileActionCommandError::CannotRecycleScanRoot);
        }
        node_by_path.insert(live_node_path(tree, node)?, node);
    }
    Ok(node_by_path)
}

/// A PNG thumbnail of `path` about `size` pixels square, from the cache or rendered by the
/// shell (a photo, a video frame, a document page, or the file type icon).
#[tauri::command]
#[specta::specta]
pub async fn get_thumbnail(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
    size: u32,
) -> Result<ThumbnailHandle, FileActionCommandError> {
    let thumbnail_cache = state.thumbnail_cache.clone();
    let asset_scope = app.asset_protocol_scope();

    tauri::async_runtime::spawn_blocking(move || {
        let thumbnail = thumbnail_cache
            .cached_thumbnail_for(&path, size)
            .map_err(|error| FileActionCommandError::ThumbnailFailed {
                path: path.display().to_string(),
                details: describe_error_with_sources(&error),
            })?;
        if let Err(error) = allow_thumbnail_folder(&asset_scope, &thumbnail.path) {
            tracing::warn!(
                error = %describe_error_with_sources(&error),
                "the thumbnail may not load through the asset protocol"
            );
        }
        Ok(ThumbnailHandle {
            png_path: thumbnail.path,
            is_file_type_icon: thumbnail.is_file_type_icon,
        })
    })
    .await?
}
