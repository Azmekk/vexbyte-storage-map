use std::io;
use std::path::Path;

use super::freedesktop_thumbnail_cache::find_cached_freedesktop_thumbnail;
use super::image_decoding::{decode_image_file, pixels_fitting_square};
use crate::shell_thumbnails::{ShellImageKind, ThumbnailPixels};

/// A picture of `path` about `size_pixels` square: the thumbnail another application already
/// cached, or else the decoded picture itself. File type icons are not rendered; without a
/// preview the caller reports that no thumbnail exists, and the UI shows none.
pub(crate) fn render_shell_image(
    path: &Path,
    size_pixels: u32,
    kind: ShellImageKind,
) -> io::Result<ThumbnailPixels> {
    match kind {
        ShellImageKind::Thumbnail => render_content_preview(path, size_pixels),
        ShellImageKind::Icon => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "file type icons are not rendered on this platform",
        )),
    }
}

fn render_content_preview(path: &Path, size_pixels: u32) -> io::Result<ThumbnailPixels> {
    let image = match find_cached_freedesktop_thumbnail(path, size_pixels) {
        Some(cached_thumbnail) => cached_thumbnail,
        None => decode_image_file(path)?,
    };
    Ok(pixels_fitting_square(&image, size_pixels))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::shell_thumbnails::{ThumbnailError, render_thumbnail_png};

    struct TemporaryFolder(PathBuf);

    impl TemporaryFolder {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("vsm-{name}-{}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TemporaryFolder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn file_types_without_a_preview_get_no_thumbnail() {
        let folder = TemporaryFolder::new("linux-no-preview");
        let source = folder.0.join("notes.vsm-unknown-type");
        fs::write(&source, b"no decoder knows this type").unwrap();

        let result = render_thumbnail_png(&source, 64);

        assert!(matches!(
            result,
            Err(ThumbnailError::RenderFailed { source, .. })
                if source.kind() == io::ErrorKind::Unsupported
        ));
    }

    #[test]
    fn icons_are_unsupported() {
        let error = render_shell_image(Path::new("/vsm-not-on-disk.png"), 64, ShellImageKind::Icon)
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }
}
