## 1. Partition queue implementation
- [x] 1.1 Replace the heap-based `PartitionQueue` with a block-based queue implementing
          `Insert`, `BatchPrepend`, and `Pull` boundary semantics.
- [x] 1.2 Add per-key location tracking and block splitting on overflow (`M`).

## 2. BMSSP hot-path optimizations
- [x] 2.1 Replace hash sets/maps in `find_pivots`, `base_case`, and recursion bookkeeping
          with index-backed vectors.
- [x] 2.2 Ensure correctness of label ordering, relaxation, and boundary handling.

## 3. Constant-degree transformation
- [x] 3.1 Build the internal port graph (ports per incident edge, 0-weight cycles).
- [x] 3.2 Run BMSSP on the port graph and project parents/costs back to original nodes.
- [x] 3.3 Add tests to compare BMSSP vs Dijkstra on multi-degree graphs.

## 4. Benchmarks
- [x] 4.1 Update/add dense directed benchmarks to compare BMSSP vs Dijkstra.
- [x] 4.2 Add a multi-target SSSP benchmark (BMSSP-all vs Dijkstra-all) on a constant-degree graph.

## 5. Validation
- [x] 5.1 Run fmt, clippy, and tests.
- [x] 5.2 Run the BMSSP/Dijkstra comparison bench.

## 6. Profiling and hot-spot optimization
- [x] 6.1 Profile `dense_graph_bmssp` on the port graph and capture hot spots.
- [x] 6.2 Reduce allocation churn in BMSSP hot paths (scratch reuse/preallocation).
- [x] 6.3 Re-run the BMSSP/Dijkstra comparison bench to measure impact.
