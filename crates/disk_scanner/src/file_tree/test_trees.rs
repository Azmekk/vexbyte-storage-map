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

pub(crate) fn child_named(tree: &FileTree, parent: NodeIndex, name: &str) -> NodeIndex {
    tree.children(parent)
        .find(|&child| tree.name(child) == name)
        .unwrap()
}
