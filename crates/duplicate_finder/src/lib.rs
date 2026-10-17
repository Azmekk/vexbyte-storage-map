//! Finds exact duplicates among personal files (photos, videos, music, documents and archives)
//! in folders the user picks. Never touches system folders, never opens cloud placeholders and
//! refuses folders on cloud virtual drives such as Google Drive for desktop.
//!
//! Pipeline: collect candidates → group by size → quick check (xxh3 over the first and last
//! 64 KiB) → full check (BLAKE3). Files on SSDs are hashed in parallel, files on hard disks one
//! at a time in path order.
//!
//! Operating system calls live behind the `platform` module; only a Windows backend exists.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod cancellation;
mod candidate_collector;
mod cloud_drive_guard;
mod confirmed_group_counter;
mod directory_entry;
mod drive_kind;
mod duplicate_group;
mod duplicate_search;
mod file_read_error;
mod full_hashing;
mod keep_rules;
mod known_folders;
mod partial_hashing;
mod platform;
mod read_scheduling;
mod removal_plan;
mod revalidation;
mod search_progress;
mod search_settings;
mod size_grouping;
mod system_folder_guard;

pub use file_categories;
pub use file_categories::{FileCategory, category_for_extension, extensions_for};

pub use cancellation::{CancellationToken, SearchCancelled};
pub use drive_kind::{DriveKind, DriveKindCache};
pub use duplicate_group::{DuplicateFile, DuplicateGroup, DuplicateGroupId};
pub use duplicate_search::{
    DuplicateSearchError, DuplicateSearchResult, FolderDriveKind, find_duplicates,
    find_duplicates_with_validated_settings,
};
pub use keep_rules::{KeepRule, apply_keep_rule, apply_keep_rule_to_all_groups};
pub use known_folders::{SuggestedFolder, suggested_duplicate_folders};
pub use removal_plan::{PlannedKeep, PlannedRemoval, RemovalPlan, RemovalPlanError};
pub use revalidation::{
    ChangedFile, FileChangeReason, RevalidationOutcome, revalidate_before_removal,
};
pub use search_progress::{DuplicateSearchProgress, DuplicateSearchStage};
pub use search_settings::{
    DEFAULT_MINIMUM_FILE_SIZE_BYTES, DuplicateSearchSettings, SearchSettingsError,
    ValidatedSearchSettings, validate_search_folder,
};
pub use system_folder_guard::SystemFolderGuard;
