use std::path::Path;

use rayon::prelude::*;

use crate::drive_kind::ReadStrategy;

/// Runs `read_file` over every item, choosing per item how its drive is read: files on SSDs are
/// read in parallel on the rayon pool, files on hard disks one at a time in path order (which
/// roughly follows their position on disk). Both sets run at the same time.
pub(crate) fn read_files_with_drive_strategy<Item, Output>(
    items: Vec<Item>,
    strategy_of: impl Fn(&Item) -> ReadStrategy,
    path_of: impl Fn(&Item) -> &Path + Sync,
    read_file: impl Fn(Item) -> Output + Sync,
) -> Vec<Output>
where
    Item: Send,
    Output: Send,
{
    let (parallel_items, mut sequential_items): (Vec<Item>, Vec<Item>) = items
        .into_iter()
        .partition(|item| strategy_of(item) == ReadStrategy::Parallel);
    sequential_items.sort_by(|left, right| path_of(left).cmp(path_of(right)));

    let (mut parallel_outputs, sequential_outputs) = rayon::join(
        || {
            parallel_items
                .into_par_iter()
                .map(&read_file)
                .collect::<Vec<_>>()
        },
        || {
            sequential_items
                .into_iter()
                .map(&read_file)
                .collect::<Vec<_>>()
        },
    );

    parallel_outputs.extend(sequential_outputs);
    parallel_outputs
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn reads_sequential_items_in_path_order_and_returns_every_output() {
        let items = vec![
            (PathBuf::from("c"), ReadStrategy::SequentialInPathOrder),
            (PathBuf::from("x"), ReadStrategy::Parallel),
            (PathBuf::from("a"), ReadStrategy::SequentialInPathOrder),
            (PathBuf::from("b"), ReadStrategy::SequentialInPathOrder),
        ];
        let sequential_read_order = Mutex::new(Vec::new());

        let mut outputs = read_files_with_drive_strategy(
            items,
            |(_, strategy)| *strategy,
            |(path, _)| path.as_path(),
            |(path, strategy)| {
                if strategy == ReadStrategy::SequentialInPathOrder {
                    sequential_read_order.lock().unwrap().push(path.clone());
                }
                path
            },
        );

        outputs.sort();
        assert_eq!(outputs.len(), 4);
        assert_eq!(
            sequential_read_order.into_inner().unwrap(),
            vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")]
        );
    }
}
