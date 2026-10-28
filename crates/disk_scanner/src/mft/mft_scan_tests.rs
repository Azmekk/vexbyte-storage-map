//! End-to-end tests of the MFT scanner on a synthesized in-memory NTFS volume.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

use super::attributes::file_name::FileNameNamespace::{Dos, Posix, Win32, Win32AndDos};
use super::attributes::{DATA_TYPE, FILE_NAME_TYPE};
use super::file_reference::FileReference;
use super::mft_record_table::MftRecordTable;
use super::mft_record_to_tree::build_tree_from_records;
use super::mft_scan::{DEFAULT_MFT_CHUNK_SIZE_BYTES, scan_mft_from_volume_bytes};
use super::mft_scan_error::MftScanError;
use super::record_fixups::apply_update_sequence_fixups;
use super::test_fixtures::{
    InMemoryVolume, RECORD_SIZE, RecordBuilder, protect_with_update_sequence,
};
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

/// What a long-lived Windows system drive looks like: `$MFT` needs an extension record, and
/// its `$ATTRIBUTE_LIST` lives in its own cluster although it is only a few entries long.
/// Record 0 maps the first 16 records; the extension record maps the rest, so the scan can only
/// reach the second half of the MFT after reading the list from the volume.
#[test]
fn locates_the_mft_through_a_non_resident_attribute_list() {
    const ATTRIBUTE_LIST_CLUSTER: u8 = 40;
    const EXTENSION_RECORD_NUMBER: usize = 15;
    let mft = FileReference::from_parts(0, 1);
    let extension = FileReference::from_parts(EXTENSION_RECORD_NUMBER as u64, 1);
    let root = FileReference::from_parts(5, 5);

    let (record_zero, attribute_list) = RecordBuilder::new(1)
        .standard_information(0, 0x06)
        .file_name(root, Win32AndDos, "$MFT")
        .non_resident_data(0, 32_768, 32_768, &[0x11, 0x04, 0x04, 0x00])
        .non_resident_attribute_list(
            &[(DATA_TYPE, 0, mft), (DATA_TYPE, 16, extension)],
            &[0x11, 0x01, ATTRIBUTE_LIST_CLUSTER, 0x00],
        );
    let extension_record = RecordBuilder::new(1)
        .extension_of(mft)
        .non_resident_data(16, 0, 0, &[0x11, 0x04, 0x14, 0x00])
        .build();

    let (mut volume, layout) = sample_volume();
    volume.write(record_offset(0), &record_zero.build());
    volume.write(record_offset(EXTENSION_RECORD_NUMBER), &extension_record);
    volume.write(
        usize::from(ATTRIBUTE_LIST_CLUSTER) * CLUSTER_SIZE,
        &attribute_list,
    );

    let tree = scan_mft_from_volume_bytes(
        &volume,
        &layout,
        PathBuf::from("X:\\"),
        DEFAULT_MFT_CHUNK_SIZE_BYTES,
        &NoProgressReporting,
        &CancellationToken::new(),
    )
    .unwrap();

    assert!(
        child_names(&tree, tree.root()).contains(&"last.txt".to_owned()),
        "record 31, in the second MFT extent, was reached through the extension record"
    );
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

/// A tiny deterministic xorshift generator, so a failing corruption reproduces exactly.
struct GarbageGenerator(u64);

impl GarbageGenerator {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % bound as u64).unwrap()
    }
}

/// Overwrites random bytes of a record's content (never the update sequence fields at 4..8),
/// then protects it again, so the corrupt record passes the fixup check and reaches the
/// attribute parsers.
fn corrupt_record(record: &mut [u8], generator: &mut GarbageGenerator) {
    let _ = apply_update_sequence_fixups(record);
    let flip_count = 1 + generator.below(24);
    for _ in 0..flip_count {
        // Header fields (offsets, sizes, flags) are hit often on purpose.
        let position = if generator.below(3) == 0 {
            8 + generator.below(40)
        } else {
            8 + generator.below(record.len() - 8)
        };
        record[position] = generator.next().to_le_bytes()[0];
    }
    protect_with_update_sequence(record, 0x0001);
}

#[test]
fn corrupt_records_never_panic_or_read_out_of_bounds() {
    let mut generator = GarbageGenerator(0x9E37_79B9_7F4A_7C15);
    let originals = sample_records();

    for round in 0..3000 {
        let mut table = MftRecordTable::with_record_count(32);
        for (record_number, original) in &originals {
            let mut record = original.clone();
            corrupt_record(&mut record, &mut generator);
            if round % 5 == 0 {
                record.truncate(generator.below(record.len()));
            }
            table.add_record(*record_number as u64, &mut record);
        }
        table.merge_extension_records();

        let tree = build_tree_from_records(&table, PathBuf::from("X:\\"), 1);
        for index in 0..tree.node_count() {
            let _ = tree.full_path(NodeIndex::from_usize(index));
        }
    }
}

#[test]
fn a_corrupt_mft_record_zero_fails_the_scan_instead_of_panicking() {
    let mut generator = GarbageGenerator(0x0123_4567_89AB_CDEF);
    let (_, layout) = sample_volume();

    for _ in 0..2000 {
        let (mut volume, _) = sample_volume();
        let (_, mut record_zero) = sample_records().swap_remove(0);
        corrupt_record(&mut record_zero, &mut generator);
        volume.write(record_offset(0), &record_zero);

        let _ = scan_mft_from_volume_bytes(
            &volume,
            &layout,
            PathBuf::from("X:\\"),
            2048,
            &NoProgressReporting,
            &CancellationToken::new(),
        );
    }
}
