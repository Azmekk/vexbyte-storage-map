//! Scans Windows drives and folders and builds a compact in-memory file tree.
//!
//! This crate knows nothing about Tauri or the user interface. Scanning is strictly read-only.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[cfg(not(windows))]
compile_error!("disk_scanner only builds for Windows.");

pub mod scan_method;
pub mod volumes;

mod windows_api;

pub use scan_method::ScanMethod;
pub use volumes::{DriveLetter, DriveType, VolumeListError, VolumeSummary, list_volumes};
