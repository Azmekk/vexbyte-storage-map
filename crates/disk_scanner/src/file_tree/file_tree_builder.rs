use std::path::PathBuf;
use std::time::SystemTime;

use super::hard_link_groups::HardLinkGroups;
use super::hard_link_tracker::{HardLinkTracker, LinkRegistration};
use super::name_storage::{ExtensionId, NameStorage};
use super::node::{Node, NodeFlags, NodeIndex, NodeKind, NodeTime, OptionalNodeIndex};
use super::tree_arena::FileTree;
use crate::file_identity::FileIdentity;

/// A caller-chosen number that identifies a directory while the tree is built, so entries can
/// name their parent before the parent itself has been added. The folder walk numbers
/// directories as it discovers them; the MFT scanner uses record numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DirectoryKey(u32);

impl DirectoryKey {
    #[must_use]
    pub const fn new(key: u32) -> Self {
        Self(key)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// One file or directory to add to the tree.
#[derive(Debug, Clone)]
pub struct NewEntry<'name> {
    pub name: &'name str,
    pub kind: NodeKind,
    /// The directory that contains this entry. `None` (or a key that is never added) puts the
    /// entry into a synthetic "Unlinked items" directory under the root.
    pub parent: Option<DirectoryKey>,
    /// The key children use to name this directory as their parent. Ignored for files.
    pub directory_key: Option<DirectoryKey>,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub modified_time: Option<SystemTime>,
    pub flags: NodeFlags,
    /// Set for files that may have several hard links; links sharing an identity are counted
    /// once.
    pub hard_link_identity: Option<FileIdentity>,
}

const NO_PENDING_ENTRY: u32 = u32::MAX;
const UNLINKED_ITEMS_DIRECTORY_NAME: &str = "Unlinked items";

/// Collects entries in any order, then links parents and aggregates sizes in `finish`.
///
/// Entries may arrive before their parent directory (the parallel folder walk and the MFT
/// both produce them out of tree order), so parents are resolved by key only once everything
/// has been collected.
pub struct FileTreeBuilder {
    names: NameStorage,
    /// Index 0 is the root.
    pending_nodes: Vec<Node>,
    /// The parent key of each pending node, `NO_PENDING_ENTRY` when it has none.
    pending_parent_keys: Vec<u32>,
    pending_entry_by_directory_key: Vec<u32>,
    inaccessible_directory_keys: Vec<DirectoryKey>,
    hard_links: HardLinkTracker,
    root_path: PathBuf,
}

impl FileTreeBuilder {
    /// Starts a tree whose root directory is `root_path`, known to children as `root_key`.
    #[must_use]
    pub fn new(root_path: PathBuf, root_key: DirectoryKey) -> Self {
        Self::with_capacity(root_path, root_key, 0)
    }

    #[must_use]
    pub fn with_capacity(
        root_path: PathBuf,
        root_key: DirectoryKey,
        entry_capacity: usize,
    ) -> Self {
        let mut names = NameStorage::with_capacity(entry_capacity);
        let root_name = names.add_name(&root_path.to_string_lossy());
        let root_node = Node::new_unlinked(root_name, ExtensionId::NONE, NodeKind::Directory);

        let mut pending_nodes = Vec::with_capacity(entry_capacity + 1);
        pending_nodes.push(root_node);
        let mut pending_parent_keys = Vec::with_capacity(entry_capacity + 1);
        pending_parent_keys.push(NO_PENDING_ENTRY);

        let mut builder = Self {
            names,
            pending_nodes,
            pending_parent_keys,
            pending_entry_by_directory_key: Vec::new(),
            inaccessible_directory_keys: Vec::new(),
            hard_links: HardLinkTracker::default(),
            root_path,
        };
        builder.register_directory_key(root_key, 0);
        builder
    }

    /// Sets the root directory's own modification time and flags.
    pub fn set_root_details(&mut self, modified_time: Option<SystemTime>, flags: NodeFlags) {
        let root = &mut self.pending_nodes[0];
        root.modified_time = modified_time.map_or(NodeTime::UNKNOWN, NodeTime::from_system_time);
        root.flags = flags;
    }

