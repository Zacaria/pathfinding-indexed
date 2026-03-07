//! Indexed graph storage and algorithms.

use num_traits::{Bounded, Signed, Zero};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::VecDeque;
use std::hash::Hash;

/// A directed graph stored as dense `usize` indices with weighted adjacency lists.
#[derive(Clone, Debug)]
pub struct IndexedGraph<C> {
    adjacency: Vec<Vec<(usize, C)>>,
}

impl<C> IndexedGraph<C> {
    /// Build a directed indexed graph from an adjacency list.
    #[must_use]
    pub const fn from_adjacency(adjacency: Vec<Vec<(usize, C)>>) -> Self {
        Self { adjacency }
    }

    /// Return the number of nodes in the graph.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Return the number of nodes in the graph.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.adjacency.len()
    }

    /// Return true if the graph contains no nodes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.adjacency.is_empty()
    }

    /// Return the adjacency list for `node`.
    #[must_use]
    pub fn successors(&self, node: usize) -> &[(usize, C)] {
        &self.adjacency[node]
    }

    /// Return all adjacency lists.
    #[must_use]
    pub fn adjacency(&self) -> &[Vec<(usize, C)>] {
        &self.adjacency
    }

    /// Run A* from `start` to a node satisfying `success`.
    pub fn astar<FH, FS>(
        &self,
        start: usize,
        mut heuristic: FH,
        mut success: FS,
    ) -> Option<(Vec<usize>, C)>
    where
        C: Zero + Ord + Copy,
        FH: FnMut(usize) -> C,
        FS: FnMut(usize) -> bool,
    {
        crate::directed::astar::astar(
            &start,
            |node| self.successors(*node).iter().copied(),
            |node| heuristic(*node),
            |node| success(*node),
        )
    }

    /// Run A* and return all shortest paths as an iterator.
    pub fn astar_bag<FH, FS>(
        &self,
        start: usize,
        mut heuristic: FH,
        mut success: FS,
    ) -> Option<(impl Iterator<Item = Vec<usize>>, C)>
    where
        C: Zero + Ord + Copy,
        FH: FnMut(usize) -> C,
        FS: FnMut(usize) -> bool,
    {
        crate::directed::astar::astar_bag(
            &start,
            |node| self.successors(*node).iter().copied(),
            |node| heuristic(*node),
            |node| success(*node),
        )
    }

    /// Run A* and collect all shortest paths into a vector.
    pub fn astar_bag_collect<FH, FS>(
        &self,
        start: usize,
        mut heuristic: FH,
        mut success: FS,
    ) -> Option<(Vec<Vec<usize>>, C)>
    where
        C: Zero + Ord + Copy,
        FH: FnMut(usize) -> C,
        FS: FnMut(usize) -> bool,
    {
        crate::directed::astar::astar_bag_collect(
            &start,
            |node| self.successors(*node).iter().copied(),
            |node| heuristic(*node),
            |node| success(*node),
        )
    }

    /// Run BFS from `start` to a node satisfying `success`.
    pub fn bfs<FS>(&self, start: usize, mut success: FS) -> Option<Vec<usize>>
    where
        FS: FnMut(usize) -> bool,
    {
        crate::directed::bfs::bfs(
            &start,
            |node| self.successors(*node).iter().map(|edge| edge.0),
            |node| success(*node),
        )
    }

    /// Run BFS and return the shortest loop starting and ending at `start`.
    #[must_use]
    pub fn bfs_loop(&self, start: usize) -> Option<Vec<usize>> {
        crate::directed::bfs::bfs_loop(&start, |node| {
            self.successors(*node).iter().map(|edge| edge.0)
        })
    }

    /// Run bidirectional BFS from `start` to `end`.
    #[must_use]
    pub fn bfs_bidirectional(&self, start: usize, end: usize) -> Option<Vec<usize>> {
        let mut predecessors = vec![Vec::new(); self.node_count()];
        for (from, edges) in self.adjacency.iter().enumerate() {
            for &(to, _) in edges {
                predecessors[to].push(from);
            }
        }

        crate::directed::bfs::bfs_bidirectional(
            &start,
            &end,
            |node| {
                self.successors(*node)
                    .iter()
                    .map(|edge| edge.0)
                    .collect::<Vec<_>>()
            },
            |node| predecessors[*node].clone(),
        )
    }

    /// Iterate over nodes reachable from `start` using BFS order.
    pub fn bfs_reach(&self, start: usize) -> impl Iterator<Item = usize> + '_ {
        crate::directed::bfs::bfs_reach(start, |node| {
            self.successors(*node).iter().map(|edge| edge.0)
        })
    }

    /// Run DFS from `start` to a node satisfying `success`.
    pub fn dfs<FS>(&self, start: usize, mut success: FS) -> Option<Vec<usize>>
    where
        FS: FnMut(usize) -> bool,
    {
        crate::directed::dfs::dfs(
            start,
            |node| self.successors(*node).iter().map(|edge| edge.0),
            |node| success(*node),
        )
    }

    /// Iterate over nodes reachable from `start` using DFS order.
    pub fn dfs_reach(&self, start: usize) -> impl Iterator<Item = usize> + '_ {
        crate::directed::dfs::dfs_reach(start, |node| {
            self.successors(*node).iter().map(|edge| edge.0)
        })
    }

    /// Run IDDFS from `start` to a node satisfying `success`.
    pub fn iddfs<FS>(&self, start: usize, mut success: FS) -> Option<Vec<usize>>
    where
        FS: FnMut(usize) -> bool,
    {
        crate::directed::iddfs::iddfs(
            start,
            |node| self.successors(*node).iter().map(|edge| edge.0),
            |node| success(*node),
        )
    }

    /// Run IDA* from `start` to a node satisfying `success`.
    pub fn idastar<FH, FS>(
        &self,
        start: usize,
        mut heuristic: FH,
        mut success: FS,
    ) -> Option<(Vec<usize>, C)>
    where
        C: Zero + Ord + Copy,
        FH: FnMut(usize) -> C,
        FS: FnMut(usize) -> bool,
    {
        crate::directed::idastar::idastar(
            &start,
            |node| self.successors(*node).iter().copied(),
            |node| heuristic(*node),
            |node| success(*node),
        )
    }

    /// Run Dijkstra from `start` to a node satisfying `success`.
    pub fn dijkstra<FS>(&self, start: usize, mut success: FS) -> Option<(Vec<usize>, C)>
    where
        C: Zero + Ord + Copy,
        FS: FnMut(usize) -> bool,
    {
        crate::directed::dijkstra::dijkstra(
            &start,
            |node| self.successors(*node).iter().copied(),
            |node| success(*node),
        )
    }

    /// Compute shortest paths to all reachable nodes from `start`.
    pub fn dijkstra_all(&self, start: usize) -> Vec<Option<(usize, C)>>
    where
        C: Zero + Ord + Copy,
    {
        let parents = crate::directed::dijkstra::dijkstra_all(&start, |node| {
            self.successors(*node).iter().copied()
        });
        let mut out = vec![None; self.node_count()];
        for (node, (parent, cost)) in parents {
            out[node] = Some((parent, cost));
        }
        out
    }

    /// Compute shortest paths until `stop` returns true.
    pub fn dijkstra_partial<FS>(
        &self,
        start: usize,
        mut stop: FS,
    ) -> (Vec<Option<(usize, C)>>, Option<usize>)
    where
        C: Zero + Ord + Copy,
        FS: FnMut(usize) -> bool,
    {
        let (parents, reached) = crate::directed::dijkstra::dijkstra_partial(
            &start,
            |node| self.successors(*node).iter().copied(),
            |node| stop(*node),
        );
        let mut out = vec![None; self.node_count()];
        for (node, (parent, cost)) in parents {
            out[node] = Some((parent, cost));
        }
        (out, reached)
    }

    /// Iterate over nodes reached by Dijkstra, yielding the node, parent, and total cost.
    pub fn dijkstra_reach(
        &self,
        start: usize,
    ) -> impl Iterator<Item = (usize, Option<usize>, C)> + '_
    where
        C: Zero + Ord + Copy + Hash,
    {
        crate::directed::dijkstra::dijkstra_reach(&start, |node| {
            self.successors(*node).iter().copied()
        })
        .map(|item| (item.node, item.parent, item.total_cost))
    }

    /// Run the BMSSP-based SSSP algorithm from `start`.
    pub fn bmssp<FS>(&self, start: usize, success: FS) -> Option<(Vec<usize>, C)>
    where
        C: Zero + Ord + Copy,
        FS: FnMut(usize) -> bool,
    {
        crate::directed::bmssp::bmssp_indexed(
            start,
            |node| self.successors(node).iter().copied(),
            success,
            self.node_count(),
        )
    }

    /// Compute BMSSP parents and costs for all reachable nodes from `start`.
    #[must_use]
    pub fn bmssp_all(&self, start: usize) -> Vec<Option<(usize, C)>>
    where
        C: Zero + Ord + Copy,
    {
        crate::directed::bmssp::bmssp_all_indexed(
            start,
            |node| self.successors(node).iter().copied(),
            self.node_count(),
        )
    }

    /// Run Fringe search from `start` to a node satisfying `success`.
    pub fn fringe<FH, FS>(
        &self,
        start: usize,
        mut heuristic: FH,
        mut success: FS,
    ) -> Option<(Vec<usize>, C)>
    where
        C: Bounded + Zero + Ord + Copy,
        FH: FnMut(usize) -> C,
        FS: FnMut(usize) -> bool,
    {
        crate::directed::fringe::fringe(
            &start,
            |node| self.successors(*node).iter().copied(),
            |node| heuristic(*node),
            |node| success(*node),
        )
    }

    /// Compute a topological ordering of all nodes.
    ///
    /// # Errors
    ///
    /// Returns `Err(node)` when a cycle is detected.
    pub fn topological_sort(&self) -> Result<Vec<usize>, usize> {
        let nodes = (0..self.node_count()).collect::<Vec<_>>();
        crate::directed::topological_sort::topological_sort(&nodes, |node| {
            self.successors(*node).iter().map(|edge| edge.0)
        })
    }

    /// Compute strongly connected components for all nodes.
    #[must_use]
    pub fn strongly_connected_components(&self) -> Vec<Vec<usize>> {
        let nodes = (0..self.node_count()).collect::<Vec<_>>();
        crate::directed::strongly_connected_components::strongly_connected_components(
            &nodes,
            |node| self.successors(*node).iter().map(|edge| edge.0),
        )
    }

    /// Compute strongly connected components reachable from `start`.
    #[must_use]
    pub fn strongly_connected_components_from(&self, start: usize) -> Vec<Vec<usize>> {
        crate::directed::strongly_connected_components::strongly_connected_components_from(
            &start,
            |node| self.successors(*node).iter().map(|edge| edge.0),
        )
    }

    /// Compute the strongly connected component containing `node`.
    #[must_use]
    pub fn strongly_connected_component(&self, node: usize) -> Vec<usize> {
        crate::directed::strongly_connected_components::strongly_connected_component(&node, |n| {
            self.successors(*n).iter().map(|edge| edge.0)
        })
    }

    /// Count the number of paths from `start` to nodes satisfying `success`.
    pub fn count_paths<FS>(&self, start: usize, mut success: FS) -> usize
    where
        FS: FnMut(usize) -> bool,
    {
        crate::directed::count_paths::count_paths(
            start,
            |node| self.successors(*node).iter().map(|edge| edge.0),
            |node| success(*node),
        )
    }

    /// Compute k-shortest paths using Yen's algorithm.
    pub fn yen<FS>(&self, start: usize, mut success: FS, k: usize) -> Vec<(Vec<usize>, C)>
    where
        C: Zero + Ord + Copy,
        FS: FnMut(usize) -> bool,
    {
        crate::directed::yen::yen(
            &start,
            |node| self.successors(*node).iter().copied(),
            |node| success(*node),
            k,
        )
    }

    /// Compute the maximum flow and minimum cut using Edmonds-Karp.
    ///
    /// # Panics
    ///
    /// Panics if `source` or `sink` are out of bounds.
    #[expect(clippy::too_many_lines)]
    #[expect(clippy::type_complexity)]
    #[must_use]
    pub fn edmonds_karp(
        &self,
        source: usize,
        sink: usize,
    ) -> (Vec<((usize, usize), C)>, C, Vec<((usize, usize), C)>)
    where
        C: Copy + Zero + Signed + Ord + Bounded,
    {
        let node_count = self.node_count();
        if node_count == 0 || source == sink {
            return (Vec::new(), Zero::zero(), Vec::new());
        }
        assert!(source < node_count, "source out of bounds");
        assert!(sink < node_count, "sink out of bounds");

        let mut capacity = vec![vec![C::zero(); node_count]; node_count];
        for (from, edges) in self.adjacency.iter().enumerate() {
            for &(to, weight) in edges {
                capacity[from][to] = capacity[from][to] + weight;
            }
        }

        let mut predecessors = vec![Vec::new(); node_count];
        for (from, edges) in self.adjacency.iter().enumerate() {
            for &(to, _) in edges {
                predecessors[to].push(from);
            }
        }

        let mut flow = vec![vec![C::zero(); node_count]; node_count];
        let mut total = C::zero();

        loop {
            let mut parent = vec![usize::MAX; node_count];
            let mut direction = vec![true; node_count];
            let mut path_capacity = vec![C::zero(); node_count];
            let mut queue = VecDeque::new();

            parent[source] = source;
            path_capacity[source] = C::max_value();
            queue.push_back(source);

            while let Some(node) = queue.pop_front() {
                let capacity_so_far = path_capacity[node];
                for &(next, _) in &self.adjacency[node] {
                    if parent[next] != usize::MAX || next == source {
                        continue;
                    }
                    let residual = capacity[node][next] - flow[node][next];
                    if residual <= Zero::zero() {
                        continue;
                    }
                    parent[next] = node;
                    direction[next] = true;
                    path_capacity[next] = if capacity_so_far < residual {
                        capacity_so_far
                    } else {
                        residual
                    };
                    if next == sink {
                        break;
                    }
                    queue.push_back(next);
                }
                if parent[sink] != usize::MAX {
                    break;
                }
                for &prev in &predecessors[node] {
                    if parent[prev] != usize::MAX || prev == source {
                        continue;
                    }
                    let residual = flow[prev][node];
                    if residual <= Zero::zero() {
                        continue;
                    }
                    parent[prev] = node;
                    direction[prev] = false;
                    path_capacity[prev] = if capacity_so_far < residual {
                        capacity_so_far
                    } else {
                        residual
                    };
                    if prev == sink {
                        break;
                    }
                    queue.push_back(prev);
                }
                if parent[sink] != usize::MAX {
                    break;
                }
            }

            if parent[sink] == usize::MAX {
                break;
            }

            let augment = path_capacity[sink];
            let mut node = sink;
            while node != source {
                let prev = parent[node];
                if direction[node] {
                    flow[prev][node] = flow[prev][node] + augment;
                } else {
                    flow[node][prev] = flow[node][prev] - augment;
                }
                node = prev;
            }
            total = total + augment;
        }

        let mut reachable = vec![false; node_count];
        let mut queue = VecDeque::new();
        reachable[source] = true;
        queue.push_back(source);
        while let Some(node) = queue.pop_front() {
            for &(next, _) in &self.adjacency[node] {
                if reachable[next] {
                    continue;
                }
                let residual = capacity[node][next] - flow[node][next];
                if residual > Zero::zero() {
                    reachable[next] = true;
                    queue.push_back(next);
                }
            }
            for &prev in &predecessors[node] {
                if reachable[prev] {
                    continue;
                }
                if flow[prev][node] > Zero::zero() {
                    reachable[prev] = true;
                    queue.push_back(prev);
                }
            }
        }

        let mut flows = Vec::new();
        for (from, edges) in self.adjacency.iter().enumerate() {
            for &(to, _) in edges {
                let value = flow[from][to];
                if value > Zero::zero() {
                    flows.push(((from, to), value));
                }
            }
        }

        let mut cuts = Vec::new();
        for (from, edges) in self.adjacency.iter().enumerate() {
            if !reachable[from] {
                continue;
            }
            for &(to, _) in edges {
                if reachable[to] {
                    continue;
                }
                let cap = capacity[from][to];
                if cap > Zero::zero() {
                    cuts.push(((from, to), cap));
                }
            }
        }

        (flows, total, cuts)
    }
}

