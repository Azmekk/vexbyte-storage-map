use serde::Serialize;
use specta::Type;

use crate::error_details::describe_error_with_sources;
use crate::update_check::{self, AvailableUpdate, UpdateCheckError};

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum UpdateCommandError {
    #[error("could not check for a newer version: {details}")]
    CheckFailed { details: String },
    #[error("the update check stopped unexpectedly: {details}")]
    BackgroundTaskFailed { details: String },
}

impl From<UpdateCheckError> for UpdateCommandError {
    fn from(error: UpdateCheckError) -> Self {
        Self::CheckFailed {
            details: describe_error_with_sources(&error),
        }
    }
}

impl From<tauri::Error> for UpdateCommandError {
    fn from(error: tauri::Error) -> Self {
        Self::BackgroundTaskFailed {
            details: error.to_string(),
        }
    }
}

/// The newest release on GitHub when it is newer than this build, otherwise `null`.
#[tauri::command]
#[specta::specta]
pub async fn check_for_update() -> Result<Option<AvailableUpdate>, UpdateCommandError> {
    let background_task =
        tauri::async_runtime::spawn_blocking(update_check::check_for_newer_release);

    let check_result = background_task.await?;
    let available_update = check_result?;

    Ok(available_update)
}
