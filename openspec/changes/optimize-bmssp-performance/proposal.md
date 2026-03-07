# Change: Optimize BMSSP performance

## Why
The current BMSSP implementation uses a heap-backed partition queue and hash-based sets. This
reintroduces a global sorting cost and large constant factors, which undermines the performance
goals described in the paper. We need a closer match to the paper’s partition-queue approach and
lighter-weight index-backed bookkeeping to improve performance while preserving correctness.
The paper’s asymptotic bound also depends on reducing the graph to constant degree; applying
that transformation internally keeps the API stable while aligning the implementation with the
paper’s assumptions.

## What Changes
- Replace the heap-based partition queue with a block-list partition queue that implements
  `Insert`, `BatchPrepend`, and `Pull` with the ordering boundary semantics from the paper.
- Use index-backed storage (e.g., `Vec<Option<_>>`, `Vec<bool>`) for BMSSP state and work sets
  instead of hash maps/sets.
- Apply the constant-degree “port graph” transformation internally before running BMSSP and
  project results back to original node indices.
- Validate projected distances and fall back to indexed Dijkstra when relaxations still improve
  (correctness safety net for degenerate cases).
- Keep public APIs unchanged; this is an internal performance refactor.
- Update/add benches to compare BMSSP and Dijkstra on dense directed graphs.
- Add a multi-target SSSP benchmark (BMSSP-all vs Dijkstra-all) on a constant-degree graph.

## Impact
- Affected specs: compute-bmssp-indexed
- Affected code: `src/directed/bmssp.rs`, benches under `benches/`
- Additional memory/time overhead from building the port graph and mapping results back.
