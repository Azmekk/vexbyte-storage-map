use std::path::{Path, PathBuf};
use std::time::Instant;

use disk_scanner::{ParsedQuery, SearchHit, SearchQueryError, SearchSortOrder};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::error_details::describe_error_with_sources;
use crate::search_index_builders::{
    start_building_all_search_indexes, start_rebuilding_search_index,
};
use crate::search_index_registry::{RefreshRequestError, SearchIndexStatus, SkipReason};

/// Search results per request are clamped to `1..=MAXIMUM_SEARCH_RESULT_COUNT`.
const MAXIMUM_SEARCH_RESULT_COUNT: u32 = 10_000;

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SearchCommandError {
    #[error("the search could not be understood: {problem}")]
    InvalidQuery { problem: SearchQueryError },
    #[error("{mount_point} is not a known volume")]
    VolumeNotFound { mount_point: String },
    #[error("{mount_point} is not indexed ({reason:?})")]
    VolumeNotIndexed {
        mount_point: String,
        reason: SkipReason,
    },
    #[error("{path} is not in the search index")]
    PathNotInIndex { path: String },
    #[error("could not show {path} in Explorer: {details}")]
    RevealFailed { path: String, details: String },
    #[error("could not open {path}: {details}")]
    OpenFailed { path: String, details: String },
    #[error("the search stopped unexpectedly: {details}")]
    BackgroundTaskFailed { details: String },
}

impl From<tauri::Error> for SearchCommandError {
    fn from(error: tauri::Error) -> Self {
        Self::BackgroundTaskFailed {
            details: error.to_string(),
        }
    }
}

impl From<RefreshRequestError> for SearchCommandError {
    fn from(error: RefreshRequestError) -> Self {
        match error {
            RefreshRequestError::VolumeNotFound { mount_point } => Self::VolumeNotFound {
                mount_point: mount_point.display().to_string(),
            },
            RefreshRequestError::VolumeNotIndexed {
                mount_point,
                reason,
            } => Self::VolumeNotIndexed {
                mount_point: mount_point.display().to_string(),
                reason,
            },
        }
    }
}

/// The answer to `search_files`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// The best matches in the requested order, at most `limit`.
    pub hits: Vec<SearchHit>,
    /// Every match on the searched volumes, also those beyond `limit`.
    pub total_match_count: u64,
    /// True when more items matched than `hits` holds.
    pub total_matches_capped: bool,
    /// The volumes whose index answered, ordered by mount point.
    pub searched_volumes: Vec<PathBuf>,
    /// Local volumes not searched because their first index is still being built.
    pub skipped_volumes_still_warming: Vec<PathBuf>,
    pub elapsed_milliseconds: u64,
}

/// Every volume's search index state, for the "Warming up search engine" screen. Afterwards
/// listen for `SearchIndexStatusChanged`.
#[tauri::command]
#[specta::specta]
pub async fn get_search_index_status(
    state: State<'_, AppState>,
) -> Result<SearchIndexStatus, SearchCommandError> {
    Ok(state.search_indexes.status())
}

/// Rebuilds the index of one volume, or of every local volume when `mount_point` is `None`
/// (which also picks up drives connected since the app started). Returns right away; the old
/// index keeps answering searches until the new one replaces it.
#[tauri::command]
#[specta::specta]
pub async fn refresh_search_index(
    app: AppHandle,
    state: State<'_, AppState>,
    mount_point: Option<PathBuf>,
) -> Result<(), SearchCommandError> {
    match mount_point {
        Some(mount_point) => {
            state.search_indexes.check_refresh_target(&mount_point)?;
            start_rebuilding_search_index(&app, mount_point);
        }
        None => start_building_all_search_indexes(&app),
    }
    Ok(())
}

/// Searches every indexed volume by name or path (see `docs/SEARCH.md` for the syntax).
/// `limit` is clamped to 1..=10,000. An empty query answers with no results right away.
#[tauri::command]
#[specta::specta]
pub async fn search_files(
    state: State<'_, AppState>,
    query: String,
    sort: SearchSortOrder,
    limit: u32,
) -> Result<SearchResponse, SearchCommandError> {
    let started_at = Instant::now();
    let limit = limit.clamp(1, MAXIMUM_SEARCH_RESULT_COUNT) as usize;
    let parsed_query = ParsedQuery::parse(&query)
        .map_err(|problem| SearchCommandError::InvalidQuery { problem })?;

    let searchable = state.search_indexes.searchable_indexes();
    let skipped_volumes_still_warming = searchable.volumes_still_warming;
    if parsed_query.is_empty() {
        return Ok(SearchResponse {
            hits: Vec::new(),
            total_match_count: 0,
            total_matches_capped: false,
            searched_volumes: Vec::new(),
            skipped_volumes_still_warming,
            elapsed_milliseconds: 0,
        });
    }

    let indexes = searchable.indexes;
    let searched_volumes = indexes
        .iter()
        .map(|index| index.mount_point().to_path_buf())
        .collect();
    let matches = tauri::async_runtime::spawn_blocking(move || {
        disk_scanner::search_volume_indexes(
            indexes.iter().map(AsRef::as_ref),
            &parsed_query,
            sort,
            limit,
        )
    })
    .await?;

    let hit_count = u64::try_from(matches.hits.len()).unwrap_or(u64::MAX);
    Ok(SearchResponse {
        total_matches_capped: matches.total_match_count > hit_count,
        total_match_count: matches.total_match_count,
        hits: matches.hits,
        searched_volumes,
        skipped_volumes_still_warming,
        elapsed_milliseconds: u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

/// Only paths that a finished index holds may be shown or opened through the search, so the
/// frontend cannot ask for arbitrary paths.
fn indexed_path(state: &AppState, path: PathBuf) -> Result<PathBuf, SearchCommandError> {
    let is_indexed = state
        .search_indexes
        .index_holding(&path)
        .and_then(|index| index.find_searchable_node_by_path(&path))
        .is_some();

    if is_indexed {
        Ok(path)
    } else {
        Err(SearchCommandError::PathNotInIndex {
            path: describe(&path),
        })
    }
}

fn describe(path: &Path) -> String {
    path.display().to_string()
}

/// Opens an Explorer window with a search result selected.
#[tauri::command]
#[specta::specta]
pub async fn reveal_search_hit(
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<(), SearchCommandError> {
    let path = indexed_path(&state, path)?;
    tauri::async_runtime::spawn_blocking(move || {
        file_actions::reveal_in_explorer(&path).map_err(|error| SearchCommandError::RevealFailed {
            path: describe(&path),
            details: describe_error_with_sources(&error),
        })
    })
    .await?
}

/// Opens a search result with its default application, or a folder in Explorer.
#[tauri::command]
#[specta::specta]
pub async fn open_search_hit(
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<(), SearchCommandError> {
    let path = indexed_path(&state, path)?;
    tauri::async_runtime::spawn_blocking(move || {
        file_actions::open_with_default_app(&path).map_err(|error| SearchCommandError::OpenFailed {
            path: describe(&path),
            details: describe_error_with_sources(&error),
        })
    })
    .await?
}
