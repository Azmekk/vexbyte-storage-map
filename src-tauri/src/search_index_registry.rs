//! The search index of every volume: which ones are indexed, which are still being read, and the
//! finished indexes that answer queries. Builds run on their own threads (see
//! `search_index_builders`); this registry only holds state, so it is easy to test.
//!
//! A finished index is an immutable `Arc<VolumeSearchIndex>`. Refreshing a volume builds a new
//! index while the old one keeps answering queries, then swaps it in as a whole.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime};

use disk_scanner::{ScanMethod, VolumeKind, VolumeSearchIndex, VolumeSummary};
use serde::Serialize;
use specta::Type;

/// Volumes read at the same time. Reading more at once mostly competes for the same disks and
/// memory bandwidth.
pub const MAXIMUM_CONCURRENT_INDEX_BUILDS: usize = 2;
/// The status event is sent at most this often per volume while it is read.
const MINIMUM_INTERVAL_BETWEEN_PROGRESS_EVENTS: Duration = Duration::from_millis(250);

/// Why a volume has no search index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SkipReason {
    /// A streamed cloud drive such as Google Drive for desktop; reading it downloads files.
    CloudDrive,
    NetworkDrive,
    RemovableDrive,
    OpticalDrive,
    RamDisk,
    UnknownDriveType,
}

impl SkipReason {
    /// `None` for a local disk, the only kind that is indexed.
    pub fn for_volume_kind(kind: VolumeKind) -> Option<Self> {
        match kind {
            VolumeKind::LocalDisk => None,
            VolumeKind::CloudVirtual => Some(Self::CloudDrive),
            VolumeKind::Network => Some(Self::NetworkDrive),
            VolumeKind::Removable => Some(Self::RemovableDrive),
            VolumeKind::Optical => Some(Self::OpticalDrive),
            VolumeKind::RamDisk => Some(Self::RamDisk),
            VolumeKind::Unknown => Some(Self::UnknownDriveType),
        }
    }
}

/// What is happening to a volume's index right now. Whether the volume can be searched is told
/// by `VolumeIndexStatus::index`, which stays set while a refresh runs or after a refresh failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum VolumeIndexState {
    /// Waiting for a free build slot.
    Idle,
    Building {
        files_so_far: u64,
        directories_so_far: u64,
        elapsed_milliseconds: u64,
        method: ScanMethod,
    },
    Ready,
    Failed {
        reason: String,
    },
    Skipped {
        reason: SkipReason,
    },
}

/// The finished index that answers queries for a volume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IndexSummary {
    pub built_at_unix_milliseconds: u64,
    pub file_count: u32,
    pub directory_count: u32,
    pub method: ScanMethod,
    /// From the start of reading the volume until the index was ready.
    pub duration_milliseconds: u64,
    pub estimated_memory_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VolumeIndexStatus {
    pub mount_point: PathBuf,
    pub label: String,
    pub volume_kind: VolumeKind,
    pub state: VolumeIndexState,
    /// Set while the volume can be searched.
    pub index: Option<IndexSummary>,
}

/// One word for the whole search engine, for the "Warming up search engine" screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum OverallIndexStatus {
    /// The volumes are still being listed, or at least one local volume is still being read for
    /// the first time. Volumes that are ready can already be searched.
    Warming,
    /// Every local volume can be searched (some may be refreshing).
    Ready,
    /// Every local volume was tried; some can be searched and some failed.
    PartiallyReady,
    /// No volume can be searched: every build failed, or there is no local volume.
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexStatus {
    /// Ordered by mount point.
    pub volumes: Vec<VolumeIndexStatus>,
    pub overall: OverallIndexStatus,
}

