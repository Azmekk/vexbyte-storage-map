use crate::file_tree::{
    ExtensionId, Node, NodeFlags, NodeIndex, NodeKind, NodeTime, OptionalNodeIndex,
};

bitflags::bitflags! {
    /// What a search needs to know about a node besides its sizes and times.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub(crate) struct SearchNodeFlags: u16 {
        const DIRECTORY = 1 << 0;
        /// Appears in search results: not the root, not a synthetic grouping folder and not
        /// inside one whose location on disk is unknown.
        const SEARCHABLE = 1 << 1;
        /// A grouping folder created by the scanner; it is not part of any path.
        const SYNTHETIC = 1 << 2;
        /// The synthetic "Unlinked items" folder; nothing below it has a path.
        const LOCATION_UNKNOWN = 1 << 3;
        /// The name is plain ASCII, so it is lowercased byte by byte.
        const NAME_IS_ASCII = 1 << 4;
        /// The name is already in lowercase, so a query reads it without lowercasing it.
        const NAME_IS_LOWERCASE = 1 << 5;
        const CLOUD_PLACEHOLDER = 1 << 6;
        /// A symbolic link, junction or mount point.
        const REPARSE_POINT = 1 << 7;
        const HARD_LINK_COUNTED_ELSEWHERE = 1 << 8;
        const INACCESSIBLE = 1 << 9;
    }
}

/// One file or folder in a `VolumeSearchIndex`: 32 bytes instead of the 56 of a scan `Node`.
///
/// The index stores the name of node `i` as name `i` of its `NameStorage`, so the node needs
/// no name id, and a pass over the nodes reads the names front to back. It has no children
/// range either: searches only walk up the tree.
#[derive(Debug, Clone)]
pub(crate) struct SearchIndexNode {
    pub(crate) parent: OptionalNodeIndex,
    pub(crate) extension: ExtensionId,
    /// The file size; for a folder, the total size of everything inside it.
    pub(crate) logical_size: u64,
    pub(crate) modified_time: NodeTime,
    /// The number of ancestors (0 for the root), so two nodes are compared by path without
    /// walking all the way up to the root.
    pub(crate) depth: u16,
    pub(crate) flags: SearchNodeFlags,
}

impl SearchIndexNode {
    /// Takes over a scan node; `depth`, `SEARCHABLE` and the name flags are set later.
    pub(crate) fn from_tree_node(node: &Node) -> Self {
        let tree_flags = node.flags;
        let mut flags = SearchNodeFlags::empty();
        let flag_pairs = [
            (NodeFlags::SYNTHETIC, SearchNodeFlags::SYNTHETIC),
            (
                NodeFlags::LOCATION_UNKNOWN,
                SearchNodeFlags::LOCATION_UNKNOWN,
            ),
            (
                NodeFlags::CLOUD_PLACEHOLDER,
                SearchNodeFlags::CLOUD_PLACEHOLDER,
            ),
            (
                NodeFlags::SYMLINK | NodeFlags::JUNCTION,
                SearchNodeFlags::REPARSE_POINT,
            ),
            (
                NodeFlags::HARD_LINK_COUNTED_ELSEWHERE,
                SearchNodeFlags::HARD_LINK_COUNTED_ELSEWHERE,
            ),
            (NodeFlags::INACCESSIBLE, SearchNodeFlags::INACCESSIBLE),
        ];
        for (tree_flag, search_flag) in flag_pairs {
            flags.set(search_flag, tree_flags.intersects(tree_flag));
        }
        flags.set(SearchNodeFlags::DIRECTORY, node.kind == NodeKind::Directory);

        Self {
            parent: node.parent,
            extension: node.extension,
            logical_size: node.logical_size,
            modified_time: node.modified_time,
            depth: 0,
            flags,
        }
    }

    pub(crate) fn kind(&self) -> NodeKind {
        if self.flags.contains(SearchNodeFlags::DIRECTORY) {
            NodeKind::Directory
        } else {
            NodeKind::File
        }
    }

    pub(crate) fn parent(&self) -> Option<NodeIndex> {
        self.parent.get()
    }

    pub(crate) fn is_searchable(&self) -> bool {
        self.flags.contains(SearchNodeFlags::SEARCHABLE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_index_node_takes_32_bytes() {
        assert_eq!(size_of::<SearchIndexNode>(), 32);
    }
}