    /// The number of entries added so far, including the root.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.pending_nodes.len()
    }

    pub fn add_entry(&mut self, entry: &NewEntry<'_>) {
        let pending_entry = u32::try_from(self.pending_nodes.len()).unwrap_or(NO_PENDING_ENTRY);
        let name = self.names.add_name(entry.name);
        let extension = match entry.kind {
            NodeKind::File => self.names.intern_extension_of(entry.name),
            NodeKind::Directory => ExtensionId::NONE,
        };

        let mut node = Node::new_unlinked(name, extension, entry.kind);
        node.flags = entry.flags - NodeFlags::HARD_LINK_COUNTED_ELSEWHERE - NodeFlags::REMOVED;
        node.modified_time = entry
            .modified_time
            .map_or(NodeTime::UNKNOWN, NodeTime::from_system_time);

        match entry.kind {
            NodeKind::File => {
                node.logical_size = entry.logical_size;
                node.allocated_size = entry.allocated_size;
                if let Some(identity) = entry.hard_link_identity {
                    let registration = self.hard_links.register_link(identity, pending_entry);
                    if matches!(registration, LinkRegistration::AdditionalLink { .. }) {
                        node.flags |= NodeFlags::HARD_LINK_COUNTED_ELSEWHERE;
                    }
                }
            }
            NodeKind::Directory => {
                if let Some(directory_key) = entry.directory_key {
                    self.register_directory_key(directory_key, pending_entry);
                }
            }
        }

        self.pending_nodes.push(node);
        self.pending_parent_keys
            .push(entry.parent.map_or(NO_PENDING_ENTRY, DirectoryKey::get));
    }

    /// Adds a directory that does not exist on disk, used to group entries.
    pub fn add_synthetic_directory(
        &mut self,
        name: &str,
        parent: Option<DirectoryKey>,
        directory_key: DirectoryKey,
    ) {
        self.add_entry(&NewEntry {
            name,
            kind: NodeKind::Directory,
            parent,
            directory_key: Some(directory_key),
            logical_size: 0,
            allocated_size: 0,
            modified_time: None,
            flags: NodeFlags::SYNTHETIC,
            hard_link_identity: None,
        });
    }

    /// Flags a directory that could not be listed. The directory may be added before or after.
    pub fn mark_directory_inaccessible(&mut self, directory_key: DirectoryKey) {
        self.inaccessible_directory_keys.push(directory_key);
    }

    fn register_directory_key(&mut self, directory_key: DirectoryKey, pending_entry: u32) {
        let key_index = directory_key.get() as usize;
        if key_index >= self.pending_entry_by_directory_key.len() {
            self.pending_entry_by_directory_key
                .resize(key_index + 1, NO_PENDING_ENTRY);
        }
        let slot = &mut self.pending_entry_by_directory_key[key_index];
        if *slot == NO_PENDING_ENTRY {
            *slot = pending_entry;
        }
    }

    fn pending_entry_for_key(&self, key: u32) -> u32 {
        self.pending_entry_by_directory_key
            .get(key as usize)
            .copied()
            .unwrap_or(NO_PENDING_ENTRY)
    }

    /// Links every entry to its parent, stores the tree in breadth-first order and aggregates
    /// sizes and counts in one post-order pass.
    #[must_use]
    pub fn finish(mut self) -> FileTree {
        self.apply_inaccessible_flags();
        let parent_entries = self.resolve_parent_entries();
        let children = ChildLists::group_by_parent(&parent_entries);
        let breadth_first_order = children.breadth_first_order();

        let mut node_index_by_pending_entry = vec![NO_PENDING_ENTRY; self.pending_nodes.len()];
        for (node_index, &pending_entry) in breadth_first_order.iter().enumerate() {
            node_index_by_pending_entry[pending_entry as usize] =
                u32::try_from(node_index).unwrap_or(NO_PENDING_ENTRY);
        }

        let mut nodes = Vec::with_capacity(breadth_first_order.len());
        for &pending_entry in &breadth_first_order {
            let pending_entry = pending_entry as usize;
            let mut node = self.pending_nodes[pending_entry].clone();

            let parent_entry = parent_entries[pending_entry];
            node.parent = if parent_entry == NO_PENDING_ENTRY {
                OptionalNodeIndex::NONE
            } else {
                OptionalNodeIndex::some(NodeIndex::new(
                    node_index_by_pending_entry[parent_entry as usize],
                ))
            };

            let child_entries = children.children_of(pending_entry);
            node.child_count = u32::try_from(child_entries.len()).unwrap_or(u32::MAX);
            node.first_child = child_entries.first().map_or(0, |&first_child| {
                node_index_by_pending_entry[first_child as usize]
            });

            nodes.push(node);
        }
        drop(self.pending_nodes);

        let link_groups = self.hard_links.into_link_groups();
        let hard_link_groups = HardLinkGroups::from_pending_groups(
            &link_groups,
            &node_index_by_pending_entry,
            &mut nodes,
        );

        aggregate_sizes_bottom_up(&mut nodes);

        FileTree::from_parts(nodes, self.names, self.root_path, hard_link_groups)
    }

