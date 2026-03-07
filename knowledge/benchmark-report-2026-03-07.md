# Benchmark Report: BMSSP vs Dijkstra

Date: 2026-03-07

## Scope

This report summarizes the BMSSP benchmark work that was added during the indexed-only API
refactor and the follow-up BMSSP optimization pass.

The goal was not to prove that BMSSP is universally faster than Dijkstra. The goal was to:

- measure the current indexed BMSSP implementation against Dijkstra,
- add repeatable workloads that cover BMSSP's intended niche,
- identify where the implementation still falls short of the paper's performance claims.

## Benchmarks

### 1. Multi-target constant-degree benchmark

Command:

```bash
cargo bench --bench bmssp_all_vs_dijkstra -- --sample-size 10
```

Workload:

- 4096 nodes
- out-degree 4
- positive integer weights in `[1, 9]`
- single-source, all-target shortest paths
- compares `IndexedGraph::dijkstra_all()` and `IndexedGraph::bmssp_all()`

Latest local result:

| Benchmark | Time |
| --- | --- |
| `constant_degree_dijkstra_all` | `465.80 µs` to `488.09 µs` |
| `constant_degree_bmssp_all` | `4.1757 ms` to `4.4823 ms` |

Interpretation:

- BMSSP improved materially relative to earlier revisions.
- Dijkstra is still much faster on this workload.
- The benchmark is still useful because it exercises BMSSP's multi-target API directly and catches
  regressions in the specialized implementation.

### 2. Dense directed single-pair benchmark

Command:

```bash
cargo bench --bench bmssp_vs_dijkstra -- --sample-size 10
```

Workload:

- 4096 nodes
- out-degree 32
- positive integer weights in `[1, 9]`
- single-source, single-target shortest path
- compares `IndexedGraph::dijkstra()` and `IndexedGraph::bmssp()`

Latest local BMSSP result:

| Benchmark | Time |
| --- | --- |
| `dense_graph_dijkstra` | `434.12 µs` to `435.59 µs` |
| `dense_graph_bmssp` | `45.286 ms` to `47.389 ms` |

Interpretation:

- The dense-graph BMSSP benchmark improved dramatically from the very slow early prototype.
- Even after the queue and batching refactors, this workload still does not show an advantage over
  Dijkstra.
- The dense benchmark remains valuable because it exposes partition-queue overhead, D1 index churn,
  and transient allocation growth.

## Profiling Notes

Sampling on the dense benchmark consistently pointed to the same hot areas:

- `bmssp_rec`
- `PartitionQueue::insert`
- `PartitionQueue::create_block`
- `PartitionQueue::update_block_max`
- `BTreeMap` insert/remove inside the D1 block index

The base-case `BinaryHeap` work is visible, but secondary. The main remaining cost is partition
queue maintenance and associated allocation churn.

## Current Conclusion

The repository now has a substantially stronger BMSSP implementation than it started with:

- the algorithm is integrated into the indexed graph API,
- dedicated BMSSP benchmarks exist and run with the other Criterion benchmarks,
- the implementation is much faster than the initial indexed prototype.

However, the current evidence does **not** support marketing BMSSP as faster than Dijkstra in
general. The honest claim is narrower:

- the crate is built around indexed graph algorithms with predictable performance,
- BMSSP is an experimental specialized shortest-path implementation,
- the current benchmarks are primarily regression tests and optimization targets.

## What Remains for Paper-Level Performance

The most plausible next steps, if BMSSP optimization resumes later, are:

1. reduce partition-queue maintenance cost further,
2. reduce D1 block index update overhead,
3. reduce transient allocation growth in recursive work,
4. explore a constant-degree transformation or other paper-faithful preprocessing for the
   workloads where BMSSP is expected to shine.