/// What a search runs against: the finished indexes ordered by mount point, and the local
/// volumes that are not searchable yet because their first build has not finished.
pub struct SearchableIndexes {
    pub indexes: Vec<Arc<VolumeSearchIndex>>,
    pub volumes_still_warming: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RefreshRequestError {
    #[error("{} is not a known volume", mount_point.display())]
    VolumeNotFound { mount_point: PathBuf },
    #[error("{} is not indexed ({reason:?})", mount_point.display())]
    VolumeNotIndexed {
        mount_point: PathBuf,
        reason: SkipReason,
    },
}

enum BuildState {
    Idle,
    Building {
        started_at: Instant,
        method: ScanMethod,
        files_so_far: u64,
        directories_so_far: u64,
        last_progress_event_at: Option<Instant>,
    },
    Ready,
    Failed {
        reason: String,
    },
    Skipped {
        reason: SkipReason,
    },
}

struct ReadyIndex {
    index: Arc<VolumeSearchIndex>,
    summary: IndexSummary,
}

struct VolumeEntry {
    volume: VolumeSummary,
    build: BuildState,
    ready: Option<ReadyIndex>,
}

impl VolumeEntry {
    fn new(volume: VolumeSummary) -> Self {
        let build = match SkipReason::for_volume_kind(volume.kind) {
            Some(reason) => BuildState::Skipped { reason },
            None => BuildState::Idle,
        };
        Self {
            volume,
            build,
            ready: None,
        }
    }

    fn is_skipped(&self) -> bool {
        matches!(self.build, BuildState::Skipped { .. })
    }

    fn is_building(&self) -> bool {
        matches!(self.build, BuildState::Building { .. })
    }

    fn is_warming(&self) -> bool {
        self.ready.is_none() && matches!(self.build, BuildState::Idle | BuildState::Building { .. })
    }

    fn status(&self) -> VolumeIndexStatus {
        let state = match &self.build {
            BuildState::Idle => VolumeIndexState::Idle,
            BuildState::Building {
                started_at,
                method,
                files_so_far,
                directories_so_far,
                ..
            } => VolumeIndexState::Building {
                files_so_far: *files_so_far,
                directories_so_far: *directories_so_far,
                elapsed_milliseconds: milliseconds_of(started_at.elapsed()),
                method: *method,
            },
            BuildState::Ready => VolumeIndexState::Ready,
            BuildState::Failed { reason } => VolumeIndexState::Failed {
                reason: reason.clone(),
            },
            BuildState::Skipped { reason } => VolumeIndexState::Skipped { reason: *reason },
        };

        VolumeIndexStatus {
            mount_point: self.volume.mount_point.clone(),
            label: self.volume.label.clone(),
            volume_kind: self.volume.kind,
            state,
            index: self.ready.as_ref().map(|ready| ready.summary.clone()),
        }
    }
}

#[derive(Default)]
struct RegistryContents {
    has_listed_volumes: bool,
    /// Ordered by mount point.
    volumes: Vec<VolumeEntry>,
    build_queue: VecDeque<PathBuf>,
    active_builder_count: usize,
}

impl RegistryContents {
    fn entry_mut(&mut self, mount_point: &Path) -> Option<&mut VolumeEntry> {
        self.volumes
            .iter_mut()
            .find(|entry| entry.volume.mount_point == mount_point)
    }

    fn overall_status(&self) -> OverallIndexStatus {
        if !self.has_listed_volumes {
            return OverallIndexStatus::Warming;
        }
        let indexable: Vec<&VolumeEntry> = self
            .volumes
            .iter()
            .filter(|entry| !entry.is_skipped())
            .collect();
        if indexable.iter().any(|entry| entry.is_warming()) {
            return OverallIndexStatus::Warming;
        }

        let searchable_count = indexable
            .iter()
            .filter(|entry| entry.ready.is_some())
            .count();
        if searchable_count == 0 {
            OverallIndexStatus::Failed
        } else if searchable_count == indexable.len() {
            OverallIndexStatus::Ready
        } else {
            OverallIndexStatus::PartiallyReady
        }
    }
}

/// Thread-safe. The lock is held only to read or update entries, never while a volume is read
/// or searched.
#[derive(Default)]
pub struct SearchIndexRegistry {
    contents: Mutex<RegistryContents>,
}

impl SearchIndexRegistry {
    fn lock(&self) -> MutexGuard<'_, RegistryContents> {
        self.contents.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Takes in the current list of volumes. Known volumes keep their index; volumes that are
    /// gone are dropped unless a build of theirs is running. Returns the local volumes in the
    /// order they should be built: the system volume first, then by mount point.
    pub fn record_volume_list(
        &self,
        listed_volumes: Vec<VolumeSummary>,
        system_volume: Option<&Path>,
    ) -> Vec<PathBuf> {
        let mut contents = self.lock();
        contents.has_listed_volumes = true;

        contents.volumes.retain(|entry| {
            entry.is_building()
                || listed_volumes
                    .iter()
                    .any(|volume| volume.mount_point == entry.volume.mount_point)
        });
        for volume in listed_volumes {
            match contents.entry_mut(&volume.mount_point) {
                Some(entry) => entry.volume = volume,
                None => contents.volumes.push(VolumeEntry::new(volume)),
            }
        }
        contents
            .volumes
            .sort_by(|first, second| first.volume.mount_point.cmp(&second.volume.mount_point));

        let mut build_order: Vec<PathBuf> = contents
            .volumes
            .iter()
            .filter(|entry| !entry.is_skipped())
            .map(|entry| entry.volume.mount_point.clone())
            .collect();
        if let Some(system_volume) = system_volume {
            build_order.sort_by_key(|mount_point| mount_point != system_volume);
        }
        build_order
    }

