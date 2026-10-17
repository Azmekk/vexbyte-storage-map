use serde::{Deserialize, Serialize};
use specta::Type;

use super::node::NodeFlags;

/// `NodeFlags` as named booleans, which is easier to use in the user interface than a bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct NodeFlagSummary {
    pub hard_link_counted_elsewhere: bool,
    pub symlink: bool,
    pub junction: bool,
    pub cloud_placeholder: bool,
    pub compressed: bool,
    pub sparse: bool,
    pub hidden: bool,
    pub system: bool,
    pub inaccessible: bool,
    pub synthetic: bool,
}

impl From<NodeFlags> for NodeFlagSummary {
    fn from(flags: NodeFlags) -> Self {
        Self {
            hard_link_counted_elsewhere: flags.contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE),
            symlink: flags.contains(NodeFlags::SYMLINK),
            junction: flags.contains(NodeFlags::JUNCTION),
            cloud_placeholder: flags.contains(NodeFlags::CLOUD_PLACEHOLDER),
            compressed: flags.contains(NodeFlags::COMPRESSED),
            sparse: flags.contains(NodeFlags::SPARSE),
            hidden: flags.contains(NodeFlags::HIDDEN),
            system: flags.contains(NodeFlags::SYSTEM),
            inaccessible: flags.contains(NodeFlags::INACCESSIBLE),
            synthetic: flags.contains(NodeFlags::SYNTHETIC),
        }
    }
}
