//! Compute a path using the [depth-first search
//! algorithm](https://en.wikipedia.org/wiki/Depth-first_search).

use std::collections::HashSet;
use std::hash::Hash;
use std::iter::FusedIterator;

use rustc_hash::{FxHashMap, FxHashSet};

/// Compute a path using the [depth-first search
/// algorithm](https://en.wikipedia.org/wiki/Depth-first_search).
///
/// The path starts from `start` up to a node for which `success`
/// returns `true` is computed and returned along with its total cost,
/// in a `Some`. If no path can be found, `None` is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node, which will be tried in order.
/// - `success` checks whether the goal has been reached. It is not a node as some problems require
///   a dynamic solution instead of a fixed node.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node. Note that the start node ownership
/// is taken by `dfs` as no clones are made.
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
/// let path = graph.dfs(0, |n| n == 3).expect("path not found");
/// assert_eq!(path, vec![0, 1, 3]);
/// ```
pub fn dfs<N, FN, IN, FS>(start: N, mut successors: FN, mut success: FS) -> Option<Vec<N>>
where
    N: Clone + Eq + Hash,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    let mut to_visit = vec![start];
    let mut visited = FxHashSet::default();
    let mut parents = FxHashMap::default();
    while let Some(node) = to_visit.pop() {
        if visited.insert(node.clone()) {
            if success(&node) {
                return Some(build_path(node, &parents));
            }
            for next in successors(&node)
                .into_iter()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                if !visited.contains(&next) {
                    parents.insert(next.clone(), node.clone());
                    to_visit.push(next);
                }
            }
        }
    }
    None
}

fn build_path<N>(mut node: N, parents: &FxHashMap<N, N>) -> Vec<N>
where
    N: Clone + Eq + Hash,
{
    let mut path = vec![node.clone()];
    while let Some(parent) = parents.get(&node).cloned() {
        path.push(parent.clone());
        node = parent;
    }
    path.into_iter().rev().collect()
}

/// Visit all nodes that are reachable from a start node. The node will be visited
/// in DFS order, starting from the `start` node and following the order returned
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
///     vec![(3, 1)],
///     vec![],
/// ]);
/// let all_nodes = graph.dfs_reach(0).collect::<Vec<_>>();
/// assert_eq!(all_nodes, vec![0, 1, 3, 2]);
/// ```
pub fn dfs_reach<N, FN, IN>(start: N, successors: FN) -> DfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    DfsReachable {
        to_see: vec![start],
        visited: HashSet::new(),
        successors,
    }
}

/// Struct returned by [`dfs_reach`].
pub struct DfsReachable<N, FN> {
    to_see: Vec<N>,
    visited: HashSet<N>,
    successors: FN,
}

impl<N, FN, IN> Iterator for DfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.to_see.pop()?;
        if self.visited.contains(&n) {
            return self.next();
        }
        self.visited.insert(n.clone());
        let mut to_insert = Vec::new();
        for s in (self.successors)(&n) {
            if !self.visited.contains(&s) {
                to_insert.push(s.clone());
            }
        }
        self.to_see.extend(to_insert.into_iter().rev());
        Some(n)
    }
}

impl<N, FN, IN> FusedIterator for DfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
}
