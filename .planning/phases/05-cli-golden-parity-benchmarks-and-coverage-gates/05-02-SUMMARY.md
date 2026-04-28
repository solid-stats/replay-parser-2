---
phase: 05-cli-golden-parity-benchmarks-and-coverage-gates
plan: 02
subsystem: comparison-harness
tags: [rust, parser-harness, parser-cli, parity, mismatch-taxonomy]

requires:
  - phase: 05-cli-golden-parity-benchmarks-and-coverage-gates
    provides: "Plan 05-00 CLI command surface and Plan 05-01 compact fixtures"
provides:
  - "Reusable parser-harness comparison report schema and selected-surface comparison logic"
  - "Public replay-parser-2 compare command for saved artifacts or selected replay input"
  - "Per-finding mismatch taxonomy and downstream impact dimensions"
  - "Generated report boundary documentation for .planning/generated/phase-05/"
affects: [phase-05-plan-03, phase-05-plan-05, parser-cli, parser-harness]

tech-stack:
  added: [parser-harness]
  patterns:
    - "Old-vs-new comparison stays in parser-harness and parser-cli adapters"
    - "Reports classify selected surfaces instead of moving legacy corpus orchestration into parser-core"

key-files:
  created:
    - crates/parser-harness/Cargo.toml
    - crates/parser-harness/src/comparison.rs
    - crates/parser-harness/src/lib.rs
    - crates/parser-harness/src/report.rs
    - crates/parser-harness/tests/comparison_report.rs
    - crates/parser-cli/tests/compare_command.rs
    - .planning/phases/05-cli-golden-parity-benchmarks-and-coverage-gates/05-02-SUMMARY.md
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/parser-cli/Cargo.toml
    - crates/parser-cli/src/main.rs

key-decisions:
  - "Comparison reports use the Phase 1 mismatch taxonomy, including human_review for unexplained drift."
  - "Every finding carries parser_artifact, server_2_persistence, server_2_recalculation, and ui_visible_public_stats impact fields."
  - "replay-parser-2 compare supports selected saved artifacts and selected replay parsing; full-corpus parity remains generated harness/CI output."
  - "Generated bulky reports belong under .planning/generated/phase-05/ and annual/yearly nomination outputs stay outside ordinary v1 parity."

patterns-established:
  - "Selected surfaces include status, replay, events, legacy player/relationship projections, bounty inputs, and vehicle_score inputs."
  - "Current-vs-regenerated old-baseline drift labels force findings to human_review."

requirements-completed: [CLI-03, TEST-02, TEST-08, TEST-09, TEST-10, TEST-11]

duration: 65min
completed: 2026-04-28
---

# Phase 5 Plan 02: Selected-Input Comparison Harness Summary

**Reusable old-vs-new comparison reports plus public `replay-parser-2 compare` command for selected artifacts or selected replay files**

## Performance

- **Duration:** 65 min
- **Started:** 2026-04-28T05:35:30Z
- **Completed:** 2026-04-28T06:40:00Z
- **Tasks:** 4
- **Files modified:** 10 plus this summary

## Accomplishments

- Added `parser-harness` workspace crate with serializable report vocabulary for mismatch categories, baseline metadata, findings, summaries, and downstream impact fields.
- Implemented selected JSON artifact comparison for `status`, `replay`, `events`, `legacy.player_game_results`, `legacy.relationships`, `bounty.inputs`, and `vehicle_score.inputs`.
- Wired `replay-parser-2 compare` so users can compare saved old/new JSON artifacts or parse a selected replay and compare it against a saved old artifact.
- Added behavior-level CLI tests for compatible saved artifacts, differing saved artifacts, replay-input comparison, and missing compare input errors.
- Documented generated report boundaries in harness code: bulky reports belong under `.planning/generated/phase-05/`; reports describe adjacent app impact but do not mutate `server-2` or `web`.

## Task Commits

1. **Task 1: Add parser-harness report types** - `fe120d0` (feat)
2. **Task 2: Implement selected-input comparison logic** - `a414c42` (feat)
3. **Task 3: Wire the public compare command** - `2b6842b` (feat)
4. **Task 4: Document generated report location and boundary checks** - `91b31aa` (fix)

## Files Created/Modified

