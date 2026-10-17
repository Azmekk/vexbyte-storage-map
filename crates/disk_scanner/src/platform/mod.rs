//! The only seam between this crate and the operating system.
//!
//! Each backend exposes the same free functions with the same signatures, taking and returning
//! only crate-owned, platform-neutral types. There is no trait and no runtime selection: the
//! compiler enforces the contract when a backend is missing a function.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use self::windows::*;
