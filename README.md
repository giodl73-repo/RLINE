# RLINE

**Reusable Rust linework for graph, context, statistics, and optimization kernels.**

RLINE is the neutral home for reusable `r*` crates that should not live inside a
single product repo. Its first job is to let BISECT, CROP, ROUTE, RPLAN, RCOUNT,
and future tools depend on shared kernels without depending on BISECT itself.

## Why RLINE

- **Clean dependency direction**: product repos consume RLINE kernels instead of
  importing reusable code from an application workspace.
- **Small generic crates**: graph, context, statistics, optimization, and history
  primitives remain product-neutral.
- **Extracted kernels**: the foundation repo now contains the shared graph,
  context, statistics, math, optimization, and history crates.
- **No product logic**: redistricting, election-audit, route, cache, or context
  packaging workflows stay in their own repos.

## Extracted crates

| Crate | Role |
|-------|------|
| `rctx-core` | context packages, crosswalk verification, provenance records |
| `rgraph-core` | graph traits, shortest paths, cuts, connectivity, cluster summaries |
| `rstat-core` | deterministic statistics and quantiles |
| `rmath-core` | deterministic numeric and linear algebra kernels |
| `ropt-core` | deterministic optimization helpers |
| `rhist-core` | history and lineage primitives layered on RCTX |
| `rhist-io` | RHIST package directory read/write helpers |
| `rhist-cli` | `rhist` command-line verifier |

## Commands

```powershell
cargo run -p rline-cli -- manifest
cargo run -p rline-cli -- packages
cargo run -p rline-cli -- packages --format json
```

`rline manifest` emits `rline.manifest.v1`, the first extraction-planning
contract for the shared kernel family. `rline packages` lists the candidate
crates and their current source paths.

## Workspace

| Crate | Purpose |
|-------|---------|
| `rline-core` | Manifest, package-family, and validation contracts for kernel extraction. |
| `rline-cli` | Small command surface for inspecting the foundation manifest. |
| `rctx-core`, `rgraph-core`, `rstat-core`, `rmath-core`, `ropt-core`, `rhist-*` | Extracted shared kernel crates. |

## Design rule

RLINE stays product-neutral. BISECT, CROP, ROUTE, FLETCH, and RCOUNT can consume
or reference RLINE, but RLINE must not depend on those application workflows.

## Specs

- [`docs\specs\rline-foundation.md`](docs/specs/rline-foundation.md) defines the
  initial manifest and extraction boundaries.
- `context\waves\` tracks implementation waves and pulse history.

## Validation

```powershell
cargo fmt
cargo test --workspace
cargo run -p rline-cli -- manifest
cargo run -p rhist-cli -- --help
```

## License

MIT. See [`LICENSE`](LICENSE).

