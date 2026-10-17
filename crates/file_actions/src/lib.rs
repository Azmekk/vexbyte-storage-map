//! Shell file actions shared by the disk viewer and the duplicate finder: moving files to the
//! Recycle Bin, revealing them in Explorer and producing shell thumbnails.
//!
//! Everything in this crate that changes the file system is exercised only by hand against
//! temporary fixtures, never by automated tests.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[cfg(not(windows))]
compile_error!("file_actions only builds for Windows.");
