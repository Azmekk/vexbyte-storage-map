use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

/// Counts files by their final (size, hash) key from many hashing threads and tells the caller
/// when a key gets its second file, which is the moment a duplicate group is confirmed. Lets the
/// progress counter of duplicate groups grow while hashing runs instead of only at the end.
pub(crate) struct ConfirmedGroupCounter<Key> {
    files_per_key: Mutex<HashMap<Key, u32>>,
}

impl<Key: Eq + Hash> ConfirmedGroupCounter<Key> {
    pub(crate) fn new() -> Self {
        Self {
            files_per_key: Mutex::new(HashMap::new()),
        }
    }

    /// Records one hashed file. Returns `true` when this file confirmed a new duplicate group.
    pub(crate) fn record_file_and_check_new_group(&self, key: Key) -> bool {
        let mut files_per_key = match self.files_per_key.lock() {
            Ok(files_per_key) => files_per_key,
            Err(poisoned) => poisoned.into_inner(),
        };
        let file_count = files_per_key.entry(key).or_insert(0);
        *file_count += 1;
        *file_count == 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_each_group_once_when_its_second_file_arrives() {
        let counter = ConfirmedGroupCounter::new();

        assert!(!counter.record_file_and_check_new_group((10_u64, 1_u8)));
        assert!(counter.record_file_and_check_new_group((10, 1)));
        assert!(!counter.record_file_and_check_new_group((10, 1)));
        assert!(!counter.record_file_and_check_new_group((10, 2)));
        assert!(!counter.record_file_and_check_new_group((20, 1)));
        assert!(counter.record_file_and_check_new_group((20, 1)));
    }
}
