//! The folders of the operating system itself on the computer running the tests.

use std::path::PathBuf;

#[cfg(windows)]
pub fn operating_system_folders() -> Vec<PathBuf> {
    ["SystemRoot", "ProgramFiles", "ProgramData"]
        .iter()
        .map(|variable| PathBuf::from(std::env::var_os(variable).unwrap()))
        .collect()
}

#[cfg(target_os = "linux")]
pub fn operating_system_folders() -> Vec<PathBuf> {
    ["/usr", "/etc", "/var"].iter().map(PathBuf::from).collect()
}
