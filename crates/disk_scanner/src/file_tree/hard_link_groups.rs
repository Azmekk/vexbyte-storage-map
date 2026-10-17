use std::collections::HashMap;

use super::node::{Node, NodeFlags, NodeIndex};

/// The nodes of a finished tree that are hard links to the same file. Only files with more
/// than one link in the tree are listed, so this stays small.
///
/// Needed when a counted link is removed from the tree: another link to the same file then
/// starts counting, because the file still occupies the disk.
#[derive(Debug, Clone, Default)]
pub(crate) struct HardLinkGroups {
    groups: Vec<Vec<NodeIndex>>,
    group_by_node: HashMap<NodeIndex, usize>,
}

impl HardLinkGroups {
    /// Translates groups of pending entries into node indices and makes sure every group
    /// counts exactly one link. Must run before sizes are aggregated.
    pub(crate) fn from_pending_groups(
        pending_groups: &[Vec<u32>],
        node_index_by_pending_entry: &[u32],
        nodes: &mut [Node],
    ) -> Self {
        let mut hard_link_groups = Self::default();

        for pending_group in pending_groups {
            let group: Vec<NodeIndex> = pending_group
                .iter()
                .filter_map(|&pending_entry| {
                    let node_index = *node_index_by_pending_entry.get(pending_entry as usize)?;
                    (node_index != u32::MAX).then_some(NodeIndex::new(node_index))
                })
                .collect();

            let Some(&first_reachable_link) = group.first() else {
                continue;
            };
            let has_counted_link = group.iter().any(|&link| {
                !nodes[link.as_usize()]
                    .flags
                    .contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE)
            });
            if !has_counted_link {
                nodes[first_reachable_link.as_usize()]
                    .flags
                    .remove(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE);
            }

            if group.len() > 1 {
                hard_link_groups.add_group(group);
            }
        }

        hard_link_groups
    }

    pub(crate) fn add_group(&mut self, group: Vec<NodeIndex>) {
        let group_index = self.groups.len();
        for &link in &group {
            self.group_by_node.insert(link, group_index);
        }
        self.groups.push(group);
    }

    /// Every link to the same file as `node`, including `node`, or an empty slice.
    pub(crate) fn links_of(&self, node: NodeIndex) -> &[NodeIndex] {
        self.group_by_node
            .get(&node)
            .and_then(|&group_index| self.groups.get(group_index))
            .map_or(&[], Vec::as_slice)
    }
}
