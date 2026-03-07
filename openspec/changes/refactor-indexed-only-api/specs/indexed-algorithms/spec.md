## ADDED Requirements
### Requirement: Directed algorithms on `IndexedGraph`
The system SHALL expose directed graph algorithms as methods on `IndexedGraph` using
`usize` indices for inputs and outputs. The method names SHALL include:
`astar`, `astar_bag`, `astar_bag_collect`, `bfs`, `bfs_bidirectional`, `bfs_loop`,
`bfs_reach`, `dfs`, `dfs_reach`, `iddfs`, `idastar`, `dijkstra`, `dijkstra_all`,
`dijkstra_partial`, `dijkstra_reach`, `bmssp`, `bmssp_all`, `fringe`,
`topological_sort`, `strongly_connected_components`, `strongly_connected_components_from`,
`strongly_connected_component`, `count_paths`, `yen`, and `edmonds_karp`.

#### Scenario: Dijkstra path on indices
- **WHEN** a user runs `graph.dijkstra(start, |i| i == goal)`
- **THEN** the result is a path of `usize` indices and a total cost

### Requirement: Undirected algorithms on `IndexedUndirectedGraph`
The system SHALL expose undirected graph algorithms as methods on `IndexedUndirectedGraph`
using `usize` indices for inputs and outputs. The method names SHALL include:
`connected_components`, `component_index`, `components`, `separate_components`,
`maximal_cliques`, `maximal_cliques_collect`, `kruskal`, `kruskal_indices`, and `prim`.

#### Scenario: Connected components on indices
- **WHEN** a user runs `graph.connected_components()` on an undirected graph
- **THEN** the result groups nodes by `usize` indices into disjoint components
