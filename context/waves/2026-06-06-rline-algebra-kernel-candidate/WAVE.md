# Wave: RLINE Algebra Kernel Candidate

## Goal

Evaluate whether RLINE should grow a small product-neutral algebra kernel for
operations that multiple portfolio repos may need beside the existing graph,
statistics, math, optimization, context, facility, and history crates.

## Thesis

RLINE already centralizes reusable graph and numeric kernels. Several repo
families may eventually need algebraic operations that make graph, history,
state, and scoring behavior composable without each product inventing its own
incompatible primitives.

The candidate direction is a narrow `ralg-core` or `ralgebra-core` crate only
after real consumer pressure appears.

## Candidate Scope

- Permutation groups for relabeling, canonicalization, symmetry checks, and
  board or unit-state transforms.
- Monoids and semigroups for composable state transitions, command histories,
  traces, and lineage operations.
- Semirings for graph path scoring, reliability scoring, provenance weights,
  and alternate shortest-path-style kernels.
- Small finite-group fixtures and law-check helpers for deterministic tests.
- Linear representation bridges where existing `rmath-core` matrices can
  represent transformations.

## Candidate Consumers

- Games repos for board symmetries and canonical state reduction.
- RPLAN, RCOUNT, and BISECT for relabeling invariance and canonical fixtures.
- CROP, CANON, LEXIS, and RHIST for context composition, identity merges,
  lineage transforms, and history operations.
- TERRAIN, ZONES, and route-like repos for semiring path costs and transform
  invariants.
- SCENE and design-lab repos for geometric transformation vocabulary.

## Non-Goals

- No broad symbolic algebra system.
- No product-specific math, legal, game, election, route, or language policy.
- No dependency from Enterprise-only LATTICE into RLINE.
- No public API commitment until at least one consumer proves a concrete need.
- No VTRACE or DCR scaffold for this wave unless the candidate becomes planned
  implementation work.

## Pulse Table

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | Candidate framing | done | Recorded algebra kernel direction, scope, consumers, and non-goals without opening implementation or VTRACE work. |
| 02 | BISECT fairness invariance note | done | Captured how group actions could support BISECT canonicalization, edge/vertex weighting audits, and fairness invariance tests. |
| 03 | Consumer pressure audit | pending | Inspect additional repo-local uses that would justify extracting a real algebra kernel. |
| 04 | Minimal API sketch | pending | Draft a narrow trait/type sketch only if at least one consumer has concrete pressure. |

## Success Criteria

- RLINE has a durable wave record for the algebra-kernel idea.
- The candidate stays narrower than a general math library.
- Future implementation is gated by consumer pressure, not speculation.
- Existing RLINE crates remain unchanged by this planning wave.
