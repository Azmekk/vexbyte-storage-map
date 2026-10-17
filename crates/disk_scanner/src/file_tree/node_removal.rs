use serde::{Deserialize, Serialize};
use specta::Type;

use super::file_tree_builder::SizeContribution;
use super::node::{NodeFlags, NodeIndex, NodeKind};
use super::tree_arena::FileTree;
use super::tree_query_error::TreeRemovalError;

/// What removing nodes from the tree changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RemovalSummary {
    /// Requested nodes that were removed; nodes already removed (for example because an
    /// ancestor was in the same request) are not counted.
    pub removed_node_count: u32,
    /// Space no longer counted in the tree. A hard-linked file whose other link stays in the
    /// tree frees nothing, because the file still exists.
    pub freed_logical_size: u64,
    pub freed_allocated_size: u64,
}

impl FileTree {
    /// Removes `nodes` and their subtrees from the tree and subtracts their sizes and counts
    /// from every ancestor, so the user interface can update after a recycle action without
    /// rescanning.
    ///
    /// This only changes the tree in memory. It never touches the file system.
    ///
    /// # Errors
    ///
    /// Nothing is changed when any node does not exist, is the root or is synthetic.
    pub fn remove_nodes_and_reaggregate(
        &mut self,
        nodes: &[NodeIndex],
    ) -> Result<RemovalSummary, TreeRemovalError> {
        for &node in nodes {
            self.check_node_can_be_removed(node)?;
        }

        let mut summary = RemovalSummary::default();
        for &node in nodes {
            if self.nodes[node.as_usize()].is_removed() {
                continue;
            }
            self.remove_subtree(node, &mut summary);
            summary.removed_node_count += 1;
        }

        Ok(summary)
    }

    fn check_node_can_be_removed(&self, node: NodeIndex) -> Result<(), TreeRemovalError> {
        let tree_node = self
            .node(node)
            .ok_or(TreeRemovalError::NodeNotFound { node })?;
        if node == self.root() {
            return Err(TreeRemovalError::CannotRemoveRoot);
        }
        if tree_node.flags.contains(NodeFlags::SYNTHETIC) {
            return Err(TreeRemovalError::CannotRemoveSyntheticNode { node });
        }
        Ok(())
    }

    fn remove_subtree(&mut self, node: NodeIndex, summary: &mut RemovalSummary) {
        let contribution = SizeContribution::of(&self.nodes[node.as_usize()]);
        let counted_hard_links = self.mark_subtree_removed(node);

        self.subtract_from_ancestors(node, contribution);
        summary.freed_logical_size += contribution.logical_size;
        summary.freed_allocated_size += contribution.allocated_size;

        for removed_link in counted_hard_links {
            let Some(promoted_link) = self.surviving_link_to_count_instead_of(removed_link) else {
                continue;
            };
            let promoted_node = &mut self.nodes[promoted_link.as_usize()];
            promoted_node
                .flags
                .remove(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE);
            let regained = SizeContribution {
                logical_size: promoted_node.counted_logical_size(),
                allocated_size: promoted_node.counted_allocated_size(),
                file_count: 0,
                directory_count: 0,
                online_only_file_count: 0,
            };
            self.add_to_ancestors(promoted_link, regained);
            summary.freed_logical_size = summary
                .freed_logical_size
                .saturating_sub(regained.logical_size);
            summary.freed_allocated_size = summary
                .freed_allocated_size
                .saturating_sub(regained.allocated_size);
        }
    }

    /// Flags `node` and all its descendants as removed and returns the hard-linked files among
    /// them whose size was counted.
    fn mark_subtree_removed(&mut self, node: NodeIndex) -> Vec<NodeIndex> {
        let mut counted_hard_links = Vec::new();
        let mut nodes_to_mark = vec![node];

        while let Some(current) = nodes_to_mark.pop() {
            let current_node = &mut self.nodes[current.as_usize()];
            if current_node.is_removed() {
                continue;
            }
            current_node.flags |= NodeFlags::REMOVED;

            let is_counted_file = current_node.kind == NodeKind::File
                && !current_node
                    .flags
                    .contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE);
            if is_counted_file && !self.hard_link_groups.links_of(current).is_empty() {
                counted_hard_links.push(current);
            }

            let child_range = self.nodes[current.as_usize()].child_index_range();
            nodes_to_mark.extend(child_range.map(NodeIndex::from_usize));
        }

