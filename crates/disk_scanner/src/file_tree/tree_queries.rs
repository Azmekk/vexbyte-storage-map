//! Small views of the tree that the user interface asks for: one page of children, the
//! details of one node, the breadcrumb path, the largest files and the file type breakdown.

use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::path::PathBuf;

use file_categories::FileCategory;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::name_storage::NameStorage;
use super::node::{Node, NodeIndex, NodeKind};
use super::node_flag_summary::NodeFlagSummary;
use super::tree_arena::FileTree;
use super::tree_query_error::TreeQueryError;

/// How the children of a directory are ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ChildSortOrder {
    LogicalSizeDescending,
    LogicalSizeAscending,
    AllocatedSizeDescending,
    AllocatedSizeAscending,
    NameAscending,
    NameDescending,
    FileCountDescending,
    FileCountAscending,
    ModifiedNewestFirst,
    ModifiedOldestFirst,
}

/// One row of the tree table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChildSummary {
    pub node: NodeIndex,
    pub name: String,
    pub kind: NodeKind,
    pub flags: NodeFlagSummary,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub file_count: u32,
    pub directory_count: u32,
    pub modified_at_unix_milliseconds: Option<u64>,
    /// The counted logical size as a fraction (0 to 1) of the parent's logical size.
    pub share_of_parent: f64,
    pub has_children: bool,
    pub extension: Option<String>,
    pub category: Option<FileCategory>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChildrenPage {
    pub parent: NodeIndex,
    pub total_child_count: u32,
    pub offset: u32,
    pub children: Vec<ChildSummary>,
}

/// Everything the inspector panel shows about one node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NodeDetails {
    pub node: NodeIndex,
    pub parent: Option<NodeIndex>,
    pub name: String,
    /// `None` for synthetic grouping nodes, which do not exist on disk.
    pub full_path: Option<PathBuf>,
    pub kind: NodeKind,
    pub flags: NodeFlagSummary,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub file_count: u32,
    pub directory_count: u32,
    pub child_count: u32,
    pub modified_at_unix_milliseconds: Option<u64>,
    pub share_of_parent: f64,
    pub extension: Option<String>,
    pub category: Option<FileCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BreadcrumbItem {
    pub node: NodeIndex,
    pub name: String,
}

/// One entry of the "Largest files" list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileSummary {
    pub node: NodeIndex,
    pub name: String,
    pub full_path: Option<PathBuf>,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub modified_at_unix_milliseconds: Option<u64>,
    pub extension: Option<String>,
    pub category: Option<FileCategory>,
    pub flags: NodeFlagSummary,
}

/// How much of a subtree one file category takes. `category: None` collects every file whose
/// extension belongs to no category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileCategoryShare {
    pub category: Option<FileCategory>,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub file_count: u32,
    pub share_of_total: f64,
}

/// How much of a subtree one extension takes. `extension: None` collects files without one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileExtensionShare {
    pub extension: Option<String>,
    pub category: Option<FileCategory>,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub file_count: u32,
    pub share_of_total: f64,
}

/// File types under a node, both sorted by logical size, largest first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileTypeBreakdown {
    pub total_logical_size: u64,
    pub categories: Vec<FileCategoryShare>,
    pub extensions: Vec<FileExtensionShare>,
}

impl FileTree {
    /// One page of a directory's children in the requested order. Files have no children and
    /// return an empty page.
    ///
    /// # Errors
    ///
    /// Fails when `parent` does not exist or was removed.
    pub fn children_page(
        &self,
        parent: NodeIndex,
        sort_order: ChildSortOrder,
        offset: u32,
        limit: u32,
    ) -> Result<ChildrenPage, TreeQueryError> {
        let parent_node = self.live_node(parent)?;
        let mut children: Vec<NodeIndex> = self.children(parent).collect();
        let total_child_count = u32::try_from(children.len()).unwrap_or(u32::MAX);

        children.sort_unstable_by(|&first, &second| {
            self.compare_children(first, second, sort_order)
                .then(first.cmp(&second))
        });

        let page = children
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|child| self.child_summary(child, parent_node))
            .collect();

