use std::panic::{AssertUnwindSafe, catch_unwind};

use disk_scanner::{ScanMethod, ScanProgressUpdate, ScanTarget};
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::app_state::AppState;
use crate::elevation::is_process_elevated;
use crate::error_details::describe_panic;
use crate::events::ScanFinished;
use crate::scan_registry::{ScanId, ScanLookupError, ScanOutcome, ScanOverview};

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ScanCommandError {
    #[error("there is no scan {scan_id}")]
    ScanNotFound { scan_id: u32 },
    #[error("the fast scan needs administrator rights")]
    FastScanRequiresElevation,
    #[error("the scan could not be started: {details}")]
    ScanStartFailed { details: String },
}

impl From<ScanLookupError> for ScanCommandError {
    fn from(error: ScanLookupError) -> Self {
        match error {
            ScanLookupError::ScanNotFound { scan_id }
            | ScanLookupError::ScanNotFinished { scan_id, .. } => Self::ScanNotFound {
                scan_id: scan_id.get(),
            },
        }
    }
}

/// Starts scanning `target` on a background thread and returns its id right away.
///
/// Progress arrives on `progress` about ten times per second. When the scan ends, its result is
/// stored and the `ScanFinished` event is emitted; from then on the tree queries work.
#[tauri::command]
#[specta::specta]
pub async fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    target: ScanTarget,
    method: ScanMethod,
    progress: Channel<ScanProgressUpdate>,
) -> Result<ScanId, ScanCommandError> {
    if method == ScanMethod::MasterFileTable && !is_process_elevated() {
        return Err(ScanCommandError::FastScanRequiresElevation);
    }

    let (scan_id, cancellation) = state.scans.register_running_scan(target.clone(), method);

    let thread_app = app.clone();
    let spawn_result = std::thread::Builder::new()
        .name(format!("vsm-scan-{}", scan_id.get()))
        .spawn(move || {
            let report_progress = |update: ScanProgressUpdate| {
                // The frontend may have dropped the channel (for example on reload); the
                // completion event still reaches it.
                let _ = progress.send(update);
            };
            let scan_result = catch_unwind(AssertUnwindSafe(|| {
                disk_scanner::scan(target, method, &report_progress, &cancellation)
            }));

            let registry = &thread_app.state::<AppState>().scans;
            let outcome = match scan_result {
                Ok(result) => registry.record_scan_end(scan_id, result),
                Err(panic_payload) => registry
                    .record_scan_thread_failure(scan_id, describe_panic(panic_payload.as_ref())),
            };
            emit_scan_finished(&thread_app, scan_id, outcome);
        });

    if let Err(error) = spawn_result {
        state
            .scans
            .record_scan_thread_failure(scan_id, error.to_string());
        return Err(ScanCommandError::ScanStartFailed {
            details: error.to_string(),
        });
    }

    Ok(scan_id)
}

fn emit_scan_finished(app: &AppHandle, scan_id: ScanId, outcome: ScanOutcome) {
    if let Err(error) = (ScanFinished { scan_id, outcome }).emit(app) {
        tracing::warn!(%error, scan_id = scan_id.get(), "could not emit the scan finished event");
    }
}

/// Asks a running scan to stop. It ends with the `cancelled` outcome shortly after. Cancelling
/// a scan that already ended does nothing.
#[tauri::command]
#[specta::specta]
pub async fn cancel_scan(
    state: State<'_, AppState>,
    scan_id: ScanId,
) -> Result<(), ScanCommandError> {
    state.scans.cancel_scan(scan_id)?;
    Ok(())
}

/// Status of a scan plus, once it finished, its summary and up-to-date totals.
#[tauri::command]
#[specta::specta]
pub async fn get_scan_summary(
    state: State<'_, AppState>,
    scan_id: ScanId,
) -> Result<ScanOverview, ScanCommandError> {
    Ok(state.scans.scan_overview(scan_id)?)
}

/// Every scan of this session, oldest first, for the "recent scans" list.
#[tauri::command]
#[specta::specta]
pub async fn list_scans(state: State<'_, AppState>) -> Result<Vec<ScanOverview>, ScanCommandError> {
    Ok(state.scans.all_scan_overviews())
}

/// Forgets a scan and frees the memory its tree uses. A running scan is cancelled first.
#[tauri::command]
#[specta::specta]
pub async fn discard_scan(
    state: State<'_, AppState>,
    scan_id: ScanId,
) -> Result<(), ScanCommandError> {
    state.scans.discard_scan(scan_id)?;
    Ok(())
}
