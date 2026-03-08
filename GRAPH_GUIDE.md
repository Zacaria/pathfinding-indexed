# Working with Indexed Graphs in pathfinding-indexed

This guide explains how to use `pathfinding-indexed` with traditional graph structures consisting of
nodes, edges, and weights. The library is index-only: nodes are dense `usize` indices, and the
algorithms are methods on indexed graph types.

## Core Concept: Dense Indices

Every node is represented by an index in `0..node_count`. You can build graphs directly from
adjacency lists or undirected edge lists, or use a mapping helper to assign indices to external
node values.

## Directed Graphs: Adjacency List

An adjacency list stores each node's outgoing neighbors and edge weights. This is efficient for
sparse graphs.

```rust
use pathfinding_indexed::IndexedGraph;

let graph = IndexedGraph::from_adjacency(vec![
    vec![(1, 4), (2, 2)],
    vec![(2, 1), (3, 5)],
    vec![(3, 8), (4, 10)],
    vec![(4, 2)],
    vec![],
]);

let result = graph.dijkstra(0, |node| node == 4);
assert_eq!(result, Some((vec![0, 1, 2, 3, 4], 12)));
```

## Directed Graphs: Adjacency Matrix

For dense graphs you might start from an adjacency matrix. Convert it to an adjacency list before
building the graph.

```rust
use pathfinding_indexed::IndexedGraph;

let matrix: Vec<Vec<Option<u32>>> = vec![
    vec![None, Some(4), Some(2), None, None],
    vec![None, None, Some(1), Some(5), None],
    vec![None, None, None, Some(8), Some(10)],
    vec![None, None, None, None, Some(2)],
    vec![None, None, None, None, None],
];

let adjacency = matrix
    .iter()
    .enumerate()
    .map(|(i, row)| {
        row.iter()
            .enumerate()
            .filter_map(|(j, weight)| weight.map(|w| (j, w)))
            .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();

let graph = IndexedGraph::from_adjacency(adjacency);
let result = graph.dijkstra(0, |node| node == 4);
assert_eq!(result, Some((vec![0, 2, 3, 4], 12)));
```

## Undirected Graphs: Edge List

Undirected graphs store each edge once and expose symmetric adjacency lists.

```rust
use pathfinding_indexed::IndexedUndirectedGraph;

let edges = vec![
    (0, 1, 4),
    (0, 2, 2),
    (1, 2, 1),
    (1, 3, 5),
    (2, 3, 8),
];

let graph = IndexedUndirectedGraph::from_edges(4, edges);
let mst = graph.kruskal();
assert_eq!(mst.len(), 3);
```

## Mapping External Node Values

If your nodes are not indices, use the mapping helper to assign dense indices and build an indexed
graph without affecting algorithm hot paths.

```rust
use pathfinding_indexed::IndexedGraphMap;
use std::collections::HashMap;

let raw: HashMap<&str, Vec<(&str, u32)>> = [
    ("A", vec![("B", 4), ("C", 2)]),
    ("B", vec![("C", 1), ("D", 5)]),
    ("C", vec![("D", 8), ("E", 10)]),
    ("D", vec![("E", 2)]),
    ("E", vec![]),
]
.into_iter()
.collect();

let mapped = IndexedGraphMap::from_nodes_and_successors(["A"], |node| {
    raw.get(node).cloned().unwrap_or_default()
});

let start = mapped.index_of(&"A").unwrap();
let goal = mapped.index_of(&"E").unwrap();

let result = mapped.graph().dijkstra(start, |node| node == goal);
let (path, cost) = result.expect(\"no path found\");
assert_eq!(*path.first().unwrap(), start);
assert_eq!(*path.last().unwrap(), goal);
assert_eq!(cost, 6);
```