    /// The volume list could not be read; the search then has nothing to search.
    pub fn record_volume_list_failure(&self) {
        self.lock().has_listed_volumes = true;
    }

    /// Checks that `mount_point` names a volume that is indexed.
    ///
    /// # Errors
    ///
    /// Fails for unknown volumes and for volumes that are skipped, such as cloud drives.
    pub fn check_refresh_target(&self, mount_point: &Path) -> Result<(), RefreshRequestError> {
        let mut contents = self.lock();
        let entry =
            contents
                .entry_mut(mount_point)
                .ok_or_else(|| RefreshRequestError::VolumeNotFound {
                    mount_point: mount_point.to_path_buf(),
                })?;
        match entry.build {
            BuildState::Skipped { reason } => Err(RefreshRequestError::VolumeNotIndexed {
                mount_point: mount_point.to_path_buf(),
                reason,
            }),
            _ => Ok(()),
        }
    }

    /// Queues builds of the given volumes. Volumes that are skipped, unknown, already queued or
    /// being built are left alone. Returns how many builder threads the caller must start; each
    /// one calls `take_next_build` until it returns `None`.
    pub fn queue_builds(&self, mount_points: &[PathBuf]) -> usize {
        let mut contents = self.lock();

        for mount_point in mount_points {
            let is_queued = contents.build_queue.contains(mount_point);
            let Some(entry) = contents.entry_mut(mount_point) else {
                continue;
            };
            if is_queued || entry.is_skipped() || entry.is_building() {
                continue;
            }
            entry.build = BuildState::Idle;
            contents.build_queue.push_back(mount_point.clone());
        }

        let free_builder_slots =
            MAXIMUM_CONCURRENT_INDEX_BUILDS.saturating_sub(contents.active_builder_count);
        let builders_to_start = free_builder_slots.min(contents.build_queue.len());
        contents.active_builder_count += builders_to_start;
        builders_to_start
    }

    /// The next volume a builder thread should read, now marked as building. `None` when the
    /// queue is empty; the builder thread must then end.
    pub fn take_next_build(&self) -> Option<VolumeSummary> {
        let mut contents = self.lock();

        while let Some(mount_point) = contents.build_queue.pop_front() {
            let Some(entry) = contents.entry_mut(&mount_point) else {
                continue;
            };
            entry.build = BuildState::Building {
                started_at: Instant::now(),
                method: disk_scanner::search_index_scan_method(&entry.volume),
                files_so_far: 0,
                directories_so_far: 0,
                last_progress_event_at: None,
            };
            return Some(entry.volume.clone());
        }

        contents.active_builder_count = contents.active_builder_count.saturating_sub(1);
        None
    }

    /// A builder thread could not be started: gives its slot back, and fails the queued builds
    /// when no builder is left to run them.
    pub fn record_builder_start_failure(&self, reason: &str) {
        let mut contents = self.lock();
        contents.active_builder_count = contents.active_builder_count.saturating_sub(1);
        if contents.active_builder_count > 0 {
            return;
        }

        let queued: Vec<PathBuf> = contents.build_queue.drain(..).collect();
        for mount_point in queued {
            if let Some(entry) = contents.entry_mut(&mount_point) {
                entry.build = BuildState::Failed {
                    reason: reason.to_owned(),
                };
            }
        }
    }

