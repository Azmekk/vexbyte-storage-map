use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;
use specta::Type;

use crate::directory_entry::{EntryFlags, EntryKind, FileStamp};
use crate::duplicate_group::DuplicateGroupId;
use crate::platform::read_file_stamp;
use crate::removal_plan::{PlannedRemoval, RemovalPlan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FileChangeReason {
    /// The file was deleted, renamed or cannot be read any more.
    Missing,
    SizeChanged,
    ModifiedTimeChanged,
    /// The file was turned into a cloud placeholder since it was compared.
    BecameOnlineOnly,
    /// No kept copy of this group is still there unchanged, so removing this one could lose the
    /// content for good.
    NoKeptCopyLeft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: PathBuf,
    pub reason: FileChangeReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevalidationOutcome {
    /// Files that still match what was compared and may be removed.
    pub still_valid: Vec<PlannedRemoval>,
    /// Files that must be skipped and reported.
    pub changed: Vec<ChangedFile>,
}

impl RevalidationOutcome {
    #[must_use]
    pub fn paths_to_remove(&self) -> Vec<PathBuf> {
        self.still_valid
            .iter()
            .map(|removal| removal.path.clone())
            .collect()
    }
}

/// Re-reads size and modification time of every planned file right before removal. A file that
/// changed is skipped, and a whole group is skipped when none of its kept copies is still intact.
/// Only file system metadata is read; no file is opened.
#[must_use]
pub fn revalidate_before_removal(plan: &RemovalPlan) -> RevalidationOutcome {
    revalidate_with_stamp_reader(plan, |path| read_file_stamp(path).ok())
}

pub(crate) fn revalidate_with_stamp_reader(
    plan: &RemovalPlan,
    read_stamp: impl Fn(&Path) -> Option<FileStamp>,
) -> RevalidationOutcome {
    let mut groups_with_intact_kept_copy = HashSet::new();
    let mut groups_without_intact_kept_copy = HashSet::new();
    let mut outcome = RevalidationOutcome {
        still_valid: Vec::new(),
        changed: Vec::new(),
    };

    for removal in plan.removals() {
        let has_intact_kept_copy = group_has_intact_kept_copy(
            plan,
            removal.group_id,
            &read_stamp,
            &mut groups_with_intact_kept_copy,
            &mut groups_without_intact_kept_copy,
        );

        let change_reason = if has_intact_kept_copy {
            find_change(
                read_stamp(&removal.path),
                removal.expected_size,
                removal.expected_modified_time,
            )
        } else {
            Some(FileChangeReason::NoKeptCopyLeft)
        };

        match change_reason {
            None => outcome.still_valid.push(removal.clone()),
            Some(reason) => outcome.changed.push(ChangedFile {
                path: removal.path.clone(),
                reason,
            }),
        }
    }

    outcome
}

fn group_has_intact_kept_copy(
    plan: &RemovalPlan,
    group_id: DuplicateGroupId,
    read_stamp: &impl Fn(&Path) -> Option<FileStamp>,
    groups_with_intact_kept_copy: &mut HashSet<DuplicateGroupId>,
    groups_without_intact_kept_copy: &mut HashSet<DuplicateGroupId>,
) -> bool {
    if groups_with_intact_kept_copy.contains(&group_id) {
        return true;
    }
    if groups_without_intact_kept_copy.contains(&group_id) {
        return false;
    }

    let is_intact = plan.kept_files_of(group_id).iter().any(|kept_file| {
        find_change(
            read_stamp(&kept_file.path),
            kept_file.expected_size,
            kept_file.expected_modified_time,
        )
        .is_none()
    });

    if is_intact {
        groups_with_intact_kept_copy.insert(group_id);
    } else {
        groups_without_intact_kept_copy.insert(group_id);
    }
    is_intact
}

fn find_change(
    current_stamp: Option<FileStamp>,
    expected_size: u64,
    expected_modified_time: SystemTime,
) -> Option<FileChangeReason> {
    let Some(stamp) = current_stamp else {
        return Some(FileChangeReason::Missing);
    };

    if stamp.kind != EntryKind::File {
        Some(FileChangeReason::Missing)
    } else if stamp.flags.contains(EntryFlags::CLOUD_PLACEHOLDER) {
        Some(FileChangeReason::BecameOnlineOnly)
    } else if stamp.size != expected_size {
        Some(FileChangeReason::SizeChanged)
    } else if stamp.modified_time != expected_modified_time {
        Some(FileChangeReason::ModifiedTimeChanged)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::collections::HashMap;
    use std::time::{Duration, UNIX_EPOCH};

    use super::*;
    use crate::duplicate_group::{DuplicateFile, DuplicateGroup};

    const SIZE: u64 = 100;

    fn plan_removing_second_copy() -> RemovalPlan {
        let files = ["kept", "removed"]
            .iter()
            .map(|name| DuplicateFile::new(PathBuf::from(name), UNIX_EPOCH, UNIX_EPOCH))
            .collect();
        let group = DuplicateGroup::new(DuplicateGroupId(0), SIZE, files);
        RemovalPlan::from_files_to_remove(&[group], &[PathBuf::from("removed")]).unwrap()
    }

    fn unchanged_stamp() -> FileStamp {
        FileStamp {
            size: SIZE,
            modified_time: UNIX_EPOCH,
            kind: EntryKind::File,
            flags: EntryFlags::empty(),
        }
    }

    fn revalidate_with(stamps: &HashMap<&str, FileStamp>) -> RevalidationOutcome {
        revalidate_with_stamp_reader(&plan_removing_second_copy(), |path| {
            stamps.get(path.to_str().unwrap()).copied()
        })
    }

    fn single_change_reason(outcome: &RevalidationOutcome) -> FileChangeReason {
        assert_eq!(outcome.still_valid.len(), 0);
        assert_eq!(outcome.changed.len(), 1);
        outcome.changed[0].reason
    }

    #[test]
    fn accepts_unchanged_files() {
        let stamps = HashMap::from([("kept", unchanged_stamp()), ("removed", unchanged_stamp())]);

        let outcome = revalidate_with(&stamps);

        assert_eq!(outcome.paths_to_remove(), vec![PathBuf::from("removed")]);
        assert_eq!(outcome.changed.len(), 0);
    }

    #[test]
    fn detects_modified_resized_missing_and_online_only_files() {
        let modified = FileStamp {
            modified_time: UNIX_EPOCH + Duration::from_secs(1),
            ..unchanged_stamp()
        };
        let resized = FileStamp {
            size: SIZE + 1,
            ..unchanged_stamp()
        };
        let online_only = FileStamp {
            flags: EntryFlags::CLOUD_PLACEHOLDER,
            ..unchanged_stamp()
        };

        let cases = [
            (Some(modified), FileChangeReason::ModifiedTimeChanged),
            (Some(resized), FileChangeReason::SizeChanged),
            (Some(online_only), FileChangeReason::BecameOnlineOnly),
            (None, FileChangeReason::Missing),
        ];
        for (removed_stamp, expected_reason) in cases {
            let mut stamps = HashMap::from([("kept", unchanged_stamp())]);
            if let Some(removed_stamp) = removed_stamp {
                stamps.insert("removed", removed_stamp);
            }

            assert_eq!(
                single_change_reason(&revalidate_with(&stamps)),
                expected_reason
            );
        }
    }

    #[test]
    fn skips_the_group_when_the_kept_copy_changed() {
        let changed_kept_copy = FileStamp {
            size: 1,
            ..unchanged_stamp()
        };
        let stamps = HashMap::from([("kept", changed_kept_copy), ("removed", unchanged_stamp())]);

        assert_eq!(
            single_change_reason(&revalidate_with(&stamps)),
            FileChangeReason::NoKeptCopyLeft
        );
    }
}
