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

## Review order

1. Use Kernel Boundary Steward for any crate extraction or dependency change.
2. Use Manifest Contract Auditor for `rline.manifest.v1`, package metadata, and CLI output.
3. Use Consumer Migration Reviewer before changing public APIs used by BISECT, CROP, ROUTE, RPLAN, RCOUNT, or future consumers.
