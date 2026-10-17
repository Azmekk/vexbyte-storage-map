use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::file_identity::FileIdentity;
use crate::file_tree::{NodeFlags, NodeKind};

bitflags::bitflags! {
    /// Platform-neutral facts about a directory entry, as reported by a platform backend.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub(crate) struct EntryFlags: u16 {
        const SYMLINK = 1 << 0;
        /// A junction or mount point: a directory that stands for another location.
        const JUNCTION = 1 << 1;
        /// Content that is only in the cloud; opening it would download it.
        const CLOUD_PLACEHOLDER = 1 << 2;
        const HIDDEN = 1 << 3;
        const SYSTEM = 1 << 4;
        const INACCESSIBLE = 1 << 5;
        const COMPRESSED = 1 << 6;
        const SPARSE = 1 << 7;
    }
}

impl EntryFlags {
    /// Symlinks, junctions and cloud placeholders are recorded but never entered.
    pub(crate) fn must_not_be_entered(self) -> bool {
        self.intersects(Self::SYMLINK | Self::JUNCTION | Self::CLOUD_PLACEHOLDER)
    }

    pub(crate) fn to_node_flags(self) -> NodeFlags {
        let flag_pairs = [
            (Self::SYMLINK, NodeFlags::SYMLINK),
            (Self::JUNCTION, NodeFlags::JUNCTION),
            (Self::CLOUD_PLACEHOLDER, NodeFlags::CLOUD_PLACEHOLDER),
            (Self::HIDDEN, NodeFlags::HIDDEN),
            (Self::SYSTEM, NodeFlags::SYSTEM),
            (Self::INACCESSIBLE, NodeFlags::INACCESSIBLE),
            (Self::COMPRESSED, NodeFlags::COMPRESSED),
            (Self::SPARSE, NodeFlags::SPARSE),
        ];

        flag_pairs
            .into_iter()
            .filter(|(entry_flag, _)| self.contains(*entry_flag))
            .fold(NodeFlags::empty(), |node_flags, (_, node_flag)| {
                node_flags | node_flag
            })
    }
}

/// One entry of a directory listing, with everything the tree needs and nothing that would
/// require opening the entry itself.
pub(crate) struct DirectoryEntry<'name> {
    pub(crate) name: &'name str,
    pub(crate) kind: NodeKind,
    /// 0 for directories.
    pub(crate) logical_size: u64,
    pub(crate) allocated_size: u64,
    pub(crate) modified_time: SystemTime,
    /// `None` when the file system reports no usable file ID.
    pub(crate) identity: Option<FileIdentity>,
    pub(crate) flags: EntryFlags,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DirectoryListError {
    #[error("could not open the directory {}", path.display())]
    OpenFailed { path: PathBuf, source: io::Error },
    #[error("could not read the entries of the directory {}", path.display())]
    ReadFailed { path: PathBuf, source: io::Error },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_entry_flag_to_its_node_flag() {
        let entry_flags = EntryFlags::all();

        let node_flags = entry_flags.to_node_flags();

        assert_eq!(
            node_flags,
            NodeFlags::SYMLINK
                | NodeFlags::JUNCTION
                | NodeFlags::CLOUD_PLACEHOLDER
                | NodeFlags::HIDDEN
                | NodeFlags::SYSTEM
                | NodeFlags::INACCESSIBLE
                | NodeFlags::COMPRESSED
                | NodeFlags::SPARSE
        );
    }

    #[test]
    fn links_and_placeholders_are_never_entered() {
        assert!(EntryFlags::JUNCTION.must_not_be_entered());
        assert!(EntryFlags::SYMLINK.must_not_be_entered());
        assert!(EntryFlags::CLOUD_PLACEHOLDER.must_not_be_entered());
        assert!(!(EntryFlags::HIDDEN | EntryFlags::SYSTEM).must_not_be_entered());
    }
}
