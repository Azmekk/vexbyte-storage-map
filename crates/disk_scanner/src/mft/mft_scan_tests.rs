//! End-to-end tests of the MFT scanner on a synthesized in-memory NTFS volume.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

use super::attributes::file_name::FileNameNamespace::{Dos, Posix, Win32, Win32AndDos};
use super::attributes::{DATA_TYPE, FILE_NAME_TYPE};
use super::file_reference::FileReference;
use super::mft_scan::{DEFAULT_MFT_CHUNK_SIZE_BYTES, scan_mft_from_volume_bytes};
use super::mft_scan_error::MftScanError;
use super::test_fixtures::{InMemoryVolume, RECORD_SIZE, RecordBuilder};
use super::volume_layout::NtfsVolumeLayout;
use crate::cancellation::CancellationToken;
use crate::file_tree::{FileTree, NodeFlags, NodeIndex};
use crate::scan_progress::NoProgressReporting;

const CLUSTER_SIZE: usize = 4096;
const FIRST_EXTENT_OFFSET: usize = 4 * CLUSTER_SIZE;
const SECOND_EXTENT_OFFSET: usize = 20 * CLUSTER_SIZE;
const RECORDS_PER_EXTENT: usize = 16;

fn record_offset(record_number: usize) -> usize {
    if record_number < RECORDS_PER_EXTENT {
        FIRST_EXTENT_OFFSET + record_number * RECORD_SIZE
    } else {
        SECOND_EXTENT_OFFSET + (record_number - RECORDS_PER_EXTENT) * RECORD_SIZE
    }
}

