---
phase: 05-cli-golden-parity-benchmarks-and-coverage-gates
plan: 04
subsystem: fault-report-gate
tags: [rust, parser-core, parser-harness, fault-injection, mutation-testing]

requires:
  - phase: 05-cli-golden-parity-benchmarks-and-coverage-gates
    provides: "Plan 05-03 strict coverage gate and behavior-test baseline"
provides:
  - "Fault report schema and validator for mutation/equivalent report gates"
  - "Deterministic parser-core fault-injection regression suite"
  - "scripts/fault-report-gate.sh with cargo-mutants preference and deterministic fallback"
  - "Generated fault report output convention under .planning/generated/phase-05/fault-report/"
affects: [phase-05-plan-05, parser-core, parser-harness]

tech-stack:
  added: [parser-harness-fault-report-check]
  patterns:
    - "Fault reports classify caught, missed, timeout, and unviable cases."
    - "High-risk missed cases block unless they carry accepted non-applicable rationale."
    - "Fallback fault coverage uses public parse_replay behavior, not parser-core internals."

key-files:
  created:
    - crates/parser-harness/src/fault_report.rs
    - crates/parser-harness/tests/fault_report_gate.rs
    - crates/parser-core/tests/fault_injection_regressions.rs
    - crates/parser-harness/src/bin/fault-report-check.rs
    - scripts/fault-report-gate.sh
    - .planning/generated/phase-05/fault-report/README.md
    - .planning/phases/05-cli-golden-parity-benchmarks-and-coverage-gates/05-04-SUMMARY.md
  modified:
    - crates/parser-harness/src/lib.rs
    - README.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md

key-decisions:
  - "Deterministic fault-injection is the local equivalent gate when cargo-mutants is not installed."
  - "The fallback report includes high-risk caught cases for vehicle score clamp/direction, same-side combat semantics, null-killer bounty exclusion, aggregate source refs, and invalid JSON failures."
  - "Generated fault reports are ignored; only the output convention README is committed."

patterns-established:
  - "Fault report JSON is validated by parser-harness binary fault-report-check."
  - "scripts/fault-report-gate.sh writes .planning/generated/phase-05/fault-report/fault-report.json."
  - "Fault regression tests stay behavior-level and use existing focused fixtures through parse_replay."

requirements-completed: [TEST-08, TEST-11, TEST-12]

duration: 14min
completed: 2026-04-28
---

# Phase 5 Plan 04: Fault Report Gate Summary

**Mutation or equivalent deterministic fault-injection gate for high-risk parser-core and aggregate behavior**

## Performance

- **Duration:** 14 min
- **Started:** 2026-04-28T08:15:21Z
- **Completed:** 2026-04-28T08:29:18Z
- **Tasks:** 3
- **Files modified:** 7 plus this summary and planning/README updates

## Accomplishments

- Added `parser-harness::fault_report` with serializable `FaultOutcome`, `FaultRisk`, `FaultCase`, `FaultReport`, summary counts, and validation errors.
- Added report validation tests for caught reports, high-risk missed blockers, accepted non-applicable missed cases, missing required targets, and timeout/unviable evidence.
- Added six public-behavior fault regression tests covering teamkill penalty clamp, vehicle score category direction, same-side combat classification, null-killer bounty exclusion, aggregate source refs, and invalid JSON failure status.
- Added `scripts/fault-report-gate.sh`, which prefers `cargo mutants` when installed and otherwise runs the deterministic fallback suite, writes a fault report JSON, and validates it with `fault-report-check`.
- Added the committed convention README under ignored `.planning/generated/phase-05/fault-report/`.

## Task Commits

1. **Task 1: Add fault report schema and validator** - `fa39d8d` (feat)
2. **Task 2: Add deterministic fault regressions** - `ed0f8dc` (test)
3. **Task 3: Add fault report gate** - `ba6cef3` (feat)
4. **Quality fix: Satisfy clippy for fault regressions** - `369d4b6` (fix)

## Verification

- `cargo test -p parser-harness fault_report_gate` - passed
- `cargo test -p parser-core fault_injection_regressions` - passed
- `scripts/fault-report-gate.sh` - passed with deterministic fallback; `cargo mutants` is not installed on this machine
- `cargo test --workspace` - passed
- `cargo fmt --all -- --check` - passed
- `cargo clippy --workspace --all-targets -- -D warnings` - passed
- `git diff --check` - passed

## Deviations from Plan

### Environment Fallback

**1. [Tooling] cargo-mutants is not installed locally**

- **Found during:** Task 3 gate verification
- **Issue:** `cargo mutants --version` reports `no such command: mutants`.
- **Fix:** Used the planned deterministic fallback path and generated a `deterministic-fault-injection` report.
- **Verification:** `scripts/fault-report-gate.sh` validated a report with `total_cases=6` and `high_risk_missed=0`.

---

**Total deviations:** 1 planned fallback
**Impact on plan:** No scope expansion. Fault reporting remains local parser/harness validation and does not change parser artifact contracts, worker messages, S3 keys, canonical identity, server persistence, APIs, UI behavior, replay discovery, or benchmark claims.

## Known Stubs

None for the deterministic fallback gate. Installing `cargo-mutants` enables the preferred mutation command path, but the local release gate is complete through the equivalent fault-injection report.

## User Setup Required

Optional: install `cargo-mutants` to exercise the preferred mutation-testing branch. Without it, `scripts/fault-report-gate.sh` uses the deterministic fallback gate.

## Next Phase Readiness

Plan 05-05 can add benchmark reports and final handoff docs on top of the CLI, comparison harness, golden fixtures, coverage gate, and fault-report gate.

## Self-Check: PASSED

- Verified `FaultOutcome` includes `caught`, `missed`, `timeout`, and `unviable`.
- Verified high-risk missed cases fail validation unless accepted as non-applicable.
- Verified required targets include `parser-core::events`, `parser-core::aggregates`, and `parser-core::vehicle_score`.
- Verified `scripts/fault-report-gate.sh` is executable and writes generated output only under `.planning/generated/phase-05/fault-report/`.
- Verified no adjacent app ownership boundaries changed.

---
*Phase: 05-cli-golden-parity-benchmarks-and-coverage-gates*
*Completed: 2026-04-28*
