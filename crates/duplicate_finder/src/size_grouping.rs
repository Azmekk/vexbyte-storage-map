use std::collections::HashMap;

use crate::candidate_collector::Candidate;

/// Groups candidates by exact size and drops sizes that only one file has, since a file with a
/// unique size cannot have a duplicate.
pub(crate) fn group_by_exact_size(candidates: Vec<Candidate>) -> Vec<Vec<Candidate>> {
    let mut candidates_by_size: HashMap<u64, Vec<Candidate>> = HashMap::new();
    for candidate in candidates {
        candidates_by_size
            .entry(candidate.size)
            .or_default()
            .push(candidate);
    }

    candidates_by_size
        .into_values()
        .filter(|group| group.len() > 1)
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;
    use std::time::SystemTime;

    use super::*;

    fn candidate(name: &str, size: u64) -> Candidate {
        Candidate {
            path: PathBuf::from(name),
            size,
            modified_time: SystemTime::UNIX_EPOCH,
            created_time: SystemTime::UNIX_EPOCH,
            search_folder_index: 0,
        }
    }

    #[test]
    fn keeps_only_sizes_shared_by_several_files() {
        let candidates = vec![
            candidate("a", 100),
            candidate("b", 200),
            candidate("c", 100),
            candidate("d", 300),
            candidate("e", 300),
            candidate("f", 300),
        ];

        let mut groups = group_by_exact_size(candidates);
        groups.sort_by_key(|group| group[0].size);

        let group_sizes: Vec<(u64, usize)> = groups
            .iter()
            .map(|group| (group[0].size, group.len()))
            .collect();
        assert_eq!(group_sizes, vec![(100, 2), (300, 3)]);
    }
}
