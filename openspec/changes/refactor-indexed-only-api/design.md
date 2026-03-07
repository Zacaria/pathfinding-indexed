## Context
The new crate `pathfinding-faster` prioritizes predictable performance. The current design
accepts successor closures over arbitrary node types, which forces hash-based maps/sets in
algorithm hot paths and makes performance sensitive to user-provided node hashing. The new
API will use dense `usize` indices and contiguous adjacency lists. Undirected graphs will be
modeled explicitly to avoid runtime deduplication and ambiguity.

## Goals / Non-Goals
- Goals:
  - Provide index-only graph types with fast adjacency access.
  - Expose algorithms as methods on graph types, with index-based inputs/outputs.
  - Separate directed and undirected graphs to enforce correct semantics.
  - Keep algorithm coverage (directed + undirected) while removing unrelated modules.
  - Allow optional mapping from external node values without affecting algorithm hot paths.
- Non-Goals:
  - Supporting dynamic/infinite graphs via successor closures.
  - Retaining grid/matrix/utils/kuhn_munkres/noderefs/cycle detection modules.
  - Preserving API compatibility with the original `pathfinding` crate.

## Decisions
- Decision: Use `IndexedGraph` for directed graphs and `IndexedUndirectedGraph` for undirected.
  - Why: Keeps invariants explicit and avoids ambiguous undirected semantics.
- Decision: Store adjacency as `Vec<Vec<(usize, C)>>` and keep algorithms index-only.
  - Why: Enables `Vec`-based visited/distances and avoids hashing in hot paths.
- Decision: Provide an optional mapping helper (builder) that returns an index-only graph
  plus node mapping data.
  - Why: Supports ergonomic construction without reintroducing hash-based storage inside
    algorithms.
- Decision: Move algorithms to methods on the graph types, keeping names aligned with
  current algorithms where possible.
  - Why: Simplifies the public API and makes graph ownership explicit.

## Alternatives Considered
- Single graph type with “undirected = symmetric edges”.
  - Rejected: It requires per-call edge deduplication for MST/cliques and risks user error.
- Always embed node mappings in the graph types.
  - Rejected: Adds memory overhead for all users and is unnecessary for index-native callers.

## Risks / Trade-offs
- Reduced flexibility: dynamic successor generation and non-index node types are no longer
  supported directly.
- Memory growth: full adjacency lists must be materialized before running algorithms.
- Additional type surface: separate directed/undirected graph types add API surface.
- Max-flow refactor: removing Matrix usage in Edmonds-Karp requires new internal
  representation for residual capacities.

## Migration Plan
- Update crate name and documentation to clearly communicate the index-only API.
- Provide mapping helper examples to ease migration from node-based graphs.
- Port all tests, examples, and benches to the new API, ensuring behavior parity where
  applicable.

## Open Questions
- None.
