---
phase: 04-event-semantics-and-aggregates
plan: 00
subsystem: parser-contract
tags: [rust, serde, schemars, parser-contract, combat-events, aggregates, vehicle-score, side-facts]

requires:
  - phase: 03-deterministic-parser-core
    provides: deterministic parse artifact shell with observed metadata and entity facts
provides:
  - Typed combat event payload contract with bounty eligibility and vehicle context
  - Typed aggregate contribution payload helpers for legacy, relationship, bounty, and vehicle-score inputs
  - Typed replay-side commander and outcome facts with explicit unknown defaults
  - Regenerated parse artifact schema and success example covering Phase 4 contract surfaces
affects: [parser-core, server-2, phase-04-parser-core-work, schema-validation]

tech-stack:
  added: []
  patterns:
    - Schema-visible typed helper definitions while preserving AggregateContributionRef.value as serde_json::Value
    - Mandatory side_facts section with explicit unknown outcome default

key-files:
  created:
    - crates/parser-contract/src/side_facts.rs
    - crates/parser-contract/tests/combat_event_contract.rs
    - crates/parser-contract/tests/aggregate_contract.rs
    - crates/parser-contract/tests/replay_side_facts_contract.rs
  modified:
    - crates/parser-contract/src/events.rs
    - crates/parser-contract/src/aggregates.rs
    - crates/parser-contract/src/artifact.rs
    - crates/parser-contract/src/lib.rs
    - crates/parser-contract/src/schema.rs
    - crates/parser-contract/tests/schema_contract.rs
    - crates/parser-contract/examples/parse_artifact_success.v1.json
    - crates/parser-contract/examples/parse_failure.v1.json
    - crates/parser-core/src/artifact.rs
    - schemas/parse-artifact-v1.schema.json

key-decisions:
  - "Aggregate contribution envelope remains forward-compatible through serde_json::Value, with typed helper schemas exported for consumers."
  - "Replay side facts are mandatory in ParseArtifact and default to empty commanders plus explicit unknown outcome."
  - "Parser-core artifact construction initializes side_facts with the explicit unknown default until later Phase 4 parser-core plans populate it."

patterns-established:
  - "Contract tests assert observable JSON serialization for typed event, aggregate, and side-fact payloads."
  - "Schema generation can expose helper payload definitions that are not directly referenced by ParseArtifact fields."

requirements-completed: [PARS-08, PARS-09, PARS-10, PARS-11, AGG-02, AGG-06, AGG-07, AGG-08, AGG-09, AGG-10, AGG-11]

duration: 14min
completed: 2026-04-27
---

# Phase 04 Plan 00: Contract Extensions Summary

**Schema-visible combat, aggregate contribution, vehicle score, and replay-side fact contracts for Phase 4 parser-core work.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-04-27T10:44:11Z
- **Completed:** 2026-04-27T10:58:03Z
- **Tasks:** 4
- **Files modified:** 17

## Accomplishments

- Added typed combat payloads on `NormalizedEvent`, including dominant combat semantic, bounty eligibility/exclusion reasons, vehicle context evidence, and legacy counter effects.
- Added typed aggregate payload helpers for legacy counters, relationships, bounty inputs, and issue #13 vehicle score inputs while keeping the existing JSON value envelope.
- Added `ReplaySideFacts` with commander facts and explicit known/unknown/inferred outcome states, then wired it into `ParseArtifact`.
- Regenerated `schemas/parse-artifact-v1.schema.json` and expanded the committed success example with combat, bounty, vehicle score, side facts, and namespaced aggregate projections.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add typed combat event payload contract** - `0cf1c73` (feat)
2. **Task 2: Add typed aggregate contribution payload helpers** - `cc06746` (feat)
3. **Task 3: Add typed replay-side commander and outcome facts** - `246210f` (feat)
4. **Task 4: Regenerate schema and success example with new contract fields** - `14c9880` (feat)

## Files Created/Modified

