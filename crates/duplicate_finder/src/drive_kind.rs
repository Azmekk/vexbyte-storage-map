use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use specta::Type;

use crate::platform::{query_volume_incurs_seek_penalty, query_volume_mount_point};

/// What kind of storage a folder lives on, which decides how its files are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DriveKind {
    /// No seek penalty: many files can be read at once.
    SolidState,
    /// Seek penalty: reading several files at once makes the heads thrash.
    Rotational,
    /// Network shares, virtual disks and drivers that do not answer.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadStrategy {
    Parallel,
    SequentialInPathOrder,
}

impl DriveKind {
    /// Unknown drives are read like hard disks: slower on an SSD, but never pathological on a
    /// spinning disk or a network share.
    #[must_use]
    pub(crate) fn read_strategy(self) -> ReadStrategy {
        match self {
            Self::SolidState => ReadStrategy::Parallel,
            Self::Rotational | Self::Unknown => ReadStrategy::SequentialInPathOrder,
        }
    }
}

/// Detects the drive kind of a path, asking the storage driver once per volume.
#[derive(Debug, Default)]
pub struct DriveKindCache {
    drive_kind_by_mount_point: Mutex<HashMap<PathBuf, DriveKind>>,
}

impl DriveKindCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn drive_kind_of(&self, path: &Path) -> DriveKind {
        let Ok(mount_point) = query_volume_mount_point(path) else {
            return DriveKind::Unknown;
        };

        let mut drive_kind_by_mount_point = match self.drive_kind_by_mount_point.lock() {
            Ok(cache) => cache,
            Err(poisoned) => poisoned.into_inner(),
        };

        *drive_kind_by_mount_point
            .entry(mount_point)
            .or_insert_with_key(|mount_point| detect_drive_kind_of_volume(mount_point))
    }
}

fn detect_drive_kind_of_volume(mount_point: &Path) -> DriveKind {
    match query_volume_incurs_seek_penalty(mount_point) {
        Ok(true) => DriveKind::Rotational,
        Ok(false) => DriveKind::SolidState,
        Err(error) => {
            tracing::debug!(mount_point = %mount_point.display(), %error, "seek penalty unknown");
            DriveKind::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn unknown_drives_are_read_sequentially() {
        assert_eq!(
            DriveKind::Unknown.read_strategy(),
            ReadStrategy::SequentialInPathOrder
        );
        assert_eq!(
            DriveKind::Rotational.read_strategy(),
            ReadStrategy::SequentialInPathOrder
        );
        assert_eq!(
            DriveKind::SolidState.read_strategy(),
            ReadStrategy::Parallel
        );
    }

    #[test]
    fn detects_the_temp_drive_and_caches_it() {
        let cache = DriveKindCache::new();
        let temp_directory = std::env::temp_dir();

        let first = cache.drive_kind_of(&temp_directory);
        let second = cache.drive_kind_of(&temp_directory);

        assert_eq!(first, second);
        assert_eq!(cache.drive_kind_by_mount_point.lock().unwrap().len(), 1);
    }
}