    fn apply_inaccessible_flags(&mut self) {
        for directory_key in std::mem::take(&mut self.inaccessible_directory_keys) {
            let pending_entry = self.pending_entry_for_key(directory_key.get());
            if let Some(node) = self.pending_nodes.get_mut(pending_entry as usize) {
                node.flags |= NodeFlags::INACCESSIBLE;
            }
        }
    }

    /// The pending entry of each entry's parent. Entries whose parent is unknown are moved into
    /// a synthetic "Unlinked items" directory under the root.
    fn resolve_parent_entries(&mut self) -> Vec<u32> {
        let mut parent_entries = vec![NO_PENDING_ENTRY; self.pending_nodes.len()];
        let mut unlinked_entries = Vec::new();

        for (pending_entry, &parent_key) in self.pending_parent_keys.iter().enumerate().skip(1) {
            let parent_entry = if parent_key == NO_PENDING_ENTRY {
                NO_PENDING_ENTRY
            } else {
                self.pending_entry_for_key(parent_key)
            };

            if parent_entry == NO_PENDING_ENTRY || parent_entry as usize == pending_entry {
                unlinked_entries.push(pending_entry);
            } else {
                parent_entries[pending_entry] = parent_entry;
            }
        }

        if !unlinked_entries.is_empty() {
            let unlinked_directory_entry = u32::try_from(self.pending_nodes.len()).unwrap_or(0);
            let name = self.names.add_name(UNLINKED_ITEMS_DIRECTORY_NAME);
            let mut unlinked_directory =
                Node::new_unlinked(name, ExtensionId::NONE, NodeKind::Directory);
            unlinked_directory.flags = NodeFlags::SYNTHETIC;
            self.pending_nodes.push(unlinked_directory);
            parent_entries.push(0);

            for pending_entry in unlinked_entries {
                parent_entries[pending_entry] = unlinked_directory_entry;
            }
        }

        parent_entries
    }
}

/// Children grouped by parent in one flat array (compressed sparse rows).
struct ChildLists {
    /// `child_entries[row_starts[parent]..row_starts[parent + 1]]` are the children of `parent`.
    row_starts: Vec<u32>,
    child_entries: Vec<u32>,
}

impl ChildLists {
    fn group_by_parent(parent_entries: &[u32]) -> Self {
        let entry_count = parent_entries.len();
        let mut row_starts = vec![0_u32; entry_count + 1];
        for &parent_entry in parent_entries {
            if parent_entry != NO_PENDING_ENTRY {
                row_starts[parent_entry as usize + 1] += 1;
            }
        }
        for row in 1..row_starts.len() {
            row_starts[row] += row_starts[row - 1];
        }

        let mut next_free_position = row_starts.clone();
        let mut child_entries = vec![0_u32; row_starts[entry_count] as usize];
        for (child_entry, &parent_entry) in parent_entries.iter().enumerate() {
            if parent_entry != NO_PENDING_ENTRY {
                let position = &mut next_free_position[parent_entry as usize];
                child_entries[*position as usize] = u32::try_from(child_entry).unwrap_or(0);
                *position += 1;
            }
        }

        Self {
            row_starts,
            child_entries,
        }
    }

