//! The one seam between the duplicate finder and the operating system. Every backend exposes the
//! same free functions with the same signatures, taking and returning only crate-owned types.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
