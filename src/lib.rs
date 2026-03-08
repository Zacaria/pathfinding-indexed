#![forbid(missing_docs)]
//! # pathfinding-indexed
//!
//! Index-only pathfinding, flow, and graph algorithms with dense `usize` indices.
//!
//! The primary API is [`IndexedGraph`] for directed graphs and
//! [`IndexedUndirectedGraph`] for undirected graphs. Algorithms are exposed as
//! methods on these types.
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
//! The minimum supported Rust version (MSRV) is Rust 1.87.0.

mod directed;
mod noderefs;
mod undirected;

pub mod indexed_graph;

pub use indexed_graph::{IndexedGraph, IndexedGraphMap, IndexedUndirectedGraph};

use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;

/// Convenience re-exports for indexed graph types.
pub mod prelude {
    pub use crate::indexed_graph::{IndexedGraph, IndexedGraphMap, IndexedUndirectedGraph};
}
