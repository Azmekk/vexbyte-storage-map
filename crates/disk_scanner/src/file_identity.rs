/// Identifies a file on its volume independently of its path, so hard links (several paths to
/// the same file) can be recognized.
///
/// Opaque by design: on Windows it holds the volume serial number and the file ID, on Unix it
/// would hold the device and inode numbers. Only equality and hashing are meaningful.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileIdentity {
    volume_id: u64,
    // Two u64 halves instead of one u128 keep the alignment at 8 bytes, which makes the hard
    // link tracker's hash map noticeably smaller.
    file_id_high: u64,
    file_id_low: u64,
}

impl FileIdentity {
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn from_volume_and_file_ids(volume_id: u64, file_id: u128) -> Self {
        Self {
            volume_id,
            file_id_high: (file_id >> 64) as u64,
            file_id_low: (file_id & u128::from(u64::MAX)) as u64,
        }
    }
}

impl std::fmt::Debug for FileIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FileIdentity")
            .finish_non_exhaustive()
    }
}