    /// Stores the counts of a running build. Returns true when a status event is due, at most
    /// four times per second per volume.
    pub fn record_build_progress(
        &self,
        mount_point: &Path,
        files_found: u64,
        directories_found: u64,
    ) -> bool {
        let mut contents = self.lock();
        let Some(entry) = contents.entry_mut(mount_point) else {
            return false;
        };
        let BuildState::Building {
            files_so_far,
            directories_so_far,
            last_progress_event_at,
            ..
        } = &mut entry.build
        else {
            return false;
        };

        *files_so_far = files_found;
        *directories_so_far = directories_found;
        let now = Instant::now();
        let is_event_due = last_progress_event_at.is_none_or(|last_event_at| {
            now.duration_since(last_event_at) >= MINIMUM_INTERVAL_BETWEEN_PROGRESS_EVENTS
        });
        if is_event_due {
            *last_progress_event_at = Some(now);
        }
        is_event_due
    }

    /// Swaps in a finished index; the previous one is freed once no running search uses it.
    pub fn record_build_success(&self, mount_point: &Path, index: VolumeSearchIndex) {
        let mut contents = self.lock();
        let Some(entry) = contents.entry_mut(mount_point) else {
            return;
        };

        let duration = match entry.build {
            BuildState::Building { started_at, .. } => started_at.elapsed(),
            _ => Duration::ZERO,
        };
        let summary = IndexSummary {
            built_at_unix_milliseconds: unix_milliseconds_now(),
            file_count: index.file_count(),
            directory_count: index.directory_count(),
            method: index.method(),
            duration_milliseconds: milliseconds_of(duration),
            estimated_memory_bytes: u64::try_from(index.estimated_memory_bytes())
                .unwrap_or(u64::MAX),
        };

        entry.ready = Some(ReadyIndex {
            index: Arc::new(index),
            summary,
        });
        entry.build = BuildState::Ready;
    }

    /// A failed refresh keeps the previous index searchable.
    pub fn record_build_failure(&self, mount_point: &Path, reason: String) {
        let mut contents = self.lock();
        if let Some(entry) = contents.entry_mut(mount_point) {
            entry.build = BuildState::Failed { reason };
        }
    }

    pub fn status(&self) -> SearchIndexStatus {
        let contents = self.lock();
        SearchIndexStatus {
            volumes: contents.volumes.iter().map(VolumeEntry::status).collect(),
            overall: contents.overall_status(),
        }
    }

    pub fn searchable_indexes(&self) -> SearchableIndexes {
        let contents = self.lock();
        let indexes = contents
            .volumes
            .iter()
            .filter_map(|entry| entry.ready.as_ref())
            .map(|ready| Arc::clone(&ready.index))
            .collect();
        let volumes_still_warming = contents
            .volumes
            .iter()
            .filter(|entry| entry.is_warming())
            .map(|entry| entry.volume.mount_point.clone())
            .collect();

        SearchableIndexes {
            indexes,
            volumes_still_warming,
        }
    }

    /// The finished index of the volume that holds `path`.
    pub fn index_holding(&self, path: &Path) -> Option<Arc<VolumeSearchIndex>> {
        let contents = self.lock();
        contents
            .volumes
            .iter()
            .filter(|entry| path.starts_with(&entry.volume.mount_point))
            .max_by_key(|entry| entry.volume.mount_point.as_os_str().len())
            .and_then(|entry| entry.ready.as_ref())
            .map(|ready| Arc::clone(&ready.index))
    }
}

fn milliseconds_of(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn unix_milliseconds_now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, milliseconds_of)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use disk_scanner::file_tree::{DirectoryKey, FileTreeBuilder, NewEntry};
    use disk_scanner::{DriveType, NodeFlags, NodeKind};

    use super::*;

    fn volume(mount_point: &str, kind: VolumeKind) -> VolumeSummary {
        VolumeSummary {
            mount_point: PathBuf::from(mount_point),
            label: String::new(),
            file_system_name: "NTFS".to_owned(),
            total_bytes: 100,
            free_bytes: 50,
            drive_type: DriveType::Fixed,
            kind,
            supports_fast_scan: false,
        }
    }

