use std::io;
use std::path::Path;

use windows::Win32::Foundation::SIZE;
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF, SIIGBF_BIGGERSIZEOK,
    SIIGBF_ICONONLY, SIIGBF_THUMBNAILONLY,
};
use windows::core::PCWSTR;

use super::bitmap_pixels::take_bitmap_pixels;
use super::com_apartment::ComApartment;
use super::io_error::to_io_error;
use super::wide_path::to_shell_wide_path;
use crate::shell_thumbnails::{ShellImageKind, ThumbnailPixels};

/// Asks the shell for a picture of `path` about `size_pixels` square: a real thumbnail (photo,
/// video frame, document page) or the file type icon.
pub(crate) fn render_shell_image(
    path: &Path,
    size_pixels: u32,
    kind: ShellImageKind,
) -> io::Result<ThumbnailPixels> {
    let _com_apartment = ComApartment::enter_single_threaded()?;
    let wide_path = to_shell_wide_path(path)?;

    // SAFETY: `wide_path` is null-terminated and outlives the call; no bind context is passed.
    let image_factory: IShellItemImageFactory =
        unsafe { SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None) }
            .map_err(|error| to_io_error(&error))?;

    let side = i32::try_from(size_pixels).unwrap_or(i32::MAX);
    let requested_size = SIZE { cx: side, cy: side };
    let flags = image_flags_for(kind);

    // SAFETY: `image_factory` is a live COM object on this apartment; the returned bitmap is
    // owned by us and handed to `take_bitmap_pixels`, which deletes it.
    let bitmap = unsafe { image_factory.GetImage(requested_size, flags) }
        .map_err(|error| to_io_error(&error))?;

    take_bitmap_pixels(bitmap)
}

fn image_flags_for(kind: ShellImageKind) -> SIIGBF {
    match kind {
        ShellImageKind::Thumbnail => SIIGBF_BIGGERSIZEOK | SIIGBF_THUMBNAILONLY,
        ShellImageKind::Icon => SIIGBF_BIGGERSIZEOK | SIIGBF_ICONONLY,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use crate::shell_thumbnails::render_thumbnail_png;

    #[test]
    fn falls_back_to_the_file_type_icon() {
        let folder = std::env::temp_dir().join(format!("vsm-icon-fallback-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let source = folder.join("notes.vsm-unknown-type");
        std::fs::write(&source, b"no previewer knows this type").unwrap();

        let thumbnail = render_thumbnail_png(&source, 64);
        std::fs::remove_dir_all(&folder).unwrap();
        let thumbnail = thumbnail.unwrap();

        assert!(thumbnail.is_file_type_icon);
        assert!(thumbnail.width > 0 && thumbnail.height > 0);
    }
}
