## ADDED Requirements
### Requirement: Indexed BMSSP APIs
The library SHALL provide `bmssp_all_indexed` and `bmssp_indexed` for directed single-source shortest paths over dense `usize` node IDs with non-negative edge costs.

#### Scenario: All-nodes computation
- **WHEN** a caller invokes `bmssp_all_indexed(start, successors, number_of_nodes)`
- **THEN** it returns a `Vec<Option<(usize, C)>>` of length `number_of_nodes`, with `None` for the start node and unreachable nodes, and `Some((parent, cost))` for reachable nodes.

#### Scenario: Goal-directed path
- **WHEN** a caller invokes `bmssp_indexed(start, successors, success, number_of_nodes)`
- **THEN** it returns `Some((path, cost))` for the minimal-cost reachable node that satisfies `success`, or `None` if no goal is reachable.

### Requirement: Prelude exports
The library SHALL re-export `bmssp_all_indexed` and `bmssp_indexed` in the prelude.

#### Scenario: Prelude usage
- **WHEN** a user writes `use pathfinding::prelude::*`
- **THEN** `bmssp_all_indexed` and `bmssp_indexed` are in scope.

### Requirement: Indexed core reuse
The generic `bmssp` and `bmssp_all` APIs SHALL delegate to the indexed core through an adapter so results remain consistent across indexed and non-indexed entry points.

#### Scenario: Consistent results
- **WHEN** the same directed graph is provided to `bmssp_all` and `bmssp_all_indexed`
- **THEN** they produce identical shortest path costs for equivalent nodes.