#[allow(clippy::too_many_lines)]
fn sample_records() -> Vec<(usize, Vec<u8>)> {
    let root = FileReference::from_parts(5, 5);
    let extend = FileReference::from_parts(11, 11);
    let users = FileReference::from_parts(16, 1);
    let big_iso = FileReference::from_parts(20, 4);
    // Records 0-15 at cluster 4, records 16-31 at cluster 20 (16 clusters further).
    let mft_runs = [0x11, 0x04, 0x04, 0x11, 0x04, 0x10, 0x00];
    let some_runs = [0x11, 0x01, 0x30, 0x00];

    let torn_record = {
        let mut record = RecordBuilder::new(1)
            .file_name(users, Win32AndDos, "torn.txt")
            .build();
        record[1022] ^= 0xFF;
        record
    };

    vec![
        (
            0,
            RecordBuilder::new(1)
                .standard_information(0, 0x06)
                .file_name(root, Win32AndDos, "$MFT")
                .non_resident_data(0, 32_768, 32_768, &mft_runs)
                .build(),
        ),
        (
            1,
            RecordBuilder::new(1)
                .file_name(root, Win32AndDos, "$MFTMirr")
                .non_resident_data(0, 4096, 4096, &some_runs)
                .build(),
        ),
        (
            2,
            RecordBuilder::new(2)
                .file_name(root, Win32AndDos, "$LogFile")
                .non_resident_data(0, 1_048_576, 1_048_576, &some_runs)
                .build(),
        ),
        (
            5,
            RecordBuilder::new(5)
                .directory()
                .standard_information(133_000_000_000_000_000, 0x06)
                .file_name(root, Win32AndDos, ".")
                .build(),
        ),
        (
            11,
            RecordBuilder::new(11)
                .directory()
                .file_name(root, Win32AndDos, "$Extend")
                .build(),
        ),
        (
            16,
            RecordBuilder::new(1)
                .directory()
                .standard_information(133_000_000_000_000_000, 0)
                .file_name(root, Win32, "Users")
                .build(),
        ),
        (
            17,
            RecordBuilder::new(3)
                .file_name(users, Win32, "Long File Name.txt")
                .file_name(users, Dos, "LONGFI~1.TXT")
                .resident_data(&[b'x'; 100])
                .build(),
        ),
        (
            18,
            RecordBuilder::new(1)
                .file_name(users, Win32AndDos, "video.mp4")
                .non_resident_data(0, 5_001_216, 5_000_000, &some_runs)
                .build(),
        ),
        (
            19,
            RecordBuilder::new(1)
                .file_name(users, Win32AndDos, "a.bin")
                .file_name(root, Posix, "b.bin")
                .non_resident_data(0, 8192, 8192, &some_runs)
                .build(),
        ),
        (
            20,
            RecordBuilder::new(4)
                .attribute_list(&[
                    (FILE_NAME_TYPE, 0, big_iso),
                    (DATA_TYPE, 0, FileReference::from_parts(21, 1)),
                ])
                .file_name(users, Win32AndDos, "big.iso")
                .build(),
        ),
        (
            21,
            RecordBuilder::new(1)
                .extension_of(big_iso)
                .non_resident_data(0, 1_003_520, 1_000_000, &some_runs)
                .build(),
        ),
        (
            22,
            RecordBuilder::new(1)
                .not_in_use()
                .file_name(users, Win32AndDos, "deleted.txt")
                .resident_data(&[1; 10])
                .build(),
        ),
        (
            23,
            RecordBuilder::new(1)
                .file_name(FileReference::from_parts(16, 9), Win32AndDos, "stale.txt")
                .build(),
        ),
        (
            24,
            RecordBuilder::new(1)
                .file_name(users, Win32, "named-stream.txt")
                .resident_data(&[2; 10])
                .named_resident_data("Zone.Identifier", &[3; 26])
                .build(),
        ),
        (
            25,
            RecordBuilder::new(1)
                .file_name(users, Win32AndDos, "packed.log")
                .compressed_non_resident_data(65_536, 60_000, 16_384)
                .build(),
        ),
        (
            26,
            RecordBuilder::new(1)
                .file_name(extend, Win32AndDos, "$Quota")
                .resident_data(&[0; 64])
                .build(),
        ),
        (
            27,
            RecordBuilder::new(1)
                .directory()
                .standard_information(0, 0x400)
                .file_name(users, Win32AndDos, "Junction")
                .reparse_point(0xA000_0003)
                .build(),
        ),
        (28, torn_record),
        (
            29,
            RecordBuilder::new(1)
                .extension_of(big_iso)
                .file_name(users, Win32AndDos, "ghost.iso")
                .build(),
        ),
        (
            30,
            RecordBuilder::new(1)
                .extension_of(FileReference::from_parts(19, 7))
                .file_name(users, Win32AndDos, "stale-link.bin")
                .build(),
        ),
        (
            31,
            RecordBuilder::new(1)
                .file_name(root, Win32AndDos, "last.txt")
                .resident_data(&[4])
                .build(),
        ),
    ]
}

/// A 256 KiB volume whose 32-record MFT is split into two fragments.
fn sample_volume() -> (InMemoryVolume, NtfsVolumeLayout) {
    let mut volume = InMemoryVolume::with_size(64 * CLUSTER_SIZE);
    for (record_number, record) in sample_records() {
        volume.write(record_offset(record_number), &record);
    }
    let layout = NtfsVolumeLayout {
        volume_serial_number: 0xABCD,
        bytes_per_sector: 512,
        bytes_per_cluster: 4096,
        bytes_per_file_record: 1024,
        total_clusters: 64,
        mft_start_cluster: 4,
        mft_valid_data_length: 32 * 1024,
    };
    (volume, layout)
}

fn scan_sample_volume(chunk_size: usize) -> FileTree {
    let (volume, layout) = sample_volume();
    scan_mft_from_volume_bytes(
        &volume,
        &layout,
        PathBuf::from("X:\\"),
        chunk_size,
        &NoProgressReporting,
        &CancellationToken::new(),
    )
    .unwrap()
}

fn child(tree: &FileTree, parent: NodeIndex, name: &str) -> NodeIndex {
    tree.children(parent)
        .find(|&child| tree.name(child) == name)
        .unwrap_or_else(|| panic!("{name} is missing"))
}

