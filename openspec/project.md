# Project Context

## Purpose
Rust library crate providing generic pathfinding, flow, and graph algorithms (A*, Dijkstra, BFS/DFS, SCC, MSTs, Kuhn-Munkres, etc.) plus helper data structures (grid, matrix). Library-only crate with no prescribed graph storage; algorithms accept caller-provided successor/weight functions. Focus on broad algorithm coverage with minimal dependencies and strong docs/examples (see `GRAPH_GUIDE.md`).

## Tech Stack
- Rust 2024 edition; MSRV 1.87.0; dual Apache-2.0/MIT license
- Core deps: `num-traits`, `indexmap` + `rustc-hash`, `integer-sqrt`, `thiserror`, `deprecate-until`
- Dev/test/bench deps: `codspeed-criterion-compat`, `iai-callgrind`, `trybuild`, `rand` family, `movingai`, `noisy_float`, `itertools`, `version_check`, `regex`
- Feature flags: `iai` enables iai-callgrind deterministic benches
- Tooling: cargo, rustfmt (stable), clippy (nightly), cargo-deny, pre-commit hooks, CodSpeed/iai benches
- CI: GitHub Actions (tests, fmt, clippy, cargo-deny, pre-commit), CodSpeed benchmarks; GitLab CI is legacy

## Project Conventions

### Code Style
- Format with `cargo +stable fmt --all`; CI enforces stable rustfmt
- Lint with `cargo +nightly clippy --all-targets -- -D warnings`; `pedantic` denied with targeted allows/denies in `Cargo.toml`
- `#![forbid(missing_docs)]` in `src/lib.rs`; public APIs require docs and examples when applicable
- Prefer generic functions over fixed graph types; expose ergonomic `prelude` re-exports
- Conventional commits (`feat|fix|chore|test(scope): message`); Unix newlines, no trailing whitespace (pre-commit enforces)

### Architecture Patterns
- Library-only crate; modules organized by graph type (`directed`, `undirected`, `kuhn_munkres`, `grid`, `matrix`, `utils`)
- Algorithms accept caller-provided successor/weight functions instead of mandating a graph struct
- `prelude` exports common algorithms and types; minimal dependency footprint prioritized
- Uses `FxHasher` via `rustc-hash` with `IndexMap`/`IndexSet` for performance-sensitive collections
- Benchmarks split between Criterion/CodSpeed (wall time) and iai-callgrind (deterministic instruction counts)

### Module Conventions
- `src/directed/`: algorithms are named after the technique; inputs are `start`/`start+goal` plus `successors` closures (unweighted `N`, weighted `(N, C)`), often return `Option<Vec<N>>`, `Option<(Vec<N>, C)>`, or iterators; some APIs accept `NodeRefs` to support single or multiple start/end nodes
- `src/undirected/`: algorithms typically operate on edge lists or adjacency groups; return iterators or collections (e.g., `kruskal` yields edges, `connected_components` returns disjoint sets); `ConnectedComponents` offers customizable collection types with convenience free functions
- `src/grid.rs`: `Grid` uses `(x, y)` coordinates with optional diagonal adjacency; operations are methods on the type with `#[must_use]` on queries; `Debug` output uses `#`/`.` (alternate `#?` uses block characters)
- `src/matrix.rs`: `Matrix` is row-major with rotation/transpose helpers; fallible builders return `Result<_, MatrixFormatError>` (`thiserror`); `matrix!` macro provides literal construction; many transforms return new matrices with `#[must_use]`
- `src/kuhn_munkres.rs`: defines `Weights` for adjacency matrices (implemented for `Matrix`); algorithm returns `(total_weight, assignments)` and documents constraints/panics
- `src/utils.rs`: small, standalone helpers with simple signatures and examples; `#[must_use]` on pure value-returning helpers
- `src/noderefs.rs`: `NodeRefs` wraps one or many node references; used by graph algorithms to accept flexible start/end inputs

### API Documentation Patterns
- Public items are documented (`#![forbid(missing_docs)]` in `src/lib.rs`)
- Function docs start with a short summary, describe arguments in hyphenated lists, and include `# Example`/`# Examples` blocks using `pathfinding::prelude::*` or `pathfinding::utils::*`
- Use `# Panics` and `# Errors` sections when behavior warrants it; some algorithms use `#[expect(clippy::missing_panics_doc)]` where panics are not practical to document
- Link to longer-form guidance in `GRAPH_GUIDE.md` or example files when appropriate

### Testing Strategy
- Run `cargo test --tests --benches` plus `cargo test --doc` for every change; benches compiled as tests
- Fast iteration with `cargo check --all-targets`; release validation via `cargo +nightly clippy --all-targets -- -D warnings`
- Formatting check: `cargo +stable fmt --all -- --check`
- MSRV guard: `sh tests/check-msrv-consistency.sh` when touching MSRV mentions
- Security/license: `cargo deny check`; optional `pre-commit run --all-files` for hygiene (trailing whitespace, codespell, commit message lint)
- Benchmarks: wall-time via Criterion/CodSpeed (`cargo bench`), deterministic via iai-callgrind (`cargo bench --features iai`)

### Git Workflow
- Conventional commits required; each commit self-contained and clean (squash during review if needed)
- Prefer opening an issue before new features; PRs validated by CI matrix (stable/beta/nightly/MSRV, fmt, clippy, cargo-deny, pre-commit, CodSpeed)
- Do not manually edit `CHANGELOG.md`; releases handled via `./release.sh` + `cargo release` + `gh release`
- If a PR should close an issue, include "Fix #123" or "Close #123" in the PR description

## Domain Context
Algorithms cover shortest paths, flows, cycle detection, SCCs, MSTs, cliques, K-shortest paths, and matching. Library intentionally avoids prescribing graph storage; callers supply successors/weights, with helpers for grids/matrices and graph guide examples.

## Important Constraints
- Maintain MSRV 1.87.0 across `Cargo.toml`, docs, and instructions
- Clippy must run on nightly; rustfmt must run on stable
- Preserve lightweight dependency set and avoid altering `CHANGELOG.md` outside release flow
- Keep Unix newlines and no trailing whitespace (pre-commit enforced)
- If MSRV changes, update `Cargo.toml`, `src/lib.rs`, and `.github/copilot-instructions.md`, then run `sh tests/check-msrv-consistency.sh`

## External Dependencies
- External services: GitHub Actions CI (tests, lint, fmt, cargo-deny, pre-commit), CodSpeed benchmarking, docs.rs for published docs, crates.io for releases
- No runtime external services; relies solely on Rust crates listed above
