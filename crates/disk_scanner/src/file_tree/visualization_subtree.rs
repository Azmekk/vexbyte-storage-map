use std::collections::VecDeque;

use file_categories::FileCategory;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::node::{NodeIndex, NodeKind};
use super::online_only_visibility::OnlineOnlyVisibility;
use super::tree_arena::FileTree;
use super::tree_query_error::TreeQueryError;

/// The treemap and sunburst must stay smooth, so a subtree never has more nodes than this.
pub const VISUALIZATION_NODE_LIMIT: usize = 5_000;
/// Deeper levels are unreadable in a sunburst and invisible in a treemap.
const MAXIMUM_VISUALIZATION_DEPTH: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum VisualizationNodeKind {
    File,
    Directory,
    /// Children below the size threshold, merged into one block.
    SmallerItems,
}

/// A pruned, nested view of a subtree for the treemap and sunburst.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VisualizationNode {
    /// `None` for merged "smaller items" blocks.
    pub node: Option<NodeIndex>,
    pub name: String,
    pub kind: VisualizationNodeKind,
    /// The counted logical size, so children always add up to at most their parent.
    pub logical_size: u64,
    pub category: Option<FileCategory>,
    /// How many items a "smaller items" block stands for; 0 for other nodes.
    pub merged_item_count: u32,
    pub children: Vec<VisualizationNode>,
}

/// A visualization node before its children are nested into it.
struct FlatVisualizationNode {
    node: VisualizationNode,
    child_slots: Vec<usize>,
}

struct DirectoryToExpand {
    tree_node: NodeIndex,
    slot: usize,
    depth: u32,
}

impl FileTree {
    /// A nested view of `node` down to `max_depth` levels, where children smaller than
    /// `minimum_share_of_parent` (0 to 1) of their parent are merged into one
    /// "N smaller items" block. Items that `visibility` hides are left out altogether.
    ///
    /// Levels are expanded breadth first until [`VISUALIZATION_NODE_LIMIT`] nodes are reached,
    /// so the top levels are always complete and deep levels are cut first.
    ///
    /// # Errors
    ///
    /// Fails when `node` does not exist or was removed.
    pub fn visualization_subtree(
        &self,
        node: NodeIndex,
        max_depth: u32,
        minimum_share_of_parent: f64,
        visibility: OnlineOnlyVisibility,
    ) -> Result<VisualizationNode, TreeQueryError> {
        self.live_node(node)?;
        let max_depth = max_depth.min(MAXIMUM_VISUALIZATION_DEPTH);
        let minimum_share_of_parent = if minimum_share_of_parent.is_finite() {
            minimum_share_of_parent.clamp(0.0, 1.0)
        } else {
            0.0
        };

        let mut flat_nodes = vec![FlatVisualizationNode {
            node: self.visualization_node_for(node),
            child_slots: Vec::new(),
        }];
        let mut directories_to_expand = VecDeque::from([DirectoryToExpand {
            tree_node: node,
            slot: 0,
            depth: 0,
        }]);

        while let Some(directory) = directories_to_expand.pop_front() {
            if directory.depth >= max_depth {
                continue;
            }
            let remaining_budget = VISUALIZATION_NODE_LIMIT.saturating_sub(flat_nodes.len());
            if remaining_budget == 0 {
                break;
            }

            let expansion = self.expand_directory(
                directory.tree_node,
                minimum_share_of_parent,
                remaining_budget,
                visibility,
            );
            for (child_tree_node, child_node) in expansion.shown_children {
                let child_slot = flat_nodes.len();
                let is_directory = child_node.kind == VisualizationNodeKind::Directory;
                flat_nodes.push(FlatVisualizationNode {
                    node: child_node,
                    child_slots: Vec::new(),
                });
                flat_nodes[directory.slot].child_slots.push(child_slot);
                if is_directory {
                    directories_to_expand.push_back(DirectoryToExpand {
                        tree_node: child_tree_node,
                        slot: child_slot,
                        depth: directory.depth + 1,
                    });
                }
            }
            if let Some(smaller_items) = expansion.smaller_items {
                let smaller_items_slot = flat_nodes.len();
                flat_nodes.push(FlatVisualizationNode {
                    node: smaller_items,
                    child_slots: Vec::new(),
                });
                flat_nodes[directory.slot]
                    .child_slots
                    .push(smaller_items_slot);
            }
        }

        Ok(nest_flat_nodes(flat_nodes))
    }

