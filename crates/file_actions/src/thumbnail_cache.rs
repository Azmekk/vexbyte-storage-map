use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

use xxhash_rust::xxh3::Xxh3;

use crate::file_stamp::FileStamp;
use crate::platform::read_file_stamp;
use crate::shell_thumbnails::{ThumbnailError, render_thumbnail_png};

/// Stores rendered thumbnails as PNG files in one cache folder (the app's cache directory) and
/// hands out their paths, so the UI can load them through the asset protocol.
///
/// The cache key covers the path, the requested size, and the file's size and modification
/// time, so an edited file gets a fresh thumbnail. Old entries are never cleaned up here.
#[derive(Debug, Clone)]
pub struct ThumbnailCache {
    directory: PathBuf,
}

/// A cached PNG thumbnail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedThumbnail {
    pub path: PathBuf,
    /// True when no content preview existed and the file type icon was cached instead.
    pub is_file_type_icon: bool,
}

/// Distinguishes temporary files written at the same moment by several threads.
static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

impl ThumbnailCache {
    /// `directory` is created on first use. Only files this cache names are ever written there.
    #[must_use]
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Returns the path of a PNG thumbnail of `source_path`, rendering and storing it first when
    /// it is not cached yet. The source file is only read, never changed.
    ///
    /// # Errors
    ///
    /// Fails when the source is missing, the shell cannot render it, or the cache folder cannot
    /// be written.
    pub fn thumbnail_path_for(
        &self,
        source_path: &Path,
        size_pixels: u32,
    ) -> Result<PathBuf, ThumbnailError> {
        self.cached_thumbnail_for(source_path, size_pixels)
            .map(|thumbnail| thumbnail.path)
    }

    /// Like [`Self::thumbnail_path_for`], and also tells whether the cached picture is the file
    /// type icon rather than a preview of the content.
    ///
    /// # Errors
    ///
    /// See [`Self::thumbnail_path_for`].
    pub fn cached_thumbnail_for(
        &self,
        source_path: &Path,
        size_pixels: u32,
    ) -> Result<CachedThumbnail, ThumbnailError> {
        let stamp =
            read_file_stamp(source_path).map_err(|source| ThumbnailError::SourceNotFound {
                path: source_path.to_path_buf(),
                source,
            })?;
        let cache_key = cache_key(source_path, size_pixels, &stamp);
        let preview_path = self.directory.join(cache_file_name(&cache_key, false));
        if preview_path.is_file() {
            return Ok(CachedThumbnail {
                path: preview_path,
                is_file_type_icon: false,
            });
        }
        let icon_path = self.directory.join(cache_file_name(&cache_key, true));
        if icon_path.is_file() {
            return Ok(CachedThumbnail {
                path: icon_path,
                is_file_type_icon: true,
            });
        }

        let thumbnail = render_thumbnail_png(source_path, size_pixels)?;
        let cached_path = if thumbnail.is_file_type_icon {
            icon_path
        } else {
            preview_path
        };
        self.store(&cached_path, &thumbnail.png_bytes)
            .map_err(|source| ThumbnailError::CacheWriteFailed {
                path: cached_path.clone(),
                source,
            })?;

        Ok(CachedThumbnail {
            path: cached_path,
            is_file_type_icon: thumbnail.is_file_type_icon,
        })
    }

    /// Writes to a temporary name first and renames, so readers never see a half-written PNG.
    fn store(&self, cached_path: &Path, png_bytes: &[u8]) -> io::Result<()> {
        std::fs::create_dir_all(&self.directory)?;

        let mut temporary_name = cached_path.as_os_str().to_os_string();
        temporary_name.push(format!(
            ".{}-{}.partial",
            std::process::id(),
            TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let temporary_path = PathBuf::from(temporary_name);

        let write_result = std::fs::File::create_new(&temporary_path)
            .and_then(|mut file| file.write_all(png_bytes))
            .and_then(|()| std::fs::rename(&temporary_path, cached_path));

        if write_result.is_err() {
            let _ = std::fs::remove_file(&temporary_path);
            // Another thread may have stored the same thumbnail first; that is a success.
            if cached_path.is_file() {
                return Ok(());
            }
        }
        write_result
    }
}

/// Everything that decides whether a cached thumbnail is still the right one.
fn cache_key(source_path: &Path, size_pixels: u32, stamp: &FileStamp) -> String {
    let modified_nanoseconds = stamp
        .modified_time
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since_epoch| since_epoch.as_nanos());

    let mut hasher = Xxh3::new();
    hasher.update(source_path.as_os_str().as_encoded_bytes());
    hasher.update(&size_pixels.to_le_bytes());
    hasher.update(&stamp.size.to_le_bytes());
    hasher.update(&modified_nanoseconds.to_le_bytes());

    format!("{:032x}-{size_pixels}", hasher.digest128())
}

/// File type icons get their own name so a cache hit still knows it holds an icon.
fn cache_file_name(cache_key: &str, is_file_type_icon: bool) -> String {
    if is_file_type_icon {
        format!("{cache_key}-icon.png")
    } else {
        format!("{cache_key}.png")
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::time::Duration;

    use super::*;

    fn stamp(size: u64, modified_second: u64) -> FileStamp {
        FileStamp {
            size,
            modified_time: UNIX_EPOCH + Duration::from_secs(modified_second),
            is_directory: false,
            is_cloud_placeholder: false,
        }
    }

    #[test]
    fn cache_names_change_with_path_size_and_file_changes() {
        let path = Path::new("photo.jpg");
        let original = cache_key(path, 256, &stamp(100, 1));

        assert_eq!(original, cache_key(path, 256, &stamp(100, 1)));
        assert_ne!(
            original,
            cache_key(Path::new("other.jpg"), 256, &stamp(100, 1))
        );
        assert_ne!(original, cache_key(path, 128, &stamp(100, 1)));
        assert_ne!(original, cache_key(path, 256, &stamp(101, 1)));
        assert_ne!(original, cache_key(path, 256, &stamp(100, 2)));
        assert!(cache_file_name(&original, false).ends_with("-256.png"));
        assert!(cache_file_name(&original, true).ends_with("-256-icon.png"));
    }
}
