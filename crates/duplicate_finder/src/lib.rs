//! Finds exact duplicates among personal files (photos, videos, music, documents and archives)
//! in folders the user picks. Never touches system folders and never opens cloud placeholders.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[cfg(not(windows))]
compile_error!("duplicate_finder only builds for Windows.");

pub mod file_categories;

pub use file_categories::{FileCategory, category_for_extension, extensions_for};
