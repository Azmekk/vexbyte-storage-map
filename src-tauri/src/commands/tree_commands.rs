use std::sync::PoisonError;

use disk_scanner::{
    BreadcrumbItem, ChildSortOrder, ChildrenPage, FileCategory, FileSummary, FileTree,
    FileTypeBreakdown, NodeDetails, NodeIndex, SearchResult, TreeQueryError, VisualizationNode,
};
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::app_state::AppState;
use crate::scan_registry::{ScanId, ScanLookupError, ScanStatus, SharedFileTree};

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TreeCommandError {
    #[error("there is no scan {scan_id}")]
    ScanNotFound { scan_id: u32 },
    #[error("scan {scan_id} has no file tree ({status:?})")]
    ScanNotFinished { scan_id: u32, status: ScanStatus },
    #[error("item {node} does not exist in this scan")]
    NodeNotFound { node: u32 },
    #[error("item {node} was moved to the Recycle Bin")]
    NodeRemoved { node: u32 },
    #[error("the query stopped unexpectedly: {details}")]
    BackgroundTaskFailed { details: String },
}

impl From<ScanLookupError> for TreeCommandError {
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

impl From<TreeQueryError> for TreeCommandError {
    fn from(error: TreeQueryError) -> Self {
        match error {
            TreeQueryError::NodeNotFound { node } => Self::NodeNotFound { node: node.get() },
            TreeQueryError::NodeRemoved { node } => Self::NodeRemoved { node: node.get() },
        }
    }
}

impl From<tauri::Error> for TreeCommandError {
    fn from(error: tauri::Error) -> Self {
        Self::BackgroundTaskFailed {
            details: error.to_string(),
        }
    }
}

/// Runs a query under the tree's read lock on the calling thread. For queries that take a few
/// milliseconds at most.
fn query_tree<Answer>(
    state: &AppState,
    scan_id: ScanId,
    query: impl FnOnce(&FileTree) -> Result<Answer, TreeQueryError>,
) -> Result<Answer, TreeCommandError> {
    let tree = state.scans.finished_tree(scan_id)?;
    let tree = tree.read().unwrap_or_else(PoisonError::into_inner);
    Ok(query(&tree)?)
}

/// Runs a query that walks a whole subtree on a blocking thread, so it never holds up other
/// commands.
async fn query_tree_in_background<Answer: Send + 'static>(
    state: &AppState,
    scan_id: ScanId,
    query: impl FnOnce(&FileTree) -> Result<Answer, TreeQueryError> + Send + 'static,
) -> Result<Answer, TreeCommandError> {
    let tree: SharedFileTree = state.scans.finished_tree(scan_id)?;

    let answer = tauri::async_runtime::spawn_blocking(move || {
        let tree = tree.read().unwrap_or_else(PoisonError::into_inner);
        query(&tree)
    })
    .await??;
    Ok(answer)
}

/// One page of a folder's children in the requested order.
#[tauri::command]
#[specta::specta]
pub async fn get_children(
    state: State<'_, AppState>,
    scan_id: ScanId,
    node: NodeIndex,
    sort: ChildSortOrder,
    offset: u32,
    limit: u32,
) -> Result<ChildrenPage, TreeCommandError> {
    query_tree(&state, scan_id, |tree| {
        tree.children_page(node, sort, offset, limit)
    })
}

/// Full path, sizes, counts, dates and flags of one item.
#[tauri::command]
#[specta::specta]
pub async fn get_node_details(
    state: State<'_, AppState>,
    scan_id: ScanId,
    node: NodeIndex,
) -> Result<NodeDetails, TreeCommandError> {
    query_tree(&state, scan_id, |tree| tree.node_details(node))
}

/// The breadcrumb trail from the scanned root down to `node`.
#[tauri::command]
#[specta::specta]
pub async fn get_path_to_root(
    state: State<'_, AppState>,
    scan_id: ScanId,
    node: NodeIndex,
) -> Result<Vec<BreadcrumbItem>, TreeCommandError> {
    query_tree(&state, scan_id, |tree| tree.path_to_root(node))
}

/// A pruned nested view of `node` for the treemap and sunburst, capped at about 5,000 nodes.
#[tauri::command]
#[specta::specta]
pub async fn get_visualization_subtree(
    state: State<'_, AppState>,
    scan_id: ScanId,
    node: NodeIndex,
    max_depth: u32,
    minimum_share_of_parent: f64,
) -> Result<VisualizationNode, TreeCommandError> {
    query_tree(&state, scan_id, |tree| {
        tree.visualization_subtree(node, max_depth, minimum_share_of_parent)
    })
}

/// The largest files under `node`, optionally of one category.
#[tauri::command]
#[specta::specta]
pub async fn get_largest_files(
    state: State<'_, AppState>,
    scan_id: ScanId,
    node: NodeIndex,
    category_filter: Option<FileCategory>,
    limit: u32,
) -> Result<Vec<FileSummary>, TreeCommandError> {
    query_tree_in_background(&state, scan_id, move |tree| {
        tree.largest_files(node, category_filter, limit)
    })
    .await
}

/// Space per file category and per extension under `node`.
#[tauri::command]
#[specta::specta]
pub async fn get_file_type_breakdown(
    state: State<'_, AppState>,
    scan_id: ScanId,
    node: NodeIndex,
) -> Result<FileTypeBreakdown, TreeCommandError> {
    query_tree_in_background(&state, scan_id, move |tree| tree.file_type_breakdown(node)).await
}

/// Items whose name contains `query`, ignoring case, largest first.
#[tauri::command]
#[specta::specta]
pub async fn search_nodes(
    state: State<'_, AppState>,
    scan_id: ScanId,
    query: String,
    limit: u32,
) -> Result<Vec<SearchResult>, TreeCommandError> {
    query_tree_in_background(&state, scan_id, move |tree| {
        Ok(tree.search_nodes(&query, limit))
    })
    .await
}
