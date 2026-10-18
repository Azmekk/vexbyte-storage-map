//! End-to-end tests of the duplicate search over real files in a temporary folder. Everything is
//! created under `std::env::temp_dir()` and removed when the fixture is dropped.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod platform;

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use duplicate_finder::{
    CancellationToken, DuplicateGroup, DuplicateSearchError, DuplicateSearchResult,
    DuplicateSearchSettings, DuplicateSearchStage, FileCategory, FileChangeReason, KeepRule,
    RemovalPlan, RemovalPlanError, SearchSettingsError, SystemFolderGuard, apply_keep_rule,
    find_duplicates, find_duplicates_with_validated_settings, revalidate_before_removal,
};

const KIB: usize = 1024;

/// Content that differs from every other fixture's content from the first byte on.
fn patterned_content(seed: u8, length: usize) -> Vec<u8> {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64 ^ (u64::from(seed) << 32 | u64::from(seed));
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state.to_le_bytes()[3]
        })
        .collect()
}

fn at_second(second: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_600_000_000 + second)
}

struct DuplicateFixture {
    root: PathBuf,
}

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

impl DuplicateFixture {
    /// Builds this tree (sizes in KiB):
    ///
    /// - three copies of a 200 KiB photo in nested folders, with different modification times
    /// - two equal-size photos that differ only in the middle (survive the quick check)
    /// - two equal-size photos that differ at the start (split by the quick check)
    /// - two identical 100 KiB PDFs (hashed completely by the quick check)
    /// - two identical 10 KiB photos (below the 64 KiB minimum)
    /// - two identical 200 KiB zip files (category not selected)
    /// - a unique photo with a hard link to it (same file, never a duplicate)
    /// - a 300 KiB video frame with a hard link and one real copy (a group of two)
    fn create() -> Self {
        let unique_suffix = format!(
            "{}-{}-{}",
            std::process::id(),
            FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(format!("vsm-duplicate-finder-test-{unique_suffix}"));
        let fixture = Self { root };

        let beach = patterned_content(1, 200 * KIB);
        fixture.write("photos/2020/beach.jpg", &beach, at_second(2_000));
        fixture.write(
            "photos/backup/nested/deep/beach_copy.jpg",
            &beach,
            at_second(1_000),
        );
        fixture.write("downloads/beach (1).jpg", &beach, at_second(3_000));

        let mut middle_variant = patterned_content(2, 200 * KIB);
        fixture.write("photos/middle_a.jpg", &middle_variant, at_second(10));
        middle_variant[100 * KIB] ^= 0xFF;
        fixture.write("photos/middle_b.jpg", &middle_variant, at_second(10));

        let mut start_variant = patterned_content(3, 200 * KIB);
        fixture.write("photos/start_a.png", &start_variant, at_second(10));
        start_variant[0] ^= 0xFF;
        fixture.write("photos/start_b.png", &start_variant, at_second(10));

        let report = patterned_content(4, 100 * KIB);
        fixture.write("documents/report.pdf", &report, at_second(10));
        fixture.write("documents/old/report copy.PDF", &report, at_second(20));

        let thumbnail = patterned_content(5, 10 * KIB);
        fixture.write("photos/tiny_a.jpg", &thumbnail, at_second(10));
        fixture.write("photos/tiny_b.jpg", &thumbnail, at_second(10));

        let archive = patterned_content(6, 200 * KIB);
        fixture.write("downloads/archive.zip", &archive, at_second(10));
        fixture.write("downloads/archive copy.zip", &archive, at_second(10));

        fixture.write(
            "photos/unique.jpg",
            &patterned_content(7, 150 * KIB),
            at_second(10),
        );
        fixture.hard_link("photos/unique.jpg", "downloads/unique link.jpg");

        let frame = patterned_content(8, 300 * KIB);
        fixture.write("videos/frame.jpg", &frame, at_second(10));
        fixture.hard_link("videos/frame.jpg", "videos/later/frame link.jpg");
        fixture.write("downloads/frame copy.jpg", &frame, at_second(20));

        fixture
    }

    fn path(&self, relative_path: &str) -> PathBuf {
        relative_path
            .split('/')
            .fold(self.root.clone(), |path, component| path.join(component))
    }

    fn write(&self, relative_path: &str, content: &[u8], modified_time: SystemTime) {
        let path = self.path(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(modified_time)
            .unwrap();
    }

    fn hard_link(&self, existing: &str, link: &str) {
        let link_path = self.path(link);
        fs::create_dir_all(link_path.parent().unwrap()).unwrap();
        fs::hard_link(self.path(existing), link_path).unwrap();
    }

    fn settings(&self) -> DuplicateSearchSettings {
        DuplicateSearchSettings::new(
            ["photos", "downloads", "documents", "videos"]
                .iter()
                .map(|folder| self.path(folder))
                .collect(),
            vec![FileCategory::Images, FileCategory::Pdfs],
        )
    }

    /// The real system folder guard would reject the fixtures, because the temp directory lies
    /// inside `AppData` (Windows) or is `/tmp` (Linux). Only the folders of the operating system
    /// itself count as system folders here.
    fn test_guard() -> SystemFolderGuard {
        SystemFolderGuard::with_system_folders(platform::operating_system_folders())
    }

    fn search(&self) -> DuplicateSearchResult {
        Self::search_with(&self.settings(), |_| {}, &CancellationToken::new()).unwrap()
    }

    fn search_with(
        settings: &DuplicateSearchSettings,
        progress_callback: impl Fn(&duplicate_finder::DuplicateSearchProgress) + Sync,
        cancellation: &CancellationToken,
    ) -> Result<DuplicateSearchResult, DuplicateSearchError> {
        let validated_settings = settings.validate_with_guard(Self::test_guard()).unwrap();
        find_duplicates_with_validated_settings(
            &validated_settings,
            progress_callback,
            cancellation,
        )
    }
}

impl Drop for DuplicateFixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.root) {
            eprintln!("could not remove {}: {error}", self.root.display());
        }
    }
}

