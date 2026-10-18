use crate::platform_info::{self, PlatformInfo};

/// The operating system the app runs on, how it treats paths and the app version. The frontend
/// reads it once before the first render.
#[tauri::command]
#[specta::specta]
pub async fn get_platform_info() -> PlatformInfo {
    platform_info::current_platform_info()
}
