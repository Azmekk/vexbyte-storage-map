use std::ffi::OsStr;
use std::path::PathBuf;

use super::file_tree_builder::{DirectoryKey, FileTreeBuilder, NewEntry};
use super::node::{NodeFlags, NodeIndex};
use super::tree_arena::FileTree;
use crate::file_identity::FileIdentity;

/// Hard link groups are re-identified by their first member when a subtree is copied; this
/// volume id keeps those identities apart from real ones.
const COPIED_LINK_GROUP_VOLUME_ID: u64 = u64::MAX;

impl FileTree {
    /// Follows `path_components` (directory and file names below the root) down the tree,
    /// comparing names case-insensitively as Windows file systems do.
    pub(crate) fn find_descendant_by_names<'component>(
        &self,
        path_components: impl IntoIterator<Item = &'component OsStr>,
    ) -> Option<NodeIndex> {
        let mut current = self.root();
        for component in path_components {
            let wanted_name = component.to_string_lossy();
            current = self.children(current).find(|&child| {
                let child_node = &self.nodes[child.as_usize()];
                !child_node.flags.contains(NodeFlags::SYNTHETIC)
                    && names_are_equal_ignoring_case(self.name(child), &wanted_name)
            })?;
        }
        Some(current)
    }

    /// Copies the subtree under `node` into a new tree whose root is `node` and whose root
    /// path is `root_path`. Sizes are aggregated anew, and a hard-linked file whose counted
    /// link lies outside the subtree is counted through one of its links inside.
    pub(crate) fn extract_subtree(&self, node: NodeIndex, root_path: PathBuf) -> FileTree {
        let root_key = directory_key_for(node);
        let mut builder = FileTreeBuilder::new(root_path, root_key);
        if let Some(root_node) = self.node(node) {
            builder.set_root_details(
                root_node.modified_time(),
                root_node.flags - NodeFlags::REMOVED,
            );
        }

        self.for_each_live_descendant(node, |descendant, descendant_node| {
            let hard_link_identity =
                self.hard_link_groups
                    .links_of(descendant)
                    .first()
                    .map(|&first_link| {
                        FileIdentity::from_volume_and_file_ids(
                            COPIED_LINK_GROUP_VOLUME_ID,
                            u128::from(first_link.get()),
                        )
                    });

            builder.add_entry(&NewEntry {
                name: self.name(descendant),
                kind: descendant_node.kind,
                parent: descendant_node.parent().map(directory_key_for),
                directory_key: Some(directory_key_for(descendant)),
                logical_size: descendant_node.logical_size,
                allocated_size: descendant_node.allocated_size,
                modified_time: descendant_node.modified_time(),
                flags: descendant_node.flags,
                hard_link_identity,
            });
        });

        builder.finish()
    }
}

fn directory_key_for(node: NodeIndex) -> DirectoryKey {
    DirectoryKey::new(node.get())
}

pub(crate) fn names_are_equal_ignoring_case(first: &str, second: &str) -> bool {
    if first.is_ascii() && second.is_ascii() {
        first.eq_ignore_ascii_case(second)
    } else {
        first.to_lowercase() == second.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::file_tree::OnlineOnlyVisibility;
    use crate::file_tree::test_trees::sample_tree;

    #[test]
    fn finds_nodes_by_name_ignoring_case_and_extracts_their_subtree() {
        let tree = sample_tree();

        let photos = tree
            .find_descendant_by_names([OsStr::new("PHOTOS")])
            .unwrap();
        let subtree = tree.extract_subtree(photos, PathBuf::from("root").join("Photos"));

        let root = subtree.node(subtree.root()).unwrap();
        assert_eq!(root.logical_size(), 1600);
        assert_eq!(root.file_count(), 3);
        assert_eq!(root.directory_count(), 1);
        let shot = subtree.search_nodes("shot", 1, OnlineOnlyVisibility::Shown)[0].node;
        assert_eq!(
            subtree.full_path(shot),
            Some(
                PathBuf::from("root")
                    .join("Photos")
                    .join("raw")
                    .join("shot.cr3")
            )
        );
        assert!(
            tree.find_descendant_by_names([OsStr::new("missing")])
                .is_none()
        );
    }
}
