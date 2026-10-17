//! The one seam between the app shell and the operating system. Every backend exposes the same
//! free functions with the same signatures, taking and returning only crate-owned types.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::*;

#[cfg(not(windows))]
compile_error!("Vexbyte Storage Map only builds for Windows so far.");
