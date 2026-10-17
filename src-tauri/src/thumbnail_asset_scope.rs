//! Keeps cached thumbnails loadable through the asset protocol when Windows stores them somewhere
//! other than the folder the configured scope names.

use std::path::{Component, Path, PathBuf};

use tauri::scope::fs::Scope;

use crate::app_state::{APP_DATA_FOLDER_NAMES, THUMBNAIL_FOLDER_NAME};

#[derive(Debug, thiserror::Error)]
pub enum ThumbnailAssetScopeError {
    #[error("could not resolve where the thumbnail {path} is stored")]
    ResolveFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("the thumbnail {path} is not stored in a thumbnail cache folder ({resolved_path})")]
    NotInThumbnailFolder {
        path: PathBuf,
        resolved_path: PathBuf,
    },
    #[error("could not allow the thumbnail folder {directory} in the asset protocol scope")]
    AllowFailed {
        directory: PathBuf,
        #[source]
        source: tauri::Error,
    },
}

/// Makes sure the asset protocol serves the cached thumbnail at `png_path`.
///
/// The asset protocol canonicalizes every requested path and matches it against patterns that
/// were canonicalized once at startup. Packaged (MSIX) processes, and every process they start,
/// get their writes to `%LOCALAPPDATA%` redirected to
/// `%LOCALAPPDATA%\Packages\<package>\LocalCache\Local`, while reads still see the real folder.
/// When the real thumbnail folder exists, the scope pattern resolves to it but a newly written
/// PNG resolves to the redirected copy, so the protocol answers 403. Allowing the folder the PNG
/// really lives in (only its files, not its subfolders) fixes that without widening the scope
/// beyond the thumbnail cache.
pub fn allow_thumbnail_folder(
    scope: &Scope,
    png_path: &Path,
) -> Result<(), ThumbnailAssetScopeError> {
    if scope.is_allowed(png_path) {
        return Ok(());
    }

    let resolved_path = std::fs::canonicalize(png_path).map_err(|source| {
        ThumbnailAssetScopeError::ResolveFailed {
            path: png_path.to_path_buf(),
            source,
        }
    })?;
    let directory = resolved_path
        .parent()
        .filter(|directory| is_thumbnail_folder(directory))
        .ok_or_else(|| ThumbnailAssetScopeError::NotInThumbnailFolder {
            path: png_path.to_path_buf(),
            resolved_path: resolved_path.clone(),
        })?;

    scope.allow_directory(directory, false).map_err(|source| {
        ThumbnailAssetScopeError::AllowFailed {
            directory: directory.to_path_buf(),
            source,
        }
    })?;
    tracing::info!(
        directory = %directory.display(),
        "allowed the redirected thumbnail folder in the asset protocol scope"
    );
    Ok(())
}

/// True when `directory` ends in `Vexbyte\Storage Map\thumbnails`.
fn is_thumbnail_folder(directory: &Path) -> bool {
    let expected_names = APP_DATA_FOLDER_NAMES
        .iter()
        .chain(std::iter::once(&THUMBNAIL_FOLDER_NAME));
    let last_names = directory
        .components()
        .rev()
        .take(APP_DATA_FOLDER_NAMES.len() + 1)
        .collect::<Vec<_>>();

    last_names.len() == APP_DATA_FOLDER_NAMES.len() + 1
        && last_names
            .into_iter()
            .rev()
            .zip(expected_names)
            .all(|(component, expected_name)| {
                matches!(component, Component::Normal(name) if name.eq_ignore_ascii_case(expected_name))
            })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::{Path, PathBuf};

    use tauri::utils::config::FsScope;

    use super::*;
    use crate::app_state::thumbnail_directory_in;

    /// A folder under the temp directory, removed when the test ends.
    struct TemporaryFolder(PathBuf);

    impl TemporaryFolder {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "vexbyte-thumbnail-scope-{name}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TemporaryFolder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn write_png(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"\x89PNG").unwrap();
    }

    /// The asset protocol scope from `tauri.conf.json`, with `$LOCALDATA` standing for
    /// `local_data_directory`.
    fn configured_scope(local_data_directory: &Path) -> Scope {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let asset_scope = &config["app"]["security"]["assetProtocol"]["scope"];
        let local_data = local_data_directory.to_str().unwrap();
        let allow = asset_scope["allow"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pattern| {
                let pattern = pattern.as_str().unwrap();
                assert!(
                    pattern.starts_with("$LOCALDATA/"),
                    "unexpected pattern {pattern}"
                );
                PathBuf::from(pattern.replacen("$LOCALDATA", local_data, 1))
            })
            .collect();
        let fs_scope = FsScope::Scope {
            allow,
            deny: Vec::new(),
            require_literal_leading_dot: asset_scope["requireLiteralLeadingDot"].as_bool(),
        };
        Scope::new(&tauri::test::mock_app(), &fs_scope).unwrap()
    }

    #[test]
    fn the_configured_scope_allows_the_thumbnails_the_app_writes() {
        let local_data = TemporaryFolder::new("configured");
        let thumbnail_directory = thumbnail_directory_in(&local_data.0);
        let preview = thumbnail_directory.join("000bf2e1d903b5cf91e6f13b5153dd76-128.png");
        let icon = thumbnail_directory.join("000bf2e1d903b5cf91e6f13b5153dd76-128-icon.png");
        write_png(&preview);
        write_png(&icon);
        let scope = configured_scope(&local_data.0);

        assert!(scope.is_allowed(&preview));
        assert!(scope.is_allowed(&icon));
        assert!(
            !scope.is_allowed(
                local_data
                    .0
                    .join("Vexbyte")
                    .join("Storage Map")
                    .join("x.png")
            )
        );
    }

    #[test]
    fn a_thumbnail_stored_in_a_redirected_folder_becomes_allowed() {
        let root = TemporaryFolder::new("redirected");
        // The scope names the real folder; the PNG lands in the redirected copy.
        let scope = configured_scope(&root.0.join("real"));
        std::fs::create_dir_all(thumbnail_directory_in(&root.0.join("real"))).unwrap();
        let redirected_directory = thumbnail_directory_in(&root.0.join("redirected"));
        let thumbnail = redirected_directory.join("ef579a2a16ef1d31ab2f484b9d803241-128.png");
        let nested_file = redirected_directory.join("nested").join("other.png");
        write_png(&thumbnail);
        write_png(&nested_file);
        assert!(!scope.is_allowed(&thumbnail));

        allow_thumbnail_folder(&scope, &thumbnail).unwrap();

        assert!(scope.is_allowed(&thumbnail));
        assert!(!scope.is_allowed(&nested_file));
        assert!(!scope.is_allowed(redirected_directory.parent().unwrap().join("x.png")));
    }

    #[test]
    fn a_file_outside_a_thumbnail_folder_is_never_allowed() {
        let root = TemporaryFolder::new("outside");
        let scope = configured_scope(&root.0.join("real"));
        let stray_file = root.0.join("Documents").join("photo.png");
        write_png(&stray_file);

        let result = allow_thumbnail_folder(&scope, &stray_file);

        assert!(matches!(
            result,
            Err(ThumbnailAssetScopeError::NotInThumbnailFolder { .. })
        ));
        assert!(!scope.is_allowed(&stray_file));
    }
}
