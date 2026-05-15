# RLINE Foundation Spec

## Goal

Create a neutral Rust workspace for reusable `r*` kernels extracted from BISECT
so BISECT, CROP, ROUTE, RPLAN, RCOUNT, and later repos can depend on them
cleanly.

## Core contract

### `rline.manifest.v1`

Describes the shared kernel family:

- `family`: package-family label, initially `rline`.
- `crates`: candidate kernel crates, current source paths, internal
  dependencies, public contracts, and migration status.
- `consumers`: repos that should consume the extracted kernels.
- `non_goals`: application logic that must stay outside RLINE.

The foundation manifest records the extracted kernel family and the remaining
consumer migration sequence.

## Extracted crates

| Crate | Kind | Current role |
|-------|------|--------------|
| `rctx-core` | context | context packages, crosswalk verification, graph/source provenance |
| `rstat-core` | statistics | deterministic summary stats, weighted stats, quantiles |
| `rmath-core` | math | deterministic numeric and linear algebra kernels |
| `ropt-core` | optimization | Pareto fronts, crowding distance, seed derivation, budget selection |
| `rgraph-core` | graph | graph traits, shortest paths, cuts, connectivity, cluster summaries |
| `rhist-core` | history | history and lineage primitives layered on RCTX |
| `rhist-io` | history IO | RHIST package directory read/write |
| `rhist-cli` | history CLI | standalone verifier CLI |

## Dependency boundary

RLINE may depend on generic Rust crates such as `serde`, `thiserror`, and
deterministic math/hash helpers. It must not depend on application crates from
BISECT, CROP, FLETCH, ROUTE, or RCOUNT.

## Non-goals

- No BISECT redistricting algorithms in RLINE.
- No RCOUNT CLI or election-audit workflow logic in RLINE.
- No CROP corpus workflow, FLETCH cache workflow, or ROUTE domain workflow in
  RLINE.
- RPLAN and RCOUNT live in their own sibling repos; they are consumers, not
  RLINE members.

## Initial CLI

```powershell
rline manifest
rline packages
rline packages --format json
```

## Migration sequence

1. Foundation manifest and repo scaffold.
2. Extract kernel crates into RLINE.
3. Update RPLAN and RCOUNT to use sibling RLINE/RPLAN dependencies.
4. Update BISECT and CROP to consume RLINE by git dependency.

