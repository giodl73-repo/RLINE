# Wave: RLINE Foundation

## Goal

Create a neutral Rust workspace for reusable RLINE kernels and plan the clean
extraction of current BISECT `r*` crates.

## Thesis

Shared graph, context, statistics, optimization, and history kernels should have
their own dependency root. BISECT, CROP, ROUTE, RCOUNT, and future repos should
consume those kernels without depending on one another.

## Pulse table

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | Workspace foundation | done | Added workspace, manifest contract, CLI manifest/package commands, specs, and wave scaffolding. |
| 02 | Kernel extraction | done | Extracted `rctx-core`, `rgraph-core`, `rstat-core`, `rmath-core`, `ropt-core`, `rhist-core`, `rhist-io`, and `rhist-cli`. |
| 03 | Consumer migration | pending | Update BISECT/CROP and candidate ROUTE use sites to consume RLINE by git dependency. |

## Success criteria

- RLINE has its own Rust workspace and git repo.
- `rline-core` exposes product-neutral manifest and validation contracts.
- Shared kernel crates build and test inside RLINE without BISECT application
  crates.
- `rline-cli` can emit the foundation manifest and package list.
- Docs identify candidate crates, consumers, dependency boundaries, and non-goals.
- Wave and pulse scaffolding exists for follow-up work.
- `cargo fmt`, `cargo test --workspace`, focused CLI smokes, and
  `git diff --check` pass.

