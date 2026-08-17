# RLINE compatibility policy

RLINE is a pre-1.0 shared foundation. Compatibility is deliberate because
multiple product repositories depend on its kernels, serialized packages, and
deterministic evidence behavior.

## Protected contract

The protected surface includes:

- public APIs exported by the shared kernel crates;
- `rline.manifest.v1`, foundation crate names, dependency edges, consumer
  declarations, and deterministic JSON round-tripping;
- serialized RCTX and RHIST package schemas and version constants;
- hash-domain prefixes, canonical byte representations, and verification
  behavior;
- deterministic graph, statistics, math, optimization, and facility results;
  and
- public error meanings used by downstream validation and operator handling.

Internal refactoring is compatible only when these observable contracts remain
stable.

## Versioning rules

- Additive API or schema changes may remain within the current `0.y` line when
  existing consumers and serialized evidence remain compatible.
- Breaking signatures, schema fields, defaults, validation behavior, error
  meanings, hash inputs, or deterministic outputs require a minor-version bump
  while the affected crate is below `1.0`.
- Prefer deprecation plus migration notes before removing a public item.
- A breaking change must identify affected consumers and include fixture or
  migration guidance.
- Downstream repositories should pin commits for reproducible evidence.
  Branch consumers must run the downstream rehearsal before updating.

## Foundation tests

From the RLINE repository:

```powershell
cargo test -p rline-core
```

These tests protect the manifest schema, foundation inventory, dependency
validation, uniqueness rules, and deterministic JSON representation. Changes
to an individual kernel must also run that crate's focused tests.

## Downstream breakage rehearsal

RCOUNT is the required first consumer rehearsal because it uses RCTX crosswalk
records and verification in district aggregation and maps count lineage into
verified RHIST packages.

Use `RCOUNT\repo-map.toml` to create the ignored local Cargo patch that maps
`rctx-core` and `rhist-core` to the sibling RLINE checkout and RPLAN crates to
the sibling RPLAN checkout. Then run from RCOUNT:

```powershell
cargo test -p rcount-district aggregation_consumes_minimal_rctx_fixture_crosswalk
cargo test -p rcount-rhist
```

A compile failure exposes public API breakage. Crosswalk, hash, lineage, or
package-verification failures expose serialized or deterministic behavior
drift.

RLINE foundation changes are not ready until the manifest tests, affected
kernel tests, and RCOUNT rehearsal pass.