- `crates/parser-contract/src/events.rs` - Combat event enums and typed combat payload on normalized events.
- `crates/parser-contract/src/aggregates.rs` - Typed helper payloads for aggregate contribution values.
- `crates/parser-contract/src/side_facts.rs` - Replay-side commander and outcome fact contract.
- `crates/parser-contract/src/artifact.rs` - Mandatory `side_facts` artifact section.
- `crates/parser-contract/src/schema.rs` - Schema export now includes aggregate helper payload definitions.
- `crates/parser-contract/tests/*.rs` - Focused behavior-level contract tests for the new JSON surfaces.
- `crates/parser-contract/examples/*.json` - Examples updated for mandatory side facts and Phase 4 success payloads.
- `crates/parser-core/src/artifact.rs` - Parser-core artifact shell initializes `side_facts` to unknown/default.
- `schemas/parse-artifact-v1.schema.json` - Regenerated committed schema.

## Decisions Made

- Aggregate payload helpers are exported in schema definitions even though `AggregateContributionRef.value` remains `serde_json::Value`.
- Missing commander/outcome data is represented as explicit unknown side facts and does not imply partial status.
- Contract-only work did not introduce canonical identity, persistence, queue, object storage, API, or UI ownership changes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated dependent artifact constructors for mandatory side_facts**
- **Found during:** Task 3 (Add typed replay-side commander and outcome facts)
- **Issue:** Adding `ParseArtifact.side_facts` broke parser-core artifact construction and test artifact constructors outside the task's listed files.
- **Fix:** Initialized `ReplaySideFacts::default()` in parser-core and contract test artifact builders; added side facts to both committed examples.
- **Files modified:** `crates/parser-core/src/artifact.rs`, `crates/parser-contract/tests/artifact_envelope.rs`, `crates/parser-contract/tests/failure_contract.rs`, `crates/parser-contract/examples/parse_artifact_success.v1.json`, `crates/parser-contract/examples/parse_failure.v1.json`
- **Verification:** `cargo test -p parser-contract replay_side_facts_contract`, `cargo test -p parser-contract artifact_envelope`, `cargo test -p parser-core`
- **Committed in:** `246210f`

**2. [Rule 2 - Missing Critical] Exported schema definitions for typed aggregate helper values**
- **Found during:** Task 4 (Regenerate schema and success example with new contract fields)
- **Issue:** Helper structs such as `VehicleScoreInputValue` are intentionally not referenced by `ParseArtifact` because the contribution envelope keeps `value: Value`, so they would not appear in generated schema automatically.
- **Fix:** Added schema export support for aggregate helper definitions while preserving the stable contribution envelope.
- **Files modified:** `crates/parser-contract/src/schema.rs`, `schemas/parse-artifact-v1.schema.json`
- **Verification:** `cargo test -p parser-contract schema_contract`, regenerated schema comparison with `cmp`
- **Committed in:** `14c9880`

**3. [Rule 3 - Blocking] Split invalid Cargo multi-filter verification commands**
- **Found during:** Task 3 and Task 4 verification
- **Issue:** The plan's `cargo test -p parser-contract replay_side_facts_contract artifact_envelope` style commands pass multiple positional filters, which Cargo rejects.
- **Fix:** Ran equivalent verification as separate Cargo test commands for each requested test filter.
- **Files modified:** None
- **Verification:** All requested suites passed individually.
- **Committed in:** N/A

---

**Total deviations:** 3 auto-fixed (2 blocking, 1 missing critical functionality).
**Impact on plan:** No scope creep; fixes were required to keep the contract buildable, schema-visible, and verifiable.

## Issues Encountered

- Local `gsd-sdk query` is unsupported in this checkout, so execution state and metadata updates were handled manually as allowed by the prompt.
- A schema helper compile error was caught during `cargo run -p parser-contract --example export_schema` and fixed before commit.

## Authentication Gates

None.

## Known Stubs

None found in created or modified files.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-contract combat_event_contract` - passed
- `cargo test -p parser-contract aggregate_contract` - passed
- `cargo test -p parser-contract replay_side_facts_contract` - passed
- `cargo test -p parser-contract schema_contract` - passed
- `cargo test -p parser-contract source_ref_contract` - passed
- `cargo fmt --all -- --check` - passed
- `git diff --check` - passed

## Next Phase Readiness

Plan 04-01 can build parser-core raw killed-event accessors against the now schema-visible combat, aggregate, vehicle-score, and side-fact contracts. No blockers remain.

## Self-Check: PASSED

- Summary file exists.
- Created side-fact module and all three new contract test files exist.
- Task commits `0cf1c73`, `cc06746`, `246210f`, and `14c9880` exist in git history.
- No unexpected file deletions were present in task commits.

---
*Phase: 04-event-semantics-and-aggregates*
*Completed: 2026-04-27*