- `Cargo.toml` - Added `crates/parser-harness` to workspace members.
- `Cargo.lock` - Locked parser-cli dependency graph after adding parser-harness.
- `crates/parser-harness/Cargo.toml` - Defined harness package and dependencies.
- `crates/parser-harness/src/lib.rs` - Exported comparison and report modules.
- `crates/parser-harness/src/report.rs` - Added report schema, mismatch taxonomy, impact dimensions, generated report boundary constant, and validation.
- `crates/parser-harness/src/comparison.rs` - Added selected-surface comparison logic and drift classification.
- `crates/parser-harness/tests/comparison_report.rs` - Covered category serialization, impact dimensions, drift review, compatible findings, insufficient data, and human review drift.
- `crates/parser-cli/Cargo.toml` - Added parser-harness dependency.
- `crates/parser-cli/src/main.rs` - Replaced reserved compare stub with selected artifact/replay compare command.
- `crates/parser-cli/tests/compare_command.rs` - Covered public compare command behavior.

## Decisions Made

- Saved artifacts are compared as `serde_json::Value` so old output snippets do not need to deserialize as current `ParseArtifact`.
- Missing selected surfaces are `insufficient_data`; exact equality is `compatible`; differing comparable values require `human_review` unless future plans add a more specific accepted category.
- A baseline label containing current/regenerated/drift forces all findings to `human_review`.
- The compare command rejects ambiguous `--replay` plus `--new-artifact` invocations and requires one selected new-side input.

## Verification

- `cargo test -p parser-harness comparison_report` - passed
- `cargo check -p parser-harness --all-targets` - passed
- `cargo test -p parser-cli compare_command` - passed
- `cargo fmt --all -- --check` - passed
- `cargo clippy -p parser-harness --all-targets -- -D warnings` - passed
- `cargo clippy -p parser-cli --all-targets -- -D warnings` - passed
- `cargo test -p parser-harness` - passed
- Boundary grep over `crates/parser-harness` and `crates/parser-cli` found only report impact fields/comments/tests, not adjacent app implementation.
- `git diff --check` - passed

## Deviations from Plan

### Auto-fixed Issues

**1. [Recovery] Completed plan inline after stalled executor**
- **Found during:** Wave 3 orchestration
- **Issue:** The executor committed Task 1, then stopped returning completion signals while the tree showed partial uncommitted harness files and no summary.
- **Fix:** The orchestrator shut down the stalled executor, preserved its committed Task 1 work, completed Tasks 2-4 inline, and committed each remaining task separately.
- **Verification:** All plan verification commands listed above.

**2. [Quality] Adjusted comparison tests so the plan filter exercises new behavior**
- **Found during:** Task 2 verification
- **Issue:** Initial new test names did not include the `comparison_report` filter used by the plan command.
- **Fix:** Renamed the new tests so `cargo test -p parser-harness comparison_report` runs the selected comparison behavior tests.
- **Verification:** `cargo test -p parser-harness comparison_report` ran 6 tests.

**3. [Quality] Resolved strict clippy findings**
- **Found during:** Task 4 verification
- **Issue:** `cargo clippy -p parser-harness --all-targets -- -D warnings` flagged needless pass-by-value, const-candidate helpers, doc markdown, and `Eq` derives.
- **Fix:** Borrowed selected JSON values, made helpers const, marked `snake_case` in docs, and derived `Eq` where valid.
- **Verification:** `cargo clippy -p parser-harness --all-targets -- -D warnings` passed.

---

**Total deviations:** 3 auto-fixed
**Impact on plan:** No scope expansion. Comparison remains a local CLI/harness concern with no worker, database, replay discovery, canonical identity, public API, UI, or yearly nomination behavior.

## Issues Encountered

- The Wave 3 subagent stalled after the first task commit. Recovery kept the committed work and completed the remaining scope inline.
- Git staging required escalation because the sandbox could not create `.git/index.lock`.

## Authentication Gates

None.

## Known Stubs

None. The previous `compare command is planned in Phase 5 Plan 02` stub was removed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 05-03 can build coverage gates on top of parser-harness and the CLI compare command. Comparison report tests now provide public behavior oracles that coverage strengthening can include without exposing private parser internals.

## Self-Check: PASSED

- Verified summary, harness files, CLI compare implementation, and compare tests exist.
- Verified task commits exist in git history: `fe120d0`, `a414c42`, `2b6842b`, and `91b31aa`.
- Verified `crates/parser-cli/src/main.rs` no longer contains `compare command is planned in Phase 5 Plan 02`.
- Verified summary whitespace with `git diff --check`.

---
*Phase: 05-cli-golden-parity-benchmarks-and-coverage-gates*
*Completed: 2026-04-28*
