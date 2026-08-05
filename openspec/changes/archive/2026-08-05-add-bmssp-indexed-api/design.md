## Context
BMSSP currently accepts generic node types and a successor closure. This is ergonomic but forces hashing, cloning, and allocations in the hot path. The paper's performance motivation is to reduce global sorting overhead, but practical wins require lower constant factors, dense indexing, and adjacency lists.

## Goals / Non-Goals
- Goals:
  - Provide indexed BMSSP APIs (`bmssp_all_indexed`, `bmssp_indexed`) for dense `usize` node IDs.
  - Provide a minimal `IndexedGraph` helper to materialize dense indices and adjacency lists.
  - Reuse a single indexed BMSSP core from both indexed and generic entry points.
- Non-Goals:
  - Implement the paper's full partition data structure `D`.
  - Introduce a new required graph storage type for all algorithms.
  - Change the semantics of existing `bmssp` / `bmssp_all` outputs.

## Decisions
- Indexed core API:
  - The indexed core operates on `usize` node IDs and accepts `successors: FnMut(usize) -> IN` where `IN: IntoIterator<Item = (usize, C)>`.
  - `bmssp_all_indexed` returns `Vec<Option<(usize, C)>>` of length `number_of_nodes`:
    - `None` for the start node and unreachable nodes.
    - `Some((parent, cost))` for reachable nodes with their optimal parent and cost.
  - `bmssp_indexed` mirrors `bmssp` and returns `Option<(Vec<usize>, C)>` for a goal predicate.
- `IndexedGraph` helper:
  - Provide a public `IndexedGraph<N, C>` that stores `nodes: Vec<N>`, `index: FxHashMap<N, usize>`, and `adj: Vec<Vec<(usize, C)>>`.
  - Constructor `IndexedGraph::from_nodes_and_successors(nodes, successors)` builds adjacency lists by calling successors for each known node, and if successors yield unknown nodes, they are added and processed until a fixed point.
  - Accessors:
    - `index_of(&N) -> Option<usize>`
    - `node(usize) -> Option<&N>`
    - `successors(usize) -> &[(usize, C)]`
    - `len()` / `is_empty()`
- Reuse in generic BMSSP:
  - `bmssp_all` and `bmssp` will delegate to the indexed core via an internal adapter that interns `N -> usize` and lazily materializes indexed successors using the original closure.
  - This keeps the behavior consistent and minimizes algorithm duplication.
- Module placement:
  - Introduce a new top-level module `indexed_graph` (e.g., `src/indexed_graph.rs`) and expose it via `pub mod indexed_graph`.
  - Re-export `IndexedGraph` in the prelude for discoverability alongside the indexed BMSSP APIs.

## Alternatives Considered
- Returning a `HashMap<usize, (usize, C)>` for `bmssp_all_indexed`: simpler but loses the density benefit of indexed APIs.
- Requiring users to pass `&[Vec<(usize, C)>]` instead of a successors closure: faster but less flexible; rejected to preserve the unified closure API style.
- Keeping separate core implementations for indexed vs generic APIs: rejected due to maintenance and divergence risk.

## Risks / Trade-offs
- `IndexedGraph::from_nodes_and_successors` can discover additional nodes; this adds work at build time and may surprise users expecting a fixed node list. This will be documented clearly.
- The indexed core still uses the simplified heap-based partition queue; asymptotic improvements from the paper are not guaranteed.

## Migration Plan
- No breaking changes; new APIs are additive.
- Existing `bmssp`/`bmssp_all` callers continue to work without changes.

## Open Questions
- None.
