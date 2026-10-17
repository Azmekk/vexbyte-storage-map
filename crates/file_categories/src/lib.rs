//! The single place that maps file categories to file extensions.
//!
//! Shared by the disk scanner (file type breakdowns, largest files filter) and the duplicate
//! finder (which file types to compare).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use serde::{Deserialize, Serialize};
use specta::Type;

/// A group of file types, used to filter and color files by what they contain.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type,
)]
#[serde(rename_all = "camelCase")]
pub enum FileCategory {
    Images,
    Videos,
    Audio,
    Pdfs,
    Documents,
    Spreadsheets,
    Presentations,
    ArchivesAndInstallers,
}

impl FileCategory {
    pub const ALL: [FileCategory; 8] = [
        FileCategory::Images,
        FileCategory::Videos,
        FileCategory::Audio,
        FileCategory::Pdfs,
        FileCategory::Documents,
        FileCategory::Spreadsheets,
        FileCategory::Presentations,
        FileCategory::ArchivesAndInstallers,
    ];
}

const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "bmp", "tif", "tiff", "cr2", "cr3", "nef",
    "arw", "dng",
];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "mkv", "avi", "wmv", "webm", "m4v", "mts"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "m4a", "aac", "ogg", "wma", "opus"];
const PDF_EXTENSIONS: &[&str] = &["pdf"];
const DOCUMENT_EXTENSIONS: &[&str] = &["doc", "docx", "odt", "rtf", "txt", "md", "epub"];
const SPREADSHEET_EXTENSIONS: &[&str] = &["xls", "xlsx", "xlsm", "ods", "csv"];
const PRESENTATION_EXTENSIONS: &[&str] = &["ppt", "pptx", "odp"];
const ARCHIVE_AND_INSTALLER_EXTENSIONS: &[&str] =
    &["zip", "rar", "7z", "tar", "gz", "iso", "exe", "msi"];

/// Lowercase extensions, without the leading dot, that belong to `category`.
#[must_use]
pub fn extensions_for(category: FileCategory) -> &'static [&'static str] {
    match category {
        FileCategory::Images => IMAGE_EXTENSIONS,
        FileCategory::Videos => VIDEO_EXTENSIONS,
        FileCategory::Audio => AUDIO_EXTENSIONS,
        FileCategory::Pdfs => PDF_EXTENSIONS,
        FileCategory::Documents => DOCUMENT_EXTENSIONS,
        FileCategory::Spreadsheets => SPREADSHEET_EXTENSIONS,
        FileCategory::Presentations => PRESENTATION_EXTENSIONS,
        FileCategory::ArchivesAndInstallers => ARCHIVE_AND_INSTALLER_EXTENSIONS,
    }
}

/// Finds the category of a file extension, ignoring case and an optional leading dot.
#[must_use]
pub fn category_for_extension(extension: &str) -> Option<FileCategory> {
    let extension_without_dot = extension.strip_prefix('.').unwrap_or(extension);

    FileCategory::ALL.into_iter().find(|&category| {
        extensions_for(category)
            .iter()
            .any(|known_extension| known_extension.eq_ignore_ascii_case(extension_without_dot))
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::collections::HashSet;

    use super::*;

    #[test]
    fn finds_category_for_known_extensions() {
        assert_eq!(category_for_extension("jpg"), Some(FileCategory::Images));
        assert_eq!(category_for_extension("mkv"), Some(FileCategory::Videos));
        assert_eq!(category_for_extension("flac"), Some(FileCategory::Audio));
        assert_eq!(category_for_extension("pdf"), Some(FileCategory::Pdfs));
        assert_eq!(
            category_for_extension("docx"),
            Some(FileCategory::Documents)
        );
        assert_eq!(
            category_for_extension("csv"),
            Some(FileCategory::Spreadsheets)
        );
        assert_eq!(
            category_for_extension("pptx"),
            Some(FileCategory::Presentations)
        );
        assert_eq!(
            category_for_extension("7z"),
            Some(FileCategory::ArchivesAndInstallers)
        );
    }

    #[test]
    fn ignores_case_and_leading_dot() {
        assert_eq!(category_for_extension("JPG"), Some(FileCategory::Images));
        assert_eq!(category_for_extension(".Cr3"), Some(FileCategory::Images));
        assert_eq!(
            category_for_extension(".MSI"),
            Some(FileCategory::ArchivesAndInstallers)
        );
    }

    #[test]
    fn returns_none_for_unknown_or_empty_extensions() {
        assert_eq!(category_for_extension("dll"), None);
        assert_eq!(category_for_extension(""), None);
        assert_eq!(category_for_extension("."), None);
    }

    #[test]
    fn every_extension_is_lowercase_without_dot_and_in_exactly_one_category() {
        let mut seen_extensions = HashSet::new();

        for category in FileCategory::ALL {
            for extension in extensions_for(category) {
                assert_eq!(*extension, extension.to_ascii_lowercase());
                assert!(!extension.starts_with('.'));
                assert!(
                    seen_extensions.insert(*extension),
                    "{extension} is listed twice"
                );
            }
        }
    }
}
