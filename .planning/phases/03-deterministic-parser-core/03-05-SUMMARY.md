---
phase: 03-deterministic-parser-core
plan: 05
subsystem: parser-core
tags: [rust, serde-json, parser-core, observed-identity, legacy-compatibility]

requires:
  - phase: 03-deterministic-parser-core
    provides: "Plan 03-04 diagnostic status policy and deterministic parser-core output"
provides:
  - "Connected-player event backfill as inferred observed entity facts with rule/source provenance"
  - "Duplicate-slot same-name compatibility hints without merging normalized entities"
  - "Focused legacy entity compatibility fixtures and behavior tests"
  - "README parser-core handoff and final Phase 3 quality-gate evidence"
affects: [03-deterministic-parser-core, 04-event-semantics-and-aggregates, parser-core]

tech-stack:
  added: []
  patterns:
    - "Raw OCAP connected-event tuple handling stays in raw.rs"
    - "Legacy identity compatibility is represented as FieldPresence::Inferred plus EntityCompatibilityHint"
    - "Duplicate same-name compatibility emits hints only; data-loss diagnostics are reserved for conflicts"

key-files:
  created:
    - .planning/phases/03-deterministic-parser-core/03-05-SUMMARY.md
    - crates/parser-core/tests/legacy_entity_compatibility.rs
    - crates/parser-core/tests/fixtures/connected-backfill.ocap.json
    - crates/parser-core/tests/fixtures/duplicate-slot-same-name.ocap.json
  modified:
    - crates/parser-core/src/raw.rs
    - crates/parser-core/src/entities.rs
    - README.md

key-decisions:
  - "Connected-player backfill uses inferred observed names and source refs, not canonical identity."
  - "Duplicate-slot same-name behavior is preserved as compatibility hints for later aggregate projection and never collapses ObservedEntity records."
  - "Normal compatibility hints do not force partial status; only conflicting present sides emit data-loss diagnostics."

patterns-established:
  - "Legacy compatibility tests assert public ParseArtifact behavior through parse_replay."
  - "Compatibility source refs include both event/name evidence and original entity JSON paths."

requirements-completed: [OUT-08, PARS-01, PARS-02, PARS-03, PARS-04, PARS-05, PARS-06, PARS-07]

duration: 12m
completed: 2026-04-27
---

# Phase 3 Plan 05: Connected-Player Backfill and Duplicate-Slot Compatibility Summary

**Auditable connected-player inferred identity and duplicate same-name compatibility hints without entity merging**

## Performance

- **Duration:** 12m
- **Started:** 2026-04-27T05:45:14Z
- **Completed:** 2026-04-27T05:56:30Z
- **Tasks:** 4
- **Files modified:** 7

## Accomplishments

- Added raw connected-event observations for `[frame, "connected", name, entity_id]` tuples.
- Implemented connected-player backfill using `FieldPresence::Inferred`, rule ID `entity.connected_player_backfill`, and event/entity source refs.
- Added duplicate-slot same-name compatibility hints with sorted related entity IDs while preserving every `ObservedEntity`.
- Added focused fixtures/tests for backfilled names, vehicle skip behavior, source refs, no-merge hints, and success status without conflicts.
- Updated README to describe the completed Phase 3 parser-core crate and validation command without claiming CLI, worker, parity, aggregate, or benchmark support exists.

## Task Commits

1. **Task 1: Add raw connected-event helpers and connected-player backfill** - `2708ed5` (feat)
2. **Task 2: Add duplicate-slot same-name compatibility hints without merging entities** - `3a01fff` (feat)
3. **Task 3: Add legacy compatibility fixtures and behavior tests** - `fbcdf75` (test)
4. **Task 4: Update README and run final Phase 3 quality gates** - `2d87924` (docs)

## Files Created/Modified

- `crates/parser-core/src/raw.rs` - Adds connected-event tuple observations with event index, frame, name, entity ID, and JSON path.
- `crates/parser-core/src/entities.rs` - Applies connected-player inferred-name backfill, duplicate-slot hints, and duplicate side-conflict diagnostics.
- `crates/parser-core/tests/legacy_entity_compatibility.rs` - Covers public parser artifact behavior for legacy entity compatibility hooks.
- `crates/parser-core/tests/fixtures/connected-backfill.ocap.json` - Focused connected-event backfill fixture.
- `crates/parser-core/tests/fixtures/duplicate-slot-same-name.ocap.json` - Focused duplicate same-name slot fixture.
- `README.md` - Documents Phase 3 parser-core completion and `cargo test -p parser-core`.