/// An undirected graph stored as dense `usize` indices with weighted adjacency lists.
#[derive(Clone, Debug)]
pub struct IndexedUndirectedGraph<C> {
    adjacency: Vec<Vec<(usize, C)>>,
    edges: Vec<(usize, usize, C)>,
}

impl<C> IndexedUndirectedGraph<C> {
    /// Build an undirected indexed graph from a list of edges.
    #[must_use]
    pub fn from_edges(node_count: usize, edges: Vec<(usize, usize, C)>) -> Self
    where
        C: Clone,
    {
        let mut adjacency = vec![Vec::new(); node_count];
        for (u, v, w) in &edges {
            debug_assert!(*u < node_count, "edge start out of bounds");
            debug_assert!(*v < node_count, "edge end out of bounds");
            adjacency[*u].push((*v, w.clone()));
            adjacency[*v].push((*u, w.clone()));
        }
        Self { adjacency, edges }
    }

    /// Return the number of nodes in the graph.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Return the adjacency list for `node`.
    #[must_use]
    pub fn successors(&self, node: usize) -> &[(usize, C)] {
        &self.adjacency[node]
    }

    /// Return the canonical edge list (each edge appears once).
    #[must_use]
    pub fn edges(&self) -> &[(usize, usize, C)] {
        &self.edges
    }