    fn children_of(&self, parent_entry: usize) -> &[u32] {
        let start = self.row_starts[parent_entry] as usize;
        let end = self.row_starts[parent_entry + 1] as usize;
        &self.child_entries[start..end]
    }

    /// Entries reachable from the root (entry 0) in breadth-first order. Entries caught in a
    /// parent cycle (only possible with a corrupt MFT) are not reachable and are left out.
    fn breadth_first_order(&self) -> Vec<u32> {
        let mut order = Vec::with_capacity(self.child_entries.len() + 1);
        order.push(0_u32);

        let mut next_to_expand = 0;
        while next_to_expand < order.len() {
            let parent_entry = order[next_to_expand] as usize;
            order.extend_from_slice(self.children_of(parent_entry));
            next_to_expand += 1;
        }

        order
    }
}

/// Adds every node's sizes and counts to its parent. Because nodes are stored in breadth-first
/// order, every child has a larger index than its parent, so one reverse pass is a post-order
/// traversal.
fn aggregate_sizes_bottom_up(nodes: &mut [Node]) {
    for node_index in (1..nodes.len()).rev() {
        let child = &nodes[node_index];
        let Some(parent_index) = child.parent() else {
            continue;
        };
        let contribution = SizeContribution::of(child);
        contribution.add_to(&mut nodes[parent_index.as_usize()]);
    }
}

/// What one node adds to the totals of each of its ancestors.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SizeContribution {
    pub(crate) logical_size: u64,
    pub(crate) allocated_size: u64,
    pub(crate) file_count: u32,
    pub(crate) directory_count: u32,
}

impl SizeContribution {
    pub(crate) fn of(node: &Node) -> Self {
        match node.kind {
            NodeKind::File => Self {
                logical_size: node.counted_logical_size(),
                allocated_size: node.counted_allocated_size(),
                file_count: 1,
                directory_count: 0,
            },
            NodeKind::Directory => Self {
                logical_size: node.logical_size,
                allocated_size: node.allocated_size,
                file_count: node.file_count,
                directory_count: node.directory_count + 1,
            },
        }
    }

    pub(crate) fn add_to(self, node: &mut Node) {
        node.logical_size = node.logical_size.saturating_add(self.logical_size);
        node.allocated_size = node.allocated_size.saturating_add(self.allocated_size);
        node.file_count = node.file_count.saturating_add(self.file_count);
        node.directory_count = node.directory_count.saturating_add(self.directory_count);
    }

    pub(crate) fn subtract_from(self, node: &mut Node) {
        node.logical_size = node.logical_size.saturating_sub(self.logical_size);
        node.allocated_size = node.allocated_size.saturating_sub(self.allocated_size);
        node.file_count = node.file_count.saturating_sub(self.file_count);
        node.directory_count = node.directory_count.saturating_sub(self.directory_count);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    pub(crate) fn file_entry(name: &str, parent: u32, size: u64) -> NewEntry<'_> {
        NewEntry {
            name,
            kind: NodeKind::File,
            parent: Some(DirectoryKey::new(parent)),
            directory_key: None,
            logical_size: size,
            allocated_size: size.next_multiple_of(4096),
            modified_time: None,
            flags: NodeFlags::empty(),
            hard_link_identity: None,
        }
    }

