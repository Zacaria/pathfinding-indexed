# build-indexed-graph Specification

## Purpose
Define how external node values map into the crate's dense indexed graph representation.
## Requirements
### Requirement: IndexedGraphMap utility
The library SHALL provide an `IndexedGraphMap<N, C>` utility that stores a dense node mapping alongside an `IndexedGraph<C>` with weighted adjacency lists.

#### Scenario: Build from nodes and successors
- **WHEN** a user constructs an `IndexedGraphMap` from a set of seed nodes and a `successors(&N) -> IntoIterator<Item = (N, C)>` closure
- **THEN** the map assigns dense indices to nodes, builds an `IndexedGraph<C>` using those indices, and discovers additional nodes yielded by `successors` until no new nodes appear.

### Requirement: IndexedGraphMap accessors
`IndexedGraphMap` SHALL provide accessors for index lookups, node lookup, and its indexed graph to support indexed algorithms.

#### Scenario: IndexedGraphMap lookups
- **WHEN** a user calls `index_of(&node)`, `node(index)`, and `graph()`
- **THEN** index and node lookups return the mapped value or `None`, and `graph()` returns the associated `IndexedGraph<C>`.
