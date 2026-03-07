# Algorithm implementation patterns in this repo (`pathfinding`)

This document describes the *common implementation approach* used across the already-implemented algorithms in this repository, with an emphasis on API shape, internal data structures, and the recurring “search loop + parent reconstruction” pattern.

The crate’s central design choice is: **no mandatory graph type**. Algorithms are implemented as generic functions that accept *successor functions* (and sometimes predecessor/neighbor functions) so they work with adjacency lists, matrices, implicit graphs, grids, etc. See the crate docs and Graph Guide for user-facing examples (`src/lib.rs`, `GRAPH_GUIDE.md`).

## 1. Code organization and re-exports

- Algorithms are split by graph type:
  - `src/directed/*` (search/path algorithms and directed-graph utilities)
  - `src/undirected/*` (MST, components, cliques, …)
  - A few algorithms/structures live at the top level (`src/kuhn_munkres.rs`, `src/matrix.rs`, `src/grid.rs`).
- `src/lib.rs` defines:
  - `#![forbid(missing_docs)]`: every public item must be documented.
  - A `prelude` module that re-exports all public algorithms for convenient import.
- `src/directed/mod.rs` contains a small shared utility: `reverse_path`, used by multiple shortest-path searches to reconstruct a path from a parent map.

## 2. Public API shape (what most algorithms look like)

### 2.1 “Successor function” interface (graph-agnostic)

Instead of requiring `Graph`/`Node` traits or a concrete graph type, algorithms typically accept:

- `start`: either `&N`, `N`, or a multi-start wrapper (`NodeRefs`)
- `successors`: `FnMut(&N) -> IN` (or similar), where `IN: IntoIterator<...>`
- `success`: `FnMut(&N) -> bool` goal predicate (when there is a “target” concept)

Examples:
- BFS (`src/directed/bfs.rs`) takes `start: S` where `S: Into<NodeRefs<'a, N>>` and `successors: FnMut(&N) -> IN` returning nodes.
- Dijkstra/A*/Fringe (`src/directed/dijkstra.rs`, `src/directed/astar.rs`, `src/directed/fringe.rs`) take `successors` that yield `(neighbor, move_cost)`.
- Topological sorting (`src/directed/topological_sort.rs`) takes a `roots: &[N]` plus `successors` (and explicitly allows discovering nodes not in `roots` in one of the variants).

Why `FnMut`?
- Many callers want successors/heuristics that close over mutable state (caches, counters, reusable buffers), and the algorithms generally work fine with `FnMut`.

### 2.2 Generic cost types via `num_traits`

Weighted searches use trait bounds like:
- `C: Zero + Ord + Copy` (Dijkstra/A*)
- sometimes also `Bounded`, `Signed` (flow, Fringe)

Two important consequences:
- `Ord` is required for priority ordering. If you want floats, you must wrap them (the crate docs explicitly recommend `ordered-float`).
- Using `num_traits::Zero` is convenient because it also implies addability in practice (via trait relationships), so code can do `new_cost = cost + move_cost` without additional boilerplate.

### 2.3 Return types: `Option`, `Result`, iterators, and “bag” solutions

Common return patterns:
- `Option<Vec<N>>` for unweighted path existence (BFS/DFS/IDDFS).
- `Option<(Vec<N>, C)>` for a shortest path plus its total cost (Dijkstra/A*/Fringe/IDA*).
- `Result<..., ...>` when “failure” is a meaningful result (e.g., cycles in `topological_sort`).
- Iterator-based APIs for “visit all reachable nodes”:
  - `bfs_reach` → `BfsReachable` (`src/directed/bfs.rs`)
  - `dijkstra_reach` → `DijkstraReachable` (`src/directed/dijkstra.rs`)
  - `dfs_reach` → `DfsReachable` (`src/directed/dfs.rs`)
- “All shortest paths” APIs:
  - `astar_bag` returns an iterator (`AstarSolution`) that yields all shortest paths (non-deterministic order) (`src/directed/astar.rs`).

## 3. Shared performance building blocks

### 3.1 Fast hashing and stable indexing

Two recurring internal type aliases (defined in `src/lib.rs`) are used heavily:
- `FxIndexMap<K, V>`: `IndexMap` + `rustc_hash::FxHasher`
- `FxIndexSet<K>`: `IndexSet` + `rustc_hash::FxHasher`

Why `IndexMap`/`IndexSet` instead of `HashMap`/`HashSet` everywhere?
- They provide *stable* insertion order.
- They provide *index-based access* (`get_index`, `index()`) which enables a common optimization:
  - store each node **once** in an `IndexMap`
  - refer to nodes in the frontier via compact `usize` indices (instead of cloning nodes into heaps/queues repeatedly)

