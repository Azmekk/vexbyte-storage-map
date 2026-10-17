use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use specta::Type;

use super::name_storage::{ExtensionId, NameId};

/// The position of a node in the tree's arena. Stable for the lifetime of a `FileTree`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type,
)]
#[serde(transparent)]
pub struct NodeIndex(u32);

impl NodeIndex {
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    pub(crate) const fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub(crate) fn from_usize(index: usize) -> Self {
        Self(u32::try_from(index).unwrap_or(u32::MAX))
    }
}

/// A `NodeIndex` that may be absent, stored in 4 bytes instead of the 8 an
/// `Option<NodeIndex>` takes, which keeps `Node` under 64 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OptionalNodeIndex(u32);

impl OptionalNodeIndex {
    pub(crate) const NONE: Self = Self(u32::MAX);

    pub(crate) const fn some(index: NodeIndex) -> Self {
        Self(index.0)
    }

    pub(crate) const fn get(self) -> Option<NodeIndex> {
        if self.0 == u32::MAX {
            None
        } else {
            Some(NodeIndex(self.0))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum NodeKind {
    File,
    Directory,
}

bitflags::bitflags! {
    /// Facts about a node that the user interface shows as badges.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct NodeFlags: u16 {
        /// Another path to the same file (a hard link) already counts its size. This node is
        /// shown with its size but adds nothing to the totals of its ancestors.
        const HARD_LINK_COUNTED_ELSEWHERE = 1 << 0;
        /// A symbolic link. Never followed.
        const SYMLINK = 1 << 1;
        /// A junction or volume mount point. Never followed.
        const JUNCTION = 1 << 2;
        /// The content lives in the cloud and is not stored on this disk. Never opened.
        const CLOUD_PLACEHOLDER = 1 << 3;
        const COMPRESSED = 1 << 4;
        const SPARSE = 1 << 5;
        const HIDDEN = 1 << 6;
        const SYSTEM = 1 << 7;
        /// The directory could not be listed, usually because access was denied.
        const INACCESSIBLE = 1 << 8;
        /// Created by the scanner to group entries (for example "NTFS metadata"); it has no path
        /// of its own and cannot be opened or recycled.
        const SYNTHETIC = 1 << 9;
        /// Removed from the tree after it was moved to the Recycle Bin.
        const REMOVED = 1 << 10;
    }
}

/// Milliseconds since the Unix epoch; 0 means unknown or before 1970.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct NodeTime(u64);

impl NodeTime {
    pub(crate) const UNKNOWN: Self = Self(0);

    pub(crate) fn from_system_time(time: SystemTime) -> Self {
        let milliseconds = time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |since_epoch| {
                u64::try_from(since_epoch.as_millis()).unwrap_or(u64::MAX)
            });
        Self(milliseconds)
    }

    pub(crate) fn unix_milliseconds(self) -> Option<u64> {
        (self.0 != 0).then_some(self.0)
    }

    fn to_system_time(self) -> Option<SystemTime> {
        let milliseconds = self.unix_milliseconds()?;
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_millis(milliseconds))
    }
}

/// One file or directory in the tree. Kept under 64 bytes; names live in `NameStorage`.
///
/// For directories the sizes and counts are aggregated over the whole subtree. For files they
/// are the file's own sizes.
#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) name: NameId,
    pub(crate) extension: ExtensionId,
    pub(crate) parent: OptionalNodeIndex,
    /// Children occupy the contiguous range `first_child..first_child + child_count`, because
    /// the arena is stored in breadth-first order.
    pub(crate) first_child: u32,
    pub(crate) child_count: u32,
    pub(crate) file_count: u32,
    pub(crate) directory_count: u32,
    pub(crate) logical_size: u64,
    pub(crate) allocated_size: u64,
    pub(crate) modified_time: NodeTime,
    pub(crate) flags: NodeFlags,
    pub(crate) kind: NodeKind,
}

impl Node {
    pub(crate) fn new_unlinked(name: NameId, extension: ExtensionId, kind: NodeKind) -> Self {
        Self {
            name,
            extension,
            parent: OptionalNodeIndex::NONE,
            first_child: 0,
            child_count: 0,
            file_count: 0,
            directory_count: 0,
            logical_size: 0,
            allocated_size: 0,
            modified_time: NodeTime::UNKNOWN,
            flags: NodeFlags::empty(),
            kind,
        }
    }

    #[must_use]
    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    #[must_use]
    pub fn is_directory(&self) -> bool {
        self.kind == NodeKind::Directory
    }

    #[must_use]
    pub fn flags(&self) -> NodeFlags {
        self.flags
    }

    #[must_use]
    pub fn parent(&self) -> Option<NodeIndex> {
        self.parent.get()
    }

    /// The file size (for directories: the sum over the subtree, with hard links counted once).
    #[must_use]
    pub fn logical_size(&self) -> u64 {
        self.logical_size
    }

    /// The space the file occupies on disk (for directories: the sum over the subtree).
    #[must_use]
    pub fn allocated_size(&self) -> u64 {
        self.allocated_size
    }

    /// The logical size this node adds to its parent: zero for a hard link counted elsewhere.
    #[must_use]
    pub fn counted_logical_size(&self) -> u64 {
        if self.flags.contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE) {
            0
        } else {
            self.logical_size
        }
    }

    /// The allocated size this node adds to its parent: zero for a hard link counted elsewhere.
    #[must_use]
    pub fn counted_allocated_size(&self) -> u64 {
        if self.flags.contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE) {
            0
        } else {
            self.allocated_size
        }
    }

    /// Files in the subtree; 0 for a file itself.
    #[must_use]
    pub fn file_count(&self) -> u32 {
        self.file_count
    }

    /// Directories in the subtree, not counting this one.
    #[must_use]
    pub fn directory_count(&self) -> u32 {
        self.directory_count
    }

    #[must_use]
    pub fn modified_time(&self) -> Option<SystemTime> {
        self.modified_time.to_system_time()
    }

    #[must_use]
    pub fn child_count(&self) -> u32 {
        self.child_count
    }

    pub(crate) fn is_removed(&self) -> bool {
        self.flags.contains(NodeFlags::REMOVED)
    }

    pub(crate) fn child_index_range(&self) -> std::ops::Range<usize> {
        let first_child = self.first_child as usize;
        first_child..first_child + self.child_count as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_stays_under_64_bytes() {
        assert!(
            size_of::<Node>() < 64,
            "Node is {} bytes; the memory target is under 64",
            size_of::<Node>()
        );
    }

    #[test]
    fn optional_node_index_round_trips() {
        assert_eq!(OptionalNodeIndex::NONE.get(), None);
        assert_eq!(
            OptionalNodeIndex::some(NodeIndex::new(7)).get(),
            Some(NodeIndex::new(7))
        );
    }

    #[test]
    fn node_time_treats_the_epoch_as_unknown() {
        assert_eq!(
            NodeTime::from_system_time(SystemTime::UNIX_EPOCH).unix_milliseconds(),
            None
        );
        let later = SystemTime::UNIX_EPOCH + Duration::from_millis(1234);
        assert_eq!(
            NodeTime::from_system_time(later).unix_milliseconds(),
            Some(1234)
        );
    }
}