    /// Return all adjacency lists.
    #[must_use]
    pub fn adjacency(&self) -> &[Vec<(usize, C)>] {
        &self.adjacency
    }

    /// Compute connected components of the graph.
    #[must_use]
    pub fn connected_components(&self) -> Vec<Vec<usize>> {
        let nodes = (0..self.node_count()).collect::<Vec<_>>();
        let components =
            crate::undirected::connected_components::connected_components(&nodes, |n| {
                self.successors(*n).iter().map(|edge| edge.0)
            });
        components
            .into_iter()
            .map(|set| set.into_iter().collect())
            .collect()
    }

    /// Return the component index for each node.
    #[must_use]
    pub fn component_index(&self) -> Vec<usize> {
        let components = self.connected_components();
        let mut index = vec![usize::MAX; self.node_count()];
        for (component_idx, component) in components.iter().enumerate() {
            for &node in component {
                index[node] = component_idx;
            }
        }
        index
    }

    /// Return connected components (alias of [`Self::connected_components`]).
    #[must_use]
    pub fn components(&self) -> Vec<Vec<usize>> {
        self.connected_components()
    }

    /// Separate connected components using neighbor groups derived from adjacency.
    #[must_use]
    pub fn separate_components(&self) -> (Vec<usize>, Vec<usize>) {
        let groups = (0..self.node_count())
            .map(|node| {
                let mut group = Vec::with_capacity(self.adjacency[node].len() + 1);
                group.push(node);
                group.extend(self.adjacency[node].iter().map(|edge| edge.0));
                group
            })
            .collect::<Vec<_>>();
        let (mapping, group_indices) =
            crate::undirected::connected_components::separate_components(&groups);
        let mut node_components = vec![usize::MAX; self.node_count()];
        for (node, component) in mapping {
            node_components[*node] = component;
        }
        (node_components, group_indices)
    }

