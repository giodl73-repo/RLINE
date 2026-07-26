# RLINE

**Reusable Rust linework for graph, context, statistics, and optimization kernels.**

**Series:** [Tools & Infrastructure](https://github.com/giodl73-repo/giodl73-repo/blob/main/series/tools-infrastructure.md).

**Review roles:** This repo uses
[ROLES](https://github.com/giodl73-repo/ROLES), the `.roles` convention for
repository-local review panels.

## R package family

RLINE is the kernel foundation for a reusable civic-evidence package family:

```text
                    ┌→ RPLAN  — district-plan packages, IO, and audits ─┐
RLINE — kernels ────┤                                                   ├→ BISECT
                    └→ RCOUNT — count packages and audit replay ────────┘
```

| Repo | Responsibility |
|------|----------------|
| **RLINE** | Product-neutral graph, context, statistics, math, optimization, facility, and history kernels. |
| [RPLAN](https://github.com/giodl73-repo/RPLAN) | Portable district-plan representation, interchange, hashing, and audit certificates. |
| [RCOUNT](https://github.com/giodl73-repo/RCOUNT) | Election-count package verification, reconciliation, aggregation, and audit replay. |
| [BISECT](https://github.com/giodl73-repo/BISECT) | Redistricting application and research workbench that consumes the reusable layers. |

The dependency direction is one-way: generic kernels and packages must not
depend on BISECT product workflows.

RLINE is the neutral home for reusable `r*` crates that should not live inside a
single product repo. Its first job is to let BISECT, CROP, ROUTE, RPLAN, RCOUNT,
and future tools depend on shared kernels without depending on BISECT itself.

## Why RLINE

- **Clean dependency direction**: product repos consume RLINE kernels instead of
  importing reusable code from an application workspace.
- **Small generic crates**: graph, context, statistics, optimization, facility,
  and history primitives remain product-neutral.
- **Extracted kernels**: the foundation repo now contains the shared graph,
  context, statistics, math, optimization, facility, and history crates.
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
| `rfacility-core` | product-neutral facility identity, category, capability, and requirement primitives |
| `rhist-core` | history and lineage primitives layered on RCTX |
| `rhist-io` | RHIST package directory read/write helpers |
| `rhist-cli` | `rhist` command-line verifier |

## Commands

```powershell
cargo run -p rline-cli -- manifest
cargo run -p rline-cli -- packages
cargo run -p rline-cli -- packages --format json
```

`rline manifest` emits `rline.manifest.v1`, the package-family contract for the
shared kernel family. `rline packages` lists the extracted crates and their
current source paths.

## Workspace

| Crate | Purpose |
|-------|---------|
| `rline-core` | Manifest, package-family, and validation contracts for kernel extraction. |
| `rline-cli` | Small command surface for inspecting the foundation manifest. |
| `rctx-core`, `rgraph-core`, `rstat-core`, `rmath-core`, `ropt-core`, `rfacility-core`, `rhist-*` | Extracted shared kernel crates. |

## Design rule

RLINE stays product-neutral. BISECT, CROP, ROUTE, FLETCH, RPLAN, and RCOUNT can
consume or reference RLINE, but RLINE must not depend on those application
workflows.

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
