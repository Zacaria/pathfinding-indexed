//! Compute a shortest path using the [breadth-first search
//! algorithm](https://en.wikipedia.org/wiki/Breadth-first_search).

use super::reverse_path;
use crate::{FxIndexMap, FxIndexSet, noderefs::NodeRefs};
use indexmap::map::Entry::Vacant;
use std::hash::Hash;
use std::iter::FusedIterator;

/// Compute a shortest path using the [breadth-first search
/// algorithm](https://en.wikipedia.org/wiki/Breadth-first_search).
///
/// The shortest path starting from `start` up to a node for which `success` returns `true` is
/// computed and returned in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
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
///     vec![(1, 1), (2, 1)],
///     vec![(3, 1)],
///     vec![(3, 1)],
///     vec![],
/// ]);
/// let path = graph.bfs(0, |n| n == 3).expect("path not found");
/// assert_eq!(path, vec![0, 1, 3]);
/// ```
pub fn bfs<'a, N, S, FN, IN, FS>(start: S, successors: FN, success: FS) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    S: Into<NodeRefs<'a, N>>,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    bfs_core(&start.into(), successors, success, true)
}

fn bfs_core<'a, N, FN, IN, FS>(
    start: &NodeRefs<'a, N>,
    mut successors: FN,
    mut success: FS,
    check_first: bool,
) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    if check_first {
        for start_node in start {
            if success(start_node) {
                return Some(vec![start_node.clone()]);
            }
        }
    }

    let mut parents: FxIndexMap<N, usize> = FxIndexMap::default();
    parents.extend(start.into_iter().map(|n| (n.clone(), usize::MAX)));

    let mut i = 0;
    while let Some((node, _)) = parents.get_index(i) {
        for successor in successors(node) {
            if success(&successor) {
                let mut path = reverse_path(&parents, |&p| p, i);
                path.push(successor);
                return Some(path);
            }
            if let Vacant(e) = parents.entry(successor) {
                e.insert(i);
            }
        }
        i += 1;
    }
    None
}

/// Return one of the shortest loop from start to start if it exists, `None` otherwise.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
///
/// Except the start node which will be included both at the beginning and the end of
/// the path, a node will never be included twice in the path as determined
/// by the `Eq` relationship.
pub fn bfs_loop<'a, N, S, FN, IN>(start: S, successors: FN) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    S: Into<NodeRefs<'a, N>>,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    let start = start.into();
    bfs_core(&start, successors, |n| start.contains(n), false)
}

/// Compute a shortest path using the [breadth-first search
/// algorithm](https://en.wikipedia.org/wiki/Breadth-first_search) with
/// [bidirectional search](https://en.wikipedia.org/wiki/Bidirectional_search).
///
/// Bidirectional search runs two simultaneous searches: one forward from the start,
/// and one backward from the end, stopping when the two meet. In many cases this gives
/// a faster result than searching only in a single direction.
///
/// The shortest path starting from `start` up to a node `end` is
/// computed and returned in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `end` is the end node.
/// - `successors_fn` returns a list of successors for a given node.
/// - `predecessors_fn` returns a list of predecessors for a given node. For an undirected graph
///   this will be the same as `successors_fn`, however for a directed graph this will be different.
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
///     vec![(1, 1), (2, 1)],
///     vec![(3, 1)],
///     vec![(3, 1)],
///     vec![],
/// ]);
/// let path = graph
///     .bfs_bidirectional(0, 3)
///     .expect("path not found");
/// assert_eq!(path.first(), Some(&0));
/// assert_eq!(path.last(), Some(&3));
/// assert_eq!(path.len(), 3);
/// ```
///
/// Find also a more interesting example, comparing regular
/// and bidirectional BFS [here](https://github.com/evenfurther/pathfinding/blob/main/examples/bfs_bidirectional.rs).
pub fn bfs_bidirectional<'a, N, S, E, FNS, FNP, IN>(
    start: S,
    end: E,
    successors_fn: FNS,
    predecessors_fn: FNP,
) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone + 'a,
    E: Into<NodeRefs<'a, N>>,
    S: Into<NodeRefs<'a, N>>,
    FNS: Fn(&N) -> IN,
    FNP: Fn(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    let start = start.into();
    let end = end.into();

    let mut predecessors: FxIndexMap<N, Option<usize>> = FxIndexMap::default();
    predecessors.extend(start.into_iter().cloned().map(|n| (n, None)));
    let mut successors: FxIndexMap<N, Option<usize>> = FxIndexMap::default();
    successors.extend(end.into_iter().cloned().map(|n| (n, None)));

    let mut i_forwards = 0;
    let mut i_backwards = 0;
    let middle = 'l: loop {
        for _ in 0..(predecessors.len() - i_forwards) {
            let node = predecessors.get_index(i_forwards).unwrap().0;
            for successor_node in successors_fn(node) {
                if !predecessors.contains_key(&successor_node) {
                    predecessors.insert(successor_node.clone(), Some(i_forwards));
                }
                if successors.contains_key(&successor_node) {
                    break 'l Some(successor_node);
                }
            }
            i_forwards += 1;
        }

        for _ in 0..(successors.len() - i_backwards) {
            let node = successors.get_index(i_backwards).unwrap().0;
            for predecessor_node in predecessors_fn(node) {
                if !successors.contains_key(&predecessor_node) {
                    successors.insert(predecessor_node.clone(), Some(i_backwards));
                }
                if predecessors.contains_key(&predecessor_node) {
                    break 'l Some(predecessor_node);
                }
            }
            i_backwards += 1;
        }

        if i_forwards == predecessors.len() && i_backwards == successors.len() {
            break 'l None;
        }
    };

    middle.map(|middle| {
        // Path found!
        // Build the path.
        let mut path = vec![];
        // From middle to the start.
        let mut node = Some(middle.clone());
        while let Some(n) = node {
            path.push(n.clone());
            node = predecessors[&n].map(|i| predecessors.get_index(i).unwrap().0.clone());
        }
        // Reverse, to put start at the front.
        path.reverse();
        // And from middle to the end.
        let mut node = successors[&middle].map(|i| successors.get_index(i).unwrap().0.clone());
        while let Some(n) = node {
            path.push(n.clone());
            node = successors[&n].map(|i| successors.get_index(i).unwrap().0.clone());
        }
        path
    })
}

/// Visit all nodes that are reachable from a start node. The node will be visited
/// in BFS order, starting from the `start` node and following the order returned
/// by the `successors` function.
///
/// # Examples
///
/// The iterator stops when there are no new nodes to visit:
///
/// ```
/// use pathfinding_faster::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 1), (2, 1)],
///     vec![(3, 1)],
///     vec![(4, 1)],
///     vec![],
///     vec![],
/// ]);
/// let all_nodes = graph.bfs_reach(0).collect::<Vec<_>>();
/// assert_eq!(all_nodes, vec![0, 1, 2, 3, 4]);
/// ```
pub fn bfs_reach<N, FN, IN>(start: N, successors: FN) -> BfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    let mut seen = FxIndexSet::default();
    seen.insert(start);
    BfsReachable {
        i: 0,
        seen,
        successors,
    }
}

/// Struct returned by [`bfs_reach`].
pub struct BfsReachable<N, FN> {
    i: usize,
    seen: FxIndexSet<N>,
    successors: FN,
}

impl<N, FN, IN> Iterator for BfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.seen.get_index(self.i)?.clone();
        for s in (self.successors)(&n) {
            self.seen.insert(s);
        }
        self.i += 1;
        Some(n)
    }
}

impl<N, FN, IN> FusedIterator for BfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
}