    /// Enumerate all maximal cliques and collect them into a vector.
    #[must_use]
    pub fn maximal_cliques_collect(&self) -> Vec<Vec<usize>> {
        let adjacency_sets = self.adjacency_sets();
        let mut connected = |a: &usize, b: &usize| adjacency_sets[*a].contains(b);
        crate::undirected::cliques::maximal_cliques_collect(0..self.node_count(), &mut connected)
            .into_iter()
            .map(|set| set.into_iter().collect())
            .collect()
    }

    /// Enumerate all maximal cliques and send them to `consumer`.
    pub fn maximal_cliques<CO>(&self, mut consumer: CO)
    where
        CO: FnMut(&Vec<usize>),
    {
        let adjacency_sets = self.adjacency_sets();
        let mut connected = |a: &usize, b: &usize| adjacency_sets[*a].contains(b);
        let mut adapter = |clique: &std::collections::HashSet<usize>| {
            let clique_vec = clique.iter().copied().collect::<Vec<_>>();
            consumer(&clique_vec);
        };
        crate::undirected::cliques::maximal_cliques(
            0..self.node_count(),
            &mut connected,
            &mut adapter,
        );
    }

    /// Compute a minimum spanning tree using Kruskal's algorithm (collected).
    #[must_use]
    pub fn kruskal(&self) -> Vec<(usize, usize, C)>
    where
        C: Clone + Ord,
    {
        self.kruskal_indices().collect()
    }

