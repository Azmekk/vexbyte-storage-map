use std::io;

use crate::shell_thumbnails::ThumbnailPixels;

pub(crate) fn encode_rgba_as_png(pixels: &ThumbnailPixels) -> io::Result<Vec<u8>> {
    let mut png_bytes = Vec::new();

    let mut encoder = png::Encoder::new(&mut png_bytes, pixels.width, pixels.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    // Thumbnails are written once and shown many times, but encoding happens while the user
    // waits; the fast setting is a good balance.
    encoder.set_compression(png::Compression::Fast);

    let mut writer = encoder.write_header().map_err(io::Error::other)?;
    writer
        .write_image_data(&pixels.rgba)
        .map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;

    Ok(png_bytes)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn encodes_pixels_that_decode_back_identically() {
        let pixels = ThumbnailPixels {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 0, 255, 128],
        };

        let png_bytes = encode_rgba_as_png(&pixels).unwrap();

        let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut decoded_pixels = vec![0; reader.output_buffer_size().unwrap()];
        let frame = reader.next_frame(&mut decoded_pixels).unwrap();
        assert_eq!((frame.width, frame.height), (2, 1));
        assert_eq!(&decoded_pixels[..frame.buffer_size()], &pixels.rgba[..]);
    }

    #[test]
    fn rejects_pixel_buffers_of_the_wrong_length() {
        let pixels = ThumbnailPixels {
            width: 2,
            height: 2,
            rgba: vec![0; 3],
        };

        assert!(encode_rgba_as_png(&pixels).is_err());
    }
}
