//! The in-memory result of a scan: a compact arena of nodes with interned names, plus the
//! queries the user interface needs. The full tree never leaves Rust.

mod file_tree_builder;
mod hard_link_groups;
mod hard_link_tracker;
mod name_storage;
mod node;
mod node_flag_summary;
mod node_removal;
mod node_search;
mod subtree_extraction;
#[cfg(test)]
mod test_trees;
mod tree_arena;
mod tree_queries;
mod tree_query_error;
mod visualization_subtree;

pub use file_tree_builder::{DirectoryKey, FileTreeBuilder, NewEntry};
pub use node::{Node, NodeFlags, NodeIndex, NodeKind};
pub use node_flag_summary::NodeFlagSummary;
pub use node_removal::RemovalSummary;
pub use node_search::SearchResult;
pub use tree_arena::{FileTree, TreeTotals};
pub use tree_queries::{
    BreadcrumbItem, ChildSortOrder, ChildSummary, ChildrenPage, FileCategoryShare,
    FileExtensionShare, FileSummary, FileTypeBreakdown, NodeDetails,
};
pub use tree_query_error::{TreeQueryError, TreeRemovalError};
pub use visualization_subtree::{
    VISUALIZATION_NODE_LIMIT, VisualizationNode, VisualizationNodeKind,
};
