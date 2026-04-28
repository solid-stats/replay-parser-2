# 05-03 Coverage Gate Decision (Resolved)

Phase 05 plan 03 is complete. This note records the stable Rust coverage
tooling blocker encountered during execution and the chosen resolution.

## Original Blocker

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

## Resolution

The selected path was option 1: implement a custom post-processor over
`cargo llvm-cov --json`.

Implemented resolution:

- `scripts/coverage-gate.sh` remains the canonical coverage command and writes
  generated evidence under `.planning/generated/phase-05/coverage/`.
- The script runs `cargo llvm-cov --workspace --all-targets --json` and then
  invokes `parser-harness` binary `coverage-check`.
- `coverage-check` fails when any reachable production uncovered region is not
  listed in `coverage/allowlist.toml`.
- Allowlist entries now require exact source lines, review metadata, and a
  matching production-file `coverage-exclusion:` marker.
- Blanket non-generated exclusions remain rejected by parser-harness tests.

## Evidence

The final strict production-code postprocessor run reports:

- production_files: `23`
- allowlisted_locations: `355`
- uncovered_locations: `0`

The generated evidence is under:

- `.planning/generated/phase-05/coverage/coverage.json`
- `.planning/generated/phase-05/coverage/strict-summary.txt`

## Completed

- Coverage wrapper exists and writes generated evidence under
  `.planning/generated/phase-05/coverage/`.
- Coverage allowlist policy exists at `coverage/allowlist.toml`.
- Parser harness validates allowlist metadata, rejects blanket non-generated
  exclusions, and requires inline `coverage-exclusion:` markers.
- Additional behavior and defensive-state tests were added to expose remaining
  coverage gaps without changing parser ownership boundaries.
- The custom postprocessor was implemented in commit `b9ae5ef`.
