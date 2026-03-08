## Context

`pathfinding-indexed` intentionally moved away from the original crate's generic closure-based API
to graph-owned methods on dense indexed graph types. That improves performance predictability, but
it also leaves common caller inputs such as blocked-cell grids and adjacency matrices one step away
from the crate's primary API.

The codebase still contains internal `grid` and `matrix` modules from the earlier crate history,
but re-exporting them as the main user story would blur the product boundary that the indexed-only
refactor established.

## Goals / Non-Goals

**Goals:**
- Let users construct indexed graphs from common matrix- and grid-shaped inputs with minimal setup.
- Preserve the indexed graph types as the primary public API.
- Return explicit mappings for grid-like helpers so callers can move between coordinates and dense
  node indices.

**Non-Goals:**
- Restoring the old `Grid`, `Matrix`, or `utils` modules as first-class public APIs.
- Reintroducing generic closure-based pathfinding entry points.
- Covering every possible input encoding in the first helper pass.

## Decisions

- Add `IndexedGraph::from_adjacency_matrix()` for square `Option<C>` matrices.
  - Rationale: adjacency matrices are a common direct graph representation and the conversion is
    unambiguous.
- Add walkable-grid helpers on `IndexedGraphMap<(usize, usize), usize>` rather than on a separate
  public `Grid` type.
  - Rationale: callers get both the indexed graph and the coordinate/index mapping in one object.
- Provide both matrix-based helpers and predicate-based grid helpers.
  - Rationale: matrix helpers cover the common "I already have a 2D array" case, while predicate
    helpers let callers adapt custom grid containers without reintroducing generic graph closures.
- Keep helper edge costs fixed at `1` for walkable-grid helpers.
  - Rationale: this matches the common blocked/unblocked grid case and keeps the first pass simple.

## Risks / Trade-offs

- [Users may want the full old helper surface back] -> Document that these helpers are adapters into
  indexed graphs, not a return to the old crate model.
- [Coordinate conventions may be ambiguous] -> Use `(row, column)` consistently in docs and helper
  signatures.
- [API creep] -> Limit the initial surface to adjacency matrices and walkable grids only.

## Migration Plan

- Add helper constructors and examples.
- Validate with unit tests and doctests.
- No data migration or release migration is required; this is additive API surface.

## Open Questions

- None for the first helper pass.
