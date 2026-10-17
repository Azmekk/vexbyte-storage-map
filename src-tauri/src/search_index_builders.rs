//! The threads that build the search indexes: one short-lived thread lists the volumes at app
//! start (and on "refresh all"), and up to `MAXIMUM_CONCURRENT_INDEX_BUILDS` builder threads
//! read the queued volumes one after another. Every change is pushed to the frontend with the
//! `SearchIndexStatusChanged` event.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use disk_scanner::{CancellationToken, ScanProgressUpdate, VolumeSummary};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::app_state::AppState;
use crate::error_details::{describe_error_with_sources, describe_panic};
use crate::events::SearchIndexStatusChanged;

/// Lists the volumes and builds the index of every local one, in the background. Called when the
/// app starts and for "refresh all", which also picks up drives connected since then. Volumes
/// that already have an index keep answering queries with it until the new one is ready.
pub fn start_building_all_search_indexes(app: &AppHandle) {
    let thread_app = app.clone();
    let spawn_result = std::thread::Builder::new()
        .name("vsm-search-index-volumes".to_owned())
        .spawn(move || list_volumes_and_queue_builds(&thread_app));

    if let Err(error) = spawn_result {
        tracing::error!(%error, "could not start listing volumes for the search index");
        app.state::<AppState>()
            .search_indexes
            .record_volume_list_failure();
        emit_search_index_status(app);
    }
}

/// Rebuilds the index of one volume. The old index keeps answering queries until the new one is
/// ready. The caller has checked that the volume is indexed.
pub fn start_rebuilding_search_index(app: &AppHandle, mount_point: PathBuf) {
    let builders_to_start = app
        .state::<AppState>()
        .search_indexes
        .queue_builds(&[mount_point]);
    emit_search_index_status(app);
    start_builder_threads(app, builders_to_start);
}

fn list_volumes_and_queue_builds(app: &AppHandle) {
    let registry = &app.state::<AppState>().search_indexes;

    let build_order = match disk_scanner::list_volumes() {
        Ok(volumes) => {
            let system_volume = disk_scanner::system_volume_mount_point();
            registry.record_volume_list(volumes, system_volume.as_deref())
        }
        Err(error) => {
            tracing::error!(
                error = %describe_error_with_sources(&error),
                "could not list the volumes for the search index"
            );
            registry.record_volume_list_failure();
            Vec::new()
        }
    };

    let builders_to_start = registry.queue_builds(&build_order);
    emit_search_index_status(app);
    start_builder_threads(app, builders_to_start);
}

fn start_builder_threads(app: &AppHandle, count: usize) {
    for builder_number in 0..count {
        let thread_app = app.clone();
        let spawn_result = std::thread::Builder::new()
            .name(format!("vsm-search-index-builder-{builder_number}"))
            .spawn(move || build_queued_volumes(&thread_app));

        if let Err(error) = spawn_result {
            tracing::error!(%error, "could not start a search index builder");
            app.state::<AppState>()
                .search_indexes
                .record_builder_start_failure(&format!(
                    "the index builder could not be started: {error}"
                ));
            emit_search_index_status(app);
        }
    }
}

fn build_queued_volumes(app: &AppHandle) {
    let registry = &app.state::<AppState>().search_indexes;

    while let Some(volume) = registry.take_next_build() {
        emit_search_index_status(app);
        build_one_volume(app, &volume);
        emit_search_index_status(app);
    }
}

fn build_one_volume(app: &AppHandle, volume: &VolumeSummary) {
    let registry = &app.state::<AppState>().search_indexes;
    let mount_point = volume.mount_point.as_path();
    tracing::info!(mount_point = %mount_point.display(), "building the search index");

    let report_progress = |update: ScanProgressUpdate| {
        let is_event_due = registry.record_build_progress(
            mount_point,
            update.files_found,
            update.directories_found,
        );
        if is_event_due {
            emit_search_index_status(app);
        }
    };
    // Builds run for the lifetime of the app; nothing cancels them yet.
    let cancellation = CancellationToken::new();

    let build_result = catch_unwind(AssertUnwindSafe(|| {
        disk_scanner::build_volume_search_index(volume, &report_progress, &cancellation)
    }));

    match build_result {
        Ok(Ok(index)) => {
            tracing::info!(
                mount_point = %mount_point.display(),
                files = index.file_count(),
                directories = index.directory_count(),
                method = ?index.method(),
                estimated_memory_bytes = index.estimated_memory_bytes(),
                "the search index is ready"
            );
            registry.record_build_success(mount_point, index);
        }
        Ok(Err(error)) => {
            let reason = describe_error_with_sources(&error);
            tracing::warn!(mount_point = %mount_point.display(), %reason, "the search index could not be built");
            registry.record_build_failure(mount_point, reason);
        }
        Err(panic_payload) => {
            let reason = describe_panic(panic_payload.as_ref());
            tracing::error!(mount_point = %mount_point.display(), %reason, "the search index builder panicked");
            registry.record_build_failure(mount_point, reason);
        }
    }
}

pub fn emit_search_index_status(app: &AppHandle) {
    let status = app.state::<AppState>().search_indexes.status();
    if let Err(error) = (SearchIndexStatusChanged { status }).emit(app) {
        tracing::warn!(%error, "could not emit the search index status");
    }
}
