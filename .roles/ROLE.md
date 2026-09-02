# RLINE Role Index

RLINE is the neutral Rust workspace for reusable `r*` kernels and contracts.
Use these roles when changing package boundaries, manifests, extracted crates,
consumer migration plans, or validation surfaces.

## Parliament

| File | Role | Primary tension |
|---|---|---|
| `parliament/kernel-boundary-steward.md` | Kernel Boundary Steward | Product-neutral kernels vs. application workflow leakage |
| `parliament/manifest-contract-auditor.md` | Manifest Contract Auditor | Stable package-family contracts vs. implementation churn |
| `parliament/consumer-migration-reviewer.md` | Consumer Migration Reviewer | Reusable extraction vs. breaking sibling repos |

## Productive tensions

| Pulls | Against | Because |
|---|---|---|
| Kernel Boundary Steward | Manifest Contract Auditor | A minimal kernel can conflict with exposing enough metadata for deterministic manifests. |
| Manifest Contract Auditor | Consumer Migration Reviewer | A cleaner manifest contract can invalidate existing files or change upgrade interpretation. |
| Consumer Migration Reviewer | Kernel Boundary Steward | Compatibility shims reduce migration risk but can permanently enlarge the kernel surface. |

Kernel leakage and non-deterministic interpretation block first. Resolve migration disputes with
old/new manifest fixtures and an explicit removal condition for every compatibility shim. If the
evidence cannot satisfy both sides, preserve the existing contract and require an owner decision
before widening the boundary.

## Review order

1. Use Kernel Boundary Steward for any crate extraction or dependency change.
2. Use Manifest Contract Auditor for `rline.manifest.v1`, package metadata, and CLI output.
3. Use Consumer Migration Reviewer before changing public APIs used by BISECT, CROP, ROUTE, RPLAN, RCOUNT, or future consumers.

## PITFALL gate routing

Invoke the Kernel Boundary Steward and Consumer Migration Reviewer before
BISECT, CROP, ROUTE, FLETCH, RPLAN, RCOUNT, or another consumer workflow term
becomes a public API, dependency, manifest field, or fixture assumption in the
shared kernel surface.

Invoke the Kernel Boundary Steward and Consumer Migration Reviewer before the
planning-only algebra candidate is described as a `ralg-core` public contract,
`ralgebra-core` public contract, implementation commitment, scheduled public
crate, dependency adoption target, or accepted API promise.

Invoke the Consumer Migration Reviewer and RCOUNT owner before public API,
serialized schema, hash-input, verifier-behavior, deterministic-output,
RCTX-crosswalk, or RHIST-package changes are promoted as RCOUNT ready, release
ready, or portfolio-snapshot ready.
