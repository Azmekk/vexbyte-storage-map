//! The one seam between the app shell and the operating system. Every backend exposes the same
//! free functions with the same signatures, taking and returning only crate-owned types.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;

#[cfg(not(any(windows, target_os = "linux")))]
compile_error!("Vexbyte Storage Map only builds for Windows and Linux so far.");