fn relative_paths(fixture: &DuplicateFixture, group: &DuplicateGroup) -> Vec<String> {
    group
        .files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(&fixture.root)
                .unwrap()
                .components()
                .map(|component| component.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect()
}

fn kept_relative_path(fixture: &DuplicateFixture, group: &DuplicateGroup) -> String {
    assert_eq!(group.kept_file_count(), 1);
    let kept_index = group.files.iter().position(|file| file.keep).unwrap();
    relative_paths(fixture, group).swap_remove(kept_index)
}

#[test]
fn finds_exactly_the_real_duplicates_sorted_by_wasted_space() {
    let fixture = DuplicateFixture::create();

    let result = fixture.search();

    let found_groups: Vec<Vec<String>> = result
        .groups
        .iter()
        .map(|group| relative_paths(&fixture, group))
        .collect();
    assert_eq!(
        found_groups,
        vec![
            vec![
                "downloads/beach (1).jpg".to_owned(),
                "photos/2020/beach.jpg".to_owned(),
                "photos/backup/nested/deep/beach_copy.jpg".to_owned(),
            ],
            vec![
                "downloads/frame copy.jpg".to_owned(),
                "videos/frame.jpg".to_owned(),
            ],
            vec![
                "documents/old/report copy.PDF".to_owned(),
                "documents/report.pdf".to_owned(),
            ],
        ]
    );

    let wasted: Vec<u64> = result
        .groups
        .iter()
        .map(|group| group.wasted_bytes)
        .collect();
    assert_eq!(wasted, vec![400 * 1024, 300 * 1024, 100 * 1024]);
    assert_eq!(result.total_wasted_bytes, 800 * 1024);
    assert_eq!(result.duplicate_file_count, 7);
    assert_eq!(result.hard_links_collapsed, 2);
    assert_eq!(result.online_only_files_skipped, 0);
    assert_eq!(result.unreadable_file_count, 0);
    // 3 beach + 2 middle + 2 start + 2 pdf + 1 unique + 2 frame (the links are collapsed).
    assert_eq!(result.files_examined, 12);
    let ids: Vec<u32> = result.groups.iter().map(|group| group.id.0).collect();
    assert_eq!(ids, vec![0, 1, 2]);
}

#[test]
fn keeps_the_oldest_copy_by_default_and_never_zero_copies() {
    let fixture = DuplicateFixture::create();

    let result = fixture.search();

    for group in &result.groups {
        assert_eq!(group.kept_file_count(), 1, "group {:?}", group.id);
    }
    assert_eq!(
        kept_relative_path(&fixture, &result.groups[0]),
        "photos/backup/nested/deep/beach_copy.jpg"
    );
}

#[test]
fn applies_every_keep_rule() {
    let fixture = DuplicateFixture::create();
    let mut beach_group = fixture.search().groups.swap_remove(0);

    let cases = [
        (KeepRule::Oldest, "photos/backup/nested/deep/beach_copy.jpg"),
        (KeepRule::Newest, "downloads/beach (1).jpg"),
        (KeepRule::ShortestPath, "photos/2020/beach.jpg"),
        (
            KeepRule::PreferredFolder(fixture.path("downloads")),
            "downloads/beach (1).jpg",
        ),
        (
            KeepRule::PreferredFolder(fixture.path("documents")),
            "photos/backup/nested/deep/beach_copy.jpg",
        ),
    ];

    for (rule, expected_kept_path) in cases {
        apply_keep_rule(&mut beach_group, &rule);
        assert_eq!(
            kept_relative_path(&fixture, &beach_group),
            expected_kept_path,
            "{rule:?}"
        );
    }
}

#[test]
fn removal_plan_refuses_to_remove_every_copy() {
    let fixture = DuplicateFixture::create();
    let mut groups = fixture.search().groups;

    let every_beach_copy: Vec<PathBuf> = groups[0]
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect();
    assert_eq!(
        RemovalPlan::from_files_to_remove(&groups, &every_beach_copy),
        Err(RemovalPlanError::GroupWouldHaveNoKeptFile {
            group_id: groups[0].id
        })
    );

    for file in &mut groups[1].files {
        file.keep = false;
    }
    assert_eq!(
        RemovalPlan::from_keep_flags(&groups),
        Err(RemovalPlanError::GroupWouldHaveNoKeptFile {
            group_id: groups[1].id
        })
    );
}

#[test]
fn revalidation_skips_a_file_modified_after_the_search() {
    let fixture = DuplicateFixture::create();
    let groups = fixture.search().groups;
    let plan = RemovalPlan::from_keep_flags(&groups).unwrap();
    assert_eq!(plan.removals().len(), 4);

    let modified_path = fixture.path("downloads/beach (1).jpg");
    OpenOptions::new()
        .append(true)
        .open(&modified_path)
        .unwrap()
        .write_all(b"edited")
        .unwrap();
    let outcome = revalidate_before_removal(&plan);

    assert_eq!(outcome.still_valid.len(), 3);
    assert_eq!(outcome.changed.len(), 1);
    assert_eq!(outcome.changed[0].path, modified_path);
    assert_eq!(outcome.changed[0].reason, FileChangeReason::SizeChanged);
}

#[test]
fn revalidation_skips_a_group_whose_kept_copy_changed() {
    let fixture = DuplicateFixture::create();
    let groups = fixture.search().groups;
    let plan = RemovalPlan::from_keep_flags(&groups).unwrap();

    let kept_beach_copy = fixture.path("photos/backup/nested/deep/beach_copy.jpg");
    File::options()
        .write(true)
        .open(&kept_beach_copy)
        .unwrap()
        .set_modified(at_second(9_999))
        .unwrap();
    let outcome = revalidate_before_removal(&plan);

    let skipped: Vec<&Path> = outcome
        .changed
        .iter()
        .map(|changed| changed.path.as_path())
        .collect();
    assert_eq!(skipped.len(), 2);
    assert!(
        outcome
            .changed
            .iter()
            .all(|changed| changed.reason == FileChangeReason::NoKeptCopyLeft)
    );
    assert_eq!(outcome.still_valid.len(), 2);
}

#[test]
fn cancellation_before_the_full_check_stops_the_search() {
    let fixture = DuplicateFixture::create();
    let cancellation = CancellationToken::new();
    let reported_stages = Mutex::new(Vec::new());

    let result = DuplicateFixture::search_with(
        &fixture.settings(),
        |progress| {
            reported_stages.lock().unwrap().push(progress.stage);
            if progress.stage == DuplicateSearchStage::QuickCheck {
                cancellation.cancel();
            }
        },
        &cancellation,
    );

    assert!(matches!(result, Err(DuplicateSearchError::Cancelled)));
    let reported_stages = reported_stages.into_inner().unwrap();
    assert!(!reported_stages.contains(&DuplicateSearchStage::FullCheck));
    assert!(!reported_stages.contains(&DuplicateSearchStage::Done));
}

#[test]
fn an_already_cancelled_search_returns_immediately() {
    let fixture = DuplicateFixture::create();
    let cancellation = CancellationToken::new();
    cancellation.cancel();

    let result = DuplicateFixture::search_with(&fixture.settings(), |_| {}, &cancellation);

    assert!(matches!(result, Err(DuplicateSearchError::Cancelled)));
}

#[test]
fn progress_ends_with_done_and_final_counts() {
    let fixture = DuplicateFixture::create();
    let last_progress = Mutex::new(None);

    DuplicateFixture::search_with(
        &fixture.settings(),
        |progress| *last_progress.lock().unwrap() = Some(progress.clone()),
        &CancellationToken::new(),
    )
    .unwrap();

    let last_progress = last_progress.into_inner().unwrap().unwrap();
    assert_eq!(last_progress.stage, DuplicateSearchStage::Done);
    assert_eq!(last_progress.files_collected, 12);
    assert_eq!(last_progress.duplicate_groups_found, 3);
    assert_eq!(
        last_progress.quick_check_files_done,
        last_progress.quick_check_files_total
    );
    // Only the beach photos, the frames and the two middle variants are fully hashed.
    assert_eq!(last_progress.full_check_files_total, 7);
    assert_eq!(
        last_progress.full_check_bytes_done,
        last_progress.full_check_bytes_total
    );
}

#[test]
fn a_lower_minimum_size_includes_small_duplicates() {
    let fixture = DuplicateFixture::create();
    let mut settings = fixture.settings();
    settings.minimum_file_size_bytes = 1;

    let result =
        DuplicateFixture::search_with(&settings, |_| {}, &CancellationToken::new()).unwrap();

    let tiny_group = result.groups.last().unwrap();
    assert_eq!(
        relative_paths(&fixture, tiny_group),
        vec!["photos/tiny_a.jpg", "photos/tiny_b.jpg"]
    );
}

#[test]
fn selecting_archives_finds_the_zip_copies() {
    let fixture = DuplicateFixture::create();
    let mut settings = fixture.settings();
    settings.categories = vec![FileCategory::ArchivesAndInstallers];

    let result =
        DuplicateFixture::search_with(&settings, |_| {}, &CancellationToken::new()).unwrap();

    assert_eq!(result.groups.len(), 1);
    assert_eq!(
        relative_paths(&fixture, &result.groups[0]),
        vec!["downloads/archive copy.zip", "downloads/archive.zip"]
    );
}

#[test]
fn rejects_system_folders() {
    let mut system_folders = platform::operating_system_folders();
    system_folders.extend([dirs::data_local_dir().unwrap(), std::env::temp_dir()]);

    // A fresh Linux home may have no ~/.local/share yet; a missing folder is rejected earlier as
    // not found, which is not what this test is about.
    for system_folder in system_folders.into_iter().filter(|folder| folder.exists()) {
        let settings =
            DuplicateSearchSettings::new(vec![system_folder.clone()], vec![FileCategory::Images]);

        let result = find_duplicates(&settings, |_| {}, &CancellationToken::new());

        assert!(
            matches!(
                result,
                Err(DuplicateSearchError::InvalidSettings(
                    SearchSettingsError::SystemFolderNotAllowed { .. }
                ))
            ),
            "{} was not rejected: {result:?}",
            system_folder.display()
        );
    }
}

#[test]
fn rejects_missing_folders_and_files() {
    let fixture = DuplicateFixture::create();
    let guard = DuplicateFixture::test_guard;

    let missing = DuplicateSearchSettings::new(
        vec![fixture.path("does-not-exist")],
        vec![FileCategory::Images],
    );
    let file_instead_of_folder = DuplicateSearchSettings::new(
        vec![fixture.path("photos/unique.jpg")],
        vec![FileCategory::Images],
    );

    assert!(matches!(
        missing.validate_with_guard(guard()),
        Err(SearchSettingsError::FolderNotFound { .. })
    ));
    assert!(matches!(
        file_instead_of_folder.validate_with_guard(guard()),
        Err(SearchSettingsError::NotAFolder { .. })
    ));
}