    fn visualization_node_for(&self, node: NodeIndex) -> VisualizationNode {
        let tree_node = &self.nodes[node.as_usize()];
        let kind = match tree_node.kind {
            NodeKind::File => VisualizationNodeKind::File,
            NodeKind::Directory => VisualizationNodeKind::Directory,
        };

        VisualizationNode {
            node: Some(node),
            name: self.name(node).to_owned(),
            kind,
            logical_size: tree_node.counted_logical_size(),
            category: self.category(node),
            merged_item_count: 0,
            children: Vec::new(),
        }
    }

    fn expand_directory(
        &self,
        directory: NodeIndex,
        minimum_share_of_parent: f64,
        remaining_budget: usize,
        visibility: OnlineOnlyVisibility,
    ) -> DirectoryExpansion {
        let directory_size = self.nodes[directory.as_usize()].logical_size;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let size_threshold = (directory_size as f64 * minimum_share_of_parent).ceil() as u64;

        let mut children: Vec<(u64, NodeIndex)> = self
            .shown_children(directory, visibility)
            .map(|child| (self.nodes[child.as_usize()].counted_logical_size(), child))
            .collect();
        children.sort_unstable_by(|first, second| second.cmp(first));

        let large_enough_count = children
            .iter()
            .take_while(|(size, _)| *size > 0 && *size >= size_threshold)
            .count();
        let everything_fits =
            large_enough_count == children.len() && children.len() <= remaining_budget;
        let shown_count = if everything_fits {
            large_enough_count
        } else {
            // One slot of the budget is kept for the "smaller items" block.
            large_enough_count.min(remaining_budget.saturating_sub(1))
        };

        let shown_children = children[..shown_count]
            .iter()
            .map(|&(_, child)| (child, self.visualization_node_for(child)))
            .collect();

        let merged_children = &children[shown_count..];
        let smaller_items = (!merged_children.is_empty()).then(|| {
            let merged_item_count = u32::try_from(merged_children.len()).unwrap_or(u32::MAX);
            VisualizationNode {
                node: None,
                name: format!("{merged_item_count} smaller items"),
                kind: VisualizationNodeKind::SmallerItems,
                logical_size: merged_children.iter().map(|(size, _)| size).sum(),
                category: None,
                merged_item_count,
                children: Vec::new(),
            }
        });

        DirectoryExpansion {
            shown_children,
            smaller_items,
        }
    }
}

struct DirectoryExpansion {
    shown_children: Vec<(NodeIndex, VisualizationNode)>,
    smaller_items: Option<VisualizationNode>,
}

