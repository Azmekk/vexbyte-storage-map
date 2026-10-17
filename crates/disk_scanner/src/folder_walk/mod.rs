//! Scan method A: lists every directory through the file system. Works on every volume and
//! needs no administrator rights.

mod directory_entry;
mod entry_batch;
mod parallel_directory_walker;
mod walk_progress;

use std::io;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::mpsc::sync_channel;

pub(crate) use directory_entry::{DirectoryEntry, DirectoryListError, EntryFlags};
use entry_batch::EntryBatch;
use parallel_directory_walker::{ParallelDirectoryWalker, ROOT_DIRECTORY_KEY};
use walk_progress::WalkProgress;

use crate::cancellation::CancellationToken;
use crate::file_tree::{FileTree, FileTreeBuilder, NewEntry};
use crate::scan_progress::{ScanPhase, ScanProgressReporter};

/// Batches waiting for the builder. Bounded so a slow consumer slows the walkers down instead
/// of buffering the whole drive in memory.
const BATCH_CHANNEL_CAPACITY: usize = 64;

#[derive(Debug, thiserror::Error)]
pub enum FolderWalkError {
    #[error("the folder {} does not exist or cannot be read", path.display())]
    RootNotFound { path: PathBuf, source: io::Error },
    #[error("{} is not a folder", path.display())]
    RootNotADirectory { path: PathBuf },
    #[error("could not start the worker threads for the folder walk")]
    WorkerPoolCreationFailed { source: rayon::ThreadPoolBuildError },
    #[error("the scan was cancelled")]
    Cancelled,
}

/// Walks `root_path` and everything below it in parallel and builds the file tree.
///
/// Never follows symlinks or junctions, never opens cloud placeholders, and records
/// directories that cannot be listed as inaccessible instead of failing.
pub(crate) fn scan_folder_with_walk(
    root_path: &Path,
    progress_reporter: &dyn ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<FileTree, FolderWalkError> {
    let root_path =
        std::path::absolute(root_path).map_err(|source| FolderWalkError::RootNotFound {
            path: root_path.to_path_buf(),
            source,
        })?;
    let root_metadata =
        std::fs::metadata(&root_path).map_err(|source| FolderWalkError::RootNotFound {
            path: root_path.clone(),
            source,
        })?;
    if !root_metadata.is_dir() {
        return Err(FolderWalkError::RootNotADirectory { path: root_path });
    }

    let worker_thread_count = walker_thread_count();
    let worker_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(worker_thread_count)
        .thread_name(|thread_number| format!("folder-walk-{thread_number}"))
        .build()
        .map_err(|source| FolderWalkError::WorkerPoolCreationFailed { source })?;

    let mut tree_builder = FileTreeBuilder::new(root_path.clone(), ROOT_DIRECTORY_KEY);
    tree_builder.set_root_details(
        root_metadata.modified().ok(),
        crate::file_tree::NodeFlags::empty(),
    );
    let mut walk_progress = WalkProgress::starting_now();

    let (batch_sender, batch_receiver) = sync_channel::<EntryBatch>(BATCH_CHANNEL_CAPACITY);
    let walker = ParallelDirectoryWalker::new(worker_thread_count, cancellation, batch_sender);

    std::thread::scope(|thread_scope| {
        let walk_root = root_path.clone();
        thread_scope.spawn(|| walker.walk(&worker_pool, walk_root));

        for batch in batch_receiver {
            add_batch_to_tree(&batch, &mut tree_builder);
            walk_progress.record_batch(&batch);
            walk_progress.report_if_due(progress_reporter);
        }
    });

    if cancellation.is_cancelled() {
        return Err(FolderWalkError::Cancelled);
    }

    progress_reporter.report_progress(walk_progress.snapshot(ScanPhase::BuildingTree));
    Ok(tree_builder.finish())
}

fn add_batch_to_tree(batch: &EntryBatch, tree_builder: &mut FileTreeBuilder) {
    for entry in batch.entries() {
        let details = &entry.details;
        tree_builder.add_entry(&NewEntry {
            name: batch.name_of(entry),
            kind: details.kind,
            parent: Some(details.parent),
            directory_key: details.directory_key,
            logical_size: details.logical_size,
            allocated_size: details.allocated_size,
            modified_time: Some(details.modified_time),
            flags: details.flags,
            hard_link_identity: details.identity,
        });
    }
    for &inaccessible_directory in batch.inaccessible_directories() {
        tree_builder.mark_directory_inaccessible(inaccessible_directory);
    }
}

/// Listing is dominated by waiting for the file system, so more threads than cores help;
/// beyond about 32 the file system's own locking stops further gains.
fn walker_thread_count() -> usize {
    let core_count = std::thread::available_parallelism().map_or(4, NonZeroUsize::get);
    (core_count * 2).clamp(4, 32)
}
