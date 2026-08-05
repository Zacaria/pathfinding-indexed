## Why

The indexed graph API is performant once callers already have dense adjacency lists, but it still
asks users with grid- or matrix-shaped inputs to write repetitive conversion code before they can
use the crate effectively. A small set of input helpers can remove that friction without restoring
the broader generic helper surface of the original crate.

## What Changes

- Add adjacency-matrix constructors for `IndexedGraph`.
- Add walkable-grid helpers that build an `IndexedGraphMap<(usize, usize), usize>` from matrix-like
  boolean inputs and custom grid predicates.
- Add docs and examples showing how to enter the indexed API from common grid/matrix inputs.
- Keep `Grid`, `Matrix`, and `utils` out of the primary public API surface; this change adds
  adapters, not a full restoration of the old helper modules.

## Capabilities

### New Capabilities
- `indexed-input-helpers`: Build indexed graphs from adjacency matrices and walkable grid inputs.

### Modified Capabilities
- `crate-surface`: The public crate surface gains indexed input helper constructors while remaining
  indexed-first.

## Impact

- Affected code: `src/indexed_graph.rs`, `src/lib.rs`, `README.md`, `GRAPH_GUIDE.md`
- Affected docs/tests: new doctests and unit tests for helper construction paths
- No new runtime dependencies
