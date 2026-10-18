//! Reads the Linux mount table (`/proc/self/mountinfo`), facts about the block devices behind
//! mounted file systems (from `/sys`) and the space of a mounted file system (`statvfs`).
//!
//! Linux only: on every other target this crate is empty. Everything here is read-only.

#![cfg(target_os = "linux")]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod block_device;
mod device_number;
mod disk_space;
mod file_system_category;
mod mount_entry;
mod mount_info_parser;
mod mount_lookup;
mod volume_label;

pub use block_device::{BlockDeviceFacts, block_device_facts};
pub use device_number::DeviceNumber;
pub use disk_space::{DiskSpace, query_disk_space};
pub use file_system_category::{FileSystemCategory, categorize_file_system};
pub use mount_entry::MountEntry;
pub use mount_info_parser::{parse_mount_info, read_mount_table};
pub use mount_lookup::mount_containing;
pub use volume_label::volume_label_of_source;
