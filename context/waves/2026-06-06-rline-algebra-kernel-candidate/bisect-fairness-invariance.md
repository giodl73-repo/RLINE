# BISECT Fairness Invariance Through Group Actions

## Purpose

This note records one concrete consumer-pressure path for a future RLINE algebra
kernel: BISECT can use group actions to make redistricting plan comparison,
search, weighting, and fairness validation more canonical and more auditable.

The goal is not to replace BISECT's recursive bisection, edge weighting, or
METIS-style partitioning. The goal is to add a reusable invariance layer around
the algorithm:

```text
irrelevant transformation -> same canonical output
authorized transformation -> possible output change, logged by mode
```

## Core Idea

A group action applies a reversible transformation to an object while preserving
the structure that should not matter.

For BISECT, the most useful transformations are:

- district-label permutations,
- tract or unit-id relabeling,
- left/right swaps in a bisection tree,
- graph isomorphisms on synthetic fixtures,
- geometry rotations/reflections for synthetic shape fixtures,
- mode-specific swaps or scrambles of forbidden input fields.

Two properties matter:

```text
invariant:   score(g * plan) = score(plan)
equivariant: build(g * input) = g * build(input)
```

Invariance says a metric or score is unchanged by irrelevant transformations.
Equivariance says the builder follows the input transformation exactly instead
of injecting order or naming bias.

## District-Label Permutation Group

District numbers are names. A plan should not become meaningfully different
because district `1` and district `2` are swapped.

For `k` districts, the relevant group is `S_k`, the permutations of district
labels. BISECT can use this for canonical plan identity:

```text
plan
-> all relevant district-label permutations
-> canonical representative
-> stable hash
```

Uses:

- `label-compare` should compare geography, not arbitrary district numbers.
- Ensemble deduplication should count unique plans modulo relabeling.
- Cache keys should avoid recomputing equivalent assignments.
- Metrics should be tested as invariant under district renumbering.

For large `k`, BISECT does not need to enumerate all of `S_k`. It can choose a
stable canonical order for districts, for example by sorted unit-set hash,
population tuple, centroid, or another declared deterministic key.

## Bisection-Tree Swap Group

Every binary split has a left/right ambiguity:

```text
parent -> child A / child B
```

Swapping child A and child B does not change the substantive cut. A recursive
bisection tree therefore has one two-element swap group per internal node:

```text
C2 x C2 x C2 ...
```

BISECT can canonicalize each split:

```text
left_hash = hash(sorted(left_units))
right_hash = hash(sorted(right_units))
split_hash = hash(sorted([left_hash, right_hash]))
```

Then the full bisection tree hash can be stable under left/right swaps. That
improves replay, round-map lineage, evidence manifests, and semantic comparison
between equivalent runs.

## Vertex-Weight Fairness

Vertex weights are where BISECT represents population and, in authorized modes,
other legally relevant quantities.

Default geographic mode should depend on:

- graph topology,
- geometry-derived edge weights,
- population vertex weights,
- chamber and district count.

Default geographic mode should not depend on:

- party fields,
- race or ethnicity fields,
- input row order,
- district labels,
- unit lexical ordering except as a declared final tie-breaker.

This can be expressed as a group-action test:

```text
canonical(build(input)) = canonical(build(transform(input)))
```

for transformations that permute unit IDs, shuffle input rows, or scramble
fields forbidden in the current mode.

In VRA mode, protected-class demographic fields may be authorized inputs, but
partisan fields should remain forbidden unless a separate mode explicitly
authorizes them.

## Edge-Weight Fairness

Edge weights declare what geography means to the cut objective.

Default edge-weighting should be invariant under unit relabeling:

```text
w(u, v) = w(pi(u), pi(v))
```

and the cut should be equivariant:

```text
cut(pi * graph, pi * weights) = pi * cut(graph, weights)
```

Mode-specific edge-weight sensitivity should be explicit:

- geographic mode may use shared boundary length and geometric continuity;
- county-sticky mode may use county-boundary relations;
- VRA mode may use demographic alignment if legally and methodologically
  authorized;
- geographic and VRA modes should not use partisan fields unless the selected
  mode says so.

This gives BISECT an executable way to prove that edge weighting follows the
declared fairness model rather than accidental input-order or label behavior.

## Fairness Goals As Symmetry Claims

Each fairness goal can be written as an allowed or forbidden sensitivity.

| Fairness Goal | Symmetry Claim |
|---|---|
| Procedural neutrality | Invariant under district renumbering and input-order shuffles. |
| Unit-name neutrality | Equivariant under tract or unit-id relabeling. |
| Geographic neutrality | Invariant under party-label swaps in geographic mode. |
| Population equality | Invariant under district labels, sensitive to population changes. |
| Compactness | Invariant under translation, rotation, and reflection of geometry. |
| VRA mode boundary | Sensitive to authorized demographic changes, invariant under party fields. |
| County-sticky mode boundary | Sensitive to county-boundary relations, invariant under unrelated metadata. |

This is a stronger statement than "the algorithm did not include partisan data."
It says the implementation was tested against transformations that would expose
forbidden influence.

## Tie-Breaking

Ties are where hidden unfairness often enters.

If two cuts have equal score and the implementation chooses the first candidate
encountered, then input order may influence the output. A group-action framing
turns that into a testable symmetry issue:

```text
tie_break(g * candidates) = g * tie_break(candidates)
```

Candidate tie-break policies:

- canonical cut hash,
- stable sorted unit-set hash,
- minimum canonical split signature,
- seeded randomness with the seed recorded as evidence,
- ensemble exploration of all tied choices when feasible.

If the tie-break cannot be made invariant, BISECT should record the arbitrary
choice as evidence rather than pretending the choice was semantic.

## Invariance Harness

A practical harness can start with three primitives:

```text
canonical_plan(assignments) -> CanonicalPlan
transform_input(input, transformation) -> Input
assert_equivariant(mode, input, transformation)
```

First transformations:

- shuffle unit order,
- permute unit IDs,
- swap district labels.

Mode-specific transformations:

- swap or scramble party fields,
- scramble demographic fields,
- rotate or reflect synthetic geometry fixtures,
- swap left/right children in bisection tree fixtures.

Expected behavior should be declared per mode:

| Transformation | Geographic Mode | VRA Mode | County-Sticky Mode |
|---|---|---|---|
| Unit-id permutation | Same canonical plan | Same canonical plan | Same canonical plan |
| Input row shuffle | Same canonical plan | Same canonical plan | Same canonical plan |
| District renumbering | Same metrics | Same metrics | Same metrics |
| Bisection child swap | Same canonical tree | Same canonical tree | Same canonical tree |
| Party-field scramble | No output change | No output change | No output change |
| Demographic-field scramble | No output change | May change if authorized | No output change unless authorized |
| County-boundary relation change | No change unless unused | No change unless unused | May change |

## RLINE Extraction Boundary

The first implementation should remain BISECT-local:

```text
canonical_plan_hash(assignments)
canonical_split_hash(left_units, right_units)
canonical_bisection_tree_hash(tree)
assert_mode_invariance(mode, fixture, transformation)
```

RLINE extraction is justified only after more than one repo needs the generic
pieces. Likely future RLINE primitives would be:

- `Permutation`,
- `GroupAction<State>`,
- canonical representative helpers,
- finite transformation fixtures,
- law-check helpers for invariance and equivariance tests.

BISECT should keep election law, fairness policy, VRA meaning, and mode-specific
interpretation local.
