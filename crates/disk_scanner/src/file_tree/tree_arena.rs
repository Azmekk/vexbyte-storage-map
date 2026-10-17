use std::path::{Path, PathBuf};

use file_categories::FileCategory;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::hard_link_groups::HardLinkGroups;
use super::name_storage::NameStorage;
use super::node::{Node, NodeFlags, NodeIndex, NodeKind};
use super::tree_query_error::TreeQueryError;

/// The finished result of a scan: every file and directory in a compact arena.
///
/// The tree is only changed by [`FileTree::remove_nodes_and_reaggregate`]. It owns all its
/// data, so it is `Send + Sync`: share it as `Arc<RwLock<FileTree>>` and run queries
/// concurrently under a read lock.
#[derive(Debug, Clone)]
pub struct FileTree {
    pub(crate) nodes: Vec<Node>,
    pub(crate) names: NameStorage,
    root_path: PathBuf,
    pub(crate) hard_link_groups: HardLinkGroups,
}

/// Totals over a whole tree, for the scan summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TreeTotals {
    /// Sum of file sizes, each hard-linked file counted once.
    pub logical_size: u64,
    pub allocated_size: u64,
    pub file_count: u32,
    pub directory_count: u32,
    pub inaccessible_directory_count: u32,
    /// Files whose content is only in the cloud. Their logical size is part of `logical_size`;
    /// they occupy (almost) no space on disk.
    pub cloud_placeholder_file_count: u32,
    pub cloud_placeholder_logical_size: u64,
    /// Additional hard links: paths shown in the tree that add no size because another path to
    /// the same file is counted.
    pub additional_hard_link_count: u32,
    /// Symlinks and junctions, recorded but never followed.
    pub link_count: u32,
}

impl FileTree {
    pub(crate) fn from_parts(
        nodes: Vec<Node>,
        names: NameStorage,
        root_path: PathBuf,
        hard_link_groups: HardLinkGroups,
    ) -> Self {
        Self {
            nodes,
            names,
            root_path,
            hard_link_groups,
        }
    }

    /// Takes the tree apart for the search index, which keeps the names and converts the nodes
    /// into its own compact form.
    pub(crate) fn into_nodes_names_and_root_path(self) -> (Vec<Node>, NameStorage, PathBuf) {
        (self.nodes, self.names, self.root_path)
    }

    #[must_use]
    pub fn root(&self) -> NodeIndex {
        NodeIndex::new(0)
    }

    /// The scanned directory, for example `C:\` or `C:\Users\someone`.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// The number of nodes in the arena, including nodes removed after a recycle action.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// The node at `index`, also when it has been removed.
    #[must_use]
    pub fn node(&self, index: NodeIndex) -> Option<&Node> {
        self.nodes.get(index.as_usize())
    }

    /// The node at `index`, unless it does not exist or has been removed.
    pub(crate) fn live_node(&self, index: NodeIndex) -> Result<&Node, TreeQueryError> {
        let node = self
            .node(index)
            .ok_or(TreeQueryError::NodeNotFound { node: index })?;
        if node.is_removed() {
            return Err(TreeQueryError::NodeRemoved { node: index });
        }
        Ok(node)
    }

    /// The node's own name; for the root, the scanned path.
    #[must_use]
    pub fn name(&self, index: NodeIndex) -> &str {
        self.node(index)
            .map_or("", |node| self.names.name(node.name))
    }

    /// The lowercase extension of a file, without the dot.
    #[must_use]
    pub fn extension(&self, index: NodeIndex) -> Option<&str> {
        self.node(index)
            .and_then(|node| self.names.extension(node.extension))
    }

    #[must_use]
    pub fn category(&self, index: NodeIndex) -> Option<FileCategory> {
        self.node(index)
            .and_then(|node| self.names.extension_category(node.extension))
    }

    /// The children of a directory that have not been removed, in storage order.
    pub fn children(&self, index: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        let child_range = self.node(index).map_or(0..0, Node::child_index_range);

        child_range
            .filter(|&child| !self.nodes[child].is_removed())
            .map(NodeIndex::from_usize)
    }

    /// Rebuilds the full path from the parent chain and the root path. Synthetic grouping
    /// directories are skipped, so `$MFT` under "NTFS metadata" resolves to `C:\$MFT`.
    ///
    /// Returns `None` for unknown nodes and for synthetic nodes themselves.
    #[must_use]
    pub fn full_path(&self, index: NodeIndex) -> Option<PathBuf> {
        let node = self.node(index)?;
        if node.flags.contains(NodeFlags::SYNTHETIC) {
            return None;
        }

        let mut names_from_node_to_root = Vec::new();
        let mut current = index;
        while let Some(parent) = self.node(current)?.parent() {
            let current_node = self.node(current)?;
            // An entry whose parent directory was not found would otherwise get a path
            // directly below the root, which names a different item (or nothing).
            if current_node.flags.contains(NodeFlags::LOCATION_UNKNOWN) {
                return None;
            }
            if !current_node.flags.contains(NodeFlags::SYNTHETIC) {
                names_from_node_to_root.push(self.names.name(current_node.name));
            }
            current = parent;
        }

        let mut full_path = self.root_path.clone();
        for name in names_from_node_to_root.into_iter().rev() {
            full_path.push(name);
        }
        Some(full_path)
    }

    /// Totals and counts over every live node of the tree.
    #[must_use]
    pub fn totals(&self) -> TreeTotals {
        let root = &self.nodes[0];
        let mut totals = TreeTotals {
            logical_size: root.logical_size,
            allocated_size: root.allocated_size,
            file_count: root.file_count,
            directory_count: root.directory_count,
            ..TreeTotals::default()
        };

        for node in self.nodes.iter().filter(|node| !node.is_removed()) {
            let flags = node.flags;
            if flags.contains(NodeFlags::INACCESSIBLE) {
                totals.inaccessible_directory_count += 1;
            }
            if flags.intersects(NodeFlags::SYMLINK | NodeFlags::JUNCTION) {
                totals.link_count += 1;
            }
            if node.kind == NodeKind::File {
                if flags.contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE) {
                    totals.additional_hard_link_count += 1;
                } else if flags.contains(NodeFlags::CLOUD_PLACEHOLDER) {
                    totals.cloud_placeholder_file_count += 1;
                    totals.cloud_placeholder_logical_size += node.logical_size;
                }
            }
        }

        totals
    }

    /// Approximate heap memory held by the tree, for diagnostics and the memory target.
    #[must_use]
    pub fn estimated_memory_bytes(&self) -> usize {
        let node_bytes = self.nodes.capacity() * size_of::<Node>();
        let name_bytes = self.names.name_text_bytes() + self.nodes.len() * size_of::<u32>();
        node_bytes + name_bytes
    }
}
