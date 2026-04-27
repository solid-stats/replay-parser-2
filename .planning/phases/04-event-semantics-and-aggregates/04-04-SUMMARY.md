---
phase: 04-event-semantics-and-aggregates
plan: 04
subsystem: parser-core
tags: [rust, parser-core, aggregates, vehicle-score, issue-13, source-refs]

requires:
  - phase: 04-event-semantics-and-aggregates
    provides: Source-backed combat events and aggregate projection foundation from Plans 04-02 and 04-03
provides:
  - Issue #13 vehicle score category mapping and weight matrix
  - Teamkill penalty clamp helper preserving raw and applied weights
  - Vehicle score award and penalty aggregate contributions with source refs
  - Per-player denominator input rows for players with vehicle-kill awards
  - Focused vehicle score fixture and behavior tests
affects: [parser-core, aggregate-projection, server-2-recalculation, phase-04-plan-06]

tech-stack:
  added: []
  patterns:
    - Vehicle score final cross-replay calculation remains downstream; parser-core emits auditable numerator and denominator inputs
    - Vehicle score contributions reuse AggregateContributionRef with typed VehicleScoreInputValue payloads

key-files:
  created:
    - crates/parser-core/src/vehicle_score.rs
    - crates/parser-core/tests/vehicle_score.rs
    - crates/parser-core/tests/fixtures/vehicle-score.ocap.json
  modified:
    - crates/parser-core/src/lib.rs
    - crates/parser-core/src/aggregates.rs

key-decisions:
  - "Vehicle score categories are mapped from raw observed vehicle class evidence, with unknown categories excluded from scoring contributions."
  - "Parser-core emits vehicle_score.inputs and vehicle_score.denominator_inputs but does not compute the final cross-replay vehicle score."
  - "Penalty contributions store both raw matrix weight and applied max(raw, 1.0) weight for auditability."

patterns-established:
  - "Contribution IDs use aggregate.vehicle_score.{event_id} with rule IDs aggregate.vehicle_score.award and aggregate.vehicle_score.penalty."
  - "Denominator input rows are grouped by legacy compatibility key and carry source contribution IDs."

requirements-completed: [PARS-09, AGG-08, AGG-09, AGG-10, AGG-11]

duration: 8m27s
completed: 2026-04-27
---

# Phase 04 Plan 04: Vehicle Score Summary

**Issue #13 vehicle score inputs with mapped categories, auditable weights, teamkill penalty clamp evidence, and per-replay denominator rows.**

## Performance

- **Duration:** 8m27s
- **Started:** 2026-04-27T11:43:15Z
- **Completed:** 2026-04-27T11:51:42Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added parser-core vehicle score taxonomy helpers for issue #13 category mapping, matrix weights, and penalty clamping.
- Extended aggregate derivation with `vehicle_score.inputs` and `vehicle_score.denominator_inputs` projections backed by `AggregateContributionRef` rows.
- Added behavior tests proving award/penalty contribution creation, raw/applied penalty weights, denominator grouping, and source-reference traceability.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add vehicle score category mapping and weight matrix** - `019f640` (feat)
2. **Task 2: Emit vehicle score contributions and denominator inputs** - `e33c9bc` (feat)
3. **Task 3: Add vehicle score fixture and behavior tests** - `fa8497e` (test)

## Files Created/Modified

- `crates/parser-core/src/vehicle_score.rs` - Issue #13 category mapping, weight matrix, and teamkill penalty clamp helper.
- `crates/parser-core/src/lib.rs` - Exports the vehicle score module.
- `crates/parser-core/src/aggregates.rs` - Emits vehicle score contribution refs and denominator input projections from normalized combat events.
- `crates/parser-core/tests/vehicle_score.rs` - Behavior tests for category mapping, matrix weights, clamp, projections, denominator rows, and source refs.
- `crates/parser-core/tests/fixtures/vehicle-score.ocap.json` - Focused OCAP fixture for vehicle score award and teamkill penalty behavior.

## Decisions Made

- Vehicle score contribution creation is limited to combat events already proven to be kills from vehicles.
- Unknown attacker or target score categories produce no score contribution, preserving raw evidence without silently awarding points.
- Parser-core emits denominator eligibility as input data only; `server-2` or the parity harness performs the final cross-replay divide.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Applied rustfmt formatting after Task 3 verification**
- **Found during:** Task 3 (Add vehicle score fixture and behavior tests)
- **Issue:** `cargo fmt --all -- --check` reported formatting changes in new vehicle score code and tests.
- **Fix:** Ran `cargo fmt --all` and re-ran the full plan verification.
- **Files modified:** `crates/parser-core/src/aggregates.rs`, `crates/parser-core/src/vehicle_score.rs`, `crates/parser-core/tests/vehicle_score.rs`
- **Verification:** `cargo fmt --all -- --check`, `cargo test -p parser-core vehicle_score`, `cargo test -p parser-core aggregate_projection`
- **Committed in:** `fa8497e`

---

**Total deviations:** 1 auto-fixed (1 blocking).
**Impact on plan:** Formatting only; no behavior or scope change.

## Issues Encountered

- Local `gsd-sdk query` is unsupported in this checkout, so execution state and metadata updates were handled manually as allowed by the prompt.

## Authentication Gates

None.

## Known Stubs

None found in created or modified files.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-core vehicle_score` - passed
- `cargo test -p parser-core aggregate_projection` - passed
- `cargo check -p parser-core --all-targets` - passed
- `cargo fmt --all -- --check` - passed
- `git diff --check` - passed

## Next Phase Readiness

Plan 04-05 can add commander-side and winner/outcome facts without changing vehicle score behavior. Vehicle score inputs are source-reference-backed and do not introduce canonical identity, persistence, queue/storage, API, or UI ownership into parser-core.

## Self-Check: PASSED

- Summary file exists.
- Created vehicle score module, behavior test file, and fixture exist.
- Task commits `019f640`, `e33c9bc`, and `fa8497e` exist in git history.
- No unexpected file deletions were present in task commits.

---
*Phase: 04-event-semantics-and-aggregates*
*Completed: 2026-04-27*