    pub(crate) fn directory_entry(name: &str, parent: u32, key: u32) -> NewEntry<'_> {
        NewEntry {
            name,
            kind: NodeKind::Directory,
            parent: Some(DirectoryKey::new(parent)),
            directory_key: Some(DirectoryKey::new(key)),
            logical_size: 0,
            allocated_size: 0,
            modified_time: None,
            flags: NodeFlags::empty(),
            hard_link_identity: None,
        }
    }

    fn find_child(tree: &FileTree, parent: NodeIndex, name: &str) -> NodeIndex {
        tree.children(parent)
            .find(|&child| tree.name(child) == name)
            .unwrap_or_else(|| panic!("no child named {name}"))
    }

    #[test]
    fn links_children_that_arrive_before_their_parents_and_aggregates_sizes() {
        let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
        builder.add_entry(&file_entry("deep.bin", 2, 300));
        builder.add_entry(&directory_entry("inner", 1, 2));
        builder.add_entry(&file_entry("top.txt", 0, 100));
        builder.add_entry(&directory_entry("outer", 0, 1));
        builder.add_entry(&file_entry("middle.txt", 1, 50));
        builder.add_entry(&directory_entry("empty", 0, 3));

        let tree = builder.finish();
        let root = tree.root();
        let outer = find_child(&tree, root, "outer");
        let inner = find_child(&tree, outer, "inner");

        assert_eq!(tree.node_count(), 7);
        assert_eq!(tree.node(root).unwrap().logical_size(), 450);
        assert_eq!(tree.node(root).unwrap().file_count(), 3);
        assert_eq!(tree.node(root).unwrap().directory_count(), 3);
        assert_eq!(tree.node(outer).unwrap().logical_size(), 350);
        assert_eq!(tree.node(outer).unwrap().directory_count(), 1);
        assert_eq!(tree.node(inner).unwrap().logical_size(), 300);
        assert_eq!(
            tree.node(root).unwrap().allocated_size(),
            4096 * 3,
            "allocated sizes are aggregated too"
        );
        for index in 1..tree.node_count() {
            let node_index = NodeIndex::from_usize(index);
            let parent = tree.node(node_index).unwrap().parent().unwrap();
            assert!(
                parent < node_index,
                "breadth-first order puts parents first"
            );
        }
    }

    #[test]
    fn counts_hard_links_once() {
        let identity = FileIdentity::from_volume_and_file_ids(1, 42);
        let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
        builder.add_entry(&directory_entry("a", 0, 1));
        builder.add_entry(&NewEntry {
            hard_link_identity: Some(identity),
            ..file_entry("link-one.bin", 1, 1000)
        });
        builder.add_entry(&NewEntry {
            hard_link_identity: Some(identity),
            ..file_entry("link-two.bin", 0, 1000)
        });

        let tree = builder.finish();
        let root = tree.root();
        let link_two = find_child(&tree, root, "link-two.bin");

        assert_eq!(tree.node(root).unwrap().logical_size(), 1000);
        assert_eq!(tree.node(root).unwrap().file_count(), 2);
        assert!(
            tree.node(link_two)
                .unwrap()
                .flags()
                .contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE)
        );
        assert_eq!(tree.node(link_two).unwrap().logical_size(), 1000);
    }

    #[test]
    fn moves_entries_with_unknown_parents_into_unlinked_items() {
        let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
        builder.add_entry(&file_entry("orphan.txt", 99, 10));
        builder.add_entry(&NewEntry {
            parent: None,
            ..file_entry("no-parent.txt", 0, 5)
        });

        let tree = builder.finish();
        let unlinked = find_child(&tree, tree.root(), UNLINKED_ITEMS_DIRECTORY_NAME);

        assert!(
            tree.node(unlinked)
                .unwrap()
                .flags()
                .contains(NodeFlags::SYNTHETIC)
        );
        assert_eq!(tree.node(unlinked).unwrap().file_count(), 2);
        assert_eq!(tree.node(tree.root()).unwrap().logical_size(), 15);
    }

    #[test]
    fn drops_entries_caught_in_a_parent_cycle() {
        let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
        builder.add_entry(&directory_entry("loop-a", 2, 1));
        builder.add_entry(&directory_entry("loop-b", 1, 2));
        builder.add_entry(&file_entry("kept.txt", 0, 1));

        let tree = builder.finish();

        assert_eq!(tree.node_count(), 2);
        assert_eq!(tree.node(tree.root()).unwrap().logical_size(), 1);
    }

    #[test]
    fn flags_inaccessible_directories_marked_before_they_were_added() {
        let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
        builder.mark_directory_inaccessible(DirectoryKey::new(1));
        builder.add_entry(&directory_entry("locked", 0, 1));

        let tree = builder.finish();
        let locked = find_child(&tree, tree.root(), "locked");

        assert!(
            tree.node(locked)
                .unwrap()
                .flags()
                .contains(NodeFlags::INACCESSIBLE)
        );
    }
}
