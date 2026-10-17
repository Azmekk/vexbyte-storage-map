use std::io;
use std::path::{Path, PathBuf};

use crate::platform::{read_file_stamp, render_shell_image};
use crate::png_encoding::encode_rgba_as_png;

/// Smallest and largest square sizes accepted; the shell renders nothing useful outside them.
pub const MINIMUM_THUMBNAIL_SIZE_PIXELS: u32 = 16;
pub const MAXIMUM_THUMBNAIL_SIZE_PIXELS: u32 = 1024;

/// Uncompressed pixels as rows of straight-alpha RGBA, top row first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailPixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Which picture the shell is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellImageKind {
    /// A preview of the content: the photo, a video frame, a document page.
    Thumbnail,
    /// The file type icon.
    Icon,
}

/// An encoded thumbnail. The shell may return a larger image than requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailPng {
    pub width: u32,
    pub height: u32,
    pub png_bytes: Vec<u8>,
    /// True when no content preview existed and the file type icon was used instead.
    pub is_file_type_icon: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ThumbnailError {
    #[error("{} could not be found", path.display())]
    SourceNotFound { path: PathBuf, source: io::Error },
    #[error("the shell could not render a thumbnail or icon for {}", path.display())]
    RenderFailed { path: PathBuf, source: io::Error },
    #[error("the thumbnail of {} could not be encoded as PNG", path.display())]
    EncodingFailed { path: PathBuf, source: io::Error },
    #[error("the thumbnail cache could not be written at {}", path.display())]
    CacheWriteFailed { path: PathBuf, source: io::Error },
}

/// Renders a PNG thumbnail of `path` about `size_pixels` square through the shell, so videos get
/// a frame and documents a page preview. Falls back to the file type icon when no preview
/// exists. Cloud placeholders always get the icon, because a preview would download them.
///
/// Uses COM on the calling thread; call it from a background thread, not the UI thread.
///
/// # Errors
///
/// Fails when the file does not exist, or the shell can produce neither preview nor icon.
pub fn render_thumbnail_png(path: &Path, size_pixels: u32) -> Result<ThumbnailPng, ThumbnailError> {
    let stamp = read_file_stamp(path).map_err(|source| ThumbnailError::SourceNotFound {
        path: path.to_path_buf(),
        source,
    })?;
    let size_pixels =
        size_pixels.clamp(MINIMUM_THUMBNAIL_SIZE_PIXELS, MAXIMUM_THUMBNAIL_SIZE_PIXELS);

    let preview = if stamp.is_cloud_placeholder {
        None
    } else {
        render_shell_image(path, size_pixels, ShellImageKind::Thumbnail).ok()
    };
    let is_file_type_icon = preview.is_none();
    let pixels = match preview {
        Some(pixels) => pixels,
        None => render_shell_image(path, size_pixels, ShellImageKind::Icon).map_err(|source| {
            ThumbnailError::RenderFailed {
                path: path.to_path_buf(),
                source,
            }
        })?,
    };

    let png_bytes =
        encode_rgba_as_png(&pixels).map_err(|source| ThumbnailError::EncodingFailed {
            path: path.to_path_buf(),
            source,
        })?;

    Ok(ThumbnailPng {
        width: pixels.width,
        height: pixels.height,
        png_bytes,
        is_file_type_icon,
    })
}
