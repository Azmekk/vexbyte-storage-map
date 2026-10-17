use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::SyncSender;

use rayon::ThreadPool;

use super::directory_entry::DirectoryEntry;
use super::entry_batch::{BatchedEntryDetails, EntryBatch};
use crate::cancellation::CancellationToken;
use crate::file_tree::{DirectoryKey, NodeKind};
use crate::platform;

/// The directory key of the scan root. Every other directory gets the next free key.
pub(crate) const ROOT_DIRECTORY_KEY: DirectoryKey = DirectoryKey::new(0);

/// Lists directories in parallel: every directory is one task on a rayon scope that lists its
/// entries and spawns one task per subdirectory.
///
/// Each worker thread collects entries in its own batch (one lock per directory, never per
/// entry) and sends full batches to the tree builder over a channel.
pub(crate) struct ParallelDirectoryWalker<'scan> {
    cancellation: &'scan CancellationToken,
    batch_sender: SyncSender<EntryBatch>,
    batches_by_worker_thread: Vec<Mutex<EntryBatch>>,
    next_directory_key: AtomicU32,
}

struct DirectoryTask {
    path: PathBuf,
    key: DirectoryKey,
    top_level_directory: Option<DirectoryKey>,
}

impl<'scan> ParallelDirectoryWalker<'scan> {
    pub(crate) fn new(
        worker_thread_count: usize,
        cancellation: &'scan CancellationToken,
        batch_sender: SyncSender<EntryBatch>,
    ) -> Self {
        let batches_by_worker_thread = (0..worker_thread_count.max(1))
            .map(|_| Mutex::new(EntryBatch::default()))
            .collect();

        Self {
            cancellation,
            batch_sender,
            batches_by_worker_thread,
            next_directory_key: AtomicU32::new(ROOT_DIRECTORY_KEY.get() + 1),
        }
    }

    /// Walks everything below `root_path` on `worker_pool` and returns once every directory
    /// has been listed (or the scan was cancelled) and every batch has been sent.
    pub(crate) fn walk(self, worker_pool: &ThreadPool, root_path: PathBuf) {
        let root_task = DirectoryTask {
            path: root_path,
            key: ROOT_DIRECTORY_KEY,
            top_level_directory: None,
        };
        worker_pool.scope(|scope| self.walk_directory(scope, &root_task));

        for batch_slot in &self.batches_by_worker_thread {
            let remaining_batch = std::mem::take(&mut *lock_ignoring_poison(batch_slot));
            if !remaining_batch.is_empty() {
                self.send_batch(remaining_batch);
            }
        }
    }

    fn walk_directory<'task>(&'task self, scope: &rayon::Scope<'task>, task: &DirectoryTask) {
        if self.cancellation.is_cancelled() {
            return;
        }

        let subdirectories = self.list_into_batch(task);

        for subdirectory in subdirectories {
            scope.spawn(move |scope| self.walk_directory(scope, &subdirectory));
        }
    }

    /// Lists one directory into the current worker's batch and returns the subdirectories to
    /// walk next. A directory that cannot be listed is flagged inaccessible instead of failing
    /// the scan.
    fn list_into_batch(&self, task: &DirectoryTask) -> Vec<DirectoryTask> {
        let worker_slot =
            rayon::current_thread_index().unwrap_or(0) % self.batches_by_worker_thread.len();
        let mut batch = lock_ignoring_poison(&self.batches_by_worker_thread[worker_slot]);
        let mut subdirectories = Vec::new();

        let listing_result =
            platform::list_directory(&task.path, &mut |entry: DirectoryEntry<'_>| {
                let is_directory = entry.kind == NodeKind::Directory;
                let directory_key = is_directory.then(|| self.allocate_directory_key());
                let top_level_directory = if task.key == ROOT_DIRECTORY_KEY {
                    directory_key
                } else {
                    task.top_level_directory
                };

                if let Some(subdirectory_key) = directory_key
                    && !entry.flags.must_not_be_entered()
                {
                    subdirectories.push(DirectoryTask {
                        path: task.path.join(entry.name),
                        key: subdirectory_key,
                        top_level_directory,
                    });
                }

                batch.add_entry(
                    entry.name,
                    BatchedEntryDetails {
                        kind: entry.kind,
                        parent: task.key,
                        directory_key,
                        top_level_directory,
                        logical_size: entry.logical_size,
                        allocated_size: entry.allocated_size,
                        modified_time: entry.modified_time,
                        flags: entry.flags.to_node_flags(),
                        identity: if is_directory { None } else { entry.identity },
                    },
                );
            });

        if let Err(error) = listing_result {
            tracing::debug!(%error, "directory could not be listed and is marked inaccessible");
            batch.mark_directory_inaccessible(task.key);
        }

        if batch.is_ready_to_send() {
            let mut full_batch = std::mem::take(&mut *batch);
            drop(batch);
            full_batch.set_latest_directory_path(task.path.clone());
            self.send_batch(full_batch);
        }

        subdirectories
    }

    fn allocate_directory_key(&self) -> DirectoryKey {
        DirectoryKey::new(self.next_directory_key.fetch_add(1, Ordering::Relaxed))
    }

    fn send_batch(&self, batch: EntryBatch) {
        // A send only fails when the builder stopped receiving, which means the scan is over.
        let _ = self.batch_sender.send(batch);
    }
}

fn lock_ignoring_poison(batch_slot: &Mutex<EntryBatch>) -> std::sync::MutexGuard<'_, EntryBatch> {
    batch_slot
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
