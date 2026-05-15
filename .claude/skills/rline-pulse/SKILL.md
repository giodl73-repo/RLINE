---
name: rline-pulse
description: Execute the next RLINE wave pulse with docs, implementation, validation, and commit-ready updates.
allowed-tools:
  - Read
  - Write
  - Glob
  - Grep
  - Bash
---

# RLINE Pulse

Use this skill for RLINE development pulses.

## Workflow

1. Read `context/waves/PHASES.md`.
2. Read the active wave `WAVE.md`.
3. Read the target pulse under `pulses\`.
4. Implement the smallest complete generic kernel or migration slice.
5. Keep application-specific behavior out of RLINE.
6. Update docs and wave/pulse status.
7. Run `cargo fmt`, `cargo test --workspace`, focused CLI smokes, and
   `git diff --check`.