    fn index_with_file(mount_point: &str, file_name: &str) -> VolumeSearchIndex {
        let mut builder = FileTreeBuilder::new(PathBuf::from(mount_point), DirectoryKey::new(0));
        builder.add_entry(&NewEntry {
            name: file_name,
            kind: NodeKind::File,
            parent: Some(DirectoryKey::new(0)),
            directory_key: None,
            logical_size: 10,
            allocated_size: 10,
            modified_time: None,
            flags: NodeFlags::empty(),
            hard_link_identity: None,
        });
        VolumeSearchIndex::from_file_tree(
            PathBuf::from(mount_point),
            builder.finish(),
            ScanMethod::FolderWalk,
        )
    }

    fn state_of(registry: &SearchIndexRegistry, mount_point: &str) -> VolumeIndexState {
        registry
            .status()
            .volumes
            .into_iter()
            .find(|volume| volume.mount_point == Path::new(mount_point))
            .unwrap()
            .state
    }

    fn two_local_volumes_and_a_cloud_drive() -> Vec<VolumeSummary> {
        vec![
            volume(r"D:\", VolumeKind::LocalDisk),
            volume(r"G:\", VolumeKind::CloudVirtual),
            volume(r"C:\", VolumeKind::LocalDisk),
        ]
    }

    #[test]
    fn is_warming_until_the_volumes_are_listed() {
        let registry = SearchIndexRegistry::default();

        assert_eq!(registry.status().overall, OverallIndexStatus::Warming);

        registry.record_volume_list_failure();
        assert_eq!(registry.status().overall, OverallIndexStatus::Failed);
    }

    #[test]
    fn builds_local_volumes_system_volume_first_and_skips_the_rest() {
        let registry = SearchIndexRegistry::default();

        let build_order = registry.record_volume_list(
            two_local_volumes_and_a_cloud_drive(),
            Some(Path::new(r"D:\")),
        );

        assert_eq!(build_order, [PathBuf::from(r"D:\"), PathBuf::from(r"C:\")]);
        assert_eq!(
            state_of(&registry, r"G:\"),
            VolumeIndexState::Skipped {
                reason: SkipReason::CloudDrive
            }
        );
        assert_eq!(state_of(&registry, r"C:\"), VolumeIndexState::Idle);
        let mount_points: Vec<_> = registry
            .status()
            .volumes
            .into_iter()
            .map(|volume| volume.mount_point)
            .collect();
        assert_eq!(
            mount_points,
            [r"C:\", r"D:\", r"G:\"].map(PathBuf::from),
            "ordered by mount point"
        );
    }

    #[test]
    fn every_kind_but_local_disks_is_skipped() {
        assert_eq!(SkipReason::for_volume_kind(VolumeKind::LocalDisk), None);
        for kind in [
            VolumeKind::CloudVirtual,
            VolumeKind::Network,
            VolumeKind::Removable,
            VolumeKind::Optical,
            VolumeKind::RamDisk,
            VolumeKind::Unknown,
        ] {
            assert!(SkipReason::for_volume_kind(kind).is_some(), "{kind:?}");
        }
    }

    #[test]
    fn runs_at_most_two_builds_at_a_time() {
        let registry = SearchIndexRegistry::default();
        let mut volumes = two_local_volumes_and_a_cloud_drive();
        volumes.push(volume(r"E:\", VolumeKind::LocalDisk));
        let build_order = registry.record_volume_list(volumes, None);

        assert_eq!(registry.queue_builds(&build_order), 2);
        assert_eq!(registry.queue_builds(&build_order), 0, "nothing new queued");

        let first = registry.take_next_build().unwrap();
        let second = registry.take_next_build().unwrap();
        let third = registry.take_next_build().unwrap();
        assert_eq!(
            [first.mount_point, second.mount_point, third.mount_point],
            [r"C:\", r"D:\", r"E:\"].map(PathBuf::from)
        );
        assert!(registry.take_next_build().is_none());
        assert!(registry.take_next_build().is_none());
        assert_eq!(
            registry.queue_builds(&[PathBuf::from(r"C:\")]),
            0,
            "C: is still building"
        );
    }

    #[test]
    fn a_builder_slot_frees_up_when_its_thread_finds_no_work() {
        let registry = SearchIndexRegistry::default();
        let build_order = registry.record_volume_list(two_local_volumes_and_a_cloud_drive(), None);
        assert_eq!(registry.queue_builds(&build_order[..1]), 1);
        let built = registry.take_next_build().unwrap();
        registry.record_build_success(&built.mount_point, index_with_file(r"C:\", "a.txt"));
        assert!(registry.take_next_build().is_none());

        assert_eq!(registry.queue_builds(&build_order[1..]), 1);
        assert_eq!(
            registry.take_next_build().unwrap().mount_point,
            Path::new(r"D:\")
        );
    }

    #[test]
    fn moves_from_warming_to_ready_as_volumes_finish() {
        let registry = SearchIndexRegistry::default();
        let build_order = registry.record_volume_list(two_local_volumes_and_a_cloud_drive(), None);
        registry.queue_builds(&build_order);

        let c_volume = registry.take_next_build().unwrap();
        assert!(matches!(
            state_of(&registry, r"C:\"),
            VolumeIndexState::Building {
                method: ScanMethod::FolderWalk,
                ..
            }
        ));
        registry.record_build_success(&c_volume.mount_point, index_with_file(r"C:\", "a.txt"));

        let status = registry.status();
        assert_eq!(status.overall, OverallIndexStatus::Warming);
        assert_eq!(status.volumes[0].state, VolumeIndexState::Ready);
        assert_eq!(status.volumes[0].index.as_ref().unwrap().file_count, 1);
        let searchable = registry.searchable_indexes();
        assert_eq!(searchable.indexes.len(), 1);
        assert_eq!(searchable.volumes_still_warming, [PathBuf::from(r"D:\")]);

        let d_volume = registry.take_next_build().unwrap();
        registry.record_build_success(&d_volume.mount_point, index_with_file(r"D:\", "b.txt"));
        assert_eq!(registry.status().overall, OverallIndexStatus::Ready);
        assert_eq!(
            registry.searchable_indexes().volumes_still_warming,
            Vec::<PathBuf>::new()
        );
    }

    #[test]
    fn failures_make_the_search_partially_ready_or_failed() {
        let registry = SearchIndexRegistry::default();
        let build_order = registry.record_volume_list(two_local_volumes_and_a_cloud_drive(), None);
        registry.queue_builds(&build_order);

        let c_volume = registry.take_next_build().unwrap();
        let d_volume = registry.take_next_build().unwrap();
        registry.record_build_failure(&c_volume.mount_point, "access denied".to_owned());
        assert_eq!(registry.status().overall, OverallIndexStatus::Warming);

        registry.record_build_success(&d_volume.mount_point, index_with_file(r"D:\", "b.txt"));
        assert_eq!(
            registry.status().overall,
            OverallIndexStatus::PartiallyReady
        );
        assert_eq!(
            state_of(&registry, r"C:\"),
            VolumeIndexState::Failed {
                reason: "access denied".to_owned()
            }
        );

        let only_failures = SearchIndexRegistry::default();
        let order =
            only_failures.record_volume_list(vec![volume(r"C:\", VolumeKind::LocalDisk)], None);
        only_failures.queue_builds(&order);
        let volume = only_failures.take_next_build().unwrap();
        only_failures.record_build_failure(&volume.mount_point, "broken".to_owned());
        assert_eq!(only_failures.status().overall, OverallIndexStatus::Failed);
    }

    #[test]
    fn a_refresh_keeps_the_old_index_searchable_until_the_new_one_replaces_it() {
        let registry = SearchIndexRegistry::default();
        let order = registry.record_volume_list(vec![volume(r"C:\", VolumeKind::LocalDisk)], None);
        registry.queue_builds(&order);
        let c_volume = registry.take_next_build().unwrap();
        registry.record_build_success(&c_volume.mount_point, index_with_file(r"C:\", "old.txt"));
        assert!(registry.take_next_build().is_none());

        assert_eq!(registry.queue_builds(&order), 1);
        registry.take_next_build().unwrap();
        let status = registry.status();
        assert!(matches!(
            status.volumes[0].state,
            VolumeIndexState::Building { .. }
        ));
        assert!(status.volumes[0].index.is_some());
        assert_eq!(status.overall, OverallIndexStatus::Ready);
        let old_index = registry.index_holding(Path::new(r"C:\old.txt")).unwrap();
        assert_eq!(
            old_index.tree().name(disk_scanner::NodeIndex::new(1)),
            "old.txt"
        );

        registry.record_build_failure(&c_volume.mount_point, "disk busy".to_owned());
        assert!(registry.index_holding(Path::new(r"C:\old.txt")).is_some());
        assert_eq!(registry.status().overall, OverallIndexStatus::Ready);

        registry.queue_builds(&order);
        registry.take_next_build().unwrap();
        registry.record_build_success(&c_volume.mount_point, index_with_file(r"C:\", "new.txt"));
        let new_index = registry.index_holding(Path::new(r"C:\new.txt")).unwrap();
        assert_eq!(
            new_index.tree().name(disk_scanner::NodeIndex::new(1)),
            "new.txt"
        );
        assert_eq!(
            old_index.tree().name(disk_scanner::NodeIndex::new(1)),
            "old.txt",
            "a search holding the old index can still finish"
        );
    }

    #[test]
    fn refresh_targets_must_be_known_local_volumes() {
        let registry = SearchIndexRegistry::default();
        registry.record_volume_list(two_local_volumes_and_a_cloud_drive(), None);

        assert_eq!(registry.check_refresh_target(Path::new(r"C:\")), Ok(()));
        assert_eq!(
            registry.check_refresh_target(Path::new(r"G:\")),
            Err(RefreshRequestError::VolumeNotIndexed {
                mount_point: PathBuf::from(r"G:\"),
                reason: SkipReason::CloudDrive
            })
        );
        assert_eq!(
            registry.check_refresh_target(Path::new(r"Z:\")),
            Err(RefreshRequestError::VolumeNotFound {
                mount_point: PathBuf::from(r"Z:\")
            })
        );
        assert_eq!(registry.queue_builds(&[PathBuf::from(r"G:\")]), 0);
    }

    #[test]
    fn a_new_volume_list_keeps_indexes_and_drops_vanished_volumes() {
        let registry = SearchIndexRegistry::default();
        let order = registry.record_volume_list(two_local_volumes_and_a_cloud_drive(), None);
        registry.queue_builds(&order);
        let c_volume = registry.take_next_build().unwrap();
        registry.record_build_success(&c_volume.mount_point, index_with_file(r"C:\", "a.txt"));
        let _d_volume_building = registry.take_next_build().unwrap();

        let order = registry.record_volume_list(vec![volume(r"C:\", VolumeKind::LocalDisk)], None);

        assert_eq!(order, [PathBuf::from(r"C:\"), PathBuf::from(r"D:\")]);
        let mount_points: Vec<_> = registry
            .status()
            .volumes
            .into_iter()
            .map(|volume| volume.mount_point)
            .collect();
        assert_eq!(
            mount_points,
            [r"C:\", r"D:\"].map(PathBuf::from),
            "D: is kept while it builds; G: is gone"
        );
        assert!(registry.index_holding(Path::new(r"C:\a.txt")).is_some());
    }

    #[test]
    fn progress_events_are_throttled() {
        let registry = SearchIndexRegistry::default();
        let order = registry.record_volume_list(vec![volume(r"C:\", VolumeKind::LocalDisk)], None);
        registry.queue_builds(&order);
        registry.take_next_build().unwrap();

        assert!(registry.record_build_progress(Path::new(r"C:\"), 10, 2));
        assert!(!registry.record_build_progress(Path::new(r"C:\"), 20, 3));
        assert!(matches!(
            state_of(&registry, r"C:\"),
            VolumeIndexState::Building {
                files_so_far: 20,
                directories_so_far: 3,
                ..
            }
        ));
        assert!(!registry.record_build_progress(Path::new(r"Z:\"), 1, 1));
    }

    #[test]
    fn a_failed_builder_start_fails_the_queued_builds() {
        let registry = SearchIndexRegistry::default();
        let order = registry.record_volume_list(two_local_volumes_and_a_cloud_drive(), None);
        assert_eq!(registry.queue_builds(&order), 2);

        registry.record_builder_start_failure("no threads");
        assert_eq!(state_of(&registry, r"C:\"), VolumeIndexState::Idle);
        registry.record_builder_start_failure("no threads");

        assert_eq!(
            state_of(&registry, r"C:\"),
            VolumeIndexState::Failed {
                reason: "no threads".to_owned()
            }
        );
        assert_eq!(registry.status().overall, OverallIndexStatus::Failed);
        assert_eq!(registry.queue_builds(&order), 2, "a refresh can try again");
    }
}
