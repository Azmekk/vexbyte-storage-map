use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use image::{DynamicImage, ImageFormat};
use md5::{Digest, Md5};

use super::file_uri::file_uri_of;

/// The size folders of the freedesktop thumbnail cache, with the longest side their thumbnails
/// have at most, smallest first.
const THUMBNAIL_SIZE_FOLDERS: [(&str, u32); 4] = [
    ("normal", 128),
    ("large", 256),
    ("x-large", 512),
    ("xx-large", 1024),
];

/// Looks for a thumbnail another application (a file manager, an image viewer) already stored
/// for `path` in the shared freedesktop thumbnail cache, large enough for `size_pixels`. Only
/// reads; a missing, stale or unreadable entry is simply not found.
pub(super) fn find_cached_freedesktop_thumbnail(
    path: &Path,
    size_pixels: u32,
) -> Option<DynamicImage> {
    let cache_root =
        thumbnail_cache_root(std::env::var_os("XDG_CACHE_HOME"), std::env::var_os("HOME"))?;
    let source_modified_seconds = std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    let thumbnail_file_name = thumbnail_file_name_of(&file_uri_of(path).ok()?);

    size_folders_large_enough_for(size_pixels)
        .map(|size_folder| cache_root.join(size_folder).join(&thumbnail_file_name))
        .find_map(|thumbnail_path| {
            read_thumbnail_if_current(&thumbnail_path, source_modified_seconds)
        })
}

/// `$XDG_CACHE_HOME/thumbnails`, or `~/.cache/thumbnails` when that is unset or not absolute.
fn thumbnail_cache_root(
    xdg_cache_home: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    let configured_cache_home = xdg_cache_home
        .map(PathBuf::from)
        .filter(|cache_home| cache_home.is_absolute());
    let cache_home = match configured_cache_home {
        Some(cache_home) => cache_home,
        None => PathBuf::from(home?).join(".cache"),
    };
    Some(cache_home.join("thumbnails"))
}

/// The cache names each thumbnail after the MD5 of the source's file URI, in lowercase hex.
fn thumbnail_file_name_of(file_uri: &str) -> String {
    let digest = Md5::digest(file_uri.as_bytes());
    format!("{digest:x}.png")
}

fn size_folders_large_enough_for(size_pixels: u32) -> impl Iterator<Item = &'static str> {
    THUMBNAIL_SIZE_FOLDERS
        .into_iter()
        .filter(move |(_, longest_side)| *longest_side >= size_pixels)
        .map(|(size_folder, _)| size_folder)
}

/// The spec requires a `Thumb::MTime` text chunk with the source's modification time in whole
/// seconds; a thumbnail without it, or with another time, is stale.
fn read_thumbnail_if_current(
    thumbnail_path: &Path,
    source_modified_seconds: u64,
) -> Option<DynamicImage> {
    let png_bytes = std::fs::read(thumbnail_path).ok()?;
    let recorded_modified_seconds = recorded_source_modified_seconds(&png_bytes)?;
    if recorded_modified_seconds != source_modified_seconds {
        return None;
    }
    image::load_from_memory_with_format(&png_bytes, ImageFormat::Png).ok()
}

fn recorded_source_modified_seconds(png_bytes: &[u8]) -> Option<u64> {
    let reader = png::Decoder::new(std::io::Cursor::new(png_bytes))
        .read_info()
        .ok()?;
    reader
        .info()
        .uncompressed_latin1_text
        .iter()
        .find(|text_chunk| text_chunk.keyword == "Thumb::MTime")
        .and_then(|text_chunk| text_chunk.text.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn names_thumbnails_after_the_md5_of_the_uri() {
        assert_eq!(
            thumbnail_file_name_of(""),
            "d41d8cd98f00b204e9800998ecf8427e.png"
        );
        assert_eq!(
            thumbnail_file_name_of("file:///home/jens/photos/me.png"),
            "c6ee772d9e49320e97ec29a7eb5b1697.png"
        );
    }

    #[test]
    fn finds_the_cache_under_xdg_cache_home_or_home() {
        assert_eq!(
            thumbnail_cache_root(
                Some("/var/cache/martin".into()),
                Some("/home/martin".into())
            ),
            Some(PathBuf::from("/var/cache/martin/thumbnails"))
        );
        assert_eq!(
            thumbnail_cache_root(Some("relative".into()), Some("/home/martin".into())),
            Some(PathBuf::from("/home/martin/.cache/thumbnails"))
        );
        assert_eq!(
            thumbnail_cache_root(None, Some("/home/martin".into())),
            Some(PathBuf::from("/home/martin/.cache/thumbnails"))
        );
        assert_eq!(thumbnail_cache_root(None, None), None);
    }

    #[test]
    fn uses_only_size_folders_large_enough() {
        assert_eq!(
            size_folders_large_enough_for(64).collect::<Vec<_>>(),
            vec!["normal", "large", "x-large", "xx-large"]
        );
        assert_eq!(
            size_folders_large_enough_for(300).collect::<Vec<_>>(),
            vec!["x-large", "xx-large"]
        );
    }

    fn png_with_text(keyword: &str, text: &str) -> Vec<u8> {
        let mut png_bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut png_bytes, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .add_text_chunk(keyword.to_owned(), text.to_owned())
            .unwrap();
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[1, 2, 3, 255]).unwrap();
        writer.finish().unwrap();
        png_bytes
    }

    #[test]
    fn reads_the_recorded_modification_time() {
        assert_eq!(
            recorded_source_modified_seconds(&png_with_text("Thumb::MTime", "1700000000")),
            Some(1_700_000_000)
        );
        assert_eq!(
            recorded_source_modified_seconds(&png_with_text("Thumb::URI", "file:///a.png")),
            None
        );
    }
}
