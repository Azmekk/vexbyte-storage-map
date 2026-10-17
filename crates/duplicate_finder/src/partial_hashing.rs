use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use xxhash_rust::xxh3::Xxh3;

use crate::file_read_error::FileReadError;
use crate::platform::open_file_for_sequential_reading;

/// Bytes read from the start and from the end of a file for the quick check.
pub(crate) const PARTIAL_HASH_SAMPLE_BYTES: u64 = 64 * 1024;

/// Files up to this size are hashed completely by the quick check, since the two samples would
/// cover (almost) the whole file anyway. Their quick hash is final.
pub(crate) const WHOLE_FILE_QUICK_CHECK_LIMIT_BYTES: u64 = 2 * PARTIAL_HASH_SAMPLE_BYTES;

/// xxh3 with 128 bits: fast, and wide enough that files hashed completely by the quick check can
/// skip the full check without a realistic chance of a collision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PartialHash(u128);

#[must_use]
pub(crate) fn quick_check_covers_whole_file(file_size: u64) -> bool {
    file_size <= WHOLE_FILE_QUICK_CHECK_LIMIT_BYTES
}

/// Hashes the first and last 64 KiB of the file, or the whole file when it is at most 128 KiB.
pub(crate) fn compute_partial_hash(
    path: &Path,
    expected_size: u64,
) -> Result<PartialHash, FileReadError> {
    let mut file = open_file_for_sequential_reading(path)?;
    let mut hasher = Xxh3::new();

    if quick_check_covers_whole_file(expected_size) {
        let mut content = Vec::with_capacity(usize::try_from(expected_size).unwrap_or(0));
        // Reading one byte past the expected size detects files that grew since they were listed.
        file.by_ref()
            .take(expected_size + 1)
            .read_to_end(&mut content)?;
        if content.len() as u64 != expected_size {
            return Err(FileReadError::SizeChanged);
        }
        hasher.update(&content);
    } else {
        if file.metadata()?.len() != expected_size {
            return Err(FileReadError::SizeChanged);
        }
        let sample_length = usize::try_from(PARTIAL_HASH_SAMPLE_BYTES).unwrap_or(usize::MAX);
        let mut sample = vec![0_u8; sample_length];

        file.read_exact(&mut sample)?;
        hasher.update(&sample);

        file.seek(SeekFrom::Start(expected_size - PARTIAL_HASH_SAMPLE_BYTES))?;
        file.read_exact(&mut sample)?;
        hasher.update(&sample);
    }

    Ok(PartialHash(hasher.digest128()))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;

    use super::*;

    struct TemporaryFolder(PathBuf);

    impl TemporaryFolder {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("vsm-partial-hash-{name}-{}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn write(&self, name: &str, content: &[u8]) -> PathBuf {
            let path = self.0.join(name);
            std::fs::write(&path, content).unwrap();
            path
        }
    }

    impl Drop for TemporaryFolder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn hash(path: &Path) -> PartialHash {
        let size = std::fs::metadata(path).unwrap().len();
        compute_partial_hash(path, size).unwrap()
    }

    #[test]
    fn small_files_are_hashed_completely() {
        let folder = TemporaryFolder::new("small");
        let mut content = vec![7_u8; 100_000];
        let original = folder.write("original.bin", &content);
        content[50_000] = 8;
        let changed_in_the_middle = folder.write("changed.bin", &content);

        assert_ne!(hash(&original), hash(&changed_in_the_middle));
    }

    #[test]
    fn large_files_only_compare_their_first_and_last_blocks() {
        let folder = TemporaryFolder::new("large");
        let mut content = vec![1_u8; 300_000];
        let original = folder.write("original.bin", &content);
        content[150_000] = 2;
        let changed_in_the_middle = folder.write("middle.bin", &content);
        content[299_999] = 3;
        let changed_at_the_end = folder.write("end.bin", &content);

        assert_eq!(hash(&original), hash(&changed_in_the_middle));
        assert_ne!(hash(&original), hash(&changed_at_the_end));
    }

    #[test]
    fn reports_files_whose_size_changed() {
        let folder = TemporaryFolder::new("changed-size");
        let path = folder.write("file.bin", &[0_u8; 1000]);

        assert!(matches!(
            compute_partial_hash(&path, 999),
            Err(FileReadError::SizeChanged)
        ));
        assert!(matches!(
            compute_partial_hash(&path, 500_000),
            Err(FileReadError::SizeChanged)
        ));
    }
}
