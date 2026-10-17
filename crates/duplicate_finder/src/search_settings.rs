use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use file_categories::{FileCategory, extensions_for};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::directory_entry::EntryKind;
use crate::platform::{is_same_or_inside_folder, normalize_folder_path, read_file_stamp};
use crate::system_folder_guard::SystemFolderGuard;

/// Files smaller than this are ignored unless the user lowers the limit: tiny files waste little
/// space and are often legitimately duplicated (icons, templates).
pub const DEFAULT_MINIMUM_FILE_SIZE_BYTES: u64 = 64 * 1024;

/// What the user asked to search: which folders and which kinds of files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSearchSettings {
    pub folders: Vec<PathBuf>,
    pub categories: Vec<FileCategory>,
    #[serde(default = "default_minimum_file_size_bytes")]
    pub minimum_file_size_bytes: u64,
}

fn default_minimum_file_size_bytes() -> u64 {
    DEFAULT_MINIMUM_FILE_SIZE_BYTES
}

#[derive(Debug, thiserror::Error)]
pub enum SearchSettingsError {
    #[error("no folders were selected")]
    NoFoldersSelected,
    #[error("no file categories were selected")]
    NoCategoriesSelected,
    #[error("the folder {} could not be found", folder.display())]
    FolderNotFound { folder: PathBuf, source: io::Error },
    #[error("{} is a file, not a folder", folder.display())]
    NotAFolder { folder: PathBuf },
    #[error("{} is a system or application folder and cannot be searched", folder.display())]
    SystemFolderNotAllowed { folder: PathBuf },
}

/// Settings that passed validation: folders exist, are absolute, contain no system folder and do
/// not overlap, and the selected categories are expanded to extensions.
#[derive(Debug, Clone)]
pub struct ValidatedSearchSettings {
    pub(crate) folders: Vec<PathBuf>,
    pub(crate) extensions: HashSet<String>,
    pub(crate) minimum_file_size_bytes: u64,
    pub(crate) system_folder_guard: SystemFolderGuard,
}

impl ValidatedSearchSettings {
    /// The folders that will be walked, normalized, with folders nested inside another selected
    /// folder removed.
    #[must_use]
    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }
}

impl DuplicateSearchSettings {
    #[must_use]
    pub fn new(folders: Vec<PathBuf>, categories: Vec<FileCategory>) -> Self {
        Self {
            folders,
            categories,
            minimum_file_size_bytes: DEFAULT_MINIMUM_FILE_SIZE_BYTES,
        }
    }

    /// Checks the settings against this computer's system folders.
    ///
    /// # Errors
    ///
    /// See [`SearchSettingsError`] for every reason settings are rejected.
    pub fn validate(&self) -> Result<ValidatedSearchSettings, SearchSettingsError> {
        self.validate_with_guard(SystemFolderGuard::for_this_computer())
    }

    /// Like [`validate`](Self::validate), with an explicit list of system folders.
    ///
    /// # Errors
    ///
    /// See [`SearchSettingsError`] for every reason settings are rejected.
    pub fn validate_with_guard(
        &self,
        system_folder_guard: SystemFolderGuard,
    ) -> Result<ValidatedSearchSettings, SearchSettingsError> {
        if self.folders.is_empty() {
            return Err(SearchSettingsError::NoFoldersSelected);
        }
        if self.categories.is_empty() {
            return Err(SearchSettingsError::NoCategoriesSelected);
        }

        let mut normalized_folders = Vec::with_capacity(self.folders.len());
        for folder in &self.folders {
            let normalized_folder = validate_folder(folder, &system_folder_guard)?;
            normalized_folders.push(normalized_folder);
        }

        Ok(ValidatedSearchSettings {
            folders: remove_nested_folders(normalized_folders),
            extensions: extensions_of_categories(&self.categories),
            minimum_file_size_bytes: self.minimum_file_size_bytes,
            system_folder_guard,
        })
    }
}

fn validate_folder(
    folder: &Path,
    system_folder_guard: &SystemFolderGuard,
) -> Result<PathBuf, SearchSettingsError> {
    let normalized_folder =
        normalize_folder_path(folder).map_err(|source| SearchSettingsError::FolderNotFound {
            folder: folder.to_path_buf(),
            source,
        })?;

    let stamp = read_file_stamp(&normalized_folder).map_err(|source| {
        SearchSettingsError::FolderNotFound {
            folder: folder.to_path_buf(),
            source,
        }
    })?;
    if stamp.kind != EntryKind::Directory {
        return Err(SearchSettingsError::NotAFolder {
            folder: folder.to_path_buf(),
        });
    }

    let system_folder_error = || SearchSettingsError::SystemFolderNotAllowed {
        folder: folder.to_path_buf(),
    };
    if system_folder_guard.is_system_path(&normalized_folder) {
        return Err(system_folder_error());
    }

    // A junction or symlink can make an innocent-looking folder point into a system folder.
    let link_target = std::fs::canonicalize(&normalized_folder)
        .ok()
        .and_then(|canonical_folder| normalize_folder_path(&canonical_folder).ok());
    if let Some(link_target) = link_target
        && system_folder_guard.is_system_path(&link_target)
    {
        return Err(system_folder_error());
    }

    Ok(normalized_folder)
}

/// Drops duplicates and folders that lie inside another selected folder, so no file is collected
/// twice.
fn remove_nested_folders(mut folders: Vec<PathBuf>) -> Vec<PathBuf> {
    folders.sort_by_key(|folder| folder.components().count());

    let mut kept_folders: Vec<PathBuf> = Vec::with_capacity(folders.len());
    for folder in folders {
        let is_covered = kept_folders
            .iter()
            .any(|kept_folder| is_same_or_inside_folder(&folder, kept_folder));
        if !is_covered {
            kept_folders.push(folder);
        }
    }

    kept_folders
}

fn extensions_of_categories(categories: &[FileCategory]) -> HashSet<String> {
    categories
        .iter()
        .flat_map(|&category| extensions_for(category).iter())
        .map(|extension| (*extension).to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn deserializes_with_the_default_minimum_size() {
        let settings: DuplicateSearchSettings =
            serde_json::from_str(r#"{"folders":["C:\\Photos"],"categories":["images"]}"#).unwrap();

        assert_eq!(settings.minimum_file_size_bytes, 64 * 1024);
        assert_eq!(settings.categories, vec![FileCategory::Images]);
    }

    #[test]
    fn rejects_empty_folder_and_category_lists() {
        let no_folders = DuplicateSearchSettings::new(vec![], vec![FileCategory::Images]);
        let no_categories = DuplicateSearchSettings::new(vec![std::env::temp_dir()], vec![]);

        assert!(matches!(
            no_folders.validate(),
            Err(SearchSettingsError::NoFoldersSelected)
        ));
        assert!(matches!(
            no_categories.validate(),
            Err(SearchSettingsError::NoCategoriesSelected)
        ));
    }

    #[test]
    fn removes_nested_and_repeated_folders() {
        let root = std::env::temp_dir();
        let folders = vec![
            root.join("a").join("b"),
            root.join("a"),
            root.join("A"),
            root.join("c"),
        ];

        let kept_folders = remove_nested_folders(folders);

        assert_eq!(kept_folders, vec![root.join("a"), root.join("c")]);
    }

    #[test]
    fn expands_categories_to_extensions() {
        let extensions = extensions_of_categories(&[FileCategory::Pdfs, FileCategory::Videos]);

        assert!(extensions.contains("pdf"));
        assert!(extensions.contains("mp4"));
        assert!(!extensions.contains("jpg"));
    }
}
