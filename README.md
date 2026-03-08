# pathfinding-indexed

[![Current Version](https://img.shields.io/crates/v/pathfinding-indexed.svg)](https://crates.io/crates/pathfinding-indexed)
[![Documentation](https://docs.rs/pathfinding-indexed/badge.svg)](https://docs.rs/pathfinding-indexed)
[![License: Apache-2.0/MIT](https://img.shields.io/crates/l/pathfinding-indexed.svg)](#license)

`pathfinding-indexed` provides index-only graph algorithms with predictable performance. Graphs are
stored as dense `usize` indices with adjacency lists, and algorithms are exposed as methods on
`IndexedGraph` (directed) and `IndexedUndirectedGraph` (undirected).

The indexed graph API is the stable core of the crate. BMSSP support is included as an
experimental specialized shortest-path implementation; current repository benchmarks track its
progress, but do not yet show it outperforming Dijkstra on the workloads in `benches/`.

## Credits

This crate builds on the original [`pathfinding`](https://crates.io/crates/pathfinding) crate and
credits Samuel Tardieu and its contributors for the original library and algorithm coverage this
work descends from.

## Using this crate

In your `Cargo.toml`, put:

```ini
[dependencies]
pathfinding-indexed = "4.14.0"
```

## Example

```rust
use pathfinding_indexed::IndexedGraph;

let graph = IndexedGraph::from_adjacency(vec![
    vec![(1, 2), (2, 4)],
    vec![(2, 1), (3, 7)],
    vec![(3, 3)],
    vec![],
]);

let result = graph.dijkstra(0, |i| i == 3);
assert_eq!(result, Some((vec![0, 1, 2, 3], 6)));
```

## More Examples

Map external node values to dense indices:

```rust
use pathfinding_indexed::IndexedGraphMap;
use std::collections::HashMap;

let raw: HashMap<&str, Vec<(&str, u32)>> = [
    ("A", vec![("B", 4), ("C", 2)]),
    ("B", vec![("C", 1), ("D", 5)]),
    ("C", vec![("D", 8)]),
    ("D", vec![]),
]
.into_iter()
.collect();

let mapped = IndexedGraphMap::from_nodes_and_successors(["A"], |node| {
    raw.get(node).cloned().unwrap_or_default()
});

let start = mapped.index_of(&"A").unwrap();
let goal = mapped.index_of(&"D").unwrap();
let result = mapped.graph().dijkstra(start, |node| node == goal);
assert_eq!(result.map(|(_, cost)| cost), Some(9));
```

Work with undirected graphs directly:

```rust
use pathfinding_indexed::IndexedUndirectedGraph;

let graph = IndexedUndirectedGraph::from_edges(
    4,
    vec![(0, 1, 7), (0, 2, 3), (1, 2, 1), (1, 3, 2), (2, 3, 6)],
);

let mst = graph.kruskal();
assert_eq!(mst.len(), 3);
```

## Working with Graphs

See the [Graph Guide](GRAPH_GUIDE.md) for examples of building indexed graphs from adjacency lists,
edge lists, adjacency matrices, and for mapping external node values to dense indices.

## License

This code is released under a dual Apache 2.0 / MIT free software license.

## Benchmarking

This repository includes two types of benchmarks:

### Wall-time Benchmarks (Criterion/CodSpeed)

Traditional wall-time benchmarks using Criterion (with CodSpeed compatibility) are located in
`benches/` with names like `algos.rs`, `edmondskarp.rs`, etc. These can be run with:

```bash
cargo bench --bench algos --bench edmondskarp --bench separate_components
```

The BMSSP-focused benchmarks currently cover:

- `bmssp_all_vs_dijkstra`: single-source, all-target shortest paths on a 4096-node constant-degree graph
- `bmssp_vs_dijkstra`: single-pair shortest path on a 4096-node denser directed graph

Recent local runs on those workloads showed:

- `constant_degree_dijkstra_all`: about `0.47-0.49 ms`
- `constant_degree_bmssp_all`: about `4.2-4.5 ms`
- `dense_graph_dijkstra`: about `0.43-0.44 ms`
- `dense_graph_bmssp`: about `45-47 ms`

These numbers are machine-specific, but the current conclusion is stable: BMSSP has improved
substantially and is worth keeping as a specialized implementation, yet it is not a general
replacement for Dijkstra in this crate today.

### Deterministic Benchmarks (iai-callgrind)

For more precise and deterministic performance measurements, we use iai-callgrind which counts CPU
instructions, cache hits/misses, and estimated cycles using Valgrind. These benchmarks are prefixed
with `iai_` and require the `iai` feature flag:

```bash
# Install valgrind first (required by iai-callgrind)
sudo apt-get install valgrind  # On Ubuntu/Debian

# Run the benchmarks with the feature flag
cargo bench --features iai --bench iai_algos --bench iai_edmondskarp --bench iai_separate_components
```

The iai-callgrind benchmarks provide consistent results across runs and are not affected by system
load, making them ideal for detecting performance regressions. They run automatically in CI for all
pull requests, comparing performance against the base branch.

## Contributing

You are welcome to contribute by opening [issues](https://github.com/Zacaria/pathfinding-indexed/issues)
or submitting [pull requests](https://github.com/Zacaria/pathfinding-indexed/pulls). Please open an issue
before implementing a new feature, in case it is a work in progress already or it is fit for this
repository.

In order to pass the continuous integration tests, your code must be formatted using the latest
`rustfmt` with the nightly rust toolchain, and pass `cargo clippy` and [`pre-commit`](https://pre-commit.com/) checks.
Those will run automatically when you submit a pull request. You can install `pre-commit` to your
checked out version of the repository by running:

```bash
$ pre-commit install --hook-type commit-msg
```

This repository uses the [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) commit message style, such as:

- feat(indexed-graphs): add `IndexedGraph::from_adjacency()`
- fix(algorithms): avoid heap churn in `dijkstra`

Each commit must be self-sufficient and clean. If during inspection or code review you need to make further changes to a commit, please squash it. You may use `git rebase -i`, or more convenient tools such as [`jj`](https://martinvonz.github.io/jj/latest/) or [`git-branchless`](https://github.com/arxanas/git-branchless), in order to manipulate your git commits.

If a pull-request should automatically close an open issue, please
include "Fix #xxx" or "Close #xxx" in the pull-request cover-letter.
