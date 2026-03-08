//! Count the total number of possible paths to reach a destination.

use std::hash::Hash;

use rustc_hash::FxHashMap;

fn cached_count_paths<T, FN, IN, FS>(
    start: T,
    successors: &mut FN,
    success: &mut FS,
    cache: &mut FxHashMap<T, usize>,
) -> usize
where
    T: Eq + Hash,
    FN: FnMut(&T) -> IN,
    IN: IntoIterator<Item = T>,
    FS: FnMut(&T) -> bool,
{
    if let Some(&n) = cache.get(&start) {
        return n;
    }

    let count = if success(&start) {
        1
    } else {
        successors(&start)
            .into_iter()
            .map(|successor| cached_count_paths(successor, successors, success, cache))
            .sum()
    };

    cache.insert(start, count);

    count
}

/// Count the total number of possible paths to reach a destination. There must be no loops
/// in the graph, or the function will overflow its stack.
///
/// # Example
///
/// ```
/// use pathfinding_indexed::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 1), (2, 1)],
///     vec![(3, 1)],
///     vec![(3, 1)],
///     vec![],
/// ]);
/// let count = graph.count_paths(0, |n| n == 3);
/// assert_eq!(count, 2);
/// ```
pub fn count_paths<T, FN, IN, FS>(start: T, mut successors: FN, mut success: FS) -> usize
where
    T: Eq + Hash,
    FN: FnMut(&T) -> IN,
    IN: IntoIterator<Item = T>,
    FS: FnMut(&T) -> bool,
{
    cached_count_paths(
        start,
        &mut successors,
        &mut success,
        &mut FxHashMap::default(),
    )
}
