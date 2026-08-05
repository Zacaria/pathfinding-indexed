## 1. Implementation
- [x] 1.1 Add an indexed BMSSP core that operates on `usize` nodes and `FnMut(usize)` successors.
- [x] 1.2 Expose `bmssp_all_indexed` and `bmssp_indexed` in `src/directed/bmssp.rs` and re-export them in the prelude.
- [x] 1.3 Add `IndexedGraph` utility to materialize dense indices + adjacency lists from user nodes.
- [x] 1.4 Refactor existing `bmssp` / `bmssp_all` to reuse the indexed core via an adapter.
- [x] 1.5 Add indexed API docs/examples and tests comparing indexed vs non-indexed results.

## 2. Validation
- [x] 2.1 `cargo fmt --all -- --check`
- [x] 2.2 `cargo test`
- [x] 2.3 `cargo clippy --all-targets -- -D warnings`
