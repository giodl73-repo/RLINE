# Pulse 02: BISECT fairness invariance note

## Goal

Record how group actions could support BISECT redistricting without adding a
crate, VTRACE scaffold, DCR, or implementation commitment.

## Changes

- Added `bisect-fairness-invariance.md`.
- Framed district-label permutations, bisection-tree swaps, vertex-weight
  sensitivity, edge-weight sensitivity, tie-breaking, and mode-specific
  fairness goals as invariance or equivariance claims.
- Kept initial implementation local to BISECT and future RLINE extraction gated
  by repeated consumer pressure.

## Validation

- Documentation-only change.
- Run `git diff --check` from the RLINE repo before closing the wave update.

## Status

Done.
