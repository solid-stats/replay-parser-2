---
phase: 02-versioned-output-contract
plan: 01
subsystem: contract
tags: [rust, serde, schemars, parser-contract, diagnostics, source-refs]
requires:
  - phase: 02-00
    provides: Rust workspace, parser-contract crate, contract version, and parser version metadata
provides:
  - Unified ParseArtifact envelope with source identity and deterministic extension map
  - Exact ParseStatus values: success, partial, skipped, failed
  - Path-based Diagnostic model with source coordinates and no raw replay snippet fields
  - Minimal success ParseArtifact v1 example validated by tests and jq
affects: [phase-02, phase-03, phase-04, phase-05, phase-06, parser-contract, server-2-integration]
tech-stack:
  added: []
  patterns: [serde snake_case enums, BTreeMap extension maps, path-based diagnostics, source-coordinate references]
key-files:
  created:
    - crates/parser-contract/tests/artifact_envelope.rs
    - crates/parser-contract/examples/parse_artifact_success.v1.json
  modified:
    - crates/parser-contract/src/artifact.rs
    - crates/parser-contract/src/source_ref.rs
    - crates/parser-contract/src/diagnostic.rs
    - crates/parser-contract/src/metadata.rs
    - crates/parser-contract/src/identity.rs
    - crates/parser-contract/src/events.rs
    - crates/parser-contract/src/aggregates.rs
    - crates/parser-contract/src/failure.rs
key-decisions:
  - "Implemented D-01 as one unified `ParseArtifact` envelope for all parse outcomes."
  - "Implemented D-14 with exact `ParseStatus` JSON values: `success`, `partial`, `skipped`, and `failed`."
  - "Kept diagnostics path/source-coordinate based and excluded raw replay snippet fields per D-08 and D-13."
patterns-established:
  - "Public contract enums use serde `snake_case` names and exact behavior tests."
  - "Dynamic extension payloads use `BTreeMap<String, serde_json::Value>` for observable deterministic ordering."
  - "Rule IDs are serialized as strings and validated through a non-empty constructor/deserializer."
requirements-completed: [OUT-01, OUT-04, OUT-05]
duration: 5m22s
completed: 2026-04-26
---

# Phase 02 Plan 01: ParseArtifact Envelope, Diagnostics, and Success Example Summary

**Unified parse artifact envelope with exact status values, source identity, path-based diagnostics, and a validated success artifact example**

## Performance

- **Duration:** 5m22s
- **Started:** 2026-04-26T05:05:17Z
- **Completed:** 2026-04-26T05:10:39Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Added `ParseArtifact` with contract/parser/source/status sections plus diagnostics, replay, entities, events, aggregates, failure, and deterministic `extensions`.
- Added `ParseStatus` with exact serialized values `success`, `partial`, `skipped`, and `failed`, directly covering the pre-wave D-01/D-14 anchor concern.
- Added path-based diagnostics with expected/observed shape, parser action, source refs, and validated non-empty rule IDs.
- Committed `parse_artifact_success.v1.json` and a test that deserializes and reserializes it without changing stable envelope fields.

## Task Commits

1. **Task 1: Define source identity, status, artifact envelope, and deterministic extension maps** - `be94f79` (feat)
2. **Task 2: Add path-based diagnostics without raw replay snippets** - `996d957` (feat)
3. **Task 3: Commit a minimal success artifact example** - `d7615d7` (feat)

## Files Created/Modified

- `crates/parser-contract/src/artifact.rs` - Defines `ParseStatus` and the top-level `ParseArtifact` envelope.
- `crates/parser-contract/src/source_ref.rs` - Defines `ReplaySource`, `SourceChecksum`, `SourceRef`, and validated `RuleId`.
- `crates/parser-contract/src/diagnostic.rs` - Defines diagnostic severity and path/action/source-reference diagnostic fields.
- `crates/parser-contract/src/metadata.rs` - Adds temporary `ReplayMetadata` section type for the envelope.
- `crates/parser-contract/src/identity.rs` - Adds temporary `ObservedEntity` section type for the envelope.
- `crates/parser-contract/src/events.rs` - Adds temporary `NormalizedEvent` section type for the envelope.
- `crates/parser-contract/src/aggregates.rs` - Adds temporary `AggregateSection` section type for the envelope.
- `crates/parser-contract/src/failure.rs` - Adds temporary `ParseFailure` section type for the envelope.
- `crates/parser-contract/tests/artifact_envelope.rs` - Tests exact status values, envelope fields, diagnostic safety, rule ID validation, and example round-trip behavior.
- `crates/parser-contract/examples/parse_artifact_success.v1.json` - Minimal success artifact example for schema and server integration review.

## Decisions Made

- Kept `ReplaySource` in `source_ref.rs` so source identity and source coordinates live together.
- Kept top-level nullable envelope fields serialized as present `null` values in the example, matching the plan's stable envelope requirement.
- Implemented `RuleId::new` and deserialization validation now because empty rule IDs would weaken D-11 auditability.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Ensured plan-filtered cargo tests execute real tests**
- **Found during:** Task 1 (artifact envelope tests)
- **Issue:** The initial Rust test names did not include the `artifact_envelope` filter string, so `cargo test -p parser-contract artifact_envelope` exited successfully while running zero envelope tests.
- **Fix:** Renamed the tests so the plan's filtered cargo command runs the status and envelope tests.
- **Files modified:** `crates/parser-contract/tests/artifact_envelope.rs`
- **Verification:** `cargo test -p parser-contract artifact_envelope` ran 2 tests and passed.
- **Committed in:** `be94f79`

---

**Total deviations:** 1 auto-fixed (1 Rule 1 bug).
**Impact on plan:** No scope change. The fix made the planned verification command meaningful.

## Issues Encountered

- Cargo test commands run in parallel can briefly wait for Cargo's artifact directory lock. This did not affect results.

## Known Stubs

These empty section structs are intentional placeholders required by the Task 1 envelope action. They do not block this plan because later Phase 2 plans own their field semantics:

- `crates/parser-contract/src/metadata.rs:5` - `ReplayMetadata`, to be filled by Plan 02.
- `crates/parser-contract/src/identity.rs:5` - `ObservedEntity`, to be filled by Plan 02.
- `crates/parser-contract/src/events.rs:5` - `NormalizedEvent`, to be filled by Plan 03.
- `crates/parser-contract/src/aggregates.rs:5` - `AggregateSection`, to be filled by Plan 03.
- `crates/parser-contract/src/failure.rs:5` - `ParseFailure`, to be filled by Plan 04.

The `null` values in `parse_artifact_success.v1.json` are not stubs; they are the plan-specified envelope representation for currently absent optional sections.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-contract artifact_envelope`
- `cargo test -p parser-contract diagnostics_are_path_based`
- `test -f crates/parser-contract/examples/parse_artifact_success.v1.json`
- `jq -e '.contract_version == "1.0.0" and .status == "success"' crates/parser-contract/examples/parse_artifact_success.v1.json`
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo test -p parser-contract`

## Next Phase Readiness

Plan 02 can replace the temporary replay metadata and observed entity section structs without changing the top-level envelope field names. Plan 03 can build on the committed source-reference and rule-ID model for events and aggregate contributions. Plan 04 can replace the temporary `ParseFailure` section and validate final examples against generated schema.

## Self-Check: PASSED

- Created files listed in the summary exist.
- Task commits `be94f79`, `996d957`, and `d7615d7` exist in git history.
- `.planning/STATE.md` and `.planning/ROADMAP.md` remain unmodified by this executor.

---
*Phase: 02-versioned-output-contract*
*Completed: 2026-04-26*
