---
phase: 03-deterministic-parser-core
plan: 02
subsystem: parser-core
tags: [rust, serde-json, parser-core, metadata, diagnostics, deterministic-output]

requires:
  - phase: 03-deterministic-parser-core
    provides: "Plan 03-00 observed entity contract extensions and source-reference invariants"
  - phase: 03-deterministic-parser-core
    provides: "Plan 03-01 parser-core pure API, deterministic artifact shell, and structured JSON/root failures"
provides:
  - "Tolerant RawReplay adapter for OCAP top-level field extraction"
  - "ReplayMetadata normalization from missionName, worldName, missionAuthor, playersCount, captureDelay, and endFrame"
  - "Path-based schema drift diagnostics with explicit unknown metadata states"
  - "Focused metadata fixtures and behavior tests"
affects: [03-deterministic-parser-core, parser-core, parser-contract-consumers]

tech-stack:
  added: []
  patterns:
    - "Raw OCAP field quirks stay in parser-core raw.rs"
    - "Metadata normalization consumes RawField present/absent/drift observations"
    - "Parser-core leaves produced_at unset and marks drifted metadata artifacts partial"

key-files:
  created:
    - .planning/phases/03-deterministic-parser-core/03-02-SUMMARY.md
    - crates/parser-core/src/raw.rs
    - crates/parser-core/src/metadata.rs
    - crates/parser-core/tests/metadata_normalization.rs
    - crates/parser-core/tests/fixtures/valid-minimal.ocap.json
    - crates/parser-core/tests/fixtures/metadata-drift.ocap.json
  modified:
    - crates/parser-core/src/lib.rs
    - crates/parser-core/src/artifact.rs
    - crates/parser-core/src/diagnostics.rs
    - crates/parser-core/tests/parser_core_api.rs

key-decisions:
  - "Valid object roots now produce ReplayMetadata immediately; entity/event/aggregate population remains deferred to later Phase 3 plans."
  - "Drifted metadata fields emit warning diagnostics, become explicit Unknown(SchemaDrift), and set artifact status to partial."
  - "Missing metadata fields become explicit Unknown(SourceFieldAbsent) with source refs but do not emit diagnostics by themselves."

patterns-established:
  - "RawReplay returns RawField::Present, RawField::Absent, or RawField::Drift with stable JSON paths and observed shapes."
  - "SourceContext copies replay ID, source file, and present checksum into metadata source refs."
  - "DiagnosticAccumulator caps emitted diagnostics while preserving a data-loss flag for status escalation."

requirements-completed: [OUT-08, PARS-01, PARS-02, PARS-03]

duration: 14m
completed: 2026-04-27
---

# Phase 3 Plan 02: Tolerant OCAP Root Decode and Replay Metadata Summary

**Tolerant OCAP top-level extraction with deterministic ReplayMetadata, explicit unknowns, and path-based drift diagnostics**

## Performance

- **Duration:** 14m
- **Started:** 2026-04-27T05:02:00Z
- **Completed:** 2026-04-27T05:15:52Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added `RawReplay` and `RawField` to isolate top-level OCAP source quirks in `raw.rs`.
- Populated `ReplayMetadata` from observed OCAP fields through the public `parse_replay` API.
- Represented absent metadata fields as explicit unknowns and drifted metadata fields as unknowns plus warning diagnostics.
- Computed deterministic frame and time bounds from `endFrame` and finite `captureDelay`.
- Added focused valid and drift fixtures plus behavior tests for metadata population, absence, drift, and derived bounds.

## Task Commits

1. **Task 1: Add tolerant raw OCAP root and top-level field helpers** - `c992d3e` (feat)
2. **Task 2: Normalize replay metadata with explicit presence and source refs** - `c2ab9f2` (feat)

## Files Created/Modified

- `crates/parser-core/src/raw.rs` - Raw OCAP root wrapper, typed top-level field helpers, and stable observed shape reporting.
- `crates/parser-core/src/metadata.rs` - Metadata normalization, source refs, derived bounds, and drift diagnostic creation.
- `crates/parser-core/src/artifact.rs` - Wires valid root objects into metadata normalization and partial status for drift.
- `crates/parser-core/src/diagnostics.rs` - Adds capped diagnostic accumulator and data-loss status flag.
- `crates/parser-core/src/lib.rs` - Exposes `raw` and `metadata` modules.
- `crates/parser-core/tests/metadata_normalization.rs` - Adds behavior tests for valid, absent, drifted, and derived metadata.
- `crates/parser-core/tests/fixtures/valid-minimal.ocap.json` - Valid focused top-level metadata fixture.
- `crates/parser-core/tests/fixtures/metadata-drift.ocap.json` - Focused drift fixture with string `playersCount` and absent `missionAuthor`.
- `crates/parser-core/tests/parser_core_api.rs` - Updates valid-root API expectation now that metadata is populated.

