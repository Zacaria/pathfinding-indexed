## ADDED Requirements
### Requirement: Directed indexed graph storage
The system SHALL provide an `IndexedGraph<C>` type that stores a directed graph using dense
`usize` indices in the range `0..node_count` with weighted adjacency lists.

#### Scenario: Build from adjacency list
- **WHEN** a user builds an `IndexedGraph` from an adjacency list with `n` rows
- **THEN** the graph reports `node_count == n`
- **AND** `successors(i)` returns the stored adjacency for each `i` in `0..n`

### Requirement: Undirected indexed graph storage
The system SHALL provide an `IndexedUndirectedGraph<C>` type that stores undirected edges
once and exposes symmetric adjacency lists.

#### Scenario: Build from undirected edges
- **WHEN** a user builds an `IndexedUndirectedGraph` from a list of undirected edges
- **THEN** `successors(u)` includes `(v, w)` and `successors(v)` includes `(u, w)`
- **AND** the canonical edge list returns each undirected edge only once

### Requirement: Mapping helper for external node values
The system SHALL provide an optional helper that assigns dense indices to external node
values and produces an index-only graph plus mapping data.

#### Scenario: Build from node values
- **WHEN** a user builds a graph from external node values
- **THEN** the helper returns a graph with dense indices
- **AND** the mapping data resolves each external node to its assigned index
