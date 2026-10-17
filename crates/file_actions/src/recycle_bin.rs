use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use specta::Type;

use crate::file_stamp::FileStamp;
use crate::platform::{path_exists, read_file_stamp, volume_has_recycle_bin};

/// What happened to each path handed to [`move_to_recycle_bin`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecycleResult {
    pub moved: Vec<PathBuf>,
    pub failed: Vec<RecycleFailure>,
    /// Sum of the sizes of the moved files. Folders count as zero; callers that know a folder's
    /// size (the disk viewer's tree) add it themselves.
    pub bytes_freed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecycleFailure {
    pub path: PathBuf,
    pub reason: RecycleFailureReason,
    /// The system's error message, when there is one.
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RecycleFailureReason {
    /// Only absolute paths are accepted, so nothing is resolved against the working directory.
    NotAbsolute,
    NotFound,
    /// Removable drives and network shares have no Recycle Bin; the file would be deleted for
    /// good, so it is left alone.
    NoRecycleBinOnDrive,
    /// A name ends with a dot or a space. The shell drops those when it parses a path, so it
    /// would act on a different item (`notes.` would become `notes`); the item is left alone.
    NameNotSupportedByShell,
    /// The shell reported an error, or the user cancelled a shell prompt, and the file is still
    /// in place.
    StillInPlace,
}

#[derive(Debug, thiserror::Error)]
pub enum RecycleBinError {
    #[error("the Recycle Bin operation stopped unexpectedly")]
    OperationThreadFailed,
}

/// Moves files and folders to the Recycle Bin, where the user can restore them.
///
/// Paths that are relative, missing, or on a drive without a Recycle Bin are refused up front and
/// reported in [`RecycleResult::failed`]; nothing is ever deleted permanently on purpose. The rest
/// go to the shell in one operation (through the `trash` crate, which uses `IFileOperation` with
/// undo enabled on Windows). Afterwards every path is checked: gone means moved, still present
/// means failed.
///
/// The shell work runs on a short-lived thread of its own, so the caller's COM state does not
/// matter. Call it from a background thread; it blocks until the shell is done.
///
/// **Verified by hand only.** No automated test executes this function, by project rule; the
/// pure planning and reporting logic around it is unit-tested.
///
/// # Errors
///
/// Fails only if the operation thread could not run to completion.
pub fn move_to_recycle_bin(paths: &[PathBuf]) -> Result<RecycleResult, RecycleBinError> {
    let preparation = prepare_recycle(
        paths,
        |path| read_file_stamp(path).ok(),
        volume_has_recycle_bin,
    );
    if preparation.paths_to_move.is_empty() {
        return Ok(summarize_recycle_outcome(preparation, None, path_exists));
    }

    let paths_to_move: Vec<&Path> = preparation
        .paths_to_move
        .iter()
        .map(|planned| planned.path.as_path())
        .collect();
    let operation_error = std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("vsm-recycle-bin".to_owned())
            .spawn_scoped(scope, || trash::delete_all(&paths_to_move))
            .map_err(|_| RecycleBinError::OperationThreadFailed)?
            .join()
            .map_err(|_| RecycleBinError::OperationThreadFailed)
    })?
    .err()
    .map(|error| error.to_string());

    Ok(summarize_recycle_outcome(
        preparation,
        operation_error.as_deref(),
        path_exists,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlannedMove {
    path: PathBuf,
    size: u64,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct RecyclePreparation {
    paths_to_move: Vec<PlannedMove>,
    refused: Vec<RecycleFailure>,
}

/// Decides which paths may be handed to the shell. Pure apart from the two lookups passed in.
fn prepare_recycle(
    paths: &[PathBuf],
    read_stamp: impl Fn(&Path) -> Option<FileStamp>,
    has_recycle_bin: impl Fn(&Path) -> bool,
) -> RecyclePreparation {
    let mut preparation = RecyclePreparation::default();
    let mut seen_paths = HashSet::new();

    for path in paths {
        if !seen_paths.insert(path) {
            continue;
        }

        let refusal_reason = if !path.is_absolute() {
            Some(RecycleFailureReason::NotAbsolute)
        } else if has_name_the_shell_would_rewrite(path) {
            Some(RecycleFailureReason::NameNotSupportedByShell)
        } else {
            match read_stamp(path) {
                None => Some(RecycleFailureReason::NotFound),
                Some(_) if !has_recycle_bin(path) => {
                    Some(RecycleFailureReason::NoRecycleBinOnDrive)
                }
                Some(stamp) => {
                    let size = if stamp.is_directory { 0 } else { stamp.size };
                    preparation.paths_to_move.push(PlannedMove {
                        path: path.clone(),
                        size,
                    });
                    None
                }
            }
        };

        if let Some(reason) = refusal_reason {
            preparation.refused.push(RecycleFailure {
                path: path.clone(),
                reason,
                detail: None,
            });
        }
    }

    preparation
}

/// Win32 path parsing, which the shell uses, strips trailing dots and spaces from every name.
fn has_name_the_shell_would_rewrite(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(name) => {
            let name = name.to_string_lossy();
            name.ends_with('.') || name.ends_with(' ')
        }
        _ => false,
    })
}

fn summarize_recycle_outcome(
    preparation: RecyclePreparation,
    operation_error: Option<&str>,
    still_exists: impl Fn(&Path) -> bool,
) -> RecycleResult {
    let mut result = RecycleResult {
        failed: preparation.refused,
        ..RecycleResult::default()
    };

    for planned_move in preparation.paths_to_move {
        if still_exists(&planned_move.path) {
            result.failed.push(RecycleFailure {
                path: planned_move.path,
                reason: RecycleFailureReason::StillInPlace,
                detail: operation_error.map(str::to_owned),
            });
        } else {
            result.bytes_freed += planned_move.size;
            result.moved.push(planned_move.path);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::time::UNIX_EPOCH;

    use super::*;

    fn absolute(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join("vsm-recycle-plan-not-on-disk")
            .join(name)
    }

    fn file_of_size(size: u64) -> FileStamp {
        FileStamp {
            size,
            modified_time: UNIX_EPOCH,
            is_directory: false,
            is_cloud_placeholder: false,
        }
    }

    #[test]
    fn refuses_relative_missing_and_unrecyclable_paths() {
        let removable_drive_file = absolute("on-usb-stick.jpg");
        let paths = vec![
            PathBuf::from("relative.jpg"),
            absolute("missing.jpg"),
            removable_drive_file.clone(),
            absolute("fine.jpg"),
            absolute("fine.jpg"),
        ];

        let preparation = prepare_recycle(
            &paths,
            |path| (!path.ends_with("missing.jpg")).then(|| file_of_size(10)),
            |path| path != removable_drive_file,
        );

        let refusals: Vec<RecycleFailureReason> = preparation
            .refused
            .iter()
            .map(|failure| failure.reason)
            .collect();
        assert_eq!(
            refusals,
            vec![
                RecycleFailureReason::NotAbsolute,
                RecycleFailureReason::NotFound,
                RecycleFailureReason::NoRecycleBinOnDrive,
            ]
        );
        assert_eq!(
            preparation.paths_to_move,
            vec![PlannedMove {
                path: absolute("fine.jpg"),
                size: 10
            }]
        );
    }

    #[test]
    fn refuses_names_the_shell_would_turn_into_another_item() {
        let paths = vec![
            absolute("notes."),
            absolute("folder ").join("inside.jpg"),
            absolute("photo.jpg"),
        ];

        let preparation = prepare_recycle(&paths, |_| Some(file_of_size(1)), |_| true);

        let refusals: Vec<RecycleFailureReason> = preparation
            .refused
            .iter()
            .map(|failure| failure.reason)
            .collect();
        assert_eq!(
            refusals,
            vec![
                RecycleFailureReason::NameNotSupportedByShell,
                RecycleFailureReason::NameNotSupportedByShell,
            ]
        );
        assert_eq!(preparation.paths_to_move.len(), 1);
        assert_eq!(preparation.paths_to_move[0].path, absolute("photo.jpg"));
    }

    #[test]
    fn folders_count_as_zero_bytes() {
        let folder_stamp = FileStamp {
            is_directory: true,
            ..file_of_size(4096)
        };

        let preparation = prepare_recycle(&[absolute("folder")], |_| Some(folder_stamp), |_| true);

        assert_eq!(preparation.paths_to_move[0].size, 0);
    }

    #[test]
    fn reports_moved_and_remaining_files_after_the_operation() {
        let preparation = RecyclePreparation {
            paths_to_move: vec![
                PlannedMove {
                    path: absolute("moved.jpg"),
                    size: 100,
                },
                PlannedMove {
                    path: absolute("locked.jpg"),
                    size: 50,
                },
            ],
            refused: vec![RecycleFailure {
                path: PathBuf::from("relative.jpg"),
                reason: RecycleFailureReason::NotAbsolute,
                detail: None,
            }],
        };

        let result = summarize_recycle_outcome(preparation, Some("file in use"), |path| {
            path.ends_with("locked.jpg")
        });

        assert_eq!(result.moved, vec![absolute("moved.jpg")]);
        assert_eq!(result.bytes_freed, 100);
        assert_eq!(result.failed.len(), 2);
        assert_eq!(result.failed[1].path, absolute("locked.jpg"));
        assert_eq!(result.failed[1].reason, RecycleFailureReason::StillInPlace);
        assert_eq!(result.failed[1].detail.as_deref(), Some("file in use"));
    }
}
