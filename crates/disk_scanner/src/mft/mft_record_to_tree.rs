use std::path::PathBuf;

use super::file_reference::{
    FIRST_ORDINARY_RECORD_NUMBER, FileReference, ROOT_DIRECTORY_RECORD_NUMBER,
};
use super::mft_record_table::{CollectedName, MftRecordTable, RecordSummary};
use super::ntfs_file_attributes::{node_flags_from_ntfs_attributes, ntfs_time_to_system_time};
use crate::file_identity::FileIdentity;
use crate::file_tree::{DirectoryKey, FileTree, FileTreeBuilder, NewEntry, NodeKind};

const NTFS_METADATA_DIRECTORY_NAME: &str = "NTFS metadata";

/// Links the collected records into a tree in a second pass, now that every parent is known.
///
/// Every name becomes a node, so a file with several hard links appears once per link and is
/// counted once. Parent references whose sequence number no longer matches point to deleted
/// directories and are dropped; such entries end up in "Unlinked items". NTFS metafiles
/// (`$MFT`, `$LogFile`, `$Bitmap` and so on) are grouped under a synthetic
/// "NTFS metadata" directory.
pub(crate) fn build_tree_from_records(
    table: &MftRecordTable,
    volume_root_path: PathBuf,
    volume_serial_number: u64,
) -> FileTree {
    let root_key = record_key(ROOT_DIRECTORY_RECORD_NUMBER);
    let mut builder =
        FileTreeBuilder::with_capacity(volume_root_path, root_key, table.names().len() + 1);

    if let Some(root_record) = table.record(ROOT_DIRECTORY_RECORD_NUMBER) {
        builder.set_root_details(
            Some(ntfs_time_to_system_time(root_record.modified_time)),
            node_flags_from_ntfs_attributes(root_record.file_attributes, None, true),
        );
    }

    let metadata_directory_key =
        DirectoryKey::new(u32::try_from(table.records().len()).unwrap_or(u32::MAX - 1));
    builder.add_synthetic_directory(
        NTFS_METADATA_DIRECTORY_NAME,
        Some(root_key),
        metadata_directory_key,
    );

    for name in table.names() {
        if name.owner.record_number() == ROOT_DIRECTORY_RECORD_NUMBER {
            continue;
        }
        let parent = presented_parent(table, name, metadata_directory_key);
        add_name_to_tree(table, name, parent, volume_serial_number, &mut builder);
    }

    builder.finish()
}

/// The directory a name appears in: its parent if that still exists, the synthetic metadata
/// directory for metafiles in the root, or `None` for unlinked entries.
fn presented_parent(
    table: &MftRecordTable,
    name: &CollectedName,
    metadata_directory_key: DirectoryKey,
) -> Option<DirectoryKey> {
    let parent = name.parent_directory;
    let parent_is_live_directory = table.is_live_reference(parent)
        && table
            .record(parent.record_number())
            .is_some_and(RecordSummary::is_directory);
    if !parent_is_live_directory {
        return None;
    }

    let is_metafile_in_root = name.owner.record_number() < FIRST_ORDINARY_RECORD_NUMBER
        && parent.record_number() == ROOT_DIRECTORY_RECORD_NUMBER;
    if is_metafile_in_root {
        Some(metadata_directory_key)
    } else {
        Some(record_key(parent.record_number()))
    }
}

fn add_name_to_tree(
    table: &MftRecordTable,
    name: &CollectedName,
    parent: Option<DirectoryKey>,
    volume_serial_number: u64,
    builder: &mut FileTreeBuilder,
) {
    let owner = name.owner;
    let Some(record) = table.record(owner.record_number()) else {
        return;
    };
    let is_directory = record.is_directory();

    let (kind, logical_size, allocated_size) = if is_directory {
        (NodeKind::Directory, 0, 0)
    } else {
        (
            NodeKind::File,
            record.data_sizes.logical_size,
            record.data_sizes.allocated_size,
        )
    };

    builder.add_entry(&NewEntry {
        name: table.name_text(name),
        kind,
        parent,
        directory_key: is_directory.then(|| record_key(owner.record_number())),
        logical_size,
        allocated_size,
        modified_time: Some(ntfs_time_to_system_time(record.modified_time)),
        flags: node_flags_from_ntfs_attributes(
            record.file_attributes,
            record.reparse_tag(),
            is_directory,
        ),
        hard_link_identity: hard_link_identity(
            owner,
            record.name_count,
            is_directory,
            volume_serial_number,
        ),
    });
}

/// Only files with more than one name need tracking; skipping the rest keeps the tracker
/// small on volumes with millions of files.
fn hard_link_identity(
    owner: FileReference,
    name_count: u16,
    is_directory: bool,
    volume_serial_number: u64,
) -> Option<FileIdentity> {
    (!is_directory && name_count > 1).then(|| {
        FileIdentity::from_volume_and_file_ids(
            volume_serial_number,
            u128::from(owner.record_number()),
        )
    })
}

fn record_key(record_number: u64) -> DirectoryKey {
    DirectoryKey::new(u32::try_from(record_number).unwrap_or(u32::MAX))
}
