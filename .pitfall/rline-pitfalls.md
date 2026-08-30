# RLINE Pitfalls

## RLINE-PF-01: Product Workflow Leaks Into Kernel Surface

**Status:** OPEN

**Pattern:** BISECT, CROP, ROUTE, FLETCH, RPLAN, RCOUNT, or another consumer's
workflow terms become RLINE public APIs, dependencies, or fixture assumptions.

**Actor:** Kernel maintainer, extraction author, downstream adopter, portfolio
dependency planner, or future agent.

**Task:** Extract a shared crate, add a public API, write a manifest field, or
prepare a consumer migration.

**Surface:** Shared kernel extraction, public APIs, manifests, and future
consumer migrations.

**Likely mistake:** Treat a useful BISECT, CROP, ROUTE, FLETCH, RPLAN, or
RCOUNT workflow concept as product-neutral because more than one repo mentions
it.

**Consequence:** RLINE becomes an application workflow repo in disguise and
forces unrelated consumers to inherit product-specific policy.

**Owner:** RLINE maintainers, with Kernel Boundary Steward review before
widening public kernel surfaces.

**Domain:** Shared kernel extraction, public APIs, manifests, and future
consumer migrations.

**Detection difficulty:** RLINE was extracted from consumer pressure, so useful
shared concepts can look product-neutral before role review names the boundary.

**Structural solution:** Require Kernel Boundary Steward review and consumer
migration notes before widening public kernel surfaces.

**Evidence:** `.roles/parliament/kernel-boundary-steward.md`,
`docs/specs/rline-foundation.md`, and `docs/compatibility.md`.

**Test:** `cargo test -p rline-core --test pitfall_policy`.

## RLINE-PF-02: Candidate Algebra Becomes Promised API

**Status:** OPEN

**Pattern:** The planning-only algebra-kernel wave is treated as an accepted
`ralg-core` or `ralgebra-core` public contract before consumer pressure,
minimal API review, tests, and migration notes exist.

**Actor:** Portfolio planner, README editor, dependency adopter, API sketch
author, downstream repo, or future agent.

**Task:** Decide whether to depend on, implement, document, or schedule the
candidate algebra kernel.

**Surface:** Algebra candidate wave, shared math surface, downstream planning,
and portfolio dependency adoption.

**Likely mistake:** Read the candidate consumer list and plausible algebra uses
as an implementation commitment or public crate promise.

**Consequence:** Repos plan against a nonexistent or unreviewed contract, and
RLINE grows a speculative algebra surface without consumer proof.

**Owner:** RLINE maintainers, with Kernel Boundary Steward and Consumer
Migration Reviewer approval before any implementation wave opens.

**Domain:** Algebra candidate wave, shared math surface, downstream planning,
and portfolio dependency adoption.

**Detection difficulty:** The wave names plausible consumers and use cases, so
portfolio planning may overread it as scheduled implementation.

**Structural solution:** Keep pulses 03 and 04 pending until concrete consumer
pressure exists, then open a scoped implementation wave.

**Evidence:** `context/waves/2026-06-06-rline-algebra-kernel-candidate/WAVE.md`
and
`context/waves/2026-06-06-rline-algebra-kernel-candidate/bisect-fairness-invariance.md`.

**Test:** `cargo test -p rline-core --test pitfall_policy`.

## RLINE-PF-03: RCOUNT Rehearsal Is Skipped

**Status:** OPEN

**Pattern:** A public API, schema, hash, verifier, or deterministic-output
change is accepted after RLINE-local tests only, without the required RCOUNT
downstream rehearsal.

**Actor:** RLINE maintainer, RCOUNT adopter, manifest editor, verifier author,
portfolio snapshotter, compatibility reviewer, or future agent.

**Task:** Change public APIs, serialized schemas, hash inputs, verifier
behavior, or deterministic outputs and decide whether the change is ready to
publish or snapshot.

**Surface:** RCTX crosswalks, RHIST packages, district aggregation, count
lineage, and portfolio dependency updates.

**Likely mistake:** Stop after RLINE workspace tests because manifest,
fixture-hash, and local verifier checks are green.

**Consequence:** RCOUNT breaks or silently changes aggregation, RHIST lineage,
or package verification behavior after the TRACKER pointer advances.

**Owner:** RLINE maintainers, with RCOUNT rehearsal evidence before affected
foundation changes are promoted.

**Domain:** RCTX crosswalks, RHIST packages, district aggregation, count
lineage, and portfolio dependency updates.

**Detection difficulty:** RLINE's workspace tests are broad and fast, so
consumer breakage can look unlikely until RCOUNT is tested.

**Structural solution:** Treat the RCOUNT rehearsal in `docs/compatibility.md`
as a release gate for affected foundation changes.

**Evidence:** `docs/compatibility.md` and
`.roles/parliament/consumer-migration-reviewer.md`.

**Test:** `cargo test -p rline-core --test pitfall_policy`.

## RLINE-PF-04: RHIST Source Index Drifts From Preserved Sources

**Status:** MITIGATED

**Pattern:** RHIST fixture `sources/source-index.json` hashes no longer match
the committed source bytes, causing CLI verification and source-custody tests to
fail before package semantics can be evaluated.

**Domain:** RHIST fixtures, source custody, package hashes, verifier CLI, and
research-paper replay.

**Detection difficulty:** The fixture directories still look complete, and the
failure appears only when raw source hashes are recomputed.

**Structural solution:** Refresh source indexes to current preserved bytes and
run `rhist-io` fixture hash refresh so manifest and proof hashes align.

**Evidence:** `cargo run -p rhist-io --example refresh_fixture_hashes`,
`cargo test -p rhist-cli --test verify_cli`, and `cargo test --workspace`.

## RLINE-PF-05: Manifest Status Becomes Migration Completion

**Status:** MITIGATED

**Pattern:** The `extracted` crate status in `rline.manifest.v1` is read as
downstream migration completion for BISECT, CROP, RPLAN, RCOUNT, ROUTE, or
FLETCH.

**Domain:** Portfolio scoring, dependency adoption, generated manifests, and
consumer migration planning.

**Detection difficulty:** The manifest is intentionally authoritative about the
kernel family, so status words can be mistaken for consumer adoption state.

**Structural solution:** Keep manifest fields scoped to RLINE extraction status
and use compatibility/wave docs for downstream migration readiness.

**Evidence:** `README.md`, `docs/specs/rline-foundation.md`,
`docs/compatibility.md`, and `cargo run -p rline-cli -- manifest`.
