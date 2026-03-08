#![forbid(missing_docs)]
//! # pathfinding-indexed
//!
//! Index-only pathfinding, flow, and graph algorithms with dense `usize` indices.
//!
//! The primary API is [`IndexedGraph`] for directed graphs and
//! [`IndexedUndirectedGraph`] for undirected graphs. Algorithms are exposed as
//! methods on these types.
//!
//! This crate builds on the original [`pathfinding`](https://crates.io/crates/pathfinding)
//! crate and credits Samuel Tardieu and its contributors for the original
//! library this indexed-only variant descends from.
//!
//! ## Example
//!
//! ```rust
//! use pathfinding_indexed::IndexedGraph;
//!
//! let graph = IndexedGraph::from_adjacency(vec![
//!     vec![(1, 2), (2, 4)],
//!     vec![(2, 1), (3, 7)],
//!     vec![(3, 3)],
//!     vec![],
//! ]);
//!
//! let result = graph.dijkstra(0, |node| node == 3);
//! assert_eq!(result, Some((vec![0, 1, 2, 3], 6)));
//! ```
//!
//! ## More Examples
//!
//! Build a graph from external node values:
//!
//! ```rust
//! use pathfinding_indexed::IndexedGraphMap;
//! use std::collections::HashMap;
//!
//! let raw: HashMap<&str, Vec<(&str, u32)>> = [
//!     ("A", vec![("B", 4), ("C", 2)]),
//!     ("B", vec![("C", 1), ("D", 5)]),
//!     ("C", vec![("D", 8)]),
//!     ("D", vec![]),
//! ]
//! .into_iter()
//! .collect();
//!
//! let mapped = IndexedGraphMap::from_nodes_and_successors(["A"], |node| {
//!     raw.get(node).cloned().unwrap_or_default()
//! });
//!
//! let start = mapped.index_of(&"A").unwrap();
//! let goal = mapped.index_of(&"D").unwrap();
//! let result = mapped.graph().dijkstra(start, |node| node == goal);
//! assert_eq!(result.map(|(_, cost)| cost), Some(9));
//! ```
//!
//! Use the undirected graph API for MST algorithms:
//!
//! ```rust
//! use pathfinding_indexed::IndexedUndirectedGraph;
//!
//! let graph = IndexedUndirectedGraph::from_edges(
//!     4,
//!     vec![(0, 1, 7), (0, 2, 3), (1, 2, 1), (1, 3, 2), (2, 3, 6)],
//! );
//!
//! let mst = graph.kruskal();
//! assert_eq!(mst.len(), 3);
//! ```
//!
//! The minimum supported Rust version (MSRV) is Rust 1.87.0.

mod directed;
mod noderefs;
mod undirected;

pub mod indexed_graph;

pub use indexed_graph::{IndexedGraph, IndexedGraphMap, IndexedInputError, IndexedUndirectedGraph};

use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;

/// Convenience re-exports for indexed graph types.
pub mod prelude {
    pub use crate::indexed_graph::{
        IndexedGraph, IndexedGraphMap, IndexedInputError, IndexedUndirectedGraph,
    };
}
