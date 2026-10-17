use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

use windows::Win32::Storage::FileSystem::{
    FILE_FLAG_SEQUENTIAL_SCAN, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
};

/// Opens `path` read-only for hashing. Other programs may keep reading, writing and renaming the
/// file meanwhile; a change shows up as a size mismatch or during revalidation.
pub(crate) fn open_file_for_sequential_reading(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .share_mode((FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE).0)
        .custom_flags(FILE_FLAG_SEQUENTIAL_SCAN.0)
        .open(path)
}
