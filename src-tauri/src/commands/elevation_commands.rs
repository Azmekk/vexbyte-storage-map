use disk_scanner::ScanTarget;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::elevation::{self, RelaunchError};
use crate::error_details::describe_error_with_sources;
use crate::startup_arguments::StartupScanRequest;

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ElevationCommandError {
    #[error("the app already runs with administrator rights")]
    AlreadyElevated,
    #[error("the request for administrator rights was declined")]
    ElevationDeclined,
    #[error("could not restart the app with administrator rights: {details}")]
    RelaunchFailed { details: String },
    #[error("the operation stopped unexpectedly: {details}")]
    BackgroundTaskFailed { details: String },
}

impl From<RelaunchError> for ElevationCommandError {
    fn from(error: RelaunchError) -> Self {
        match error {
            RelaunchError::DeclinedByUser => Self::ElevationDeclined,
            RelaunchError::LaunchFailed { .. } => Self::RelaunchFailed {
                details: describe_error_with_sources(&error),
            },
        }
    }
}

impl From<tauri::Error> for ElevationCommandError {
    fn from(error: tauri::Error) -> Self {
        Self::BackgroundTaskFailed {
            details: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ElevationStatus {
    pub is_elevated: bool,
}

/// Whether this instance runs with administrator rights, which the fast scan needs.
#[tauri::command]
#[specta::specta]
pub async fn get_elevation_status() -> Result<ElevationStatus, ElevationCommandError> {
    Ok(ElevationStatus {
        is_elevated: elevation::is_process_elevated(),
    })
}

/// Restarts the app with administrator rights and a fast scan of `target`. Windows shows the
/// UAC prompt; when the user accepts, the new instance starts and this one exits. When the user
/// declines, this returns `elevationDeclined` and the app keeps running.
#[tauri::command]
#[specta::specta]
pub async fn relaunch_elevated(
    app: AppHandle,
    target: ScanTarget,
) -> Result<(), ElevationCommandError> {
    if elevation::is_process_elevated() {
        return Err(ElevationCommandError::AlreadyElevated);
    }

    tauri::async_runtime::spawn_blocking(move || elevation::relaunch_elevated_for_scan(&target))
        .await??;

    app.exit(0);
    Ok(())
}

/// Restarts the app with administrator rights on the home screen, without starting a scan.
/// Windows shows the UAC prompt; when the user accepts, the new instance starts and this one
/// exits. When the user declines, this returns `elevationDeclined` and the app keeps running.
#[tauri::command]
#[specta::specta]
pub async fn relaunch_as_administrator(app: AppHandle) -> Result<(), ElevationCommandError> {
    if elevation::is_process_elevated() {
        return Err(ElevationCommandError::AlreadyElevated);
    }

    tauri::async_runtime::spawn_blocking(elevation::relaunch_elevated).await??;

    app.exit(0);
    Ok(())
}

/// The scan this instance was started for (by `relaunch_elevated`), if any. The frontend starts
/// it once on load. It stays the same for the lifetime of the instance.
#[tauri::command]
#[specta::specta]
pub async fn get_startup_scan_request(
    state: State<'_, AppState>,
) -> Result<Option<StartupScanRequest>, ElevationCommandError> {
    Ok(state.startup_scan_request.clone())
}
