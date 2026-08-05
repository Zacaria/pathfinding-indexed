## ADDED Requirements

### Requirement: Indexed graphs can be built from adjacency matrices
The system SHALL provide a constructor that builds an `IndexedGraph` from a square adjacency matrix
whose cells contain optional edge weights.

#### Scenario: Valid adjacency matrix
- **WHEN** a caller provides a square `Option<C>` matrix
- **THEN** the system returns an `IndexedGraph`
- **AND** every `Some(weight)` cell becomes a directed edge to the corresponding column

#### Scenario: Invalid adjacency matrix shape
- **WHEN** a caller provides a ragged or non-square adjacency matrix
- **THEN** the system returns an error describing the invalid matrix shape

### Requirement: Indexed graph maps can be built from walkable grids
The system SHALL provide helpers that build an `IndexedGraphMap<(usize, usize), usize>` from
walkable grid inputs while preserving a coordinate-to-index mapping.

#### Scenario: Walkable boolean matrix with orthogonal neighbors
- **WHEN** a caller provides a boolean matrix and requests 4-neighbor connectivity
- **THEN** the system returns a graph map whose nodes are the walkable `(row, column)` cells
- **AND** edges connect orthogonally adjacent walkable cells with unit cost

#### Scenario: Walkable boolean matrix with diagonal neighbors
- **WHEN** a caller provides a boolean matrix and requests 8-neighbor connectivity
- **THEN** the system returns a graph map whose nodes are the walkable `(row, column)` cells
- **AND** edges connect orthogonally and diagonally adjacent walkable cells with unit cost

#### Scenario: Custom walkable grid predicate
- **WHEN** a caller provides grid dimensions and a predicate describing which cells are walkable
- **THEN** the system returns a graph map for the walkable cells
- **AND** the caller can recover dense indices from `(row, column)` coordinates through the map
