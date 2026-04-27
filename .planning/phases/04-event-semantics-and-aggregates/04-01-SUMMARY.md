---
phase: 04-event-semantics-and-aggregates
plan: 01
subsystem: parser-core
tags: [rust, parser-core, raw-events, source-refs, killed-events]

requires:
  - phase: 04-event-semantics-and-aggregates
    provides: Phase 04 Plan 00 contract extensions for event and aggregate semantics
provides:
  - Tolerant raw killed-event tuple observations with source event index, frame, JSON path, entity IDs, weapon, and distance evidence
  - SourceContext event source-ref helper carrying frame, event index, entity ID, JSON path, rule ID, replay ID, source file, and checksum
  - Focused raw killed-event fixture and behavior tests for normal, null-killer, empty-weapon, malformed, and ignored non-killed tuples
affects: [parser-core, combat-event-normalization, aggregate-source-refs, phase-04-plan-02]

tech-stack:
  added: []
  patterns:
    - Raw OCAP event tuple accessors preserve source evidence without emitting semantic diagnostics
    - Event source references are built from deterministic caller-provided source metadata and explicit coordinates

key-files:
  created:
    - crates/parser-core/tests/raw_event_accessors.rs
    - crates/parser-core/tests/fixtures/killed-events.ocap.json
  modified:
    - crates/parser-core/src/raw.rs
    - crates/parser-core/src/artifact.rs

key-decisions:
  - "Raw killed-event accessors preserve malformed kill-info shapes as source-backed observations and defer semantic diagnostics to Plan 04-02."
  - "Event source refs are constructed from SourceContext metadata and explicit event coordinates only; no wall-clock or filesystem metadata is introduced."

patterns-established:
  - "KilledEventObservation keeps raw tuple evidence isolated from semantic combat classification."
  - "SourceContext::event_source_ref is the event-coordinate counterpart to existing top-level and entity source-ref helpers."

requirements-completed: [PARS-08, PARS-09, AGG-02]

duration: 5m31s
completed: 2026-04-27
---

# Phase 04 Plan 01: Raw Killed-Event Accessors Summary

**Tolerant parser-core killed tuple observations and event-coordinate source refs for later combat semantics and auditable aggregates.**

## Performance

- **Duration:** 5m31s
- **Started:** 2026-04-27T11:02:28Z
- **Completed:** 2026-04-27T11:07:59Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Added `KilledEventObservation`, `KilledEventKillInfo`, and `killed_events` to preserve one raw observation per stable-frame `killed` source tuple.
- Added `SourceContext::event_source_ref` for event source refs with frame, event index, entity ID, JSON path, and rule ID coordinates.
- Added focused raw accessor tests covering normal killer/weapon/distance, null killer, empty weapon, malformed kill-info shape, ignored non-killed events, and event source refs.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement raw killed-event tuple accessors** - `f301dcb` (feat)
2. **Task 2: Add event source reference helper** - `6543250` (feat)
3. **Task 3: Add raw killed-event accessor fixture and tests** - `cc17a75` (test)

## Files Created/Modified

- `crates/parser-core/src/raw.rs` - Added tolerant killed-event observation structs, kill-info enum, and top-level `killed_events` accessor.
- `crates/parser-core/src/artifact.rs` - Added `SourceContext::event_source_ref`.
- `crates/parser-core/tests/raw_event_accessors.rs` - Added behavior-focused raw event accessor and event source-ref tests.
- `crates/parser-core/tests/fixtures/killed-events.ocap.json` - Added focused killed-event OCAP fixture.

## Decisions Made

- Followed the plan's boundary: raw accessors do not emit diagnostics or semantic counters.
- Malformed `event[3]` kill-info values remain observable as `KilledEventKillInfo::Malformed` with a coarse observed shape.
- Events without a numeric frame are skipped because they lack the stable event coordinate required by the plan.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Local `gsd-sdk query` is unsupported in this checkout, so execution state and metadata updates were handled manually as allowed by the prompt.

## Authentication Gates

None.

## Known Stubs

None found in created or modified files.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-core raw_event_accessors` - passed
- `cargo check -p parser-core --all-targets` - passed
- `cargo fmt --all -- --check` - passed
- `cargo clippy -p parser-core --all-targets -- -D warnings` - passed
- `git diff --check` - passed

## Next Phase Readiness

Plan 04-02 can consume `KilledEventObservation` values and `SourceContext::event_source_ref` to normalize combat events while preserving raw source coordinates. No semantic counters, aggregate projections, or parser artifact event population were added in this plan.

## Self-Check: PASSED

- Summary file exists.
- Raw event accessor test file and killed-event fixture exist.
- Task commits `f301dcb`, `6543250`, and `cc17a75` exist in git history.
- No unexpected file deletions were present in task commits.

---
*Phase: 04-event-semantics-and-aggregates*
*Completed: 2026-04-27*
