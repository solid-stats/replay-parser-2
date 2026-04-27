---
phase: 03-deterministic-parser-core
plan: 04
subsystem: parser-core
tags: [rust, serde-json, parser-core, diagnostics, deterministic-output]

requires:
  - phase: 03-deterministic-parser-core
    provides: "Plan 03-02 RawReplay adapter, metadata normalization, and initial drift diagnostics"
  - phase: 03-deterministic-parser-core
    provides: "Plan 03-03 observed entity normalization and stable entity ordering"
provides:
  - "Capped diagnostic accumulator with omitted diagnostic summary"
  - "ParseStatus escalation from data-loss diagnostics to partial artifacts"
  - "Focused schema drift status and diagnostic cap tests"
  - "Repeated serde_json::to_string determinism tests for parser-core artifacts"
affects: [03-deterministic-parser-core, parser-core, parser-contract-consumers]

tech-stack:
  added: []
  patterns:
    - "DiagnosticAccumulator::finish returns both emitted diagnostics and successful-parse status"
    - "DiagnosticImpact classifies parser diagnostics as info, non-loss warning, or data loss"
    - "Parser-core determinism is tested through public ParseArtifact serialization"

key-files:
  created:
    - .planning/phases/03-deterministic-parser-core/03-04-SUMMARY.md
    - crates/parser-core/tests/schema_drift_status.rs
    - crates/parser-core/tests/deterministic_output.rs
    - crates/parser-core/tests/fixtures/diagnostic-cap.ocap.json
  modified:
    - crates/parser-core/src/artifact.rs
    - crates/parser-core/src/diagnostics.rs
    - crates/parser-core/src/entities.rs
    - crates/parser-core/src/metadata.rs

key-decisions:
  - "Data-loss diagnostics now drive successful root parses to ParseStatus::Partial through DiagnosticAccumulator::finish."
  - "Diagnostic caps keep the configured number of concrete diagnostics and append one diagnostic.limit_exceeded summary with omitted count."
  - "Parser-core determinism remains adapter-free: produced_at stays None and repeated serde_json::to_string output is byte-identical for the same ParserInput."

patterns-established:
  - "Normalization modules pass DiagnosticImpact::DataLoss for drift-caused unknowns, dropped entity rows, skipped entity sections, and lost audit evidence."
  - "Diagnostic cap behavior is verified with a fixture containing repeated malformed entity rows."
  - "Deterministic output tests assert public artifact behavior rather than private sort helpers."

requirements-completed: [OUT-08, PARS-01, PARS-02, PARS-03, PARS-04, PARS-05]

duration: 7m
completed: 2026-04-27
---

# Phase 3 Plan 04: Schema-Drift Diagnostics and Deterministic Output Summary

**Capped diagnostics with partial-status escalation and byte-identical parser-core serialization tests**

## Performance

- **Duration:** 7m
- **Started:** 2026-04-27T05:35:15Z
- **Completed:** 2026-04-27T05:41:43Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Replaced the boolean diagnostic flag with `DiagnosticImpact`, `DiagnosticAccumulator`, and `DiagnosticReport`.
- Added capped diagnostic behavior that appends `diagnostic.limit_exceeded` with omitted-count context.
- Routed successful artifact status through the diagnostic report so data-loss drift yields `ParseStatus::Partial`.
- Added schema-drift status, diagnostic cap, and deterministic serialization behavior tests.

## Task Commits

1. **Task 1: Implement diagnostic accumulator and status escalation policy** - `310a449` (feat)
2. **Task 2: Add schema-drift status and diagnostic cap tests** - `56b1828` (test)
3. **Task 3: Prove deterministic serialized output for Phase 3 artifact sections** - `76cdb84` (test)
4. **Auto-fix: satisfy diagnostic accumulator clippy gate** - `3d78d07` (fix)

## Files Created/Modified

