## ADDED Requirements
### Requirement: Constant-degree port graph
The BMSSP indexed implementation SHALL internally transform the input directed graph into a
constant-degree port graph by creating a port for each incident edge, adding a directed 0-weight
cycle across the ports of each vertex, and replacing each original edge `(u, v, w)` with a port
edge carrying weight `w`. The algorithm SHALL run BMSSP on the port graph and project the
resulting parents and costs back to the original node indices.

#### Scenario: Internal transformation for multi-degree vertex
- **GIVEN** a vertex with multiple incoming and outgoing edges
- **WHEN** `bmssp_all_indexed` is called
- **THEN** the internal graph includes per-edge ports connected by 0-weight cycle edges and
  the returned parent/costs refer to the original node indices

## MODIFIED Requirements
### Requirement: Partition queue boundary semantics
The BMSSP implementation SHALL maintain a partition queue with `Insert`, `BatchPrepend`, and
`Pull` operations that preserve the boundary ordering semantics described in the paper.

#### Scenario: Boundary separation
- **GIVEN** a partition queue with `M = 2` containing labels `{2, 4}` and upper bound `B = 10`
- **WHEN** `BatchPrepend` inserts a label `1` and `Pull` is called
- **THEN** `Pull` returns the two smallest labels `{1, 2}` and a boundary `x` such that
  `2 < x <= 4`
