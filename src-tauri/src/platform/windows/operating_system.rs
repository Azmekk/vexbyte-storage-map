use crate::platform_info::OperatingSystem;

pub(crate) fn current_operating_system() -> OperatingSystem {
    OperatingSystem::Windows
}

/// NTFS can hold case-sensitive folders, but Windows treats paths case-insensitively by default.
pub(crate) fn paths_are_case_sensitive() -> bool {
    false
}
