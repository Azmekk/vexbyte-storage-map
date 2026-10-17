//! Scans drives and folders and builds a compact in-memory file tree.
//!
//! This crate knows nothing about Tauri or the user interface. Scanning is strictly read-only.
//! All operating system calls live in `platform`; Windows is the only backend so far.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod cancellation;
pub mod fast_scan_availability;
pub mod file_identity;
pub mod file_tree;
pub mod folder_walk;
mod mft;
pub mod scan;
pub mod scan_method;
pub mod scan_progress;
pub mod volumes;

mod platform;

pub use cancellation::{CancellationToken, ScanCancelled};
pub use fast_scan_availability::{
    FastScanAvailability, fast_scan_availability, is_process_elevated,
};
pub use file_categories::FileCategory;
pub use file_identity::FileIdentity;
pub use file_tree::{
    BreadcrumbItem, ChildSortOrder, ChildSummary, ChildrenPage, FileCategoryShare,
    FileExtensionShare, FileSummary, FileTree, FileTypeBreakdown, Node, NodeDetails,
    NodeFlagSummary, NodeFlags, NodeIndex, NodeKind, RemovalSummary, SearchResult, TreeQueryError,
    TreeRemovalError, TreeTotals, VISUALIZATION_NODE_LIMIT, VisualizationNode,
    VisualizationNodeKind,
};
pub use folder_walk::FolderWalkError;
pub use mft::{
    DEFAULT_MFT_CHUNK_SIZE_BYTES, DataRunError, MftChunkReadError, MftExtentError, MftScanError,
};
pub use scan::{
    ScanError, ScanOptions, ScanResult, ScanSummary, ScanTarget, scan, scan_with_options,
};
pub use scan_method::ScanMethod;
pub use scan_progress::{
    NoProgressReporting, ScanPhase, ScanProgressReporter, ScanProgressUpdate,
    TopLevelDirectoryProgress,
};
pub use volumes::{DriveType, VolumeListError, VolumeSummary, list_volumes};
