use std::path::{Path, PathBuf};

use super::lowercase_name_table::LowercaseNameTable;
use crate::file_tree::{FileTree, NodeFlags, NodeIndex};
use crate::scan_method::ScanMethod;

/// Marks a node that never appears in search results: the root, synthetic grouping folders and
/// entries whose location on disk is unknown.
pub(crate) const NOT_SEARCHABLE: u32 = u32::MAX;

/// Everything on one volume, prepared for fast name searches.
///
/// It owns the volume's `FileTree` and adds a lowercase copy of every name plus the position
/// of every node in path order, so a query neither lowercases names nor builds paths to sort.
/// It is immutable: a refresh builds a new index and replaces the old one as a whole.
#[derive(Debug)]
pub struct VolumeSearchIndex {
    mount_point: PathBuf,
    method: ScanMethod,
    pub(crate) tree: FileTree,
    pub(crate) lowercase_names: LowercaseNameTable,
    /// The position of each node when all searchable nodes are sorted by full path
    /// (component by component, ignoring case), or `NOT_SEARCHABLE`.
    pub(crate) path_ranks: Vec<u32>,
}

impl VolumeSearchIndex {
    /// Prepares `tree`, the scan of the volume mounted at `mount_point`, for searching. Takes
    /// about a second for three million nodes.
    #[must_use]
    pub fn from_file_tree(mount_point: PathBuf, tree: FileTree, method: ScanMethod) -> Self {
        let lowercase_names = LowercaseNameTable::of_tree(&tree);
        let path_ranks = rank_nodes_in_path_order(&tree, &lowercase_names);

        Self {
            mount_point,
            method,
            tree,
            lowercase_names,
            path_ranks,
        }
    }

    /// Where the volume is mounted, for example `C:\`.
    #[must_use]
    pub fn mount_point(&self) -> &Path {
        &self.mount_point
    }

    /// How the volume was read.
    #[must_use]
    pub fn method(&self) -> ScanMethod {
        self.method
    }

    #[must_use]
    pub fn tree(&self) -> &FileTree {
        &self.tree
    }

    #[must_use]
    pub fn file_count(&self) -> u32 {
        self.tree.totals().file_count
    }

    #[must_use]
    pub fn directory_count(&self) -> u32 {
        self.tree.totals().directory_count
    }

    /// Approximate heap memory of the tree plus the search tables.
    #[must_use]
    pub fn estimated_memory_bytes(&self) -> usize {
        self.tree.estimated_memory_bytes()
            + self.lowercase_names.memory_bytes()
            + self.path_ranks.capacity() * size_of::<u32>()
    }

    /// The node at exactly `path`, if `path` lies on this volume and appears in search results.
    /// Used to check that a path the user interface sends back really came from a search.
    #[must_use]
    pub fn find_searchable_node_by_path(&self, path: &Path) -> Option<NodeIndex> {
        let path_below_root = path.strip_prefix(self.tree.root_path()).ok()?;
        let node = self.tree.find_descendant_by_names(
            path_below_root
                .components()
                .map(std::path::Component::as_os_str),
        )?;

        let is_searchable = self.path_ranks.get(node.as_usize()) != Some(&NOT_SEARCHABLE);
        let has_exactly_this_path = self.tree.full_path(node).as_deref() == Some(path);
        (is_searchable && has_exactly_this_path).then_some(node)
    }

    pub(crate) fn is_searchable(&self, position: usize) -> bool {
        self.path_ranks[position] != NOT_SEARCHABLE
    }
}