## Decisions Made

- Kept compatibility data local to observed entity facts and hints; no combat events, aggregate formulas, canonical identity, game-type filters, yearly nomination behavior, CLI, worker, S3/RabbitMQ, or database behavior were added.
- Treated empty entity names as eligible for legacy connected-player backfill while preserving directly observed non-empty names.
- Classified duplicate-slot same-name side conflicts as data loss, because later aggregate projection cannot safely infer one side without losing source disagreement.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Formatted new compatibility code for strict workspace gates**
- **Found during:** Task 3 verification hardening
- **Issue:** `cargo fmt --all -- --check` reported formatting drift in the new legacy compatibility test and one `entities.rs` line from the compatibility helpers.
- **Fix:** Ran `cargo fmt --all` and re-ran the targeted legacy compatibility test.
- **Files modified:** `crates/parser-core/src/entities.rs`, `crates/parser-core/tests/legacy_entity_compatibility.rs`
- **Verification:** `cargo fmt --all -- --check` and `cargo test -p parser-core legacy_entity_compatibility` passed.
- **Committed in:** `fbcdf75`

**2. [Rule 3 - Blocking] Fixed strict clippy blockers in compatibility helpers**
- **Found during:** Task 4 final quality gates
- **Issue:** Workspace clippy rejected pass-by-reference `RawReplay` helper arguments, nested `if` control flow, and non-const pure helpers.
- **Fix:** Passed `RawReplay` by value where copy-sized, collapsed the nested inference branch, and marked pure helpers as `const fn`.
- **Files modified:** `crates/parser-core/src/raw.rs`, `crates/parser-core/src/entities.rs`
- **Verification:** `cargo clippy --workspace --all-targets -- -D warnings` passed.
- **Committed in:** `2d87924`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** No scope expansion. Both fixes were required to satisfy the plan's strict Rust quality gates.

## Issues Encountered

- The plan's legacy reference path under `/home/afgan0r/Projects/SolidGames/replays-parser` was absent in this environment. The equivalent old parser source was read from `/home/alexandr/Projects/SolidGames/sg-replay-parser`, matching the previous Phase 3 execution summaries.
- The local `node_modules/@gsd-build/sdk` CLI path was absent, and `gsd-sdk query` on PATH did not support query mode. Per the user-provided commit protocol, execution used normal git commits and did not update `STATE.md` or `ROADMAP.md`.
- An unrelated `.gitignore` modification adding `.codex` appeared during final verification. It was not part of this plan and was not committed.

## Known Stubs

None found in files created or modified by this plan.

## Threat Flags

None. This plan added no network endpoints, auth paths, file access patterns, queue/S3/database behavior, canonical identity fields, or UI-visible API behavior.

## Verification

| Command | Result |
| --- | --- |
| `rg -n "connected_player_backfill\|entity\\.connected_player_backfill\|legacy connected event player backfill" crates/parser-core/src` | PASS |
| `cargo check -p parser-core --all-targets` | PASS |
| `rg -n "duplicate_slot_same_name\|compat\\.entity_duplicate_side_conflict\|kept_entities_unmerged" crates/parser-core/src` | PASS |
| `cargo test -p parser-core legacy_entity_compatibility` | PASS - 5 targeted tests passed |
| `cargo test -p parser-core` | PASS - 27 parser-core tests passed plus doc-tests |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS - workspace tests and doc-tests passed |
| `cargo doc --workspace --no-deps` | PASS |
| `rg -n "crates/parser-core\|cargo test -p parser-core" README.md` | PASS |
| `rg -n "implemented.*sg-replay-parser parse\|parse command is implemented\|CLI parse.*implemented" README.md` | PASS - no matches |
| `git diff --check` | PASS |

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 4 can build combat event semantics and aggregate projection on top of populated metadata, observed entity facts, connected-player inferred identity, duplicate same-name compatibility hints, deterministic output, and data-loss status policy.

## Self-Check: PASSED

- Found `.planning/phases/03-deterministic-parser-core/03-05-SUMMARY.md`.
- Found `crates/parser-core/tests/legacy_entity_compatibility.rs`.
- Found `crates/parser-core/tests/fixtures/connected-backfill.ocap.json`.
- Found `crates/parser-core/tests/fixtures/duplicate-slot-same-name.ocap.json`.
- Found task commits `2708ed5`, `3a01fff`, `fbcdf75`, and `2d87924` in git history.

---
*Phase: 03-deterministic-parser-core*
*Completed: 2026-04-27*
