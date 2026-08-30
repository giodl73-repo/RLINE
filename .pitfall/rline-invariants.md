# RLINE Invariants

## RLINE-INV-01: Application Crates Do Not Become RLINE Dependencies

**Status:** MITIGATED

**Claim:** RLINE crates remain reusable kernels and do not depend on BISECT,
CROP, ROUTE, FLETCH, RPLAN, RCOUNT, or other application crates.

**Why it matters:** Shared kernels stop being shared when one consumer's
workflow becomes part of the foundation.

**Enforcement:** Foundation spec, role review, manifest non-goals, and
workspace tests protect dependency direction.

**Evidence:** `README.md`, `docs/specs/rline-foundation.md`,
`.roles/parliament/kernel-boundary-steward.md`, and
`cargo run -p rline-cli -- manifest`.

## RLINE-INV-02: Manifest Inventory Is Deterministic

**Status:** MITIGATED

**Claim:** `rline.manifest.v1` round-trips deterministically and rejects
duplicate crates or unknown internal dependencies.

**Why it matters:** Consumers and automation need stable package-family
metadata.

**Enforcement:** `rline-core` tests protect manifest schema, dependency
validation, uniqueness, and JSON representation.

**Evidence:** `cargo test -p rline-core` and
`cargo run -p rline-cli -- manifest`.

## RLINE-INV-03: RHIST Fixtures Verify Against Preserved Sources

**Status:** MITIGATED

**Claim:** RHIST fixture source indexes match committed source bytes, package
hashes match refreshed package content, and CLI verification passes or fails by
fixture intent.

**Why it matters:** Fixture packages are evidence only when source custody and
package hashes are current.

**Enforcement:** `rhist-io` source-hash tests, RHIST CLI tests, and the fixture
hash refresh example keep source indexes and package hashes aligned.

**Evidence:** `cargo run -p rhist-io --example refresh_fixture_hashes`,
`cargo test -p rhist-cli --test verify_cli`, and `cargo test --workspace`.

## RLINE-INV-04: Deterministic Kernels Reject Invalid Numeric Or Graph Inputs

**Status:** MITIGATED

**Claim:** Graph, statistics, math, and optimization kernels reject
out-of-bounds, non-finite, overflowed, duplicate, malformed, or ambiguous inputs
instead of silently producing plausible outputs.

**Why it matters:** RLINE kernels become reusable evidence primitives for
downstream repos, so invalid inputs must fail loudly.

**Enforcement:** Workspace tests cover graph connectivity, shortest paths,
boundary metrics, numeric overflow, summaries, MCMC diagnostics, probability,
resampling, Pareto sorting, and seed derivation.

**Evidence:** `cargo test --workspace`.

## RLINE-INV-05: RCOUNT Rehearsal Remains A Release Gate

**Status:** MITIGATED

**Claim:** A foundation change that affects RCTX/RHIST contracts or kernel APIs
requires RCOUNT rehearsal before it is treated as ready.

**Why it matters:** RCOUNT is the first consumer of RCTX crosswalk and RHIST
verification behavior.

**Enforcement:** Compatibility policy names focused RCOUNT tests and expected
failure signals.

**Evidence:** `docs/compatibility.md` and
`.roles/parliament/consumer-migration-reviewer.md`.
