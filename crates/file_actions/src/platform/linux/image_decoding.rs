use std::io;
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageError, ImageFormat, ImageReader};

use crate::shell_thumbnails::ThumbnailPixels;

/// Larger files are not decoded for a thumbnail; reading them would stall the thumbnail threads.
const MAXIMUM_DECODED_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// Decodes a picture file the `image` crate supports (JPEG, PNG, WebP, GIF, BMP), turned upright
/// as its EXIF orientation says. Other types fail with [`io::ErrorKind::Unsupported`].
pub(super) fn decode_image_file(path: &Path) -> io::Result<DynamicImage> {
    let has_supported_extension =
        ImageFormat::from_path(path).is_ok_and(|format| format.reading_enabled());
    if !has_supported_extension {
        return Err(unsupported("no decoder for this file type"));
    }

    let metadata = std::fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(unsupported("only regular files are decoded"));
    }
    if metadata.len() > MAXIMUM_DECODED_FILE_BYTES {
        return Err(unsupported(
            "the file is too large to decode for a thumbnail",
        ));
    }

    let reader = ImageReader::open(path)?.with_guessed_format()?;
    let mut decoder = reader.into_decoder().map_err(to_io_error)?;
    let orientation = decoder.orientation().map_err(to_io_error)?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(to_io_error)?;
    image.apply_orientation(orientation);
    Ok(image)
}

/// Scales `image` down to fit a `size_pixels` square, keeping its aspect ratio. Smaller pictures
/// keep their size.
pub(super) fn pixels_fitting_square(image: &DynamicImage, size_pixels: u32) -> ThumbnailPixels {
    let fits_already = image.width() <= size_pixels && image.height() <= size_pixels;
    let rgba_image = if fits_already {
        image.to_rgba8()
    } else {
        image.thumbnail(size_pixels, size_pixels).into_rgba8()
    };

    ThumbnailPixels {
        width: rgba_image.width(),
        height: rgba_image.height(),
        rgba: rgba_image.into_raw(),
    }
}

fn unsupported(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

fn to_io_error(error: ImageError) -> io::Error {
    match error {
        ImageError::IoError(io_error) => io_error,
        ImageError::Unsupported(_) => io::Error::new(io::ErrorKind::Unsupported, error),
        other => io::Error::new(io::ErrorKind::InvalidData, other),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use image::RgbaImage;

    use super::*;

    #[test]
    fn scales_down_keeping_the_aspect_ratio() {
        let wide_image = DynamicImage::ImageRgba8(RgbaImage::new(400, 100));

        let pixels = pixels_fitting_square(&wide_image, 200);

        assert_eq!((pixels.width, pixels.height), (200, 50));
        assert_eq!(pixels.rgba.len(), 200 * 50 * 4);
    }

    #[test]
    fn keeps_small_pictures_at_their_size() {
        let small_image = DynamicImage::ImageRgba8(RgbaImage::new(30, 20));

        let pixels = pixels_fitting_square(&small_image, 200);

        assert_eq!((pixels.width, pixels.height), (30, 20));
    }

    #[test]
    fn refuses_unknown_file_types_without_reading_them() {
        let error =
            decode_image_file(Path::new("/vsm-not-on-disk/notes.vsm-unknown-type")).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }
}
