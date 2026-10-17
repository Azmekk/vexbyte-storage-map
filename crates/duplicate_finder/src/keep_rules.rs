use std::cmp::Ordering;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::duplicate_group::{DuplicateFile, DuplicateGroup};
use crate::platform::is_same_or_inside_folder;

/// Picks which copy of each group stays. Every rule keeps exactly one copy; the user can then
/// change any group by hand.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind", content = "folder")]
pub enum KeepRule {
    /// The copy modified longest ago, usually the original.
    #[default]
    Oldest,
    Newest,
    /// The oldest copy inside this folder; groups without a copy there keep their oldest copy.
    PreferredFolder(PathBuf),
    /// The copy with the shortest full path, usually the one in the tidiest place.
    ShortestPath,
}

/// Marks exactly one file of `group` as kept, according to `rule`, and every other file for
/// removal.
pub fn apply_keep_rule(group: &mut DuplicateGroup, rule: &KeepRule) {
    let Some(kept_index) = index_of_file_to_keep(&group.files, rule) else {
        return;
    };

    for (index, file) in group.files.iter_mut().enumerate() {
        file.keep = index == kept_index;
    }
}

pub fn apply_keep_rule_to_all_groups(groups: &mut [DuplicateGroup], rule: &KeepRule) {
    for group in groups {
        apply_keep_rule(group, rule);
    }
}

fn index_of_file_to_keep(files: &[DuplicateFile], rule: &KeepRule) -> Option<usize> {
    let indexed_files = files.iter().enumerate();

    let best = match rule {
        KeepRule::Oldest => indexed_files.min_by(|(_, left), (_, right)| oldest_first(left, right)),
        KeepRule::Newest => indexed_files.min_by(|(_, left), (_, right)| newest_first(left, right)),
        KeepRule::ShortestPath => {
            indexed_files.min_by(|(_, left), (_, right)| shortest_path_first(left, right))
        }
        KeepRule::PreferredFolder(folder) => {
            let oldest_in_folder = files
                .iter()
                .enumerate()
                .filter(|(_, file)| is_same_or_inside_folder(&file.path, folder))
                .min_by(|(_, left), (_, right)| oldest_first(left, right));
            return oldest_in_folder
                .map(|(index, _)| index)
                .or_else(|| index_of_file_to_keep(files, &KeepRule::Oldest));
        }
    };

    best.map(|(index, _)| index)
}

/// Ties fall back to the creation time and then the path, so every rule is deterministic.
fn oldest_first(left: &DuplicateFile, right: &DuplicateFile) -> Ordering {
    left.modified_time()
        .cmp(&right.modified_time())
        .then_with(|| left.created_time().cmp(&right.created_time()))
        .then_with(|| left.path.cmp(&right.path))
}

fn newest_first(left: &DuplicateFile, right: &DuplicateFile) -> Ordering {
    right
        .modified_time()
        .cmp(&left.modified_time())
        .then_with(|| right.created_time().cmp(&left.created_time()))
        .then_with(|| left.path.cmp(&right.path))
}

fn shortest_path_first(left: &DuplicateFile, right: &DuplicateFile) -> Ordering {
    let left_length = left.path.as_os_str().len();
    let right_length = right.path.as_os_str().len();

    left_length
        .cmp(&right_length)
        .then_with(|| oldest_first(left, right))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::duplicate_group::DuplicateGroupId;

    fn at_second(second: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(second)
    }

    fn sample_group() -> DuplicateGroup {
        let root = std::env::temp_dir();
        DuplicateGroup::new(
            DuplicateGroupId(0),
            10,
            vec![
                DuplicateFile::new(
                    root.join("Photos").join("2020").join("beach.jpg"),
                    at_second(200),
                    at_second(200),
                ),
                DuplicateFile::new(
                    root.join("Backup").join("beach.jpg"),
                    at_second(100),
                    at_second(300),
                ),
                DuplicateFile::new(
                    root.join("Downloads")
                        .join("Old")
                        .join("x")
                        .join("beach.jpg"),
                    at_second(300),
                    at_second(300),
                ),
            ],
        )
    }

    fn kept_path(group: &DuplicateGroup) -> PathBuf {
        assert_eq!(group.kept_file_count(), 1);
        group
            .files
            .iter()
            .find(|file| file.keep)
            .unwrap()
            .path
            .clone()
    }

    #[test]
    fn oldest_keeps_the_earliest_modified_copy() {
        let mut group = sample_group();
        apply_keep_rule(&mut group, &KeepRule::Oldest);

        assert!(kept_path(&group).starts_with(std::env::temp_dir().join("Backup")));
    }

    #[test]
    fn newest_keeps_the_latest_modified_copy() {
        let mut group = sample_group();
        apply_keep_rule(&mut group, &KeepRule::Newest);

        assert!(kept_path(&group).starts_with(std::env::temp_dir().join("Downloads")));
    }

    #[test]
    fn shortest_path_keeps_the_shortest_path() {
        let mut group = sample_group();
        apply_keep_rule(&mut group, &KeepRule::ShortestPath);

        assert!(kept_path(&group).starts_with(std::env::temp_dir().join("Backup")));
    }

    #[test]
    fn preferred_folder_keeps_the_copy_inside_it() {
        let mut group = sample_group();
        let folder = std::env::temp_dir().join("photos");
        apply_keep_rule(&mut group, &KeepRule::PreferredFolder(folder));

        assert!(kept_path(&group).starts_with(std::env::temp_dir().join("Photos")));
    }

    #[test]
    fn preferred_folder_falls_back_to_the_oldest_copy() {
        let mut group = sample_group();
        let folder = std::env::temp_dir().join("Elsewhere");
        apply_keep_rule(&mut group, &KeepRule::PreferredFolder(folder));

        assert!(kept_path(&group).starts_with(std::env::temp_dir().join("Backup")));
    }

    #[test]
    fn serializes_rules_with_a_kind_tag() {
        let rule: KeepRule =
            serde_json::from_str(r#"{"kind":"preferredFolder","folder":"D:\\Photos"}"#).unwrap();
        assert_eq!(rule, KeepRule::PreferredFolder(PathBuf::from(r"D:\Photos")));

        let oldest = serde_json::to_string(&KeepRule::Oldest).unwrap();
        assert_eq!(oldest, r#"{"kind":"oldest"}"#);
    }
}
