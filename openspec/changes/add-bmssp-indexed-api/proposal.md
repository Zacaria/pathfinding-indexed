# Change: add indexed BMSSP APIs and IndexedGraph utility

## Why
The current BMSSP APIs operate on generic node types via successor closures. For large graphs, users may want to avoid per-node hashing/cloning and run BMSSP on dense `usize` indices for better locality and lower overhead. This change adds indexed BMSSP entry points and a small helper to materialize an indexed graph while keeping the existing closure-based APIs usable.

## What Changes
- Add `bmssp_all_indexed` and `bmssp_indexed` (indexed `usize` node APIs), re-exported in the prelude.
- Introduce `IndexedGraph` as a helper for building dense indices and adjacency lists for use with indexed algorithms.
- Refactor existing `bmssp` / `bmssp_all` to delegate to the new indexed core via an adapter where possible.
- Add docs/examples and tests for the indexed APIs and `IndexedGraph` utility.

## Impact
- Affected specs:
  - `compute-bmssp-indexed`
  - `build-indexed-graph`
- Affected code:
  - `src/directed/bmssp.rs`
  - `src/lib.rs` (prelude)
  - new module for `IndexedGraph` (location TBD)
  - `tests/` and `examples/`
