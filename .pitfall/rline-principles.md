# RLINE Principles

## RLINE-P-01: Kernels Stay Product-Neutral

**Decision rule:** RLINE may own reusable graph, context, statistics, math,
optimization, facility, and history kernels, but it must not absorb BISECT,
CROP, ROUTE, FLETCH, RPLAN, RCOUNT, or other product workflows.

**Rationale:** RLINE exists to fix dependency direction; product logic moving
into RLINE would recreate the application-workspace coupling it was meant to
remove.

**Test:** Kernel Boundary Steward review and workspace tests preserve generic
crate surfaces and dependency boundaries.

**Evidence:** `README.md`, `docs/specs/rline-foundation.md`,
`.roles/parliament/kernel-boundary-steward.md`, and `cargo test --workspace`.

## RLINE-P-02: Manifest Output Is A Contract

**Decision rule:** `rline.manifest.v1` must expose crate names, source paths,
internal dependencies, migration status, consumers, non-goals, and deterministic
JSON behavior honestly.

**Rationale:** Consumers should understand the kernel family without reverse
engineering every crate or inferring migration status from code layout.

**Test:** Manifest tests and `rline manifest` CLI smoke protect schema,
inventory, dependency validation, uniqueness rules, and deterministic output.

**Evidence:** `docs/compatibility.md`,
`.roles/parliament/manifest-contract-auditor.md`, and
`cargo run -p rline-cli -- manifest`.

## RLINE-P-03: Source Hashes Bind Preserved Bytes

**Decision rule:** RCTX/RHIST source indexes must match the exact preserved
source bytes in fixture packages, and package-content hashes must be refreshed
after source-index repair.

**Rationale:** Source custody is only meaningful if declared hashes verify
against the files committed in the package.

**Test:** RHIST IO and CLI tests verify source hashes and package hashes across
synthetic and real Rhode Island fixtures.

**Evidence:** `docs/fixtures/rhist/README.md`,
`crates/rhist-io/src/lib.rs`, and `cargo test --workspace`.

## RLINE-P-04: Consumer Rehearsal Gates Foundation Changes

**Decision rule:** Public API, schema, hash, deterministic-output, or verifier
changes are not ready until affected RLINE tests and the required RCOUNT
rehearsal pass or are explicitly scoped out.

**Rationale:** RCOUNT consumes RCTX crosswalks and RHIST packages; RLINE-local
green tests do not prove downstream compatibility.

**Test:** Compatibility policy names the RCOUNT rehearsal and failure signals.

**Evidence:** `docs/compatibility.md` and
`.roles/parliament/consumer-migration-reviewer.md`.

## RLINE-P-05: Candidate Algebra Is Not A Public API

**Decision rule:** The algebra-kernel wave remains planning-only until concrete
consumer pressure justifies a narrow implementation and public contract.

**Rationale:** A speculative algebra crate would widen the shared foundation
before consumers have proved the need.

**Test:** Active wave records scope, non-goals, pending consumer-pressure audit,
and no implementation commitment.

**Evidence:** `context/waves/2026-06-06-rline-algebra-kernel-candidate/WAVE.md`
and `context/waves/2026-06-06-rline-algebra-kernel-candidate/pulses/pulse-02.md`.
