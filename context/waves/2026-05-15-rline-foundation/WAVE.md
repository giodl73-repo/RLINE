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
| 02 | Source audit | pending | Inventory current `r*` crate APIs, dependency edges, tests, and downstream consumers. |
| 03 | First extraction | pending | Move the lowest-risk zero-application-dependency crate(s), likely `rstat-core` and `ropt-core`. |
| 04 | Graph/context extraction plan | pending | Split `rgraph-core`, `rctx-core`, and `rhist-core` with compatibility shims. |
| 05 | Consumer migration | pending | Update BISECT/CROP and candidate ROUTE use sites to consume RLINE. |

## Success criteria

- RLINE has its own Rust workspace and git repo.
- `rline-core` exposes product-neutral manifest and validation contracts.
- `rline-cli` can emit the foundation manifest and package list.
- Docs identify candidate crates, consumers, dependency boundaries, and non-goals.
- Wave and pulse scaffolding exists for follow-up work.
- `cargo fmt`, `cargo test --workspace`, focused CLI smokes, and
  `git diff --check` pass.

