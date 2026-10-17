//! The duplicate groups of one finished search, as the review screen sees and edits them:
//! pages in a chosen order, keep/remove choices, and the groups after a recycle action.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use duplicate_finder::{
    DuplicateGroup, DuplicateGroupId, KeepRule, RemovalPlan, RemovalPlanError,
    apply_keep_rule_to_all_groups,
};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateGroupSortOrder {
    #[default]
    WastedBytesDescending,
    FileSizeDescending,
    FileCountDescending,
    PathAscending,
}

/// How much the current keep/remove choices would free.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSelectionSummary {
    pub files_selected_for_removal: u64,
    pub bytes_selected_for_removal: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroupsPage {
    pub total_group_count: u32,
    pub offset: u32,
    pub groups: Vec<DuplicateGroup>,
    /// Over all groups, not just this page.
    pub total_wasted_bytes: u64,
    pub total_duplicate_file_count: u64,
    pub selection: DuplicateSelectionSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeepChangeError {
    #[error("there is no duplicate group {}", group_id.0)]
    GroupNotFound { group_id: DuplicateGroupId },
    #[error("{} is not part of duplicate group {}", path.display(), group_id.0)]
    FileNotInGroup {
        group_id: DuplicateGroupId,
        path: PathBuf,
    },
    #[error("duplicate group {} must keep at least one file", group_id.0)]
    GroupWouldHaveNoKeptFile { group_id: DuplicateGroupId },
}

/// One page of `groups` in `sort_order`.
pub fn page_of_groups(
    groups: &[DuplicateGroup],
    sort_order: DuplicateGroupSortOrder,
    offset: u32,
    limit: u32,
) -> DuplicateGroupsPage {
    let mut ordered: Vec<&DuplicateGroup> = groups.iter().collect();
    ordered.sort_by(|first, second| {
        let ordering = match sort_order {
            DuplicateGroupSortOrder::WastedBytesDescending => {
                second.wasted_bytes.cmp(&first.wasted_bytes)
            }
            DuplicateGroupSortOrder::FileSizeDescending => second.file_size.cmp(&first.file_size),
            DuplicateGroupSortOrder::FileCountDescending => {
                second.files.len().cmp(&first.files.len())
            }
            DuplicateGroupSortOrder::PathAscending => first_path(first).cmp(&first_path(second)),
        };
        ordering.then(first.id.cmp(&second.id))
    });

    let page_groups = ordered
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .cloned()
        .collect();

    DuplicateGroupsPage {
        total_group_count: u32::try_from(groups.len()).unwrap_or(u32::MAX),
        offset,
        groups: page_groups,
        total_wasted_bytes: groups.iter().map(|group| group.wasted_bytes).sum(),
        total_duplicate_file_count: groups.iter().map(|group| group.files.len() as u64).sum(),
        selection: selection_summary(groups),
    }
}

fn first_path(group: &DuplicateGroup) -> Option<&Path> {
    group.files.first().map(|file| file.path.as_path())
}

pub fn selection_summary(groups: &[DuplicateGroup]) -> DuplicateSelectionSummary {
    let files_selected_for_removal = groups
        .iter()
        .map(|group| (group.files.len() - group.kept_file_count()) as u64)
        .sum();
    let bytes_selected_for_removal = groups
        .iter()
        .map(DuplicateGroup::bytes_selected_for_removal)
        .sum();

    DuplicateSelectionSummary {
        files_selected_for_removal,
        bytes_selected_for_removal,
    }
}

/// Applies `rule` to every group, replacing any choices made by hand.
pub fn apply_keep_rule(
    groups: &mut [DuplicateGroup],
    rule: &KeepRule,
) -> DuplicateSelectionSummary {
    apply_keep_rule_to_all_groups(groups, rule);
    selection_summary(groups)
}

/// Keeps or removes one file by hand and returns the changed group.
///
/// # Errors
///
/// Refuses to remove the last kept file of a group, and fails for unknown groups or files.
pub fn set_file_keep(
    groups: &mut [DuplicateGroup],
    group_id: DuplicateGroupId,
    path: &Path,
    keep: bool,
) -> Result<DuplicateGroup, KeepChangeError> {
    let group = groups
        .iter_mut()
        .find(|group| group.id == group_id)
        .ok_or(KeepChangeError::GroupNotFound { group_id })?;
    let file_index = group
        .files
        .iter()
        .position(|file| file.path == path)
        .ok_or_else(|| KeepChangeError::FileNotInGroup {
            group_id,
            path: path.to_path_buf(),
        })?;

    let is_last_kept_file = group.files[file_index].keep && group.kept_file_count() == 1;
    if !keep && is_last_kept_file {
        return Err(KeepChangeError::GroupWouldHaveNoKeptFile { group_id });
    }

    group.files[file_index].keep = keep;
    Ok(group.clone())
}

/// Builds the plan for moving `files_to_remove` to the Recycle Bin.
///
/// # Errors
///
/// Refuses plans that would remove every copy of a group, and paths outside every group.
pub fn plan_removal(
    groups: &[DuplicateGroup],
    files_to_remove: &[PathBuf],
) -> Result<RemovalPlan, RemovalPlanError> {
    RemovalPlan::from_files_to_remove(groups, files_to_remove)
}

/// Takes files that were moved to the Recycle Bin out of their groups. Groups left with a single
/// file are no longer duplicates and are dropped.
pub fn remove_recycled_files(groups: &mut Vec<DuplicateGroup>, recycled_paths: &[PathBuf]) {
    if recycled_paths.is_empty() {
        return;
    }
    let recycled_paths: HashSet<&Path> = recycled_paths.iter().map(PathBuf::as_path).collect();

    for group in groups.iter_mut() {
        group
            .files
            .retain(|file| !recycled_paths.contains(file.path.as_path()));
        group.wasted_bytes = group.file_size * (group.files.len().saturating_sub(1) as u64);
    }
    groups.retain(|group| group.files.len() >= 2);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::time::{Duration, UNIX_EPOCH};

    use duplicate_finder::DuplicateFile;

    use super::*;

    fn group(id: u32, file_size: u64, names: &[&str]) -> DuplicateGroup {
        let files = names
            .iter()
            .enumerate()
            .map(|(position, name)| {
                let time = UNIX_EPOCH + Duration::from_secs(position as u64);
                DuplicateFile::new(PathBuf::from(name), time, time)
            })
            .collect();
        DuplicateGroup::new(DuplicateGroupId(id), file_size, files)
    }

    fn sample_groups() -> Vec<DuplicateGroup> {
        vec![
            group(0, 100, &["a1", "a2"]),
            group(1, 10, &["b1", "b2", "b3", "b4"]),
            group(2, 50, &["c1", "c2", "c3"]),
        ]
    }

    fn page_ids(page: &DuplicateGroupsPage) -> Vec<u32> {
        page.groups.iter().map(|group| group.id.0).collect()
    }

    #[test]
    fn pages_in_the_requested_order() {
        let groups = sample_groups();

        let by_waste = page_of_groups(
            &groups,
            DuplicateGroupSortOrder::WastedBytesDescending,
            0,
            10,
        );
        let by_count = page_of_groups(&groups, DuplicateGroupSortOrder::FileCountDescending, 0, 10);
        let second_by_size =
            page_of_groups(&groups, DuplicateGroupSortOrder::FileSizeDescending, 1, 1);

        assert_eq!(page_ids(&by_waste), vec![0, 2, 1]);
        assert_eq!(page_ids(&by_count), vec![1, 2, 0]);
        assert_eq!(page_ids(&second_by_size), vec![2]);
        assert_eq!(second_by_size.total_group_count, 3);
        assert_eq!(by_waste.total_wasted_bytes, 100 + 30 + 100);
        assert_eq!(by_waste.selection, DuplicateSelectionSummary::default());
    }

    #[test]
    fn keep_rules_select_all_but_one_copy() {
        let mut groups = sample_groups();

        let selection = apply_keep_rule(&mut groups, &KeepRule::Oldest);

        assert_eq!(selection.files_selected_for_removal, 1 + 3 + 2);
        assert_eq!(selection.bytes_selected_for_removal, 100 + 30 + 100);
        assert!(groups[1].files[0].keep);
    }

    #[test]
    fn refuses_to_unkeep_the_last_kept_file() {
        let mut groups = sample_groups();
        apply_keep_rule(&mut groups, &KeepRule::Oldest);

        let refused = set_file_keep(&mut groups, DuplicateGroupId(0), Path::new("a1"), false);
        assert_eq!(
            refused,
            Err(KeepChangeError::GroupWouldHaveNoKeptFile {
                group_id: DuplicateGroupId(0)
            })
        );

        let kept_second =
            set_file_keep(&mut groups, DuplicateGroupId(0), Path::new("a2"), true).unwrap();
        assert_eq!(kept_second.kept_file_count(), 2);
        let removed_first =
            set_file_keep(&mut groups, DuplicateGroupId(0), Path::new("a1"), false).unwrap();
        assert_eq!(removed_first.kept_file_count(), 1);
    }

    #[test]
    fn reports_unknown_groups_and_files() {
        let mut groups = sample_groups();

        assert_eq!(
            set_file_keep(&mut groups, DuplicateGroupId(9), Path::new("a1"), true),
            Err(KeepChangeError::GroupNotFound {
                group_id: DuplicateGroupId(9)
            })
        );
        assert!(matches!(
            set_file_keep(&mut groups, DuplicateGroupId(0), Path::new("b1"), true),
            Err(KeepChangeError::FileNotInGroup { .. })
        ));
    }

    #[test]
    fn plans_that_would_empty_a_group_are_rejected() {
        let groups = sample_groups();

        let emptying = plan_removal(&groups, &[PathBuf::from("a1"), PathBuf::from("a2")]);
        assert_eq!(
            emptying,
            Err(RemovalPlanError::GroupWouldHaveNoKeptFile {
                group_id: DuplicateGroupId(0)
            })
        );

        let partial = plan_removal(&groups, &[PathBuf::from("c2"), PathBuf::from("c3")]).unwrap();
        assert_eq!(partial.total_bytes(), 100);
    }

    #[test]
    fn recycled_files_leave_their_groups_and_singles_are_dropped() {
        let mut groups = sample_groups();

        remove_recycled_files(
            &mut groups,
            &[
                PathBuf::from("a2"),
                PathBuf::from("c2"),
                PathBuf::from("c3"),
            ],
        );

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].id, DuplicateGroupId(1));
        assert_eq!(groups[0].wasted_bytes, 30);

        let mut partly_recycled = sample_groups();
        remove_recycled_files(&mut partly_recycled, &[PathBuf::from("b4")]);
        assert_eq!(partly_recycled[1].files.len(), 3);
        assert_eq!(partly_recycled[1].wasted_bytes, 20);
    }
}
