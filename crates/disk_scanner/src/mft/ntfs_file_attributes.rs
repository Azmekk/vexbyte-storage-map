//! Maps the file attribute bits NTFS stores in `$STANDARD_INFORMATION` (the same values Win32
//! reports as `FILE_ATTRIBUTE_*`) and the reparse tag to node flags.

use std::time::{Duration, SystemTime};

use crate::file_tree::NodeFlags;

const HIDDEN: u32 = 0x0000_0002;
const SYSTEM: u32 = 0x0000_0004;
const SPARSE_FILE: u32 = 0x0000_0200;
const COMPRESSED: u32 = 0x0000_0800;
const OFFLINE: u32 = 0x0000_1000;
const RECALL_ON_OPEN: u32 = 0x0004_0000;
const RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;

const REPARSE_TAG_SYMLINK: u32 = 0xA000_000C;
const REPARSE_TAG_LX_SYMLINK: u32 = 0xA000_001D;
/// Name surrogate tags (junctions, mount points, symlinks) stand for another location.
const REPARSE_TAG_NAME_SURROGATE_BIT: u32 = 0x2000_0000;

/// Seconds between 1601-01-01 (the NTFS epoch) and 1970-01-01.
const SECONDS_FROM_NTFS_EPOCH_TO_UNIX_EPOCH: u64 = 11_644_473_600;
const NTFS_TICKS_PER_SECOND: u64 = 10_000_000;

pub(crate) fn node_flags_from_ntfs_attributes(
    file_attributes: u32,
    reparse_tag: Option<u32>,
    is_directory: bool,
) -> NodeFlags {
    let mut flags = NodeFlags::empty();
    flags.set(NodeFlags::HIDDEN, file_attributes & HIDDEN != 0);
    flags.set(NodeFlags::SYSTEM, file_attributes & SYSTEM != 0);
    flags.set(NodeFlags::SPARSE, file_attributes & SPARSE_FILE != 0);
    flags.set(NodeFlags::COMPRESSED, file_attributes & COMPRESSED != 0);

    let cloud_attributes = if is_directory {
        RECALL_ON_OPEN | OFFLINE
    } else {
        RECALL_ON_OPEN | OFFLINE | RECALL_ON_DATA_ACCESS
    };
    flags.set(
        NodeFlags::CLOUD_PLACEHOLDER,
        file_attributes & cloud_attributes != 0,
    );

    if let Some(tag) = reparse_tag {
        if tag == REPARSE_TAG_SYMLINK || tag == REPARSE_TAG_LX_SYMLINK {
            flags |= NodeFlags::SYMLINK;
        } else if tag & REPARSE_TAG_NAME_SURROGATE_BIT != 0 {
            flags |= NodeFlags::JUNCTION;
        }
    }

    flags
}

/// Converts an NTFS timestamp (100 ns ticks since 1601) into a `SystemTime`; times before 1970
/// become the Unix epoch, which the tree treats as unknown.
pub(crate) fn ntfs_time_to_system_time(ntfs_ticks: u64) -> SystemTime {
    let seconds = ntfs_ticks / NTFS_TICKS_PER_SECOND;
    let Some(seconds_since_unix_epoch) = seconds.checked_sub(SECONDS_FROM_NTFS_EPOCH_TO_UNIX_EPOCH)
    else {
        return SystemTime::UNIX_EPOCH;
    };
    let nanoseconds = u32::try_from((ntfs_ticks % NTFS_TICKS_PER_SECOND) * 100).unwrap_or(0);

    SystemTime::UNIX_EPOCH
        .checked_add(Duration::new(seconds_since_unix_epoch, nanoseconds))
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_attributes_and_reparse_tags() {
        assert_eq!(
            node_flags_from_ntfs_attributes(HIDDEN | SYSTEM, None, false),
            NodeFlags::HIDDEN | NodeFlags::SYSTEM
        );
        assert_eq!(
            node_flags_from_ntfs_attributes(0, Some(0xA000_0003), true),
            NodeFlags::JUNCTION
        );
        assert_eq!(
            node_flags_from_ntfs_attributes(RECALL_ON_DATA_ACCESS, Some(0x9000_601A), false),
            NodeFlags::CLOUD_PLACEHOLDER
        );
        assert_eq!(
            node_flags_from_ntfs_attributes(RECALL_ON_DATA_ACCESS, Some(0x9000_601A), true),
            NodeFlags::empty(),
            "a synced cloud folder is an ordinary folder"
        );
    }

    #[test]
    fn converts_ntfs_timestamps() {
        assert_eq!(
            ntfs_time_to_system_time(116_444_736_000_000_000),
            SystemTime::UNIX_EPOCH
        );
        assert_eq!(
            ntfs_time_to_system_time(116_444_736_020_000_000),
            SystemTime::UNIX_EPOCH + Duration::from_secs(2)
        );
        assert_eq!(ntfs_time_to_system_time(5), SystemTime::UNIX_EPOCH);
    }
}
