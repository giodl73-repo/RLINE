# Pulse 01: Workspace foundation

## Goal

Make RLINE real as a local Rust workspace with enough contract surface to plan
the `r*` kernel extraction out of BISECT.

## Changes

- Added `rline-core` and `rline-cli`.
- Added `rline.manifest.v1` structs and validation.
- Added foundation manifest entries for `rctx-core`, `rgraph-core`,
  `rstat-core`, `ropt-core`, and `rhist-core`.
- Added `rline manifest` and `rline packages`.
- Added README, foundation spec, and wave scaffolding.

## Validation

- `cargo fmt`
- `cargo test --workspace`
- CLI smoke for `rline manifest`
- CLI smoke for `rline packages`
- `git diff --check`

## Status

Done.

