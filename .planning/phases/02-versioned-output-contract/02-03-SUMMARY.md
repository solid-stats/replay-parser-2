---
phase: 02-versioned-output-contract
plan: 03
subsystem: contract
tags: [rust, serde, parser-contract, source-refs, normalized-events, aggregates, rule-ids]
requires:
  - phase: 02-02
    provides: FieldPresence, observed identity types, metadata contract, and initial SourceRef/RuleId placeholders
provides:
  - Validated SourceRef coordinate model with stable namespaced RuleId values
  - NormalizedEvent and EventActorRef skeletons with source_refs and rule_id fields
  - AggregateContributionRef and AggregateSection skeletons for Phase 4 formulas
affects: [phase-02, phase-04, server-2-integration, schema-generation]
tech-stack:
  added: []
  patterns: [serde snake_case enums, BTreeMap extension maps, behavior-level contract tests]
key-files:
  created:
    - crates/parser-contract/tests/source_ref_contract.rs
  modified:
    - crates/parser-contract/src/source_ref.rs
    - crates/parser-contract/src/events.rs
    - crates/parser-contract/src/aggregates.rs
    - crates/parser-contract/examples/parse_artifact_success.v1.json
key-decisions:
  - "RuleId validation now requires non-empty lowercase namespaced IDs with dot-separated non-empty segments."
  - "Normalized events and aggregate contributions require source_refs plus rule_id for auditability, while formulas remain Phase 4 scope."
  - "AggregateSection is now concrete with contributions and deterministic BTreeMap projections."
patterns-established:
  - "Contract tests include cargo-filter-friendly names so required plan verification commands execute real assertions."
  - "Audit skeletons use source coordinates and stable rule IDs instead of embedding raw replay snippets."
requirements-completed: [OUT-05]
duration: 4m53s
completed: 2026-04-26
---

# Phase 02 Plan 03: Source References, Events, and Aggregate Contribution Refs Summary

**Auditable event and aggregate skeletons with validated stable rule IDs**

## Performance

- **Duration:** 4m53s
- **Started:** 2026-04-26T05:23:33Z
- **Completed:** 2026-04-26T05:28:26Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Tightened `RuleId::new` so rule IDs must be lowercase, ASCII, namespaced, and dot-separated.
- Defined `NormalizedEventKind`, `EventActorRef`, and `NormalizedEvent` with `source_refs`, `rule_id`, and deterministic `BTreeMap` attributes.
- Defined `AggregateContributionKind`, `AggregateContributionRef`, and `AggregateSection` with contribution audit refs and deterministic projections.
- Added source-reference contract tests covering rule IDs, source coordinates, normalized events, and vehicle score contribution references.

## Task Commits

1. **Task 1: Tighten SourceRef and RuleId contract** - `1c5ca91` (feat)
2. **Task 2: Define normalized event skeleton with source refs** - `af4466c` (feat)
3. **Task 3: Define aggregate contribution reference skeleton** - `395b9e7` (feat)

## Files Created/Modified

- `crates/parser-contract/src/source_ref.rs` - Enforces stable lowercase namespaced `RuleId` values and keeps the required source coordinate fields.
- `crates/parser-contract/src/events.rs` - Defines normalized event kinds, actor refs, source refs, rule IDs, and attribute map shape.
- `crates/parser-contract/src/aggregates.rs` - Defines aggregate contribution refs and concrete aggregate sections.
- `crates/parser-contract/tests/source_ref_contract.rs` - Adds behavior tests for source refs, normalized events, and aggregate contribution refs.
- `crates/parser-contract/examples/parse_artifact_success.v1.json` - Updates the success artifact example to use the concrete aggregate section shape.

## Decisions Made

- Kept Phase 2 strictly at contract skeleton level: no kill/teamkill semantics and no aggregate formulas were implemented.
- Used `BTreeMap` for event attributes and aggregate projections to preserve deterministic serialized ordering.
- Included `source_ref_contract`, `normalized_event_source_refs`, and `aggregate_contribution_refs` in test names so the plan's filtered cargo commands run real tests.

## Pre-Wave Note

The orchestrator's `verify.key-links` planned-anchor miss for `AggregateContributionRef` was expected for this wave. This was not a prior-wave dependency failure; `AggregateContributionRef` was implemented and tested in this plan per D-12.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Updated success artifact aggregate example**
- **Found during:** Task 3 (Define aggregate contribution reference skeleton)
- **Issue:** The existing success example still serialized `aggregates` as `{}` after the aggregate section became a concrete contract type.
- **Fix:** Updated `parse_artifact_success.v1.json` to include empty `contributions` and `projections` fields.
- **Files modified:** `crates/parser-contract/examples/parse_artifact_success.v1.json`
- **Verification:** `cargo test -p parser-contract` and the required filtered aggregate test passed.
- **Committed in:** `395b9e7`

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The adjustment kept the existing contract example aligned with the new aggregate model without adding formulas or broadening scope.

## Issues Encountered

None.

## Known Stubs

None - the empty example `contributions` and `projections` are intentional default contract values, not unwired placeholders.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-contract source_ref_contract`
- `cargo test -p parser-contract normalized_event_source_refs`
- `cargo test -p parser-contract aggregate_contribution_refs`
- `rg -n "AggregateContributionRef|NormalizedEvent|RuleId" crates/parser-contract/src`
- `git diff --check`
- `cargo test -p parser-contract`
- `cargo fmt --all -- --check`

## Next Phase Readiness

Plan 04 can generate schema and structured failure contracts against concrete event and aggregate skeletons. Phase 4 can plan event semantics and aggregate formulas against `AggregateContributionRef` without changing the audit contribution model.

Orchestrator-owned `.planning/STATE.md` and `.planning/ROADMAP.md` were not modified by this executor.

## Self-Check: PASSED

- Created summary file exists.
- Modified contract source, example, and test files exist.
- Task commits `1c5ca91`, `af4466c`, and `395b9e7` exist in git history.
- `.planning/STATE.md` and `.planning/ROADMAP.md` remain unmodified by this executor.

---
*Phase: 02-versioned-output-contract*
*Completed: 2026-04-26*
