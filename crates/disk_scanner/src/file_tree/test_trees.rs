//! Small trees shared by the unit tests of the tree queries.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use super::{DirectoryKey, FileTree, FileTreeBuilder, NewEntry, NodeFlags, NodeIndex, NodeKind};

pub(crate) fn entry(
    name: &str,
    kind: NodeKind,
    parent: u32,
    key: Option<u32>,
    size: u64,
) -> NewEntry<'_> {
    NewEntry {
        name,
        kind,
        parent: Some(DirectoryKey::new(parent)),
        directory_key: key.map(DirectoryKey::new),
        logical_size: size,
        allocated_size: size,
        modified_time: None,
        flags: NodeFlags::empty(),
        hard_link_identity: None,
    }
}

/// root
/// ├─ Photos/            (key 1)
/// │  ├─ beach.JPG       400
/// │  ├─ city.png        300
/// │  └─ raw/            (key 2)
/// │     └─ shot.cr3     900
/// ├─ notes.txt          50
/// ├─ setup.exe          700
/// └─ Makefile           10
pub(crate) fn sample_tree() -> FileTree {
    let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
    builder.add_entry(&entry("Photos", NodeKind::Directory, 0, Some(1), 0));
    builder.add_entry(&entry("beach.JPG", NodeKind::File, 1, None, 400));
    builder.add_entry(&entry("city.png", NodeKind::File, 1, None, 300));
    builder.add_entry(&entry("raw", NodeKind::Directory, 1, Some(2), 0));
    builder.add_entry(&entry("shot.cr3", NodeKind::File, 2, None, 900));
    builder.add_entry(&entry("notes.txt", NodeKind::File, 0, None, 50));
    builder.add_entry(&entry("setup.exe", NodeKind::File, 0, None, 700));
    builder.add_entry(&entry("Makefile", NodeKind::File, 0, None, 10));
    builder.finish()
}

/// An entry with its own allocated size and flags, for cloud placeholders.
pub(crate) fn entry_with_flags(
    name: &str,
    kind: NodeKind,
    parent: u32,
    key: Option<u32>,
    logical_size: u64,
    allocated_size: u64,
    flags: NodeFlags,
) -> NewEntry<'_> {
    NewEntry {
        allocated_size,
        flags,
        ..entry(name, kind, parent, key, logical_size)
    }
}

/// root
/// ├─ Cloud/            (key 1) four online-only JPEGs of 1,000,000 each
/// ├─ Mixed/            (key 2)
/// │  ├─ online.pdf     500,000 online-only
/// │  └─ local.pdf      300 (4096 on disk)
/// ├─ Empty/            (key 3)
/// ├─ Unpopulated/      (key 4) a cloud folder that was never listed
/// ├─ video.mp4         4,000,000 online-only
/// ├─ partial.iso       4,000,000 of which 65,536 are on disk
/// └─ zero.txt          an ordinary empty file
pub(crate) fn cloud_tree() -> FileTree {
    let file = NodeKind::File;
    let directory = NodeKind::Directory;
    let placeholder = NodeFlags::CLOUD_PLACEHOLDER;
    let none = NodeFlags::empty();

    let mut builder = FileTreeBuilder::new(PathBuf::from("root"), DirectoryKey::new(0));
    builder.add_entry(&entry_with_flags(
        "Cloud",
        directory,
        0,
        Some(1),
        0,
        0,
        none,
    ));
    for name in ["a.jpg", "b.jpg", "c.jpg", "d.jpg"] {
        builder.add_entry(&entry_with_flags(
            name,
            file,
            1,
            None,
            1_000_000,
            0,
            placeholder,
        ));
    }
    builder.add_entry(&entry_with_flags(
        "Mixed",
        directory,
        0,
        Some(2),
        0,
        0,
        none,
    ));
    builder.add_entry(&entry_with_flags(
        "online.pdf",
        file,
        2,
        None,
        500_000,
        0,
        placeholder,
    ));
    builder.add_entry(&entry_with_flags(
        "local.pdf",
        file,
        2,
        None,
        300,
        4096,
        none,
    ));
    builder.add_entry(&entry_with_flags(
        "Empty",
        directory,
        0,
        Some(3),
        0,
        0,
        none,
    ));
    builder.add_entry(&entry_with_flags(
        "Unpopulated",
        directory,
        0,
        Some(4),
        0,
        0,
        placeholder,
    ));
    builder.add_entry(&entry_with_flags(
        "video.mp4",
        file,
        0,
        None,
        4_000_000,
        0,
        placeholder,
    ));
    builder.add_entry(&entry_with_flags(
        "partial.iso",
        file,
        0,
        None,
        4_000_000,
        65_536,
        placeholder,
    ));
    builder.add_entry(&entry_with_flags("zero.txt", file, 0, None, 0, 0, none));
    builder.finish()
}

/// The first node anywhere in the tree with this name.
pub(crate) fn node_named(tree: &FileTree, name: &str) -> NodeIndex {
    (0..tree.node_count())
        .map(NodeIndex::from_usize)
        .find(|&index| tree.name(index) == name)
        .unwrap()
}

pub(crate) fn child_named(tree: &FileTree, parent: NodeIndex, name: &str) -> NodeIndex {
    tree.children(parent)
        .find(|&child| tree.name(child) == name)
        .unwrap()
}