/// Numbers the nodes in a depth-first walk that visits the children of every folder in
/// lowercase name order, which is the order of their full paths. Sorting results by path is
/// then a comparison of two integers.
fn rank_nodes_in_path_order(tree: &FileTree, lowercase_names: &LowercaseNameTable) -> Vec<u32> {
    let nodes = &tree.nodes;
    let mut path_ranks = vec![NOT_SEARCHABLE; nodes.len()];
    if nodes.is_empty() {
        return path_ranks;
    }

    let mut next_rank: u32 = 0;
    // Each entry is an arena position and whether the node has a known location on disk.
    let mut pending: Vec<(usize, bool)> = vec![(0, true)];

    while let Some((position, has_known_location)) = pending.pop() {
        let node = &nodes[position];
        let has_known_location =
            has_known_location && !node.flags.contains(NodeFlags::LOCATION_UNKNOWN);
        let is_searchable = position != 0
            && has_known_location
            && !node
                .flags
                .intersects(NodeFlags::SYNTHETIC | NodeFlags::REMOVED);
        if is_searchable {
            path_ranks[position] = next_rank;
            next_rank = next_rank.saturating_add(1);
        }

        let first_pending_child = pending.len();
        pending.extend(
            node.child_index_range()
                .filter(|&child| !nodes[child].is_removed())
                .map(|child| (child, has_known_location)),
        );
        // Pushed in descending order, so the smallest name is taken next.
        pending[first_pending_child..].sort_unstable_by(|(first, _), (second, _)| {
            lowercase_names
                .name(*second)
                .cmp(lowercase_names.name(*first))
                .then(second.cmp(first))
        });
    }

    path_ranks
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::file_tree::test_trees::entry;
    use crate::file_tree::{DirectoryKey, FileTreeBuilder, NewEntry, NodeKind};

    fn index_of(builder: FileTreeBuilder) -> VolumeSearchIndex {
        VolumeSearchIndex::from_file_tree(
            PathBuf::from(r"C:\"),
            builder.finish(),
            ScanMethod::FolderWalk,
        )
    }

    fn names_in_path_order(index: &VolumeSearchIndex) -> Vec<&str> {
        let mut ranked: Vec<(u32, usize)> = index
            .path_ranks
            .iter()
            .enumerate()
            .filter(|(_, rank)| **rank != NOT_SEARCHABLE)
            .map(|(position, rank)| (*rank, position))
            .collect();
        ranked.sort_unstable();
        ranked
            .into_iter()
            .map(|(_, position)| index.tree.name(NodeIndex::from_usize(position)))
            .collect()
    }

    #[test]
    fn ranks_nodes_by_path_ignoring_case() {
        let mut builder = FileTreeBuilder::new(PathBuf::from(r"C:\"), DirectoryKey::new(0));
        builder.add_entry(&entry("zeta", NodeKind::Directory, 0, Some(1), 0));
        builder.add_entry(&entry("Alpha", NodeKind::Directory, 0, Some(2), 0));
        builder.add_entry(&entry("b.txt", NodeKind::File, 2, None, 1));
        builder.add_entry(&entry("A.txt", NodeKind::File, 2, None, 1));
        builder.add_entry(&entry("inner.txt", NodeKind::File, 1, None, 1));
        builder.add_entry(&entry("middle.txt", NodeKind::File, 0, None, 1));

        let index = index_of(builder);

        assert_eq!(
            names_in_path_order(&index),
            ["Alpha", "A.txt", "b.txt", "middle.txt", "zeta", "inner.txt"]
        );
        assert!(!index.is_searchable(0), "the root is never a result");
    }

    #[test]
    fn synthetic_folders_and_unlinked_items_are_not_searchable() {
        let mut builder = FileTreeBuilder::new(PathBuf::from(r"C:\"), DirectoryKey::new(0));
        builder.add_synthetic_directory(
            "NTFS metadata",
            Some(DirectoryKey::new(0)),
            DirectoryKey::new(1),
        );
        builder.add_entry(&entry("$MFT", NodeKind::File, 1, None, 10));
        builder.add_entry(&NewEntry {
            parent: None,
            ..entry("orphan.txt", NodeKind::File, 0, None, 5)
        });

        let index = index_of(builder);

        assert_eq!(names_in_path_order(&index), ["$MFT"]);
    }

    #[test]
    fn finds_only_searchable_nodes_by_their_exact_path() {
        let mut builder = FileTreeBuilder::new(PathBuf::from(r"C:\"), DirectoryKey::new(0));
        builder.add_entry(&entry("Users", NodeKind::Directory, 0, Some(1), 0));
        builder.add_entry(&entry("notes.txt", NodeKind::File, 1, None, 5));
        let index = index_of(builder);

        let found = index.find_searchable_node_by_path(Path::new(r"C:\Users\notes.txt"));

        assert_eq!(index.tree.name(found.unwrap()), "notes.txt");
        assert_eq!(index.find_searchable_node_by_path(Path::new(r"C:\")), None);
        assert_eq!(
            index.find_searchable_node_by_path(Path::new(r"C:\Users\missing.txt")),
            None
        );
        assert_eq!(
            index.find_searchable_node_by_path(Path::new(r"D:\Users\notes.txt")),
            None
        );
        assert_eq!(
            index.find_searchable_node_by_path(Path::new(r"C:\Users\..\Users\notes.txt")),
            None
        );
    }
}
