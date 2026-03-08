//! Compute a shortest path using the [Dijkstra search
//! algorithm](https://en.wikipedia.org/wiki/Dijkstra's_algorithm).

use super::reverse_path;
use crate::FxIndexMap;
use indexmap::map::Entry::{Occupied, Vacant};
use num_traits::Zero;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;

/// Compute a shortest path using the [Dijkstra search
/// algorithm](https://en.wikipedia.org/wiki/Dijkstra's_algorithm).
///
/// The shortest path starting from `start` up to a node for which `success` returns `true` is
/// computed and returned along with its total cost, in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node, along with the cost for moving
///   from the node to the successor. This cost must be non-negative.
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
/// use pathfinding_indexed::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 2), (2, 4)],
///     vec![(2, 1), (3, 7)],
///     vec![(3, 3)],
///     vec![],
/// ]);
/// let result = graph.dijkstra(0, |n| n == 3);
/// assert_eq!(result, Some((vec![0, 1, 2, 3], 6)));
/// ```
pub fn dijkstra<N, C, FN, IN, FS>(
    start: &N,
    mut successors: FN,
    mut success: FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FS: FnMut(&N) -> bool,
{
    dijkstra_internal(start, &mut successors, &mut success)
}

pub(crate) fn dijkstra_internal<N, C, FN, IN, FS>(
    start: &N,
    successors: &mut FN,
    success: &mut FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FS: FnMut(&N) -> bool,
{
    let (parents, reached) = run_dijkstra(start, successors, success);
    reached.map(|target| {
        (
            reverse_path(&parents, |&(p, _)| p, target),
            parents.get_index(target).unwrap().1.1,
        )
    })
}

/// Determine all reachable nodes from a starting point as well as the
/// minimum cost to reach them and a possible optimal parent node
/// using the [Dijkstra search
/// algorithm](https://en.wikipedia.org/wiki/Dijkstra's_algorithm).
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node, along with the cost for moving
///   from the node to the successor.
///
/// The result associates every reachable node (not including `start`) with an optimal parent
/// node and a cost from the start node.
///
/// # Example
///
/// We use a graph of indexed nodes, each node leading to its children with a cost of 10.
///
/// ```
/// use pathfinding_indexed::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 10), (2, 10)],
///     vec![(3, 10), (4, 10)],
///     vec![(5, 10), (6, 10)],
///     vec![(7, 10), (8, 10)],
///     vec![],
///     vec![],
///     vec![],
///     vec![],
///     vec![],
/// ]);
/// let reachables = graph.dijkstra_all(0);
/// assert_eq!(reachables[1], Some((0, 10)));
/// assert_eq!(reachables[3], Some((1, 20)));
/// assert_eq!(reachables[8], Some((3, 30)));
/// ```
pub fn dijkstra_all<N, C, FN, IN>(start: &N, successors: FN) -> HashMap<N, (N, C)>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
{
    dijkstra_partial(start, successors, |_| false).0
}

/// Determine some reachable nodes from a starting point as well as the minimum cost to
/// reach them and a possible optimal parent node
/// using the [Dijkstra search algorithm](https://en.wikipedia.org/wiki/Dijkstra's_algorithm).
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node, along with the cost for moving
///   from the node to the successor.
/// - `stop` is a function which is called every time a node is examined (including `start`).
///   A `true` return value will stop the algorithm.
///
/// The result is a map where every node examined before the algorithm stopped (not including
/// `start`) is associated with an optimal parent node and a cost from the start node, as well
/// as the node which caused the algorithm to stop if any.
///
pub fn dijkstra_partial<N, C, FN, IN, FS>(
    start: &N,
    mut successors: FN,
    mut stop: FS,
) -> (HashMap<N, (N, C)>, Option<N>)
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FS: FnMut(&N) -> bool,
{
    let (parents, reached) = run_dijkstra(start, &mut successors, &mut stop);
    (
        parents
            .iter()
            .skip(1)
            .map(|(n, (p, c))| (n.clone(), (parents.get_index(*p).unwrap().0.clone(), *c))) // unwrap() cannot fail
            .collect(),
        reached.map(|i| parents.get_index(i).unwrap().0.clone()),
    )
}

fn run_dijkstra<N, C, FN, IN, FS>(
    start: &N,
    successors: &mut FN,
    stop: &mut FS,
) -> (FxIndexMap<N, (usize, C)>, Option<usize>)
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FS: FnMut(&N) -> bool,
{
    let mut to_see = BinaryHeap::new();
    to_see.push(SmallestHolder {
        cost: Zero::zero(),
        index: 0,
    });
    let mut parents: FxIndexMap<N, (usize, C)> = FxIndexMap::default();
    parents.insert(start.clone(), (usize::MAX, Zero::zero()));
    let mut target_reached = None;
    while let Some(SmallestHolder { cost, index }) = to_see.pop() {
        let successors = {
            let (node, &(_, c)) = parents.get_index(index).unwrap();
            if stop(node) {
                target_reached = Some(index);
                break;
            }
            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            if cost > c {
                continue;
            }
            successors(node)
        };
        for (successor, move_cost) in successors {
            let new_cost = cost + move_cost;
            let n;
            match parents.entry(successor) {
                Vacant(e) => {
                    n = e.index();
                    e.insert((index, new_cost));
                }
                Occupied(mut e) => {
                    if e.get().1 > new_cost {
                        n = e.index();
                        e.insert((index, new_cost));
                    } else {
                        continue;
                    }
                }
            }

            to_see.push(SmallestHolder {
                cost: new_cost,
                index: n,
            });
        }
    }
    (parents, target_reached)
}

struct SmallestHolder<K> {
    cost: K,
    index: usize,
}

impl<K: PartialEq> PartialEq for SmallestHolder<K> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl<K: PartialEq> Eq for SmallestHolder<K> {}

impl<K: Ord> PartialOrd for SmallestHolder<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord> Ord for SmallestHolder<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

/// Struct returned by [`dijkstra_reach`].
pub struct DijkstraReachable<N, C, FN> {
    to_see: BinaryHeap<SmallestHolder<C>>,
    seen: FxHashSet<usize>,
    parents: FxIndexMap<N, (usize, C)>,
    total_costs: FxHashMap<N, C>,
    successors: FN,
}

/// Information about a node reached by [`dijkstra_reach`].
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DijkstraReachableItem<N, C> {
    /// The node that was reached by [`dijkstra_reach`].
    pub node: N,
    /// The previous node that the current node came from.
    /// If the node is the first node, there will be no parent.
    pub parent: Option<N>,
    /// The total cost from the starting node.
    pub total_cost: C,
}

impl<N, C, FN, IN> Iterator for DijkstraReachable<N, C, FN>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy + Hash,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
{
    type Item = DijkstraReachableItem<N, C>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(SmallestHolder { cost, index }) = self.to_see.pop() {
            if !self.seen.insert(index) {
                continue;
            }
            let item;
            let successors = {
                let (node, (parent_index, _)) = self.parents.get_index(index).unwrap();
                let total_cost = self.total_costs[node];
                item = Some(DijkstraReachableItem {
                    node: node.clone(),
                    parent: self.parents.get_index(*parent_index).map(|x| x.0.clone()),
                    total_cost,
                });
                (self.successors)(node)
            };
            for (successor, move_cost) in successors {
                let new_cost = cost + move_cost;
                let n;
                match self.parents.entry(successor.clone()) {
                    Vacant(e) => {
                        n = e.index();
                        e.insert((index, new_cost));
                        self.total_costs.insert(successor.clone(), new_cost);
                    }
                    Occupied(mut e) => {
                        if e.get().1 > new_cost {
                            n = e.index();
                            e.insert((index, new_cost));
                            self.total_costs.insert(successor.clone(), new_cost);
                        } else {
                            continue;
                        }
                    }
                }

                self.to_see.push(SmallestHolder {
                    cost: new_cost,
                    index: n,
                });
            }
            return item;
        }

        None
    }
}

/// Visit all nodes that are reachable from a start node. The node
/// will be visited in order of cost, with the closest nodes first.
///
/// The `successors` function receives the current node, and returns
/// an iterator of successors associated with their move cost.
pub fn dijkstra_reach<N, C, FN, IN>(start: &N, successors: FN) -> DijkstraReachable<N, C, FN>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
{
    let mut to_see = BinaryHeap::new();
    to_see.push(SmallestHolder {
        cost: Zero::zero(),
        index: 0,
    });

    let mut parents: FxIndexMap<N, (usize, C)> = FxIndexMap::default();
    parents.insert(start.clone(), (usize::MAX, Zero::zero()));

    let mut total_costs = FxHashMap::default();
    total_costs.insert(start.clone(), Zero::zero());

    let seen = FxHashSet::default();

    DijkstraReachable {
        to_see,
        seen,
        parents,
        total_costs,
        successors,
    }
}
