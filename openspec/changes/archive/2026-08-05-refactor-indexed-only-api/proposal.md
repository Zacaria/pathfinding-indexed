# Change: Refactor to index-only graph APIs

## Why
This crate is being repositioned as `pathfinding-indexed` to focus on predictable, high performance
by using dense node indices and contiguous storage. The existing closure-based APIs and helper
modules rely on hashing in hot paths and do not align with that goal.

## What Changes
- **BREAKING**: Replace closure-based algorithms with methods on `IndexedGraph` (directed)
  and `IndexedUndirectedGraph` (undirected).
- **BREAKING**: Remove `grid`, `matrix`, `utils`, `kuhn_munkres`, `noderefs`, and cycle
  detection modules from the public API.
- Add index-only graph core types plus an optional mapping helper that builds graphs from
  external node values without affecting algorithm hot paths.
- Port all algorithms, examples, tests, and benches to the indexed API.
- Rename crate metadata to `pathfinding-indexed` and update docs to match.

## Impact
- Affected specs: indexed-graphs, indexed-algorithms, crate-surface
- Affected code: `src/lib.rs`, `src/indexed_graph.rs`, `src/directed/*`, `src/undirected/*`,
  `tests/*`, `examples/*`, `benches/*`, `README.md`, `GRAPH_GUIDE.md`, `Cargo.toml`
