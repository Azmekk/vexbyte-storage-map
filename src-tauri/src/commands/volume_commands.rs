use disk_scanner::{VolumeListError, VolumeSummary};
use serde::Serialize;
use specta::Type;

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum VolumeCommandError {
    #[error("Windows could not report the drives on this computer: {details}")]
    VolumeListFailed { details: String },
    #[error("The drive listing stopped unexpectedly: {details}")]
    BackgroundTaskFailed { details: String },
}

impl From<VolumeListError> for VolumeCommandError {
    fn from(error: VolumeListError) -> Self {
        let details = match &error {
            VolumeListError::LogicalDrivesQueryFailed { source } => source.message(),
        };

        Self::VolumeListFailed { details }
    }
}

impl From<tauri::Error> for VolumeCommandError {
    fn from(error: tauri::Error) -> Self {
        Self::BackgroundTaskFailed {
            details: error.to_string(),
        }
    }
}

/// Lists the mounted volumes that are ready to be read.
#[tauri::command]
#[specta::specta]
pub async fn list_volumes() -> Result<Vec<VolumeSummary>, VolumeCommandError> {
    let background_task = tauri::async_runtime::spawn_blocking(disk_scanner::list_volumes);

    let list_result = background_task.await?;
    let volumes = list_result?;

    Ok(volumes)
}
