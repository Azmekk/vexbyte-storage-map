//! Which operating system the app runs on, so the UI can use the matching wording.

use serde::Serialize;
use specta::Type;

use crate::platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
#[allow(
    dead_code,
    reason = "each build only constructs the variant of the system it is built for"
)]
pub enum OperatingSystem {
    Windows,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: OperatingSystem,
    pub paths_are_case_sensitive: bool,
    pub app_version: &'static str,
}

pub fn current_platform_info() -> PlatformInfo {
    PlatformInfo {
        os: platform::current_operating_system(),
        paths_are_case_sensitive: platform::paths_are_case_sensitive(),
        app_version: env!("CARGO_PKG_VERSION"),
    }
}