This “node table + indices” pattern appears in:
- BFS path building: `parents: FxIndexMap<N, usize>` is both the visited set and the queue (`src/directed/bfs.rs`).
- Dijkstra/A*: `parents: FxIndexMap<N, (usize, C)>`, and the heap stores `(cost, index)` (`src/directed/dijkstra.rs`, `src/directed/astar.rs`).

### 3.2 `reverse_path`: reconstructing a path from parent indices

`src/directed/mod.rs` defines:
- `reverse_path(parents, parent_index_fn, start_index) -> Vec<N>`

The pattern:
- During the search, store each node once in an `FxIndexMap`.
- For each discovered node, store the index of its predecessor (and often also its best-known cost).
- When a goal is reached, call `reverse_path` to walk indices backward and then reverse the collected nodes.

This is used by BFS, Dijkstra, A*, Fringe, etc.

### 3.3 Priority queues as min-heaps via wrappers

Rust’s `BinaryHeap` is a max-heap, so shortest-path searches implement `Ord` wrappers that invert ordering:
- `SmallestHolder` in Dijkstra (`src/directed/dijkstra.rs`)
- `SmallestCostHolder` in A* (`src/directed/astar.rs`)

This is also used elsewhere with `std::cmp::Reverse` when the stored element already has an `Ord` that can be flipped (e.g., Prim and Yen use `Reverse(...)`).

### 3.4 The “stale entry” technique (avoid decrease-key)

Dijkstra and A* both use a standard approach:
- When a better path to a node is found, push a new heap entry with the better cost.
- When popping from the heap, check whether the popped cost matches the current best-known cost; if not, discard it.

In code, this is the recurring guard:
- “If popped_cost > recorded_best_cost { continue; }”

This avoids the need for a decrease-key capable heap and keeps the implementation simple while still performant.

### 3.5 Index-based `Entry` updates

A recurring idiom relies on `IndexMap`’s entry API:

- `match parents.entry(successor)`:
  - `Vacant(e)`:
    - capture `e.index()` for the successor’s assigned stable index
    - insert `(parent_index, new_cost)` (or similar)
  - `Occupied(e)`:
    - update only if `new_cost` improves the recorded best cost

This pattern keeps “node table”, “best known cost”, and “parent pointers” in one structure.

## 4. Patterns by algorithm family

### 4.1 Unweighted shortest path (BFS)

Key implementation technique in this repo’s BFS (`src/directed/bfs.rs`):
- The visited set and queue are unified:
  - `parents: FxIndexMap<N, usize>`
  - process nodes in increasing `i` (index) order: `while let Some((node,_)) = parents.get_index(i) { ...; i += 1; }`
- Parent reconstruction uses `reverse_path`.

Multi-source support:
- BFS accepts `start: Into<NodeRefs<'a, N>>` so you can provide one start or many.
- `NodeRefs` (`src/noderefs.rs`) is a `FxHashSet<&N>` wrapper that supports `From<&N>` and `FromIterator<&N>`.

Bidirectional BFS (`bfs_bidirectional`) uses two `FxIndexMap`s to grow two frontiers until they meet.

### 4.2 Weighted shortest path (Dijkstra / A* / Fringe)

Common structure:
1. Frontier: `BinaryHeap` (Dijkstra/A*) or `VecDeque` (Fringe uses two queues).
2. Best-known cost + parent pointers: `FxIndexMap`.
3. Loop:
   - pick next node by algorithm-specific rule
   - skip stale frontier entries if needed
   - check goal predicate
   - iterate successors and relax/update
4. Reconstruct path via `reverse_path`.

Algorithm-specific differences:
- Dijkstra: ordering is by known cost only.
- A*: ordering is by `cost + heuristic(node)`; note the `SmallestCostHolder` tie-breaking prefers higher `cost` when `estimated_cost` matches, to reduce expansions (see comment in `src/directed/astar.rs`).
- Fringe: uses an `f`-limit (`flimit`) and partitions work into “now” and “later” queues (`VecDeque`) (`src/directed/fringe.rs`).

### 4.3 Iterative deepening searches (IDDFS / IDA*)

The iterative deepening algorithms use a different style: recursive depth-limited search.

- IDDFS (`src/directed/iddfs.rs`):
  - owns the `start` node to avoid cloning it.
  - maintains the current `path: Vec<N>` and checks `path.contains(...)` to avoid loops.
  - repeats depth-bounded DFS increasing depth until it finds a solution or proves impossibility.

