use std::cmp::Ordering;
use std::path::{Component, Path, PathBuf};

use rayon::prelude::*;

use super::lowercase_names::{compare_names_ignoring_case, name_case_flags};
use super::search_index_node::{SearchIndexNode, SearchNodeFlags};
use crate::file_tree::{
    FileTree, NameStorage, Node, NodeFlags, NodeIndex, names_are_equal_ignoring_case,
};
use crate::scan_method::ScanMethod;

/// Everything on one volume, prepared for fast name searches.
///
/// It keeps only what a search needs: a compact node per file or folder (parent, extension,
/// size, time, flags) and the names, stored in node order. Case is ignored by lowercasing names
/// while matching, and results are ordered by path by walking up the tree, so neither a
/// lowercase copy of the names nor a path order table is stored. It is immutable: a refresh
/// builds a new index and replaces the old one as a whole.
#[derive(Debug)]
pub struct VolumeSearchIndex {
    mount_point: PathBuf,
    method: ScanMethod,
    /// The scanned directory the root node stands for; the start of every path.
    root_path: PathBuf,
    file_count: u32,
    directory_count: u32,
    pub(crate) nodes: Vec<SearchIndexNode>,
    /// Name `i` is the name of node `i`.
    pub(crate) names: NameStorage,
}

impl VolumeSearchIndex {
    /// Prepares `tree`, the scan of the volume mounted at `mount_point`, for searching. The tree
    /// is consumed: its names are kept and its nodes are replaced by compact ones, so the scan
    /// arena is freed as soon as the index is ready. Takes well under a second for three
    /// million nodes.
    #[must_use]
    pub fn from_file_tree(mount_point: PathBuf, tree: FileTree, method: ScanMethod) -> Self {
        let totals = tree.totals();
        let (tree_nodes, tree_names, root_path) = tree.into_nodes_names_and_root_path();

        let names = tree_names.into_reordered(tree_nodes.iter().map(|node| node.name));
        let mut nodes: Vec<SearchIndexNode> = tree_nodes
            .par_iter()
            .enumerate()
            .map(|(position, tree_node)| {
                let mut node = SearchIndexNode::from_tree_node(tree_node);
                node.flags |= name_case_flags(names.name_at(position));
                node
            })
            .collect();
        mark_depths_and_searchable_nodes(&tree_nodes, &mut nodes);
        drop(tree_nodes);

        Self {
            mount_point,
            method,
            root_path,
            file_count: totals.file_count,
            directory_count: totals.directory_count,
            nodes,
            names,
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

    /// The number of nodes, including the root and synthetic grouping folders.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// The node's own name; for the root, the scanned path.
    #[must_use]
    pub fn name(&self, node: NodeIndex) -> &str {
        self.name_at(node.as_usize())
    }

    #[must_use]
    pub fn file_count(&self) -> u32 {
        self.file_count
    }

    #[must_use]
    pub fn directory_count(&self) -> u32 {
        self.directory_count
    }

    /// Approximate heap memory of the nodes and names.
    #[must_use]
    pub fn estimated_memory_bytes(&self) -> usize {
        self.nodes.capacity() * size_of::<SearchIndexNode>() + self.names.memory_bytes()
    }

    /// The node at `path`, if `path` lies on this volume and appears in search results. Used
    /// to check that a path the user interface sends back really came from a search.
    ///
    /// Names are compared ignoring case, as NTFS does, so `c:\users\ME\notes.TXT` finds
    /// `C:\Users\me\notes.txt`. When several nodes have that path (which a case-sensitive
    /// folder allows), the first in the arena is returned. Scans the names of all nodes in
    /// parallel: a few milliseconds for three million nodes.
    #[must_use]
    pub fn find_searchable_node_by_path(&self, path: &Path) -> Option<NodeIndex> {
        let path_below_root = path.strip_prefix(&self.root_path).ok()?;
        let wanted_names = path_below_root
            .components()
            .map(|component| match component {
                Component::Normal(name) => name.to_str(),
                _ => None,
            })
            .collect::<Option<Vec<&str>>>()?;
        let (wanted_name, wanted_ancestor_names) = wanted_names.split_last()?;
        let wanted_name_flags = name_case_flags(wanted_name);

        let found = (0..self.nodes.len())
            .into_par_iter()
            .find_first(|&position| {
                let node = &self.nodes[position];
                node.is_searchable()
                    && name_equals_ignoring_case(
                        (self.name_at(position), node.flags),
                        (wanted_name, wanted_name_flags),
                    )
                    && self.has_ancestor_names(position, wanted_ancestor_names)
            })?;
        Some(NodeIndex::from_usize(found))
    }

    /// True when the folders on the path of `position` (below the root, without synthetic
    /// folders) are exactly `wanted_names`, ignoring case.
    fn has_ancestor_names(&self, position: usize, wanted_names: &[&str]) -> bool {
        let mut remaining_names = wanted_names;
        let mut current = self.nodes[position].parent();
        while let Some(ancestor) = current {
            let ancestor_node = &self.nodes[ancestor.as_usize()];
            let Some(ancestor_parent) = ancestor_node.parent() else {
                break;
            };
            if !ancestor_node.flags.contains(SearchNodeFlags::SYNTHETIC) {
                let Some((wanted_name, rest)) = remaining_names.split_last() else {
                    return false;
                };
                if !names_are_equal_ignoring_case(self.name(ancestor), wanted_name) {
                    return false;
                }
                remaining_names = rest;
            }
            current = Some(ancestor_parent);
        }
        remaining_names.is_empty()
    }

    /// Rebuilds the full path from the parent chain and the root path. Synthetic grouping
    /// folders are skipped, so `$MFT` under "NTFS metadata" resolves to `C:\$MFT`. `None` for
    /// synthetic nodes and for nodes whose location on disk is unknown.
    pub(crate) fn full_path(&self, position: usize) -> Option<PathBuf> {
        let node = self.nodes.get(position)?;
        if node.flags.contains(SearchNodeFlags::SYNTHETIC) {
            return None;
        }

        let mut names_from_node_to_root = Vec::new();
        let mut current = position;
        while let Some(parent) = self.nodes[current].parent() {
            let current_node = &self.nodes[current];
            if current_node
                .flags
                .contains(SearchNodeFlags::LOCATION_UNKNOWN)
            {
                return None;
            }
            if !current_node.flags.contains(SearchNodeFlags::SYNTHETIC) {
                names_from_node_to_root.push(self.name_at(current));
            }
            current = parent.as_usize();
        }

        let mut full_path = self.root_path.clone();
        for name in names_from_node_to_root.into_iter().rev() {
            full_path.push(name);
        }
        Some(full_path)
    }

    pub(crate) fn name_at(&self, position: usize) -> &str {
        self.names.name_at(position)
    }

    /// Orders two nodes as a depth-first walk that visits the children of every folder by
    /// lowercase name (then by arena position) would: by full path, component by component,
    /// ignoring case, with a folder before its contents. Only walks up to the nodes' closest
    /// common ancestor, which for results in the same folder is one step.
    pub(crate) fn compare_in_path_order(&self, first: usize, second: usize) -> Ordering {
        if first == second {
            return Ordering::Equal;
        }
        let first_depth = self.nodes[first].depth;
        let second_depth = self.nodes[second].depth;

        let mut first_ancestor = first;
        let mut second_ancestor = second;
        for _ in second_depth..first_depth {
            first_ancestor = self.parent_position(first_ancestor);
        }
        for _ in first_depth..second_depth {
            second_ancestor = self.parent_position(second_ancestor);
        }
        if first_ancestor == second_ancestor {
            return first_depth.cmp(&second_depth);
        }

        loop {
            let first_parent = self.parent_position(first_ancestor);
            let second_parent = self.parent_position(second_ancestor);
            if first_parent == second_parent {
                break;
            }
            first_ancestor = first_parent;
            second_ancestor = second_parent;
        }

        self.compare_names_ignoring_case(first_ancestor, second_ancestor)
            .then(first_ancestor.cmp(&second_ancestor))
    }

    pub(crate) fn compare_names_ignoring_case(&self, first: usize, second: usize) -> Ordering {
        compare_names_ignoring_case(
            (self.name_at(first), self.nodes[first].flags),
            (self.name_at(second), self.nodes[second].flags),
        )
    }

    /// The root is its own parent here, which ends every walk up the tree.
    fn parent_position(&self, position: usize) -> usize {
        self.nodes[position]
            .parent()
            .map_or(position, NodeIndex::as_usize)
    }
}

fn name_equals_ignoring_case(
    (first, first_flags): (&str, SearchNodeFlags),
    (second, second_flags): (&str, SearchNodeFlags),
) -> bool {
    let both_are_ascii = first_flags.contains(SearchNodeFlags::NAME_IS_ASCII)
        && second_flags.contains(SearchNodeFlags::NAME_IS_ASCII);
    if both_are_ascii {
        first.eq_ignore_ascii_case(second)
    } else {
        compare_names_ignoring_case((first, first_flags), (second, second_flags)).is_eq()
    }
}

/// Sets every node's depth and marks the nodes that appear in search results, walking down
/// from the root over the scan tree's children ranges. Nodes that cannot be reached from the
/// root (none in a scan, but removed subtrees in an edited tree) are never searchable.
fn mark_depths_and_searchable_nodes(tree_nodes: &[Node], nodes: &mut [SearchIndexNode]) {
    if tree_nodes.is_empty() {
        return;
    }

    // Each entry is an arena position, its depth and whether its location on disk is known.
    let mut pending: Vec<(usize, u16, bool)> = vec![(0, 0, true)];
    while let Some((position, depth, has_known_location)) = pending.pop() {
        let tree_node = &tree_nodes[position];
        let has_known_location =
            has_known_location && !tree_node.flags.contains(NodeFlags::LOCATION_UNKNOWN);
        let is_searchable = position != 0
            && has_known_location
            && !tree_node
                .flags
                .intersects(NodeFlags::SYNTHETIC | NodeFlags::REMOVED);

        let node = &mut nodes[position];
        node.depth = depth;
        node.flags.set(SearchNodeFlags::SEARCHABLE, is_searchable);

        // A Windows path has at most 32,767 characters, so no real tree comes near u16::MAX.
        let child_depth = depth.saturating_add(1);
        pending.extend(
            tree_node
                .child_index_range()
                .filter(|&child| !tree_nodes[child].is_removed())
                .map(|child| (child, child_depth, has_known_location)),
        );
    }
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
        let mut searchable: Vec<usize> = (0..index.node_count())
            .filter(|&position| index.nodes[position].is_searchable())
            .collect();
        searchable.sort_unstable_by(|&first, &second| index.compare_in_path_order(first, second));
        searchable
            .into_iter()
            .map(|position| index.name_at(position))
            .collect()
    }

    #[test]
    fn orders_nodes_by_path_ignoring_case() {
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
        assert!(
            !index.nodes[0].is_searchable(),
            "the root is never a result"
        );
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
        let mft = index.find_searchable_node_by_path(Path::new(r"C:\$MFT"));
        assert_eq!(index.name(mft.unwrap()), "$MFT");
        for path in [
            r"C:\NTFS metadata\$MFT",
            r"C:\NTFS metadata",
            r"C:\orphan.txt",
            r"C:\Unlinked items\orphan.txt",
            r"C:\Unlinked items",
        ] {
            assert_eq!(index.find_searchable_node_by_path(Path::new(path)), None);
        }
    }

    #[test]
    fn finds_only_searchable_nodes_by_their_path() {
        let mut builder = FileTreeBuilder::new(PathBuf::from(r"C:\"), DirectoryKey::new(0));
        builder.add_entry(&entry("Users", NodeKind::Directory, 0, Some(1), 0));
        builder.add_entry(&entry("notes.txt", NodeKind::File, 1, None, 5));
        builder.add_entry(&entry("Ärger.TXT", NodeKind::File, 1, None, 5));
        let index = index_of(builder);

        let found = index.find_searchable_node_by_path(Path::new(r"C:\Users\notes.txt"));

        assert_eq!(index.name(found.unwrap()), "notes.txt");
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
        assert_eq!(
            index.find_searchable_node_by_path(Path::new(r"C:\notes.txt")),
            None
        );
        assert_eq!(
            index.find_searchable_node_by_path(Path::new(r"C:\Other\notes.txt")),
            None
        );
        assert_eq!(
            index.find_searchable_node_by_path(Path::new(r"C:\Users\notes.txt\notes.txt")),
            None
        );
    }

    #[test]
    fn a_path_in_a_different_case_finds_the_same_node() {
        let mut builder = FileTreeBuilder::new(PathBuf::from(r"C:\"), DirectoryKey::new(0));
        builder.add_entry(&entry("Users", NodeKind::Directory, 0, Some(1), 0));
        builder.add_entry(&entry("notes.txt", NodeKind::File, 1, None, 5));
        builder.add_entry(&entry("Ärger.TXT", NodeKind::File, 1, None, 5));
        let index = index_of(builder);

        let notes = index.find_searchable_node_by_path(Path::new(r"C:\Users\notes.txt"));
        let anger = index.find_searchable_node_by_path(Path::new(r"C:\Users\Ärger.TXT"));

        assert!(notes.is_some() && anger.is_some());
        assert_eq!(
            index.find_searchable_node_by_path(Path::new(r"C:\USERS\NOTES.TXT")),
            notes
        );
        assert_eq!(
            index.find_searchable_node_by_path(Path::new(r"C:\users\ärger.txt")),
            anger
        );
    }

    #[test]
    fn keeps_totals_and_frees_the_scan_arena() {
        let mut builder = FileTreeBuilder::new(PathBuf::from(r"C:\"), DirectoryKey::new(0));
        builder.add_entry(&entry("Users", NodeKind::Directory, 0, Some(1), 0));
        builder.add_entry(&entry("notes.txt", NodeKind::File, 1, None, 5));
        builder.add_entry(&entry("todo.txt", NodeKind::File, 1, None, 5));
        let index = index_of(builder);

        assert_eq!(index.file_count(), 2);
        assert_eq!(index.directory_count(), 1);
        assert_eq!(index.node_count(), 4);
        assert_eq!(index.name(NodeIndex::new(0)), r"C:\");
        assert!(index.estimated_memory_bytes() < 4 * 64 + 256);
    }
}
