## 1. Repository and crate rename
- [x] 1.1 Update `Cargo.toml` crate name, package metadata, and badges/links.
- [x] 1.2 Update `README.md` and `GRAPH_GUIDE.md` to describe the index-only API.

## 2. Core indexed graph types
- [x] 2.1 Replace `IndexedGraph` with an index-only directed graph (adjacency list).
- [x] 2.2 Add `IndexedUndirectedGraph` with canonical undirected edge storage.
- [x] 2.3 Add a mapping helper (builder) to create graphs from external node values.

## 3. Directed algorithms on `IndexedGraph`
- [x] 3.1 Refactor traversal/shortest-path algorithms to methods on `IndexedGraph`.
- [x] 3.2 Refactor max-flow (Edmonds-Karp) to index-only storage without `Matrix`.
- [x] 3.3 Remove cycle detection module.

## 4. Undirected algorithms on `IndexedUndirectedGraph`
- [x] 4.1 Refactor connected components, cliques, Kruskal, and Prim.

## 5. Public API cleanup
- [x] 5.1 Remove grid, matrix, utils, kuhn_munkres, and noderefs modules from the crate.
- [x] 5.2 Update module layout and re-exports (including any prelude).

## 6. Tests, examples, and benches
- [x] 6.1 Port tests to index-based APIs.
- [x] 6.2 Port examples to index-based APIs.
- [x] 6.3 Port benches and ensure feature flags still work.

## 7. Validation
- [x] 7.1 Run fmt, clippy, and tests; compile benches if available.
