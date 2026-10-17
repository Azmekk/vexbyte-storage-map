use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;

/// A personal folder offered on the duplicate search setup screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedFolder {
    /// The English display name, for example `Pictures`.
    pub name: String,
    pub path: PathBuf,
}

/// The current user's Pictures, Videos, Music, Documents, Downloads and Desktop folders, in that
/// order. Folders the system does not define, or that do not exist, are left out.
#[must_use]
pub fn suggested_duplicate_folders() -> Vec<SuggestedFolder> {
    let candidates = [
        ("Pictures", dirs::picture_dir()),
        ("Videos", dirs::video_dir()),
        ("Music", dirs::audio_dir()),
        ("Documents", dirs::document_dir()),
        ("Downloads", dirs::download_dir()),
        ("Desktop", dirs::desktop_dir()),
    ];

    let mut seen_paths = Vec::new();
    let mut suggested_folders = Vec::new();
    for (name, path) in candidates {
        let Some(path) = path else { continue };
        if !path.is_dir() || seen_paths.contains(&path) {
            continue;
        }
        seen_paths.push(path.clone());
        suggested_folders.push(SuggestedFolder {
            name: name.to_owned(),
            path,
        });
    }

    suggested_folders
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn suggests_existing_folders_without_repeats() {
        let suggested_folders = suggested_duplicate_folders();

        for (index, folder) in suggested_folders.iter().enumerate() {
            assert!(folder.path.is_dir(), "{}", folder.path.display());
            assert!(
                suggested_folders[index + 1..]
                    .iter()
                    .all(|other| other.path != folder.path)
            );
        }
    }
}
