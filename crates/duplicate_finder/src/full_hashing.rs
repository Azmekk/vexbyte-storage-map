use std::io::Read;
use std::path::Path;

use crate::cancellation::CancellationToken;
use crate::file_read_error::FileReadError;
use crate::platform::open_file_for_sequential_reading;

/// Large reads keep the drive streaming; 1 MiB is past the point where bigger buffers help.
const FULL_HASH_BUFFER_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ContentHash([u8; 32]);

/// Streams the whole file through BLAKE3. `report_bytes_read` is called after every buffer, and
/// cancellation is checked between buffers so large videos stop promptly.
pub(crate) fn compute_full_hash(
    path: &Path,
    expected_size: u64,
    cancellation: &CancellationToken,
    mut report_bytes_read: impl FnMut(u64),
) -> Result<ContentHash, FileReadError> {
    let mut file = open_file_for_sequential_reading(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; FULL_HASH_BUFFER_BYTES];
    let mut total_bytes_read = 0_u64;

    loop {
        if cancellation.is_cancelled() {
            return Err(FileReadError::Cancelled);
        }

        let bytes_read = match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes_read) => bytes_read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error.into()),
        };

        hasher.update(&buffer[..bytes_read]);
        total_bytes_read += bytes_read as u64;
        report_bytes_read(bytes_read as u64);

        if total_bytes_read > expected_size {
            return Err(FileReadError::SizeChanged);
        }
    }

    if total_bytes_read != expected_size {
        return Err(FileReadError::SizeChanged);
    }

    Ok(ContentHash(*hasher.finalize().as_bytes()))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn hashes_identical_content_identically_and_counts_bytes() {
        let folder =
            std::env::temp_dir().join(format!("vsm-full-hash-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let content = vec![42_u8; 2 * FULL_HASH_BUFFER_BYTES + 17];
        let first = folder.join("first.bin");
        let second = folder.join("second.bin");
        let different = folder.join("different.bin");
        std::fs::write(&first, &content).unwrap();
        std::fs::write(&second, &content).unwrap();
        let mut changed_content = content.clone();
        changed_content[FULL_HASH_BUFFER_BYTES + 5] = 0;
        std::fs::write(&different, &changed_content).unwrap();

        let size = content.len() as u64;
        let token = CancellationToken::new();
        let mut bytes_reported = 0;
        let first_hash =
            compute_full_hash(&first, size, &token, |bytes| bytes_reported += bytes).unwrap();
        let second_hash = compute_full_hash(&second, size, &token, |_| {}).unwrap();
        let different_hash = compute_full_hash(&different, size, &token, |_| {}).unwrap();
        std::fs::remove_dir_all(&folder).unwrap();

        assert_eq!(first_hash, second_hash);
        assert_ne!(first_hash, different_hash);
        assert_eq!(bytes_reported, size);
    }
}
