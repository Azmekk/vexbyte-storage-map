use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use specta::Type;

/// Identifies a group within one search result. Groups are numbered by descending wasted space.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Type,
)]
#[serde(transparent)]
pub struct DuplicateGroupId(pub u32);

/// One copy of a duplicated file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateFile {
    pub path: PathBuf,
    /// Milliseconds since 1970-01-01 UTC, ready for `new Date(...)`.
    pub modified_unix_milliseconds: i64,
    pub created_unix_milliseconds: i64,
    /// Whether this copy stays. Every group keeps at least one copy.
    pub keep: bool,
    #[serde(skip)]
    modified_time: SystemTime,
    #[serde(skip)]
    created_time: SystemTime,
}

impl DuplicateFile {
    #[must_use]
    pub fn new(path: PathBuf, modified_time: SystemTime, created_time: SystemTime) -> Self {
        Self {
            path,
            modified_unix_milliseconds: unix_milliseconds(modified_time),
            created_unix_milliseconds: unix_milliseconds(created_time),
            keep: true,
            modified_time,
            created_time,
        }
    }

    /// The exact modification time, used to detect files that changed before removal.
    #[must_use]
    pub fn modified_time(&self) -> SystemTime {
        self.modified_time
    }

    #[must_use]
    pub fn created_time(&self) -> SystemTime {
        self.created_time
    }
}

/// Files with byte-for-byte identical content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub id: DuplicateGroupId,
    /// Size of each copy.
    pub file_size: u64,
    /// Sorted by path.
    pub files: Vec<DuplicateFile>,
    /// Space that would be freed by keeping a single copy.
    pub wasted_bytes: u64,
}

impl DuplicateGroup {
    #[must_use]
    pub fn new(id: DuplicateGroupId, file_size: u64, mut files: Vec<DuplicateFile>) -> Self {
        files.sort_by(|left, right| left.path.cmp(&right.path));
        let wasted_bytes = file_size * (files.len().saturating_sub(1) as u64);

        Self {
            id,
            file_size,
            files,
            wasted_bytes,
        }
    }

    #[must_use]
    pub fn kept_file_count(&self) -> usize {
        self.files.iter().filter(|file| file.keep).count()
    }

    /// Bytes freed by removing every copy not marked to keep.
    #[must_use]
    pub fn bytes_selected_for_removal(&self) -> u64 {
        let removed_count = self.files.len() - self.kept_file_count();
        self.file_size * removed_count as u64
    }
}

fn unix_milliseconds(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(since_epoch) => i64::try_from(since_epoch.as_millis()).unwrap_or(i64::MAX),
        Err(before_epoch) => {
            i64::try_from(before_epoch.duration().as_millis()).map_or(i64::MIN, |millis| -millis)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::time::Duration;

    use super::*;

    #[test]
    fn computes_wasted_bytes_and_sorts_files_by_path() {
        let time = UNIX_EPOCH + Duration::from_millis(1_500);
        let group = DuplicateGroup::new(
            DuplicateGroupId(0),
            1_000,
            vec![
                DuplicateFile::new(PathBuf::from("b"), time, time),
                DuplicateFile::new(PathBuf::from("a"), time, time),
                DuplicateFile::new(PathBuf::from("c"), time, time),
            ],
        );

        assert_eq!(group.wasted_bytes, 2_000);
        assert_eq!(group.files[0].path, PathBuf::from("a"));
        assert_eq!(group.files[0].modified_unix_milliseconds, 1_500);
    }

    #[test]
    fn converts_times_before_1970() {
        assert_eq!(
            unix_milliseconds(UNIX_EPOCH - Duration::from_secs(1)),
            -1_000
        );
    }
}