- `crates/parser-core/src/diagnostics.rs` - Diagnostic impact enum, capped accumulator, omitted-summary diagnostic, and report/status policy.
- `crates/parser-core/src/artifact.rs` - Uses `DiagnosticAccumulator::finish` for final diagnostics and successful parse status.
- `crates/parser-core/src/metadata.rs` - Marks metadata schema drift diagnostics as data loss.
- `crates/parser-core/src/entities.rs` - Marks dropped rows, skipped sections, drift unknowns, and audit-loss diagnostics as data loss.
- `crates/parser-core/tests/schema_drift_status.rs` - Covers partial/success status policy and diagnostic cap behavior.
- `crates/parser-core/tests/deterministic_output.rs` - Covers byte-identical serialization, stable entity order, and `produced_at: None`.
- `crates/parser-core/tests/fixtures/diagnostic-cap.ocap.json` - Focused fixture with five malformed entity rows lacking usable IDs.

## Decisions Made

- Kept diagnostic capping inside parser-core rather than contract types, because cap behavior is parser policy and does not require artifact shape changes.
- Preserved insertion order for emitted diagnostics; current parser-core diagnostic production is deterministic and the summary is appended last.
- Did not add adjacent app changes: this plan stays in local parser-core behavior and tests, with no queue, S3, API, database, or canonical identity surface.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed strict clippy blocker in diagnostic finish logic**
- **Found during:** Final plan verification after Task 3
- **Issue:** Clippy rejected nested `if` logic in `DiagnosticAccumulator::finish` under workspace `-D warnings`.
- **Fix:** Collapsed the condition while preserving omitted-summary behavior.
- **Files modified:** `crates/parser-core/src/diagnostics.rs`
- **Verification:** `cargo clippy --workspace --all-targets -- -D warnings` passed.
- **Committed in:** `3d78d07`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** No scope expansion. The fix was required by the existing strict Rust quality gate.

## Issues Encountered

- The local `node_modules/@gsd-build/sdk` CLI path was absent, and `gsd-sdk query` on PATH did not support query mode. Per the user-provided commit protocol, this execution used normal git commits and did not update `STATE.md` or `ROADMAP.md`.

## Known Stubs

- `crates/parser-core/src/artifact.rs:71` still emits empty `events` and default empty aggregates for successful artifacts. This is intentional until Phase 4 event semantics and aggregate projection.
- `crates/parser-core/src/artifact.rs:91-94` keeps failed artifacts at empty diagnostics/entities/events, preserving the structured failure shell from Plan 03-01.
- `crates/parser-core/src/entities.rs:122` still emits `compatibility_hints: Vec::new()` for each entity. This is intentional until Plan 03-05 connected-player backfill and duplicate-slot hints.

## Threat Flags

None. This plan added no network endpoints, auth paths, file access patterns, queue/S3/database behavior, canonical identity fields, or UI-visible API behavior.

## Verification

| Command | Result |
| --- | --- |
| `rg -n "DiagnosticAccumulator\|DiagnosticImpact\|diagnostic\\.limit_exceeded\|summarized_repeated_diagnostics" crates/parser-core/src` | PASS |
| `cargo check -p parser-core --all-targets` | PASS |
| `cargo test -p parser-core schema_drift_status` | PASS - 4 targeted tests passed |
| `cargo test -p parser-core deterministic_output` | PASS - 3 targeted tests passed |
| `cargo test -p parser-core` | PASS - 22 parser-core tests passed plus doc-tests |
| `cargo test --workspace` | PASS - workspace tests and doc-tests passed |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03-05 can build connected-player backfill and duplicate-slot compatibility hints on top of the capped diagnostic/status policy and deterministic artifact serialization tests.

## Self-Check: PASSED

- Found `.planning/phases/03-deterministic-parser-core/03-04-SUMMARY.md`.
- Found `crates/parser-core/tests/schema_drift_status.rs`.
- Found `crates/parser-core/tests/deterministic_output.rs`.
- Found `crates/parser-core/tests/fixtures/diagnostic-cap.ocap.json`.
- Found task commits `310a449`, `56b1828`, `76cdb84`, and `3d78d07` in git history.

---
*Phase: 03-deterministic-parser-core*
*Completed: 2026-04-27*