        counted_hard_links
    }

    fn surviving_link_to_count_instead_of(&self, removed_link: NodeIndex) -> Option<NodeIndex> {
        self.hard_link_groups
            .links_of(removed_link)
            .iter()
            .copied()
            .find(|&link| !self.nodes[link.as_usize()].is_removed())
    }

    fn subtract_from_ancestors(&mut self, node: NodeIndex, contribution: SizeContribution) {
        let mut ancestor = self.nodes[node.as_usize()].parent();
        while let Some(ancestor_index) = ancestor {
            let ancestor_node = &mut self.nodes[ancestor_index.as_usize()];
            contribution.subtract_from(ancestor_node);
            ancestor = ancestor_node.parent();
        }
    }

    fn add_to_ancestors(&mut self, node: NodeIndex, contribution: SizeContribution) {
        let mut ancestor = self.nodes[node.as_usize()].parent();
        while let Some(ancestor_index) = ancestor {
            let ancestor_node = &mut self.nodes[ancestor_index.as_usize()];
            contribution.add_to(ancestor_node);
            ancestor = ancestor_node.parent();
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;

    use super::*;
    use crate::file_identity::FileIdentity;
    use crate::file_tree::test_trees::{child_named, sample_tree};
    use crate::file_tree::{
        DirectoryKey, FileTreeBuilder, NewEntry, OnlineOnlyVisibility, TreeQueryError,
    };

    #[test]
    fn removes_a_subtree_and_updates_every_ancestor() {
        let mut tree = sample_tree();
        let photos = child_named(&tree, tree.root(), "Photos");
        let raw = child_named(&tree, photos, "raw");
        let shot = child_named(&tree, raw, "shot.cr3");

        let summary = tree.remove_nodes_and_reaggregate(&[raw, shot]).unwrap();

        assert_eq!(summary.removed_node_count, 1, "shot.cr3 went with raw");
        assert_eq!(summary.freed_logical_size, 900);
        let root = tree.node(tree.root()).unwrap();
        assert_eq!(root.logical_size(), 1460);
        assert_eq!(root.file_count(), 5);
        assert_eq!(root.directory_count(), 1);
        assert_eq!(tree.node(photos).unwrap().logical_size(), 700);
        assert_eq!(tree.children(photos).count(), 2);
        assert_eq!(
            tree.node_details(shot, OnlineOnlyVisibility::Shown),
            Err(TreeQueryError::NodeRemoved { node: shot })
        );
        assert_eq!(
            tree.search_nodes("shot", 10, OnlineOnlyVisibility::Shown),
            []
        );
    }

    #[test]
    fn rejects_the_root_without_changing_anything() {
        let mut tree = sample_tree();
        let photos = child_named(&tree, tree.root(), "Photos");

        let result = tree.remove_nodes_and_reaggregate(&[photos, tree.root()]);

        assert_eq!(result, Err(TreeRemovalError::CannotRemoveRoot));
        assert_eq!(tree.node(tree.root()).unwrap().logical_size(), 2360);
    }

    #[test]
    fn counts_the_other_link_when_the_counted_hard_link_is_removed() {
        let identity = FileIdentity::from_volume_and_file_ids(3, 77);
        let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
        for name in ["first.bin", "second.bin"] {
            builder.add_entry(&NewEntry {
                name,
                kind: NodeKind::File,
                parent: Some(DirectoryKey::new(0)),
                directory_key: None,
                logical_size: 500,
                allocated_size: 512,
                modified_time: None,
                flags: NodeFlags::empty(),
                hard_link_identity: Some(identity),
            });
        }
        let mut tree = builder.finish();
        let first = child_named(&tree, tree.root(), "first.bin");
        let second = child_named(&tree, tree.root(), "second.bin");

        let summary = tree.remove_nodes_and_reaggregate(&[first]).unwrap();

        assert_eq!(summary.freed_logical_size, 0, "the file still exists");
        assert_eq!(tree.node(tree.root()).unwrap().logical_size(), 500);
        assert_eq!(tree.node(tree.root()).unwrap().file_count(), 1);
        assert!(
            !tree
                .node(second)
                .unwrap()
                .flags()
                .contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE)
        );

        let summary = tree.remove_nodes_and_reaggregate(&[second]).unwrap();
        assert_eq!(summary.freed_logical_size, 500);
        assert_eq!(tree.node(tree.root()).unwrap().logical_size(), 0);
    }
}
