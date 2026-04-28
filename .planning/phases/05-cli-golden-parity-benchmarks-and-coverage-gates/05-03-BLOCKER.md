# 05-03 Coverage Gate Blocker

Phase 05 plan 03 is not complete.

## Blocker

`scripts/coverage-gate.sh` can enforce native `cargo llvm-cov` 100% line,
function, and region thresholds, but stable Rust does not provide a usable
line-level production-code exclusion mechanism for the planned allowlist model.

`#[coverage(off)]` would be the direct Rust-side mechanism, but it is still an
experimental compiler feature on the installed stable toolchain:

- `rustc 1.95.0 (59807616e 2026-04-14)`
- `#[coverage(off)]` fails with `E0658`

`cargo llvm-cov` supports `--ignore-filename-regex`, but that excludes whole
files, which conflicts with the plan requirement that blanket module/file
exclusions are not acceptable.

## Evidence

The strict production-code run with `cfg(coverage)` enabled, which excludes
source `#[cfg(test)]` modules from the denominator, currently reports:

- lines: `3722/3883`
- functions: `385/399`
- regions: `4760/5049`

The generated evidence is under:

- `.planning/generated/phase-05/coverage/coverage.json`
- `.planning/generated/phase-05/coverage/strict-missing-lines.txt`

## Completed So Far

- Coverage wrapper exists and writes generated evidence under
  `.planning/generated/phase-05/coverage/`.
- Coverage allowlist policy exists at `coverage/allowlist.toml`.
- Parser harness validates allowlist metadata, rejects blanket non-generated
  exclusions, and requires inline `coverage-exclusion:` markers.
- Additional behavior and defensive-state tests were added to expose remaining
  coverage gaps without changing parser ownership boundaries.

## Required Decision

Choose one path before 05-03 can be completed:

1. Implement a custom post-processor that reads `cargo llvm-cov --json`, applies
   narrow allowlist entries to uncovered lines/regions, and fails on any
   uncovered production code not allowlisted.
2. Keep native `cargo llvm-cov` thresholds and remove/refactor every remaining
   defensive branch until line/function/region coverage reaches 100% with an
   empty allowlist.
3. Relax the gate to native line/function coverage plus generated missing-region
   evidence, explicitly changing D-09/D-10 in the Phase 05 context.

