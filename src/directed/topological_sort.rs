//! Find a topological order in a directed graph if one exists.

use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

/// Find a topological order in a directed graph if one exists.
///
/// - `roots` is a collection of nodes that ought to be explored.
/// - `successors` returns a list of successors for a given node, including possibly
///   nodes that were not present in `roots`.
///
/// The function returns an acceptable topological order of nodes given as roots or
/// discovered, or an error if a cycle is detected.
///
/// # Errors
///
/// If a cycle is found, `Err(n)` is returned with `n` being an arbitrary node involved in a cycle.
/// In this case case, the strongly connected set can then be found using the
/// [`strongly_connected_component`](super::strongly_connected_components::strongly_connected_component)
/// function, or if only one of the loops is needed the [`bfs_loop`](super::bfs::bfs_loop) function
/// can be used instead to identify one of the shortest loops involving this node.
///
/// # Examples
///
/// Sort a simple chain:
///
/// ```
/// use pathfinding_faster::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 1)],
///     vec![(2, 1)],
///     vec![(3, 1)],
///     vec![],
/// ]);
/// let sorted = graph.topological_sort().expect("graph has a cycle");
/// assert_eq!(sorted, vec![0, 1, 2, 3]);
/// ```
///
/// If there is a cycle, a node from that cycle is returned as an error:
///
/// ```
/// use pathfinding_faster::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 1)],
///     vec![(2, 1)],
///     vec![(1, 1)],
/// ]);
/// assert!(graph.topological_sort().is_err());
///
/// let loop_path = graph.bfs_loop(1).expect("loop not found");
/// assert_eq!(loop_path, vec![1, 2, 1]);
///
/// let mut component = graph.strongly_connected_component(1);
/// component.sort();
/// assert_eq!(component, vec![1, 2]);
/// ```
pub fn topological_sort<N, FN, IN>(roots: &[N], mut successors: FN) -> Result<Vec<N>, N>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    let mut marked = HashSet::with_capacity(roots.len());
    let mut temp = HashSet::new();
    let mut sorted = VecDeque::with_capacity(roots.len());
    let mut roots: HashSet<N> = roots.iter().cloned().collect::<HashSet<_>>();
    while let Some(node) = roots.iter().next().cloned() {
        temp.clear();
        visit(
            &node,
            &mut successors,
            &mut roots,
            &mut marked,
            &mut temp,
            &mut sorted,
        )?;
    }
    Ok(sorted.into_iter().collect())
}

fn visit<N, FN, IN>(
    node: &N,
    successors: &mut FN,
    unmarked: &mut HashSet<N>,
    marked: &mut HashSet<N>,
    temp: &mut HashSet<N>,
    sorted: &mut VecDeque<N>,
) -> Result<(), N>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    unmarked.remove(node);
    if marked.contains(node) {
        return Ok(());
    }
    if temp.contains(node) {
        return Err(node.clone());
    }
    temp.insert(node.clone());
    for n in successors(node) {
        visit(&n, successors, unmarked, marked, temp, sorted)?;
    }
    marked.insert(node.clone());
    sorted.push_front(node.clone());
    Ok(())
}
