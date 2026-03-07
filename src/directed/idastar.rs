//! Compute a shortest path using the [IDA* search
//! algorithm](https://en.wikipedia.org/wiki/Iterative_deepening_A*).

use indexmap::IndexSet;
use num_traits::Zero;
use std::{hash::Hash, ops::ControlFlow};

/// Compute a shortest path using the [IDA* search
/// algorithm](https://en.wikipedia.org/wiki/Iterative_deepening_A*).
///
/// The shortest path starting from `start` up to a node for which `success` returns `true` is
/// computed and returned along with its total cost, in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node, along with the cost for moving
///   from the node to the successor. This cost must be non-negative.
/// - `heuristic` returns an approximation of the cost from a given node to the goal. The
///   approximation must not be greater than the real cost, or a wrong shortest path may be returned.
/// - `success` checks whether the goal has been reached. It is not a node as some problems require
///   a dynamic solution instead of a fixed node.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node.
///
/// # Example
///
/// ```
/// use pathfinding_faster::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 1), (2, 2)],
///     vec![(2, 1), (3, 5)],
///     vec![(3, 1)],
///     vec![],
/// ]);
/// let goal = 3usize;
/// let result = graph.idastar(0, |n| goal.saturating_sub(n), |n| n == goal);
/// assert_eq!(result, Some((vec![0, 1, 2, 3], 3)));
/// ```
pub fn idastar<N, C, FN, IN, FH, FS>(
    start: &N,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Clone + Hash,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let mut path = IndexSet::from([start.clone()]);

    std::iter::repeat(())
        .try_fold(heuristic(start), |bound, ()| {
            search(
                &mut path,
                Zero::zero(),
                bound,
                &mut successors,
                &mut heuristic,
                &mut success,
            )
            .map_break(Some)?
            // .filter(|min| *min > bound)
            .map_or(ControlFlow::Break(None), ControlFlow::Continue)
        })
        .break_value()
        .unwrap_or_default() // To avoid a missing panics section, as this always break
}

fn search<N, C, FN, IN, FH, FS>(
    path: &mut IndexSet<N>,
    cost: C,
    bound: C,
    successors: &mut FN,
    heuristic: &mut FH,
    success: &mut FS,
) -> ControlFlow<(Vec<N>, C), Option<C>>
where
    N: Eq + Clone + Hash,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let neighbs = {
        let start = &path[path.len() - 1];
        let f = cost + heuristic(start);
        if f > bound {
            return ControlFlow::Continue(Some(f));
        }
        if success(start) {
            return ControlFlow::Break((path.iter().cloned().collect(), f));
        }
        let mut neighbs: Vec<(N, C, C)> = successors(start)
            .into_iter()
            .filter_map(|(n, c)| {
                (!path.contains(&n)).then(|| {
                    let h = heuristic(&n);
                    (n, c, c + h)
                })
            })
            .collect::<Vec<_>>();
        neighbs.sort_unstable_by(|(_, _, c1), (_, _, c2)| c1.cmp(c2));
        neighbs
    };
    let mut min = None;
    for (node, extra, _) in neighbs {
        let (idx, _) = path.insert_full(node);
        match search(path, cost + extra, bound, successors, heuristic, success)? {
            Some(m) if min.is_none_or(|n| n >= m) => min = Some(m),
            _ => (),
        }
        path.swap_remove_index(idx);
    }
    ControlFlow::Continue(min)
}
