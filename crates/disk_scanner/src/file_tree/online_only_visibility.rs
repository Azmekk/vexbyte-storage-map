use serde::{Deserialize, Serialize};
use specta::Type;

use super::node::{Node, NodeFlags, NodeKind};

/// Whether the tree queries list online-only items: cloud placeholders whose content is not on
/// this disk, and folders that hold nothing else.
///
/// Online-only items take almost no space, so a folder of them would look large while freeing
/// nothing. They are hidden by default and only ever shown on request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum OnlineOnlyVisibility {
    #[default]
    Hidden,
    Shown,
}

impl OnlineOnlyVisibility {
    /// Whether `node` is listed under this setting.
    #[must_use]
    pub fn shows(self, node: &Node) -> bool {
        match self {
            Self::Shown => true,
            Self::Hidden => !node.is_online_only(),
        }
    }
}

impl Node {
    /// True for a cloud placeholder file with no data on this disk, and for a folder whose files
    /// are all online-only (or that has not been populated from the cloud yet). A placeholder
    /// that is partly or fully downloaded has data on disk and is not online-only.
    #[must_use]
    pub fn is_online_only(&self) -> bool {
        let is_placeholder = self.flags.contains(NodeFlags::CLOUD_PLACEHOLDER);
        match self.kind {
            NodeKind::File => is_placeholder && self.allocated_size == 0,
            NodeKind::Directory => {
                is_placeholder
                    || (self.file_count > 0 && self.online_only_file_count == self.file_count)
            }
        }
    }

    /// The number of files in the subtree that this setting lists; for a file itself, 0.
    #[must_use]
    pub fn file_count_under(&self, visibility: OnlineOnlyVisibility) -> u32 {
        match visibility {
            OnlineOnlyVisibility::Shown => self.file_count,
            OnlineOnlyVisibility::Hidden => self.local_file_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::file_tree::FileTree;
    use crate::file_tree::test_trees::{cloud_tree, node_named};

    fn named<'tree>(tree: &'tree FileTree, name: &str) -> &'tree Node {
        tree.node(node_named(tree, name)).unwrap()
    }

    #[test]
    fn a_placeholder_without_data_on_disk_is_online_only() {
        let tree = cloud_tree();

        assert!(named(&tree, "video.mp4").is_online_only());
        assert!(!named(&tree, "partial.iso").is_online_only());
        assert!(
            !named(&tree, "zero.txt").is_online_only(),
            "an empty local file is not a cloud file"
        );
    }

    #[test]
    fn a_folder_is_online_only_when_all_its_files_are() {
        let tree = cloud_tree();

        assert!(named(&tree, "Cloud").is_online_only());
        assert!(!named(&tree, "Mixed").is_online_only());
        assert!(
            !named(&tree, "Empty").is_online_only(),
            "an empty folder is empty, not online-only"
        );
        assert!(
            named(&tree, "Unpopulated").is_online_only(),
            "an unpopulated cloud folder has no local content"
        );
        assert!(!named(&tree, "root").is_online_only());
    }

    #[test]
    fn hidden_lists_local_items_only_and_counts_local_files() {
        let tree = cloud_tree();
        let mixed = named(&tree, "Mixed");
        let root = named(&tree, "root");

        assert!(!OnlineOnlyVisibility::Hidden.shows(named(&tree, "video.mp4")));
        assert!(OnlineOnlyVisibility::Shown.shows(named(&tree, "video.mp4")));
        assert!(OnlineOnlyVisibility::Hidden.shows(named(&tree, "local.pdf")));
        assert_eq!(mixed.file_count_under(OnlineOnlyVisibility::Hidden), 1);
        assert_eq!(mixed.file_count_under(OnlineOnlyVisibility::Shown), 2);
        assert_eq!(root.online_only_file_count(), 6);
        assert_eq!(root.file_count_under(OnlineOnlyVisibility::Hidden), 3);
    }

    #[test]
    fn online_only_files_add_only_their_bytes_on_disk_to_folder_sizes() {
        let tree = cloud_tree();
        let root = named(&tree, "root");

        assert_eq!(named(&tree, "Cloud").logical_size(), 0);
        assert_eq!(named(&tree, "Mixed").logical_size(), 300);
        assert_eq!(
            root.logical_size(),
            300 + 65_536,
            "the partly downloaded file counts what it has on disk"
        );
        assert_eq!(root.allocated_size(), 4096 + 65_536);
        assert_eq!(
            named(&tree, "video.mp4").logical_size(),
            4_000_000,
            "the file itself keeps reporting its real length"
        );
    }
}
