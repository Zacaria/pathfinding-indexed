## Context
BMSSP’s performance in the paper relies on a partition queue (`Insert`/`BatchPrepend`/`Pull`)
that avoids maintaining a fully sorted global frontier. The current implementation uses a heap
and hash-based sets, which reintroduces the sorting cost and large constants.

## Goals
- Implement a block-based partition queue that preserves the boundary semantics described in
  `knowledge/bmssp-analysis.md`.
- Reduce hashing overhead in BMSSP hot loops by using index-backed arrays.
- Apply the constant-degree transformation internally while preserving the existing public API.
- Keep the public API unchanged and maintain current correctness.

## Non-Goals
- Changing the public BMSSP API or requiring callers to opt into the transformation.

## Design Overview

### Partition Queue (`𝒟`)
The queue maintains two block sequences:
- `D0`: blocks created by `BatchPrepend` (all values smaller than any in `D1`).
- `D1`: blocks created by `Insert` (range-ordered by block upper bound).

Each block stores:
- `items: Vec<usize>` (node indices)
- `max: Label<C>` (largest label in the block)

Each key has a location table:
- `best[key] -> Option<Label<C>>`
- `loc[key] -> Option<(block_id, index_in_block)>`

Block size is capped at `M` (the layer bound); `D1` blocks are split at median when they exceed `M`.

### Insert
1. If `best[key]` is `None` or the new label is smaller, remove the old entry (if any).
2. Find the first `D1` block with `block.max >= label` (via `BTreeMap` keyed by `(max, block_id)`).
3. Insert the key into that block, update `best`/`loc`, and refresh block metadata.
4. If the block exceeds `M`, split it by median using `select_nth_unstable_by`, producing two
   blocks with updated `max` values and `BTreeMap` entries.

### BatchPrepend
All items are guaranteed smaller than any existing value:
1. Build blocks of size ≤ `M` from the input list.
2. Order blocks by `max` ascending (via `select_nth_unstable_by` partitioning).
3. Prepend blocks to `D0` so that `D0` is range-ordered from smallest to largest.

### Pull
1. Gather candidate blocks from `D0` in order until the total candidates ≥ `M`, then continue with
   `D1` blocks in ascending `max` order until reaching `M` (at most one extra block).
2. If candidates ≤ `M`, return all and boundary = upper bound.
3. Otherwise, select the `M` smallest candidates using `select_nth_unstable_by`; return them and
   set boundary to the minimum label among the remaining candidates.
4. Remove returned items from blocks, delete empty blocks, and recompute `max` when necessary.

### Array-backed Sets
Replace `FxHashSet` and `HashMap` usage in BMSSP hot paths with:
- `Vec<bool>` for membership (e.g., `W`, `extracted`, `out_set`)
- `Vec<Option<Label<C>>>` for `best`
- `Vec<Option<Location>>` for `loc`

### Constant-Degree Port Graph
The transformation builds an internal “port graph” that replaces each original vertex `v` with
`deg_in(v) + deg_out(v)` ports (or a single dummy port for an isolated start node). Each original
edge `(u, v, w)` is replaced by an edge from a dedicated outgoing port of `u` to a dedicated
incoming port of `v` with weight `w`. The ports for each original vertex are connected in a
directed 0-weight cycle so any incoming port can reach any outgoing port.

Implementation details:
- Build adjacency lists from the original `successors` closure once.
- Allocate contiguous port ranges per original vertex, with outgoing ports first, then incoming
  ports.
- Add 0-weight cycle edges within each vertex’s port range (skip for single-port ranges).
- Track `in_port_source[port] -> original source` and `in_ports_by_node[orig] -> [ports]`.
- Run BMSSP on the port graph, then project results by selecting the minimum-labeled incoming
  port per original vertex and mapping its `in_port_source` back to a parent index.
- After projection, validate that the resulting distances satisfy edge relaxations; fall back to
  indexed Dijkstra if any edge can still relax (small-graph safety net).

## Risks / Trade-offs
- Maintaining block metadata requires occasional rescans when the maximum item is removed.
- Correctness depends on keeping `best`/`loc` in sync during removals and splits.
- The median-based partitioning reduces full sorting but is still more complex than the current
  heap-based queue.
- The port graph increases memory and preprocessing time (nodes ≈ `2m`, edges ≈ `3m`).