## Verification

| Command | Result |
| --- | --- |
| `rg -n "pub mod raw|RawReplay|RawField|observed_shape" crates/parser-core/src` | PASS |
| `cargo check -p parser-core --all-targets` | PASS |
| `cargo test -p parser-core metadata_normalization` | PASS - 4 metadata tests passed |
| `rg -n "metadata\\.mission_name\\.observed|schema\\.metadata_field|diagnostic\\.schema_drift\\.metadata" crates/parser-core/src crates/parser-core/tests` | PASS |
| `cargo test -p parser-core parser_core_api` | PASS - 2 filtered API tests passed |
| `cargo test --workspace` | PASS - workspace tests and doc-tests passed |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |

## Decisions Made

- Kept top-level source shape tolerance in `raw.rs`; `metadata.rs` only consumes explicit observations.
- Used field-specific rule IDs such as `metadata.mission_name.observed` for metadata source refs and `diagnostic.schema_drift.metadata` for drift diagnostics.
- Chose `ParseStatus::Partial` for schema drift diagnostics because drift causes explicit unknowns and data loss under Phase 3 decision D-08.
- Left entity, event, aggregate, CLI, worker, S3, RabbitMQ, database, and canonical identity behavior out of scope.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed RawReplay helper ownership error**
- **Found during:** Task 1 verification
- **Issue:** Generic field parsing moved `json_path` into one closure and reused it in another, blocking `cargo check`.
- **Fix:** Rewrote the helper with an explicit `match` so present and drift branches own the path once.
- **Files modified:** `crates/parser-core/src/raw.rs`
- **Verification:** `cargo check -p parser-core --all-targets` passed.
- **Committed in:** `c992d3e`

**2. [Rule 1 - Bug] Updated parser-core API expectation after metadata became populated**
- **Found during:** Task 2 implementation
- **Issue:** Existing parser-core API test still expected valid object roots to return `replay: None`, which contradicted this plan's metadata output.
- **Fix:** Updated the test to require `artifact.replay.is_some()` while preserving deterministic shell assertions.
- **Files modified:** `crates/parser-core/tests/parser_core_api.rs`
- **Verification:** `cargo test -p parser-core parser_core_api` passed.
- **Committed in:** `c2ab9f2`

**3. [Rule 3 - Blocking] Resolved strict clippy blockers in new parser-core code**
- **Found during:** Task 2 verification
- **Issue:** Workspace clippy gates required `Eq`, an `Option::map_or_else` rewrite, a const test helper, and local allows for plan-mandated borrowed `RawReplay` signatures.
- **Fix:** Applied the clippy-required changes without changing parser behavior.
- **Files modified:** `crates/parser-core/src/diagnostics.rs`, `crates/parser-core/src/metadata.rs`, `crates/parser-core/src/raw.rs`, `crates/parser-core/tests/metadata_normalization.rs`
- **Verification:** `cargo clippy --workspace --all-targets -- -D warnings` passed.
- **Committed in:** `c2ab9f2`

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact on plan:** No scope expansion. Fixes were required for correctness and the existing strict Rust quality gate.

## Issues Encountered

- The Task 1 read_first path `/home/afgan0r/Projects/SolidGames/replays-parser/src/0 - types/replay.d.ts` was not present in this environment, and no readable local replacement was available. This did not affect the metadata implementation because the task's executable top-level field evidence came from the Phase 1 corpus manifest and parser-contract metadata types.
- The local `node_modules/@gsd-build/sdk` CLI path was absent and the `gsd-sdk query` command on PATH did not support query mode. Per the user-provided commit protocol, execution used normal git commits and did not update `STATE.md` or `ROADMAP.md`.

## Known Stubs

- `crates/parser-core/src/artifact.rs` still emits empty `entities`, `events`, `aggregates`, and `extensions` for successful metadata parses. These are intentional deferred Phase 3/4 surfaces; this plan only owns replay metadata normalization.
- `crates/parser-core/src/artifact.rs` keeps failed artifacts at `replay: None` with empty diagnostics/entities/events, preserving the structured failure behavior from Plan 03-01.

## Threat Flags

None. This plan added no network endpoints, auth paths, file access patterns, queue/S3/database behavior, canonical identity fields, or parser-owned timestamps.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03-03 can build observed unit/player, vehicle, and static weapon entity normalization on top of the `RawReplay` adapter and `SourceContext` source-reference pattern introduced here.

## Self-Check: PASSED

- Found `.planning/phases/03-deterministic-parser-core/03-02-SUMMARY.md`.
- Found `crates/parser-core/src/raw.rs`.
- Found `crates/parser-core/src/metadata.rs`.
- Found `crates/parser-core/tests/metadata_normalization.rs`.
- Found task commits `c992d3e` and `c2ab9f2` in git history.

---
*Phase: 03-deterministic-parser-core*
*Completed: 2026-04-27*