/// Nests the breadth-first list into a tree. Children always have larger slots than their
/// parent, so building from the last slot backwards finds every child already complete.
fn nest_flat_nodes(flat_nodes: Vec<FlatVisualizationNode>) -> VisualizationNode {
    let mut child_slots_by_slot = Vec::with_capacity(flat_nodes.len());
    let mut finished_nodes: Vec<Option<VisualizationNode>> = Vec::with_capacity(flat_nodes.len());
    for flat_node in flat_nodes {
        child_slots_by_slot.push(flat_node.child_slots);
        finished_nodes.push(Some(flat_node.node));
    }

    for slot in (0..finished_nodes.len()).rev() {
        let children: Vec<VisualizationNode> = child_slots_by_slot[slot]
            .iter()
            .filter_map(|&child_slot| finished_nodes[child_slot].take())
            .collect();
        if let Some(node) = finished_nodes[slot].as_mut() {
            node.children = children;
        }
    }

    finished_nodes
        .into_iter()
        .next()
        .flatten()
        .unwrap_or(VisualizationNode {
            node: None,
            name: String::new(),
            kind: VisualizationNodeKind::Directory,
            logical_size: 0,
            category: None,
            merged_item_count: 0,
            children: Vec::new(),
        })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;

    use super::*;
    use crate::file_tree::test_trees::{cloud_tree, sample_tree};
    use crate::file_tree::{DirectoryKey, FileTreeBuilder, NewEntry, NodeFlags};

    const SHOWN: OnlineOnlyVisibility = OnlineOnlyVisibility::Shown;

    fn count_nodes(node: &VisualizationNode) -> usize {
        1 + node.children.iter().map(count_nodes).sum::<usize>()
    }

    #[test]
    fn merges_small_children_and_respects_depth() {
        let tree = sample_tree();

        let subtree = tree
            .visualization_subtree(tree.root(), 1, 0.1, SHOWN)
            .unwrap();

        let names: Vec<_> = subtree
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect();
        assert_eq!(names, ["Photos", "setup.exe", "2 smaller items"]);
        assert_eq!(subtree.children[2].logical_size, 60);
        assert_eq!(subtree.children[2].merged_item_count, 2);
        assert!(
            subtree.children[0].children.is_empty(),
            "depth 1 stops below the first level"
        );

        let deeper = tree
            .visualization_subtree(tree.root(), 3, 0.0, SHOWN)
            .unwrap();
        let photos = &deeper.children[0];
        assert_eq!(photos.children.len(), 3);
        assert_eq!(photos.children[0].name, "raw");
        assert_eq!(photos.children[0].children[0].name, "shot.cr3");
    }

    #[test]
    fn never_exceeds_the_node_limit() {
        let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
        let names: Vec<String> = (0..20_000)
            .map(|number| format!("file-{number}.bin"))
            .collect();
        for (number, name) in names.iter().enumerate() {
            builder.add_entry(&NewEntry {
                name,
                kind: NodeKind::File,
                parent: Some(DirectoryKey::new(0)),
                directory_key: None,
                logical_size: 1000 + number as u64,
                allocated_size: 0,
                modified_time: None,
                flags: NodeFlags::empty(),
                hard_link_identity: None,
            });
        }
        let tree = builder.finish();

        let subtree = tree
            .visualization_subtree(tree.root(), 5, 0.0, SHOWN)
            .unwrap();

        assert_eq!(count_nodes(&subtree), VISUALIZATION_NODE_LIMIT);
        let smaller_items = subtree.children.last().unwrap();
        assert_eq!(smaller_items.kind, VisualizationNodeKind::SmallerItems);
        let children_total: u64 = subtree
            .children
            .iter()
            .map(|child| child.logical_size)
            .sum();
        assert_eq!(children_total, subtree.logical_size);
    }

    #[test]
    fn hidden_online_only_items_are_not_even_counted_as_smaller_items() {
        let tree = cloud_tree();

        let hidden = tree
            .visualization_subtree(tree.root(), 2, 0.0, OnlineOnlyVisibility::Hidden)
            .unwrap();
        let names: Vec<_> = hidden
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect();
        assert_eq!(names, ["partial.iso", "Mixed", "2 smaller items"]);
        assert_eq!(
            hidden.children[2].merged_item_count, 2,
            "Empty and zero.txt"
        );

        let shown = tree
            .visualization_subtree(tree.root(), 2, 0.0, SHOWN)
            .unwrap();
        let smaller_items = shown.children.last().unwrap();
        assert_eq!(
            smaller_items.merged_item_count, 5,
            "the online-only items take no space, so they end up among the smaller items"
        );
        assert_eq!(smaller_items.logical_size, 0);
    }
}
