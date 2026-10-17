use crate::folder_walk::EntryFlags;

const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x0000_0004;
const FILE_ATTRIBUTE_SPARSE_FILE: u32 = 0x0000_0200;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
const FILE_ATTRIBUTE_COMPRESSED: u32 = 0x0000_0800;
const FILE_ATTRIBUTE_OFFLINE: u32 = 0x0000_1000;
const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x0004_0000;
const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;

const IO_REPARSE_TAG_SYMLINK: u32 = 0xA000_000C;
const IO_REPARSE_TAG_LX_SYMLINK: u32 = 0xA000_001D;
/// Bit 29 of a reparse tag marks a "name surrogate": the reparse point stands for another named
/// entity (junction, volume mount point, symlink). Other tags (cloud files, deduplication,
/// WOF compression) mark ordinary files and directories whose content is stored differently.
const REPARSE_TAG_NAME_SURROGATE_BIT: u32 = 0x2000_0000;

/// Maps Win32 file attributes and the reparse tag of an entry to platform-neutral flags.
pub(super) fn entry_flags_from_attributes(
    attributes: u32,
    reparse_tag: u32,
    is_directory: bool,
) -> EntryFlags {
    let mut flags = EntryFlags::empty();

    flags.set(EntryFlags::HIDDEN, attributes & FILE_ATTRIBUTE_HIDDEN != 0);
    flags.set(EntryFlags::SYSTEM, attributes & FILE_ATTRIBUTE_SYSTEM != 0);
    flags.set(
        EntryFlags::SPARSE,
        attributes & FILE_ATTRIBUTE_SPARSE_FILE != 0,
    );
    flags.set(
        EntryFlags::COMPRESSED,
        attributes & FILE_ATTRIBUTE_COMPRESSED != 0,
    );
    flags.set(
        EntryFlags::CLOUD_PLACEHOLDER,
        is_cloud_placeholder(attributes, is_directory),
    );

    if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        if reparse_tag == IO_REPARSE_TAG_SYMLINK || reparse_tag == IO_REPARSE_TAG_LX_SYMLINK {
            flags |= EntryFlags::SYMLINK;
        } else if reparse_tag & REPARSE_TAG_NAME_SURROGATE_BIT != 0 {
            flags |= EntryFlags::JUNCTION;
        }
    }

    flags
}

/// Cloud files (`OneDrive` and other cloud sync providers) mark content that is not on disk with
/// recall attributes. Opening such a file downloads it. A synced directory is listed normally,
/// but a directory marked recall-on-open has not been populated yet and listing it would fetch
/// its contents from the cloud.
fn is_cloud_placeholder(attributes: u32, is_directory: bool) -> bool {
    let directory_recall_attributes = FILE_ATTRIBUTE_RECALL_ON_OPEN | FILE_ATTRIBUTE_OFFLINE;
    let file_recall_attributes = directory_recall_attributes | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS;

    let recall_attributes = if is_directory {
        directory_recall_attributes
    } else {
        file_recall_attributes
    };

    attributes & recall_attributes != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_simple_attributes() {
        let flags = entry_flags_from_attributes(
            FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_COMPRESSED,
            0,
            false,
        );

        assert_eq!(
            flags,
            EntryFlags::HIDDEN | EntryFlags::SYSTEM | EntryFlags::COMPRESSED
        );
    }

    #[test]
    fn recognizes_junctions_and_symlinks_by_reparse_tag() {
        let junction_tag = 0xA000_0003;

        let junction =
            entry_flags_from_attributes(FILE_ATTRIBUTE_REPARSE_POINT, junction_tag, true);
        let symlink =
            entry_flags_from_attributes(FILE_ATTRIBUTE_REPARSE_POINT, IO_REPARSE_TAG_SYMLINK, true);

        assert_eq!(junction, EntryFlags::JUNCTION);
        assert_eq!(symlink, EntryFlags::SYMLINK);
    }

    #[test]
    fn treats_a_synced_cloud_directory_as_a_normal_directory() {
        let cloud_files_tag = 0x9000_601A;

        let flags =
            entry_flags_from_attributes(FILE_ATTRIBUTE_REPARSE_POINT, cloud_files_tag, true);

        assert_eq!(flags, EntryFlags::empty());
    }

    #[test]
    fn flags_cloud_placeholders() {
        let online_only_file =
            entry_flags_from_attributes(FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS, 0, false);
        let unpopulated_directory =
            entry_flags_from_attributes(FILE_ATTRIBUTE_RECALL_ON_OPEN, 0, true);

        assert_eq!(online_only_file, EntryFlags::CLOUD_PLACEHOLDER);
        assert_eq!(unpopulated_directory, EntryFlags::CLOUD_PLACEHOLDER);
    }
}