fn child_names(tree: &FileTree, parent: NodeIndex) -> Vec<String> {
    let mut names: Vec<String> = tree
        .children(parent)
        .map(|child| tree.name(child).to_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn builds_the_tree_of_a_synthesized_volume() {
    for chunk_size in [DEFAULT_MFT_CHUNK_SIZE_BYTES, 2048] {
        let tree = scan_sample_volume(chunk_size);
        let root = tree.root();

        assert_eq!(
            child_names(&tree, root),
            [
                "NTFS metadata",
                "Unlinked items",
                "Users",
                "b.bin",
                "last.txt"
            ]
        );

        let metadata = child(&tree, root, "NTFS metadata");
        assert_eq!(
            child_names(&tree, metadata),
            ["$Extend", "$LogFile", "$MFT", "$MFTMirr"]
        );
        let extend = child(&tree, metadata, "$Extend");
        assert_eq!(child_names(&tree, extend), ["$Quota"]);
        let mft = child(&tree, metadata, "$MFT");
        assert_eq!(tree.full_path(mft), Some(Path::new("X:\\").join("$MFT")));

        let users = child(&tree, root, "Users");
        assert_eq!(
            child_names(&tree, users),
            [
                "Junction",
                "Long File Name.txt",
                "a.bin",
                "big.iso",
                "named-stream.txt",
                "packed.log",
                "video.mp4"
            ],
            "DOS names, deleted, torn and stale extension records are skipped"
        );

        let long_name = child(&tree, users, "Long File Name.txt");
        assert_eq!(tree.node(long_name).unwrap().logical_size(), 100);
        assert_eq!(tree.node(long_name).unwrap().allocated_size(), 0);
        let big_iso = child(&tree, users, "big.iso");
        assert_eq!(
            tree.node(big_iso).unwrap().logical_size(),
            1_000_000,
            "the size comes from the extension record"
        );
        let named_stream = child(&tree, users, "named-stream.txt");
        assert_eq!(tree.node(named_stream).unwrap().logical_size(), 10);
        let packed = child(&tree, users, "packed.log");
        assert_eq!(tree.node(packed).unwrap().allocated_size(), 16_384);
        let junction = child(&tree, users, "Junction");
        assert!(
            tree.node(junction)
                .unwrap()
                .flags()
                .contains(NodeFlags::JUNCTION)
        );

        let b_link = child(&tree, root, "b.bin");
        assert!(
            tree.node(b_link)
                .unwrap()
                .flags()
                .contains(NodeFlags::HARD_LINK_COUNTED_ELSEWHERE)
        );

        let unlinked = child(&tree, root, "Unlinked items");
        assert_eq!(child_names(&tree, unlinked), ["stale.txt"]);

        let totals = tree.totals();
        assert_eq!(totals.logical_size, 7_153_807);
        assert_eq!(totals.file_count, 13);
        assert_eq!(totals.additional_hard_link_count, 1);
        assert_eq!(totals.link_count, 1);
        assert!(tree.node(root).unwrap().modified_time().is_some());
    }
}

#[test]
fn stops_when_cancelled() {
    let (volume, layout) = sample_volume();
    let cancellation = CancellationToken::new();
    cancellation.cancel();

    let result = scan_mft_from_volume_bytes(
        &volume,
        &layout,
        PathBuf::from("X:\\"),
        2048,
        &NoProgressReporting,
        &cancellation,
    );

    assert!(matches!(result, Err(MftScanError::Cancelled)));
}

#[test]
fn rejects_an_inconsistent_layout() {
    let (volume, layout) = sample_volume();
    let broken_layout = NtfsVolumeLayout {
        bytes_per_file_record: 1000,
        ..layout
    };

    let result = scan_mft_from_volume_bytes(
        &volume,
        &broken_layout,
        PathBuf::from("X:\\"),
        2048,
        &NoProgressReporting,
        &CancellationToken::new(),
    );

    assert!(matches!(
        result,
        Err(MftScanError::InvalidVolumeLayout { .. })
    ));
}