    /// Compute a minimum spanning tree using Kruskal's algorithm (iterator).
    pub fn kruskal_indices(&self) -> impl Iterator<Item = (usize, usize, C)>
    where
        C: Clone + Ord,
    {
        crate::undirected::kruskal::kruskal_indices(self.node_count(), self.edges())
    }

    /// Compute a minimum spanning tree using Prim's algorithm.
    #[must_use]
    pub fn prim(&self) -> Vec<(usize, usize, C)>
    where
        C: Clone + Ord,
    {
        crate::undirected::prim::prim(self.edges())
            .into_iter()
            .map(|(a, b, c)| (*a, *b, c))
            .collect()
    }

    fn adjacency_sets(&self) -> Vec<FxHashSet<usize>> {
        self.adjacency
            .iter()
            .map(|edges| edges.iter().map(|edge| edge.0).collect())
            .collect()
    }
}

/// Helper that maps external node values to dense indices and builds an indexed graph.
#[derive(Clone, Debug)]
pub struct IndexedGraphMap<N, C> {
    graph: IndexedGraph<C>,
    nodes: Vec<N>,
    index: FxHashMap<N, usize>,
}

impl<N, C> IndexedGraphMap<N, C>
where
    N: Eq + Hash + Clone,
{
    /// Build a mapped graph from seed nodes and a successor function.
    pub fn from_nodes_and_successors<I, FN, IN>(nodes: I, mut successors: FN) -> Self
    where
        I: IntoIterator<Item = N>,
        FN: FnMut(&N) -> IN,
        IN: IntoIterator<Item = (N, C)>,
    {
        let mut mapped = Self {
            graph: IndexedGraph::from_adjacency(Vec::new()),
            nodes: Vec::new(),
            index: FxHashMap::default(),
        };

        for node in nodes {
            mapped.ensure_index(node);
        }

        let mut cursor = 0usize;
        while cursor < mapped.nodes.len() {
            let node = mapped.nodes[cursor].clone();
            let mut edges = Vec::new();
            for (neighbor, cost) in successors(&node) {
                let neighbor_idx = mapped.ensure_index(neighbor);
                edges.push((neighbor_idx, cost));
            }
            if cursor >= mapped.graph.adjacency.len() {
                mapped.graph.adjacency.push(edges);
            } else {
                mapped.graph.adjacency[cursor] = edges;
            }
            cursor += 1;
        }

        mapped
    }

    /// Return the indexed graph.
    #[must_use]
    pub const fn graph(&self) -> &IndexedGraph<C> {
        &self.graph
    }

    /// Return the indexed graph, consuming the mapping helper.
    #[must_use]
    pub fn into_graph(self) -> IndexedGraph<C> {
        self.graph
    }

    /// Return the number of nodes in the mapped graph.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Return the index assigned to `node`.
    #[must_use]
    pub fn index_of(&self, node: &N) -> Option<usize> {
        self.index.get(node).copied()
    }

    /// Return the external node value for a given index.
    #[must_use]
    pub fn node(&self, index: usize) -> Option<&N> {
        self.nodes.get(index)
    }

    /// Return the mapped node values in index order.
    #[must_use]
    pub fn nodes(&self) -> &[N] {
        &self.nodes
    }

    fn ensure_index(&mut self, node: N) -> usize {
        if let Some(&idx) = self.index.get(&node) {
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(node.clone());
        self.index.insert(node, idx);
        self.graph.adjacency.push(Vec::new());
        idx
    }
}
