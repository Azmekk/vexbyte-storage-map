use std::io;
use std::mem::size_of;

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
    DeleteObject, GetDIBits, GetObjectW, HBITMAP, HGDIOBJ,
};

use crate::shell_thumbnails::ThumbnailPixels;

const BYTES_PER_PIXEL: usize = 4;

/// Deletes the bitmap when dropped, so it is freed on every path out of `take_bitmap_pixels`.
struct OwnedBitmap(HBITMAP);

impl Drop for OwnedBitmap {
    fn drop(&mut self) {
        // SAFETY: the bitmap was handed to us by the shell, is not selected into any device
        // context and is deleted exactly once.
        let _ = unsafe { DeleteObject(HGDIOBJ(self.0.0)) };
    }
}

/// Copies the pixels out of a 32-bit shell bitmap as straight (not premultiplied) RGBA, top row
/// first, and deletes the bitmap.
pub(super) fn take_bitmap_pixels(bitmap: HBITMAP) -> io::Result<ThumbnailPixels> {
    let bitmap = OwnedBitmap(bitmap);

    let mut bitmap_description = BITMAP::default();
    // SAFETY: the bitmap handle is valid and `bitmap_description` is a writable BITMAP whose
    // size is passed along.
    let copied_bytes = unsafe {
        GetObjectW(
            HGDIOBJ(bitmap.0.0),
            i32::try_from(size_of::<BITMAP>()).unwrap_or(i32::MAX),
            Some((&raw mut bitmap_description).cast()),
        )
    };
    if copied_bytes == 0 {
        return Err(io::Error::other("the shell returned an unreadable bitmap"));
    }

    let width = bitmap_description.bmWidth;
    let height = bitmap_description.bmHeight.abs();
    let (Ok(width_pixels), Ok(height_pixels)) = (u32::try_from(width), u32::try_from(height))
    else {
        return Err(io::Error::other("the shell returned a bitmap with no size"));
    };

    let mut bitmap_information = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).unwrap_or(u32::MAX),
            biWidth: width,
            // A negative height asks for a top-down bitmap, the row order PNG uses.
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..BITMAPINFOHEADER::default()
        },
        ..BITMAPINFO::default()
    };
    let mut pixels = vec![0_u8; width_pixels as usize * height_pixels as usize * BYTES_PER_PIXEL];

    // SAFETY: a memory device context compatible with the screen needs no window.
    let device_context = unsafe { CreateCompatibleDC(None) };
    // SAFETY: `pixels` holds exactly width * height 32-bit pixels, matching the requested
    // format in `bitmap_information`; the device context and bitmap are valid.
    let copied_rows = unsafe {
        GetDIBits(
            device_context,
            bitmap.0,
            0,
            height_pixels,
            Some(pixels.as_mut_ptr().cast()),
            &raw mut bitmap_information,
            DIB_RGB_COLORS,
        )
    };
    // SAFETY: the device context was created above and is deleted exactly once.
    let _ = unsafe { DeleteDC(device_context) };

    if copied_rows != height {
        return Err(io::Error::other("could not copy the thumbnail pixels"));
    }

    convert_premultiplied_bgra_to_straight_rgba(&mut pixels);
    Ok(ThumbnailPixels {
        width: width_pixels,
        height: height_pixels,
        rgba: pixels,
    })
}

/// Shell bitmaps are BGRA with premultiplied alpha. Bitmaps without an alpha channel leave every
/// alpha byte at zero; those are treated as fully opaque.
fn convert_premultiplied_bgra_to_straight_rgba(pixels: &mut [u8]) {
    let has_alpha_channel = pixels
        .as_chunks::<BYTES_PER_PIXEL>()
        .0
        .iter()
        .any(|pixel| pixel[3] != 0);

    for pixel in pixels.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
        pixel.swap(0, 2);

        if !has_alpha_channel {
            pixel[3] = u8::MAX;
            continue;
        }

        let alpha = u16::from(pixel[3]);
        if alpha != 0 && alpha != u16::from(u8::MAX) {
            for channel in &mut pixel[..3] {
                let straight = (u16::from(*channel) * u16::from(u8::MAX) + alpha / 2) / alpha;
                *channel = u8::try_from(straight).unwrap_or(u8::MAX);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn swaps_blue_and_red_and_makes_alphaless_bitmaps_opaque() {
        let mut pixels = vec![10, 20, 30, 0, 40, 50, 60, 0];

        convert_premultiplied_bgra_to_straight_rgba(&mut pixels);

        assert_eq!(pixels, vec![30, 20, 10, 255, 60, 50, 40, 255]);
    }

    #[test]
    fn removes_premultiplied_alpha() {
        let mut pixels = vec![0, 0, 64, 128, 0, 0, 0, 0];

        convert_premultiplied_bgra_to_straight_rgba(&mut pixels);

        assert_eq!(pixels, vec![128, 0, 0, 128, 0, 0, 0, 0]);
    }
}
