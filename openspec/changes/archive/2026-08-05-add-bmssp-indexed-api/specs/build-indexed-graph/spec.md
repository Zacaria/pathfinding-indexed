## ADDED Requirements
### Requirement: IndexedGraph utility
The library SHALL provide an `IndexedGraph<N, C>` utility that stores a dense node index and adjacency lists for weighted directed graphs.

#### Scenario: Build from nodes and successors
- **WHEN** a user constructs an `IndexedGraph` from a set of seed nodes and a `successors(&N) -> IntoIterator<Item = (N, C)>` closure
- **THEN** the graph assigns dense indices to nodes, builds adjacency lists using those indices, and discovers additional nodes yielded by `successors` until no new nodes appear.

### Requirement: IndexedGraph accessors
`IndexedGraph` SHALL provide accessors for index lookups, node lookup, and adjacency lists to support indexed algorithms.

#### Scenario: IndexedGraph lookups
- **WHEN** a user calls `index_of(&node)`, `node(index)`, and `successors(index)`
- **THEN** they receive the corresponding index, node reference, and adjacency slice for that index (or `None` if the node/index is unknown).
