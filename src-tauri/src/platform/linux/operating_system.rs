use crate::platform_info::OperatingSystem;

pub(crate) fn current_operating_system() -> OperatingSystem {
    OperatingSystem::Linux
}

pub(crate) fn paths_are_case_sensitive() -> bool {
    true
}
