# RLINE

**Reusable Rust linework for graph, context, statistics, and optimization kernels.**

RLINE is the neutral home for reusable `r*` crates that should not live inside a
single product repo. Its first job is to let BISECT, CROP, ROUTE, and future
tools depend on shared kernels without depending on BISECT itself.

## Why RLINE

- **Clean dependency direction**: product repos consume RLINE kernels instead of
  importing reusable code from an application workspace.
- **Small generic crates**: graph, context, statistics, optimization, and history
  primitives remain product-neutral.
- **Planned extraction**: the foundation manifest records source paths,
  dependencies, consumers, and non-goals before code moves.
- **No product logic**: redistricting, election-audit, route, cache, or context
  packaging workflows stay in their own repos.

## Initial candidates

| Crate | Current source | Role |
|-------|----------------|------|
| `rctx-core` | `C:\src\apportionment\crates\rctx-core` | context packages, crosswalk verification, provenance records |
| `rgraph-core` | `C:\src\apportionment\crates\rgraph-core` | graph traits, shortest paths, cuts, connectivity, cluster summaries |
| `rstat-core` | `C:\src\apportionment\crates\rstat-core` | deterministic statistics and quantiles |
| `ropt-core` | `C:\src\apportionment\crates\ropt-core` | deterministic optimization helpers |
| `rhist-core` | `C:\src\apportionment\crates\rhist-core` | history and lineage primitives layered on RCTX |

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
```

## License

MIT. See [`LICENSE`](LICENSE).

