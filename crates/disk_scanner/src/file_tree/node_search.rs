use std::path::PathBuf;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::node::{NodeFlags, NodeIndex, NodeKind};
use super::tree_arena::FileTree;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub node: NodeIndex,
    pub name: String,
    pub kind: NodeKind,
    pub full_path: Option<PathBuf>,
    pub logical_size: u64,
}

/// Name comparisons work on chunks of nodes in parallel; a chunk this size keeps the per-task
/// overhead negligible.
const SEARCH_CHUNK_SIZE: usize = 16 * 1024;

impl FileTree {
    /// Finds nodes whose name contains `query`, ignoring case, largest first.
    ///
    /// Scans every name in parallel, so it takes tens of milliseconds for a million nodes;
    /// call it from a background thread.
    #[must_use]
    pub fn search_nodes(&self, query: &str, limit: u32) -> Vec<SearchResult> {
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() || limit == 0 {
            return Vec::new();
        }
        let matcher = NameMatcher::new(trimmed_query);

        let mut matching_nodes: Vec<(u64, NodeIndex)> = self
            .nodes
            .par_chunks(SEARCH_CHUNK_SIZE)
            .enumerate()
            .flat_map_iter(|(chunk_number, chunk)| {
                let chunk_start = chunk_number * SEARCH_CHUNK_SIZE;
                chunk
                    .iter()
                    .enumerate()
                    .filter(|(position, node)| {
                        let is_searchable = *position + chunk_start != 0
                            && !node
                                .flags
                                .intersects(NodeFlags::REMOVED | NodeFlags::SYNTHETIC);
                        is_searchable && matcher.matches(self.names.name(node.name))
                    })
                    .map(move |(position, node)| {
                        (
                            node.logical_size,
                            NodeIndex::from_usize(chunk_start + position),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        matching_nodes.sort_unstable_by(|first, second| second.cmp(first));
        matching_nodes.truncate(limit as usize);

        matching_nodes
            .into_iter()
            .map(|(logical_size, node)| SearchResult {
                node,
                name: self.name(node).to_owned(),
                kind: self.nodes[node.as_usize()].kind,
                full_path: self.full_path(node),
                logical_size,
            })
            .collect()
    }
}

/// Case-insensitive substring matching with a fast path for ASCII queries, which compares
/// bytes without allocating.
enum NameMatcher {
    Ascii { lowercase_query: Vec<u8> },
    Unicode { lowercase_query: String },
}

impl NameMatcher {
    fn new(query: &str) -> Self {
        if query.is_ascii() {
            Self::Ascii {
                lowercase_query: query.to_ascii_lowercase().into_bytes(),
            }
        } else {
            Self::Unicode {
                lowercase_query: query.to_lowercase(),
            }
        }
    }

    fn matches(&self, name: &str) -> bool {
        match self {
            Self::Ascii { lowercase_query } => name
                .as_bytes()
                .windows(lowercase_query.len())
                .any(|window| window.eq_ignore_ascii_case(lowercase_query)),
            Self::Unicode { lowercase_query } => name.to_lowercase().contains(lowercase_query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_tree::test_trees::sample_tree;

    #[test]
    fn finds_names_case_insensitively_largest_first() {
        let tree = sample_tree();

        let results = tree.search_nodes("P", 10);
        let names: Vec<_> = results.iter().map(|result| result.name.as_str()).collect();

        assert_eq!(names, ["Photos", "setup.exe", "beach.JPG", "city.png"]);
    }

    #[test]
    fn limits_results_and_ignores_empty_queries() {
        let tree = sample_tree();

        assert_eq!(tree.search_nodes("e", 2).len(), 2);
        assert_eq!(tree.search_nodes("   ", 10), []);
        assert_eq!(tree.search_nodes("does-not-exist", 10), []);
    }

    #[test]
    fn matches_non_ascii_queries() {
        let matcher = NameMatcher::new("ÜBER");

        assert!(matcher.matches("Überweisung.pdf"));
        assert!(!matcher.matches("Uberweisung.pdf"));
    }
}