        Ok(ChildrenPage {
            parent,
            total_child_count,
            offset,
            children: page,
        })
    }

    fn compare_children(
        &self,
        first: NodeIndex,
        second: NodeIndex,
        sort_order: ChildSortOrder,
    ) -> Ordering {
        let first_node = &self.nodes[first.as_usize()];
        let second_node = &self.nodes[second.as_usize()];

        match sort_order {
            ChildSortOrder::LogicalSizeDescending => {
                second_node.logical_size.cmp(&first_node.logical_size)
            }
            ChildSortOrder::LogicalSizeAscending => {
                first_node.logical_size.cmp(&second_node.logical_size)
            }
            ChildSortOrder::AllocatedSizeDescending => {
                second_node.allocated_size.cmp(&first_node.allocated_size)
            }
            ChildSortOrder::AllocatedSizeAscending => {
                first_node.allocated_size.cmp(&second_node.allocated_size)
            }
            ChildSortOrder::NameAscending => {
                compare_names_ignoring_case(self.name(first), self.name(second))
            }
            ChildSortOrder::NameDescending => {
                compare_names_ignoring_case(self.name(second), self.name(first))
            }
            ChildSortOrder::FileCountDescending => {
                second_node.file_count.cmp(&first_node.file_count)
            }
            ChildSortOrder::FileCountAscending => {
                first_node.file_count.cmp(&second_node.file_count)
            }
            ChildSortOrder::ModifiedNewestFirst => {
                second_node.modified_time.cmp(&first_node.modified_time)
            }
            ChildSortOrder::ModifiedOldestFirst => {
                first_node.modified_time.cmp(&second_node.modified_time)
            }
        }
    }

    fn child_summary(&self, child: NodeIndex, parent_node: &Node) -> ChildSummary {
        let node = &self.nodes[child.as_usize()];

        ChildSummary {
            node: child,
            name: self.name(child).to_owned(),
            kind: node.kind,
            flags: node.flags.into(),
            logical_size: node.logical_size,
            allocated_size: node.allocated_size,
            file_count: node.file_count,
            directory_count: node.directory_count,
            modified_at_unix_milliseconds: node.modified_time.unix_milliseconds(),
            share_of_parent: share_of(node.counted_logical_size(), parent_node.logical_size),
            has_children: self.children(child).next().is_some(),
            extension: self.extension(child).map(str::to_owned),
            category: self.category(child),
        }
    }

    /// # Errors
    ///
    /// Fails when `node` does not exist or was removed.
    pub fn node_details(&self, node: NodeIndex) -> Result<NodeDetails, TreeQueryError> {
        let tree_node = self.live_node(node)?;
        let parent_logical_size = tree_node
            .parent()
            .and_then(|parent| self.node(parent))
            .map_or(tree_node.logical_size, |parent| parent.logical_size);

        Ok(NodeDetails {
            node,
            parent: tree_node.parent(),
            name: self.name(node).to_owned(),
            full_path: self.full_path(node),
            kind: tree_node.kind,
            flags: tree_node.flags.into(),
            logical_size: tree_node.logical_size,
            allocated_size: tree_node.allocated_size,
            file_count: tree_node.file_count,
            directory_count: tree_node.directory_count,
            child_count: u32::try_from(self.children(node).count()).unwrap_or(u32::MAX),
            modified_at_unix_milliseconds: tree_node.modified_time.unix_milliseconds(),
            share_of_parent: share_of(tree_node.counted_logical_size(), parent_logical_size),
            extension: self.extension(node).map(str::to_owned),
            category: self.category(node),
        })
    }

    /// The breadcrumb trail from the root down to `node`, both included.
    ///
    /// # Errors
    ///
    /// Fails when `node` does not exist or was removed.
    pub fn path_to_root(&self, node: NodeIndex) -> Result<Vec<BreadcrumbItem>, TreeQueryError> {
        self.live_node(node)?;

        let mut breadcrumbs = Vec::new();
        let mut current = Some(node);
        while let Some(current_index) = current {
            breadcrumbs.push(BreadcrumbItem {
                node: current_index,
                name: self.name(current_index).to_owned(),
            });
            current = self.node(current_index).and_then(Node::parent);
        }
        breadcrumbs.reverse();

        Ok(breadcrumbs)
    }

    /// The `limit` largest files under `node` by logical size, optionally of one category.
    ///
    /// # Errors
    ///
    /// Fails when `node` does not exist or was removed.
    pub fn largest_files(
        &self,
        node: NodeIndex,
        category_filter: Option<FileCategory>,
        limit: u32,
    ) -> Result<Vec<FileSummary>, TreeQueryError> {
        self.live_node(node)?;
        let limit = limit as usize;
        if limit == 0 {
            return Ok(Vec::new());
        }

        let category_by_extension = category_lookup_table(&self.names);
        let mut smallest_kept_first: BinaryHeap<Reverse<(u64, NodeIndex)>> =
            BinaryHeap::with_capacity(limit + 1);

        self.for_each_live_descendant(node, |file_index, file_node| {
            if file_node.kind != NodeKind::File {
                return;
            }
            if let Some(wanted_category) = category_filter
                && category_by_extension[file_node.extension.as_usize()] != Some(wanted_category)
            {
                return;
            }

            smallest_kept_first.push(Reverse((file_node.logical_size, file_index)));
            if smallest_kept_first.len() > limit {
                smallest_kept_first.pop();
            }
        });

        let mut largest_files: Vec<(u64, NodeIndex)> = smallest_kept_first
            .into_iter()
            .map(|Reverse(entry)| entry)
            .collect();
        largest_files.sort_unstable_by(|first, second| second.cmp(first));

        Ok(largest_files
            .into_iter()
            .map(|(_, file_index)| self.file_summary(file_index))
            .collect())
    }

    pub(crate) fn file_summary(&self, file_index: NodeIndex) -> FileSummary {
        let node = &self.nodes[file_index.as_usize()];

        FileSummary {
            node: file_index,
            name: self.name(file_index).to_owned(),
            full_path: self.full_path(file_index),
            logical_size: node.logical_size,
            allocated_size: node.allocated_size,
            modified_at_unix_milliseconds: node.modified_time.unix_milliseconds(),
            extension: self.extension(file_index).map(str::to_owned),
            category: self.category(file_index),
            flags: node.flags.into(),
        }
    }

    /// Sizes per file category and per extension under `node`, with hard links counted once.
    ///
    /// # Errors
    ///
    /// Fails when `node` does not exist or was removed.
    pub fn file_type_breakdown(
        &self,
        node: NodeIndex,
    ) -> Result<FileTypeBreakdown, TreeQueryError> {
        self.live_node(node)?;

        let mut totals_by_extension = vec![TypeTotals::default(); self.names.extension_id_count()];
        self.for_each_live_descendant(node, |_, file_node| {
            if file_node.kind == NodeKind::File {
                let totals = &mut totals_by_extension[file_node.extension.as_usize()];
                totals.logical_size += file_node.counted_logical_size();
                totals.allocated_size += file_node.counted_allocated_size();
                totals.file_count += 1;
            }
        });

        let total_logical_size: u64 = totals_by_extension
            .iter()
            .map(|totals| totals.logical_size)
            .sum();

        let mut categories: Vec<FileCategoryShare> = Vec::new();
        let mut extensions: Vec<FileExtensionShare> = Vec::new();
        for (extension_index, totals) in totals_by_extension.iter().enumerate() {
            if totals.file_count == 0 {
                continue;
            }
            let extension_id = NameStorage::extension_id_from_usize(extension_index);
            let category = self.names.extension_category(extension_id);

            extensions.push(FileExtensionShare {
                extension: self.names.extension(extension_id).map(str::to_owned),
                category,
                logical_size: totals.logical_size,
                allocated_size: totals.allocated_size,
                file_count: totals.file_count,
                share_of_total: share_of(totals.logical_size, total_logical_size),
            });

            if let Some(category_share) = categories
                .iter_mut()
                .find(|share| share.category == category)
            {
                category_share.logical_size += totals.logical_size;
                category_share.allocated_size += totals.allocated_size;
                category_share.file_count += totals.file_count;
            } else {
                categories.push(FileCategoryShare {
                    category,
                    logical_size: totals.logical_size,
                    allocated_size: totals.allocated_size,
                    file_count: totals.file_count,
                    share_of_total: 0.0,
                });
            }
        }

        for category_share in &mut categories {
            category_share.share_of_total =
                share_of(category_share.logical_size, total_logical_size);
        }
        categories.sort_by_key(|share| Reverse(share.logical_size));
        extensions.sort_by_key(|share| Reverse(share.logical_size));

        Ok(FileTypeBreakdown {
            total_logical_size,
            categories,
            extensions,
        })
    }

    /// Visits every live node below `node` (not `node` itself), depth first.
    pub(crate) fn for_each_live_descendant(
        &self,
        node: NodeIndex,
        mut visit: impl FnMut(NodeIndex, &Node),
    ) {
        let mut directories_to_visit = vec![node];
        while let Some(directory) = directories_to_visit.pop() {
            for child in self.children(directory) {
                let child_node = &self.nodes[child.as_usize()];
                visit(child, child_node);
                if child_node.child_count > 0 {
                    directories_to_visit.push(child);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TypeTotals {
    logical_size: u64,
    allocated_size: u64,
    file_count: u32,
}

fn category_lookup_table(names: &NameStorage) -> Vec<Option<FileCategory>> {
    (0..names.extension_id_count())
        .map(|index| names.extension_category(NameStorage::extension_id_from_usize(index)))
        .collect()
}

/// `part / whole` as a fraction from 0 to 1; 0 when `whole` is 0.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn share_of(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        (part as f64 / whole as f64).clamp(0.0, 1.0)
    }
}

/// Orders names like Explorer does for plain ASCII: case-insensitively.
fn compare_names_ignoring_case(first: &str, second: &str) -> Ordering {
    let first_folded = first.bytes().map(|byte| byte.to_ascii_lowercase());
    let second_folded = second.bytes().map(|byte| byte.to_ascii_lowercase());
    first_folded.cmp(second_folded)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;

    use super::*;
    use crate::file_identity::FileIdentity;
    use crate::file_tree::test_trees::{child_named, entry, sample_tree};
    use crate::file_tree::{DirectoryKey, FileTreeBuilder, NewEntry};

    #[test]
    fn sorts_and_pages_children() {
        let tree = sample_tree();

        let by_size = tree
            .children_page(tree.root(), ChildSortOrder::LogicalSizeDescending, 0, 10)
            .unwrap();
        let names: Vec<_> = by_size
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect();
        assert_eq!(names, ["Photos", "setup.exe", "notes.txt", "Makefile"]);
        assert_eq!(by_size.total_child_count, 4);
        assert!(by_size.children[0].has_children);
        assert!((by_size.children[0].share_of_parent - 1600.0 / 2360.0).abs() < 1e-9);

        let second_page = tree
            .children_page(tree.root(), ChildSortOrder::NameAscending, 1, 2)
            .unwrap();
        let names: Vec<_> = second_page
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect();
        assert_eq!(names, ["notes.txt", "Photos"]);
    }

    #[test]
    fn reports_details_and_breadcrumbs() {
        let tree = sample_tree();
        let photos = child_named(&tree, tree.root(), "Photos");
        let raw = child_named(&tree, photos, "raw");
        let shot = child_named(&tree, raw, "shot.cr3");

        let details = tree.node_details(shot).unwrap();
        assert_eq!(
            details.full_path,
            Some(
                PathBuf::from("root")
                    .join("Photos")
                    .join("raw")
                    .join("shot.cr3")
            )
        );
        assert_eq!(details.extension.as_deref(), Some("cr3"));
        assert_eq!(details.category, Some(FileCategory::Images));
        assert!((details.share_of_parent - 1.0).abs() < 1e-9);

        let breadcrumbs = tree.path_to_root(shot).unwrap();
        let names: Vec<_> = breadcrumbs.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, ["root", "Photos", "raw", "shot.cr3"]);
    }

    #[test]
    fn finds_largest_files_with_and_without_category() {
        let tree = sample_tree();

        let largest = tree.largest_files(tree.root(), None, 3).unwrap();
        let names: Vec<_> = largest.iter().map(|file| file.name.as_str()).collect();
        assert_eq!(names, ["shot.cr3", "setup.exe", "beach.JPG"]);

        let images = tree
            .largest_files(tree.root(), Some(FileCategory::Images), 10)
            .unwrap();
        let names: Vec<_> = images.iter().map(|file| file.name.as_str()).collect();
        assert_eq!(names, ["shot.cr3", "beach.JPG", "city.png"]);
    }

    #[test]
    fn breaks_down_file_types_by_category_and_extension() {
        let tree = sample_tree();

        let breakdown = tree.file_type_breakdown(tree.root()).unwrap();

        assert_eq!(breakdown.total_logical_size, 2360);
        assert_eq!(breakdown.categories[0].category, Some(FileCategory::Images));
        assert_eq!(breakdown.categories[0].logical_size, 1600);
        assert_eq!(breakdown.categories[0].file_count, 3);
        let uncategorized = breakdown
            .categories
            .iter()
            .find(|share| share.category.is_none())
            .unwrap();
        assert_eq!(uncategorized.logical_size, 10);
        assert_eq!(breakdown.extensions[0].extension.as_deref(), Some("cr3"));
        let jpg = breakdown
            .extensions
            .iter()
            .find(|share| share.extension.as_deref() == Some("jpg"))
            .unwrap();
        assert_eq!(jpg.logical_size, 400);
    }

    #[test]
    fn breakdown_counts_hard_links_once() {
        let identity = FileIdentity::from_volume_and_file_ids(1, 9);
        let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
        for name in ["a.mp4", "b.mp4"] {
            builder.add_entry(&NewEntry {
                hard_link_identity: Some(identity),
                ..entry(name, NodeKind::File, 0, None, 1000)
            });
        }
        let tree = builder.finish();

        let breakdown = tree.file_type_breakdown(tree.root()).unwrap();

        assert_eq!(breakdown.total_logical_size, 1000);
        assert_eq!(breakdown.categories[0].file_count, 2);
    }

    #[test]
    fn rejects_unknown_nodes() {
        let tree = sample_tree();
        let missing = NodeIndex::new(999);

        assert_eq!(
            tree.node_details(missing),
            Err(TreeQueryError::NodeNotFound { node: missing })
        );
    }
}
