//! Shell file actions shared by the disk viewer and the duplicate finder: moving files to the
//! Recycle Bin, revealing them in Explorer, opening them and producing shell thumbnails.
//!
//! Everything in this crate that changes the file system (the Recycle Bin) or opens windows and
//! applications is exercised only by hand, never by automated tests.
//!
//! Operating system calls live behind the `platform` module; only a Windows backend exists.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod file_stamp;
mod open_with_default_app;
mod platform;
mod png_encoding;
mod recycle_bin;
mod reveal_in_explorer;
mod shell_thumbnails;
mod thumbnail_cache;

pub use open_with_default_app::{OpenWithDefaultAppError, open_with_default_app};
pub use recycle_bin::{
    RecycleBinError, RecycleFailure, RecycleFailureReason, RecycleResult, move_to_recycle_bin,
};
pub use reveal_in_explorer::{RevealInExplorerError, reveal_in_explorer};
pub use shell_thumbnails::{
    MAXIMUM_THUMBNAIL_SIZE_PIXELS, MINIMUM_THUMBNAIL_SIZE_PIXELS, ThumbnailError, ThumbnailPixels,
    ThumbnailPng, render_thumbnail_png,
};
pub use thumbnail_cache::{CachedThumbnail, ThumbnailCache};
