use std::collections::HashMap;

use crate::file_identity::FileIdentity;

/// Whether a path is the first one seen for its file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkRegistration {
    /// The first path to this file. Its size is counted.
    FirstLink,
    /// The file was already seen through another path. Its size is not counted again.
    AdditionalLink { first_link_entry: u32 },
}

/// Remembers every file identity already counted while a tree is built, so that each file
/// with several hard links adds its size to the totals exactly once.
///
/// Entries are identified by their position in the builder's pending list.
#[derive(Default)]
pub(crate) struct HardLinkTracker {
    first_link_entry_by_identity: HashMap<FileIdentity, u32>,
    /// `(first link, additional link)` pairs, in the order the additional links were seen.
    additional_links: Vec<(u32, u32)>,
}

impl HardLinkTracker {
    pub(crate) fn register_link(&mut self, identity: FileIdentity, entry: u32) -> LinkRegistration {
        match self.first_link_entry_by_identity.entry(identity) {
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(entry);
                LinkRegistration::FirstLink
            }
            std::collections::hash_map::Entry::Occupied(occupied) => {
                let first_link_entry = *occupied.get();
                self.additional_links.push((first_link_entry, entry));
                LinkRegistration::AdditionalLink { first_link_entry }
            }
        }
    }

    /// Groups of entries that are links to the same file, first link first.
    pub(crate) fn into_link_groups(self) -> Vec<Vec<u32>> {
        let mut group_index_by_first_link: HashMap<u32, usize> = HashMap::new();
        let mut groups: Vec<Vec<u32>> = Vec::new();

        for (first_link_entry, additional_link_entry) in self.additional_links {
            let group_index = *group_index_by_first_link
                .entry(first_link_entry)
                .or_insert_with(|| {
                    groups.push(vec![first_link_entry]);
                    groups.len() - 1
                });
            groups[group_index].push(additional_link_entry);
        }

        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(file_id: u128) -> FileIdentity {
        FileIdentity::from_volume_and_file_ids(1, file_id)
    }

    #[test]
    fn counts_each_identity_once() {
        let mut tracker = HardLinkTracker::default();

        assert_eq!(
            tracker.register_link(identity(10), 0),
            LinkRegistration::FirstLink
        );
        assert_eq!(
            tracker.register_link(identity(11), 1),
            LinkRegistration::FirstLink
        );
        assert_eq!(
            tracker.register_link(identity(10), 2),
            LinkRegistration::AdditionalLink {
                first_link_entry: 0
            }
        );
        assert_eq!(
            tracker.register_link(identity(10), 3),
            LinkRegistration::AdditionalLink {
                first_link_entry: 0
            }
        );

        assert_eq!(tracker.into_link_groups(), vec![vec![0, 2, 3]]);
    }

    #[test]
    fn distinguishes_volumes() {
        let mut tracker = HardLinkTracker::default();

        let on_first_volume = FileIdentity::from_volume_and_file_ids(1, 5);
        let on_second_volume = FileIdentity::from_volume_and_file_ids(2, 5);

        assert_eq!(
            tracker.register_link(on_first_volume, 0),
            LinkRegistration::FirstLink
        );
        assert_eq!(
            tracker.register_link(on_second_volume, 1),
            LinkRegistration::FirstLink
        );
    }
}
