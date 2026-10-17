//! Renders shell thumbnails of files this test creates in its own temporary folder. The source
//! files are only read; the folder is removed at the end.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use file_actions::{ThumbnailCache, render_thumbnail_png};

struct TemporaryFolder(PathBuf);

impl TemporaryFolder {
    fn new() -> Self {
        let nanoseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "vsm-thumbnail-test-{}-{nanoseconds}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TemporaryFolder {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!("could not remove {}: {error}", self.0.display());
        }
    }
}

fn write_gradient_png(path: &Path, side: u32) {
    let mut rgba = Vec::new();
    for row in 0..side {
        for column in 0..side {
            let red = u8::try_from(column * 4 % 256).unwrap();
            let green = u8::try_from(row * 4 % 256).unwrap();
            rgba.extend_from_slice(&[red, green, 128, 255]);
        }
    }

    let file = fs::File::create(path).unwrap();
    let mut encoder = png::Encoder::new(file, side, side);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&rgba).unwrap();
    writer.finish().unwrap();
}

fn decoded_size(png_bytes: &[u8]) -> (u32, u32) {
    let reader = png::Decoder::new(std::io::Cursor::new(png_bytes))
        .read_info()
        .unwrap();
    let information = reader.info();
    (information.width, information.height)
}

#[test]
fn renders_a_thumbnail_of_a_png_without_touching_it() {
    let folder = TemporaryFolder::new();
    let source = folder.0.join("gradient.png");
    write_gradient_png(&source, 64);
    let original_bytes = fs::read(&source).unwrap();
    let original_modified = fs::metadata(&source).unwrap().modified().unwrap();

    let thumbnail = render_thumbnail_png(&source, 48).unwrap();

    assert!(thumbnail.width > 0 && thumbnail.height > 0);
    assert_eq!(
        decoded_size(&thumbnail.png_bytes),
        (thumbnail.width, thumbnail.height)
    );
    assert_eq!(fs::read(&source).unwrap(), original_bytes);
    assert_eq!(
        fs::metadata(&source).unwrap().modified().unwrap(),
        original_modified
    );
}

#[test]
fn falls_back_to_the_file_type_icon() {
    let folder = TemporaryFolder::new();
    let source = folder.0.join("notes.vsm-unknown-type");
    fs::write(&source, b"no previewer knows this type").unwrap();

    let thumbnail = render_thumbnail_png(&source, 64).unwrap();

    assert!(thumbnail.is_file_type_icon);
    assert!(thumbnail.width > 0 && thumbnail.height > 0);
}

#[test]
fn caches_thumbnails_as_png_files_and_reuses_them() {
    let folder = TemporaryFolder::new();
    let source = folder.0.join("gradient.png");
    write_gradient_png(&source, 64);
    let cache = ThumbnailCache::new(folder.0.join("cache"));

    let first_path = cache.thumbnail_path_for(&source, 64).unwrap();
    let first_written = fs::metadata(&first_path).unwrap().modified().unwrap();
    let second_path = cache.thumbnail_path_for(&source, 64).unwrap();
    let other_size_path = cache.thumbnail_path_for(&source, 32).unwrap();

    assert!(first_path.starts_with(cache.directory()));
    assert_eq!(first_path, second_path);
    assert_eq!(
        fs::metadata(&second_path).unwrap().modified().unwrap(),
        first_written
    );
    assert_ne!(first_path, other_size_path);
    let (width, height) = decoded_size(&fs::read(&first_path).unwrap());
    assert!(width > 0 && height > 0);
    let leftover_partial_files = fs::read_dir(cache.directory())
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".partial")
        })
        .count();
    assert_eq!(leftover_partial_files, 0);
}

#[test]
fn reports_missing_sources() {
    let folder = TemporaryFolder::new();

    let result = render_thumbnail_png(&folder.0.join("missing.png"), 64);

    assert!(matches!(
        result,
        Err(file_actions::ThumbnailError::SourceNotFound { .. })
    ));
}
