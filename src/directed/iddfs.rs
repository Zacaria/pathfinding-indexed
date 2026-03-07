//! Compute a shortest path using the [iterative deepening depth-first search
//! algorithm](https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search).

/// Compute a shortest path using the [iterative deepening depth-first search
/// algorithm](https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search).
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
/// The returned path comprises both the start and end node. Note that the start node ownership
/// is taken by `iddfs` as no clones are made.
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
/// let path = graph.iddfs(0, |n| n == 3).expect("path not found");
/// assert_eq!(path, vec![0, 1, 3]);
/// ```
pub fn iddfs<N, FN, IN, FS>(start: N, mut successors: FN, mut success: FS) -> Option<Vec<N>>
where
    N: Eq,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    let mut path = vec![start];

    let mut current_max_depth: usize = 1;

    loop {
        match step(&mut path, &mut successors, &mut success, current_max_depth) {
            Path::FoundOptimum => return Some(path),
            Path::NoneAtThisDepth => current_max_depth += 1,
            Path::Impossible => return None,
        }
    }
}

#[derive(Debug)]
enum Path {
    FoundOptimum,
    Impossible,
    NoneAtThisDepth,
}

fn step<N, FN, IN, FS>(
    path: &mut Vec<N>,
    successors: &mut FN,
    success: &mut FS,
    depth: usize,
) -> Path
where
    N: Eq,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    if depth == 0 {
        Path::NoneAtThisDepth
    } else if success(path.last().unwrap()) {
        Path::FoundOptimum
    } else {
        let successors_it = successors(path.last().unwrap());

        let mut best_result = Path::Impossible;

        for n in successors_it {
            if !path.contains(&n) {
                path.push(n);
                match step(path, successors, success, depth - 1) {
                    Path::FoundOptimum => return Path::FoundOptimum,
                    Path::NoneAtThisDepth => best_result = Path::NoneAtThisDepth,
                    Path::Impossible => (),
                }
                path.pop();
            }
        }

        best_result
    }
}
