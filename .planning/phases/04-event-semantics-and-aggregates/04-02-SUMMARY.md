---
phase: 04-event-semantics-and-aggregates
plan: 02
subsystem: parser-core
tags: [rust, parser-core, combat-events, killed-events, bounty-exclusions, source-refs]

requires:
  - phase: 04-event-semantics-and-aggregates
    provides: Raw killed-event tuple observations and event-coordinate source refs from Plan 04-01
provides:
  - Source-backed normalized combat events for enemy kills, teamkills, suicides, null killers, vehicle victims, and unknown actors
  - Explicit bounty eligibility/exclusion metadata on combat events
  - Legacy counter effects carried on combat events for later aggregate projection
  - Behavior tests and focused fixture covering combat classification and event source refs
affects: [parser-core, aggregate-projection, bounty-inputs, vehicle-score, phase-04-plan-03]

tech-stack:
  added: []
  patterns:
    - Combat events derive from raw killed observations plus normalized observed entities
    - Unknown or unauditable killed tuples emit source-backed unknown events and data-loss diagnostics

key-files:
  created:
    - crates/parser-core/src/events.rs
    - crates/parser-core/tests/combat_event_semantics.rs
    - crates/parser-core/tests/fixtures/combat-events.ocap.json
  modified:
    - crates/parser-core/src/lib.rs
    - crates/parser-core/src/artifact.rs

key-decisions:
  - "Combat normalization emits one dominant normalized event per source killed tuple and preserves legacy counter effects on the event."
  - "Only enemy player kills are bounty eligible; teamkills, suicides, null killers, vehicle victims, and unknown actors carry explicit bounty exclusion reasons."
  - "Unknown actor or malformed killed tuples become partial, diagnostic-backed unknown events with no legacy counter effects."

patterns-established:
  - "Parser-core combat events use SourceContext::event_source_ref with semantic rule IDs for auditable event coordinates."
  - "Behavior tests parse fixtures through the public parse_replay API and assert observable artifact output."

requirements-completed: [PARS-08, PARS-09, AGG-02, AGG-06, AGG-07]

duration: 8m27s
completed: 2026-04-27
---

# Phase 04 Plan 02: Combat Event Normalization Summary

**Source-backed killed tuple normalization with explicit combat semantics, bounty exclusions, legacy effects, and partial diagnostics for unauditable actors.**

## Performance

- **Duration:** 8m27s
- **Started:** 2026-04-27T11:12:09Z
- **Completed:** 2026-04-27T11:20:36Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added parser-core combat event normalization for enemy kills, teamkills, suicides, null-killer deaths, vehicle victims, and unknown actor cases.
- Wired normalized combat events into successful parse artifact assembly while leaving aggregate projections and side facts defaulted for later plans.
- Added a focused OCAP fixture and behavior tests proving event count, semantic classification, bounty eligibility/exclusions, partial status, and event-coordinate source refs.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement combat event normalization module** - `243d49b` (feat)
2. **Task 2: Wire combat events into parse artifact assembly** - `123c031` (feat)
3. **Task 3: Add combat semantics fixture and behavior tests** - `cc664e8` (test)

## Files Created/Modified

- `crates/parser-core/src/events.rs` - Normalizes raw killed observations into typed `NormalizedEvent` combat payloads with source refs, bounty state, legacy effects, and diagnostics.
- `crates/parser-core/src/lib.rs` - Exports the new parser-core `events` module.
- `crates/parser-core/src/artifact.rs` - Adds combat event normalization to successful parse artifact assembly.
- `crates/parser-core/tests/combat_event_semantics.rs` - Behavior-level tests for combat classification and source refs through `parse_replay`.
- `crates/parser-core/tests/fixtures/combat-events.ocap.json` - Focused killed-event fixture covering six required combat cases.

## Decisions Made

- Null-killer player deaths, same-side teamkills, suicides, vehicle victims, and unknown actors are emitted as auditable combat events but never bounty-eligible events.
- Unknown actor/entity lookup and malformed killed shapes use `DiagnosticImpact::DataLoss`, making the artifact `partial` while preserving a source-backed unknown event.
- Vehicle score taxonomy fields are populated conservatively from existing observed entity class/name evidence without adding aggregate contributions in this plan.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Local `gsd-sdk query` is unsupported in this checkout, so execution state and metadata updates were handled manually where safe as allowed by the prompt.
- ROADMAP already counted Plan 04-01 as complete but its checkbox was still unchecked; the metadata update aligned the checkbox with the existing completed summary and Phase 4 count.

## Authentication Gates

None.

## Known Stubs

None found in created or modified files.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-core combat_event_semantics` - passed
- `cargo test -p parser-core deterministic_output` - passed
- `cargo check -p parser-core --all-targets` - passed
- `cargo fmt --all -- --check` - passed
- `git diff --check` - passed

## Next Phase Readiness

Plan 04-03 can derive legacy per-replay projections, relationship summaries, and bounty input projections from normalized combat events and their source refs. No canonical identity, persistence, queue/storage, API, or UI responsibility was introduced.

## Self-Check: PASSED

- Summary file exists.
- Created combat event module, combat semantics test file, and combat-events fixture exist.
- Task commits `243d49b`, `123c031`, and `cc664e8` exist in git history.
- No unexpected file deletions were present in task commits.

---
*Phase: 04-event-semantics-and-aggregates*
*Completed: 2026-04-27*
