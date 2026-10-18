use std::fs::File;
use std::io;
use std::path::Path;

use rustix::fs::{Advice, fadvise};

/// Opens `path` read-only for hashing and tells the kernel the file will be read front to back,
/// so it reads ahead more aggressively.
pub(crate) fn open_file_for_sequential_reading(path: &Path) -> io::Result<File> {
    let file = File::open(path)?;

    // The advice only tunes read-ahead; some file systems (for example FUSE) reject it, and
    // the file reads correctly either way.
    let _ = fadvise(&file, 0, None, Advice::Sequential);

    Ok(file)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::io::Read;

    use super::*;

    #[test]
    fn reads_a_temporary_file() {
        let directory =
            std::env::temp_dir().join(format!("vsm-file-reading-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("file.bin");
        std::fs::write(&path, b"contents").unwrap();

        let mut contents = Vec::new();
        let read_result = open_file_for_sequential_reading(&path)
            .and_then(|mut file| file.read_to_end(&mut contents));
        std::fs::remove_dir_all(&directory).unwrap();

        read_result.unwrap();
        assert_eq!(contents, b"contents");
    }
}
