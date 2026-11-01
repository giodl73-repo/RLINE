# Pulse 01 - PITFALL Use-Case Integration

Date: 2026-08-29

## Scope

Second-pass PITFALL integration for RLINE, focused on kernel-boundary,
planning-candidate, and downstream-rehearsal mistakes:

- `RLINE-PF-01` - product workflow leaks into kernel surface
- `RLINE-PF-02` - candidate algebra becomes promised API
- `RLINE-PF-03` - RCOUNT rehearsal is skipped

## Changes

- Added actor, task, surface, likely mistake, consequence, owner, and retained
  test fields to the three open PITFALL entries.
- Added `crates/rline-core/tests/pitfall_policy.rs` so kernel-boundary,
  planning-only algebra, and RCOUNT rehearsal rules are test-cited.
- Tightened README language so candidate kernels remain planning-only until
  consumer pressure, API review, tests, and migration notes exist.

## Validation

Run before commit:

```powershell
C:\Users\giodl\.cargo\bin\cargo.exe fmt --check
C:\Users\giodl\.cargo\bin\cargo.exe test -p rline-core --test pitfall_policy
C:\Users\giodl\.cargo\bin\cargo.exe test --workspace
C:\Users\giodl\.cargo\bin\cargo.exe run --manifest-path C:\src\TRACKER\repos\standards-protocols\pitfall\Cargo.toml -q -p pitfall-cli -- C:\src\TRACKER\repos\tools-infra\rline --format json
python C:\src\TRACKER\repos\standards-protocols\pitfall\tools\check_pitfall.py C:\src\TRACKER\repos\tools-infra\rline
git diff --check
```
