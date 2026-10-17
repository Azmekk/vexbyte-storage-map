use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::duplicate_group::{DuplicateGroup, DuplicateGroupId};

/// A copy the plan will remove, with what it looked like when it was compared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRemoval {
    pub path: PathBuf,
    pub group_id: DuplicateGroupId,
    pub expected_size: u64,
    pub expected_modified_time: SystemTime,
}

/// A copy the plan keeps. Revalidation checks that at least one of these still exists unchanged
/// before anything in its group is removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedKeep {
    pub path: PathBuf,
    pub expected_size: u64,
    pub expected_modified_time: SystemTime,
}

/// Which files to remove, built only when every affected group keeps at least one copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovalPlan {
    removals: Vec<PlannedRemoval>,
    kept_files_by_group: HashMap<DuplicateGroupId, Vec<PlannedKeep>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RemovalPlanError {
    #[error("duplicate group {} would lose every copy", group_id.0)]
    GroupWouldHaveNoKeptFile { group_id: DuplicateGroupId },
    #[error("{} is not part of any duplicate group", path.display())]
    FileNotInAnyGroup { path: PathBuf },
}

impl RemovalPlan {
    /// Removes every file whose `keep` flag is off.
    ///
    /// # Errors
    ///
    /// Fails with [`RemovalPlanError::GroupWouldHaveNoKeptFile`] when a group has no kept file.
    pub fn from_keep_flags(groups: &[DuplicateGroup]) -> Result<Self, RemovalPlanError> {
        let files_to_remove: HashSet<&Path> = groups
            .iter()
            .flat_map(|group| &group.files)
            .filter(|file| !file.keep)
            .map(|file| file.path.as_path())
            .collect();

        Self::build(groups, &files_to_remove)
    }

    /// Removes exactly the listed files, ignoring the groups' `keep` flags. This is the form the
    /// UI sends after the user changed individual toggles.
    ///
    /// # Errors
    ///
    /// Fails when a path belongs to no group, or when a group would lose all of its copies.
    pub fn from_files_to_remove(
        groups: &[DuplicateGroup],
        files_to_remove: &[PathBuf],
    ) -> Result<Self, RemovalPlanError> {
        let known_paths: HashSet<&Path> = groups
            .iter()
            .flat_map(|group| &group.files)
            .map(|file| file.path.as_path())
            .collect();

        if let Some(unknown_path) = files_to_remove
            .iter()
            .find(|path| !known_paths.contains(path.as_path()))
        {
            return Err(RemovalPlanError::FileNotInAnyGroup {
                path: unknown_path.clone(),
            });
        }

        let files_to_remove: HashSet<&Path> =
            files_to_remove.iter().map(PathBuf::as_path).collect();
        Self::build(groups, &files_to_remove)
    }

    fn build(
        groups: &[DuplicateGroup],
        files_to_remove: &HashSet<&Path>,
    ) -> Result<Self, RemovalPlanError> {
        let mut removals = Vec::new();
        let mut kept_files_by_group = HashMap::new();

        for group in groups {
            let (removed_files, kept_files): (Vec<_>, Vec<_>) = group
                .files
                .iter()
                .partition(|file| files_to_remove.contains(file.path.as_path()));

            if removed_files.is_empty() {
                continue;
            }
            if kept_files.is_empty() {
                return Err(RemovalPlanError::GroupWouldHaveNoKeptFile { group_id: group.id });
            }

            removals.extend(removed_files.into_iter().map(|file| PlannedRemoval {
                path: file.path.clone(),
                group_id: group.id,
                expected_size: group.file_size,
                expected_modified_time: file.modified_time(),
            }));

            let planned_keeps = kept_files
                .into_iter()
                .map(|file| PlannedKeep {
                    path: file.path.clone(),
                    expected_size: group.file_size,
                    expected_modified_time: file.modified_time(),
                })
                .collect();
            kept_files_by_group.insert(group.id, planned_keeps);
        }

        Ok(Self {
            removals,
            kept_files_by_group,
        })
    }

    #[must_use]
    pub fn removals(&self) -> &[PlannedRemoval] {
        &self.removals
    }

    #[must_use]
    pub fn kept_files_of(&self, group_id: DuplicateGroupId) -> &[PlannedKeep] {
        self.kept_files_by_group
            .get(&group_id)
            .map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        self.removals
            .iter()
            .map(|removal| removal.expected_size)
            .sum()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.removals.is_empty()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::time::UNIX_EPOCH;

    use super::*;
    use crate::duplicate_group::DuplicateFile;

    fn group(id: u32, names: &[&str]) -> DuplicateGroup {
        let files = names
            .iter()
            .map(|name| DuplicateFile::new(PathBuf::from(name), UNIX_EPOCH, UNIX_EPOCH))
            .collect();
        DuplicateGroup::new(DuplicateGroupId(id), 100, files)
    }

    #[test]
    fn removes_unkept_files_and_remembers_the_kept_ones() {
        let mut groups = vec![group(0, &["a1", "a2", "a3"]), group(1, &["b1", "b2"])];
        groups[0].files[1].keep = false;
        groups[0].files[2].keep = false;

        let plan = RemovalPlan::from_keep_flags(&groups).unwrap();

        let removed: Vec<_> = plan.removals().iter().map(|r| r.path.clone()).collect();
        assert_eq!(removed, vec![PathBuf::from("a2"), PathBuf::from("a3")]);
        assert_eq!(plan.total_bytes(), 200);
        assert_eq!(plan.kept_files_of(DuplicateGroupId(0)).len(), 1);
        assert_eq!(plan.kept_files_of(DuplicateGroupId(1)), []);
    }

    #[test]
    fn refuses_to_empty_a_group_from_keep_flags() {
        let mut groups = vec![group(0, &["a1", "a2"]), group(7, &["b1", "b2"])];
        for file in &mut groups[1].files {
            file.keep = false;
        }

        assert_eq!(
            RemovalPlan::from_keep_flags(&groups),
            Err(RemovalPlanError::GroupWouldHaveNoKeptFile {
                group_id: DuplicateGroupId(7)
            })
        );
    }

    #[test]
    fn refuses_to_empty_a_group_from_an_explicit_list() {
        let groups = vec![group(3, &["a1", "a2"])];
        let files_to_remove = vec![PathBuf::from("a1"), PathBuf::from("a2")];

        assert_eq!(
            RemovalPlan::from_files_to_remove(&groups, &files_to_remove),
            Err(RemovalPlanError::GroupWouldHaveNoKeptFile {
                group_id: DuplicateGroupId(3)
            })
        );
    }

    #[test]
    fn refuses_files_outside_every_group() {
        let groups = vec![group(0, &["a1", "a2"])];

        assert_eq!(
            RemovalPlan::from_files_to_remove(&groups, &[PathBuf::from("elsewhere")]),
            Err(RemovalPlanError::FileNotInAnyGroup {
                path: PathBuf::from("elsewhere")
            })
        );
    }

    #[test]
    fn explicit_list_ignores_keep_flags() {
        let mut groups = vec![group(0, &["a1", "a2"])];
        groups[0].files[0].keep = false;

        let plan = RemovalPlan::from_files_to_remove(&groups, &[PathBuf::from("a2")]).unwrap();

        assert_eq!(plan.removals().len(), 1);
        assert_eq!(plan.removals()[0].path, PathBuf::from("a2"));
        assert_eq!(
            plan.kept_files_of(DuplicateGroupId(0))[0].path,
            PathBuf::from("a1")
        );
    }
}
