//! Platform-neutral description of one directory entry, produced by the `platform` listing.

use std::ffi::OsString;
use std::time::SystemTime;

/// Identifies a file on its volume, so several paths (hard links) to the same file can be
/// recognized. Opaque on purpose: only equality and hashing are meaningful.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileIdentity {
    volume_serial_number: u64,
    file_id: u128,
}

impl FileIdentity {
    pub(crate) fn new(volume_serial_number: u64, file_id: u128) -> Self {
        Self {
            volume_serial_number,
            file_id,
        }
    }
}

impl std::fmt::Debug for FileIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("FileIdentity(..)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct EntryFlags: u16 {
        const SYMLINK = 1 << 0;
        const JUNCTION = 1 << 1;
        /// Content lives in the cloud; reading it would download it.
        const CLOUD_PLACEHOLDER = 1 << 2;
        const HIDDEN = 1 << 3;
        const SYSTEM = 1 << 4;
        const INACCESSIBLE = 1 << 5;
        const COMPRESSED = 1 << 6;
        const SPARSE = 1 << 7;
    }
}

impl EntryFlags {
    /// Links that point somewhere else and must never be followed or hashed.
    #[must_use]
    pub fn is_link(self) -> bool {
        self.intersects(Self::SYMLINK | Self::JUNCTION)
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: OsString,
    pub kind: EntryKind,
    pub logical_size: u64,
    pub modified_time: SystemTime,
    pub created_time: SystemTime,
    /// `None` when the file system does not report stable file ids (for example some network
    /// shares); such files are never collapsed as hard links.
    pub identity: Option<FileIdentity>,
    pub flags: EntryFlags,
}

/// Size, modification time and flags of a single path, read without opening the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStamp {
    pub size: u64,
    pub modified_time: SystemTime,
    pub kind: EntryKind,
    pub flags: EntryFlags,
}