- IDA* (`src/directed/idastar.rs`):
  - uses an `IndexSet` for the current path so it can both preserve path order and do fast membership tests.
  - sorts successors by `cost + heuristic` to prioritize likely good branches.

### 4.4 Enumerating multiple solutions (A* bag, Yen)

Two notable patterns:

- `astar_bag` (`src/directed/astar.rs`):
  - stores **multiple parents** per node when multiple best paths reach it with identical cost.
  - returns an iterator (`AstarSolution`) that lazily enumerates all shortest paths from the recorded parent DAG.
  - also provides a helper `astar_bag_collect` for convenience (with an explicit warning about combinatorial explosion).

- `yen` (`src/directed/yen.rs`):
  - reuses `dijkstra_internal` for spur-path computations (explicit internal function reuse).
  - builds candidate paths in a min-heap (`BinaryHeap<Reverse<Path<...>>>`).
  - recalculates costs via `make_cost` when needed (since root path cost is not cached).

### 4.5 Graph property algorithms (SCC, topological sort)

These are implemented in a more “classic textbook” style, still generic over nodes and successor functions:

- Strongly connected components (`src/directed/strongly_connected_components.rs`):
  - path-based SCC algorithm, with internal stacks and preorder numbering.
  - provides “from start”, “single component”, and “all components” entry points.

- Topological sort (`src/directed/topological_sort.rs`):
  - provides a DFS-based ordering algorithm returning `Result<Vec<N>, N>` (an arbitrary node in a detected cycle).
  - also provides “into groups” variant that returns independent layers and a partial result on cycle detection.

### 4.6 “Index remapping” for algorithms that want dense integer IDs (Kruskal, Edmonds–Karp)

Several algorithms are simpler or faster when they operate on `usize` indices and arrays/matrices.
The common approach in this repo is:

1. Build an `IndexSet` / `FxIndexSet` to map user nodes to indices.
2. Run the core algorithm on indices.
3. Map results back to node references/values.

Examples:
- Kruskal (`src/undirected/kruskal.rs`):
  - converts nodes in edge list to indices with `FxIndexSet`.
  - runs union-find over `Vec<usize>` parents/ranks.
- Edmonds–Karp (`src/directed/edmonds_karp.rs`):
  - maps `vertices: &[N]` to indices.
  - offers dense (`Matrix`) and sparse (maps) implementations behind a common `EdmondsKarp` trait.
  - returns flows and min-cut edges mapped back to node pairs.

### 4.7 Simple DP/caching algorithms (count_paths)

`count_paths` (`src/directed/count_paths.rs`) is implemented as memoized recursion:
- It uses `FxHashMap` as a cache keyed by node.
- It assumes no cycles; otherwise recursion can overflow the stack (documented in comments).

## 5. Error handling, panics, and clippy policy

The codebase generally uses:
- `Option` for “might not find a path”.
- `Result` for “algorithm can fail in a meaningful way” (cycle detection, etc.).
- `panic!` / `unwrap()` when a failure indicates a *programmer error* or an invariant breach (e.g., “source not found in vertices” in `edmonds_karp`).

You will see targeted `#[expect(clippy::missing_panics_doc)]` on functions that use internal `unwrap()` where the authors consider it unavoidably safe due to invariants.

## 6. Documentation and testing approach

Documentation is a first-class part of the implementation:
- Every public function has doc comments describing parameters, semantics, and includes runnable examples (doctests).

Testing is layered:
- Integration tests in `tests/` validate core behavior and cover regressions (e.g., BFS multiple starts in `tests/test_bfs_multiple_starts.rs`, Yen issues in `tests/yen-issue-507.rs`).
- Some modules also include focused unit tests (e.g., `src/undirected/kruskal.rs` tests path-halving in union-find).

## 7. How to add a new algorithm in the “house style”

If you implement another algorithm and want it to fit the existing style:

1. Put it under `src/directed/` or `src/undirected/` as appropriate, and expose it from the corresponding `mod.rs`.
2. Prefer the “successor function” API:
   - `FnMut(&N) -> IntoIterator<Item = ...>`
   - accept a `success` predicate if there is a goal concept
3. Use `FxIndexMap`/`FxIndexSet` when you benefit from stable insertion order or index-based storage.
4. If you use a `BinaryHeap`, consider the stale-entry pattern instead of decrease-key.
5. Provide:
   - a path+cost variant (if weighted)
   - a “reach” iterator variant if it naturally fits
6. Write doc comments + examples and add at least one integration test in `tests/` for core correctness and edge cases.

