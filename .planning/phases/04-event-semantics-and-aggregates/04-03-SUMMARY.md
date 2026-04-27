---
phase: 04-event-semantics-and-aggregates
plan: 03
subsystem: parser-core
tags: [rust, parser-core, aggregates, legacy-projections, bounty-inputs, source-refs]

requires:
  - phase: 04-event-semantics-and-aggregates
    provides: Source-backed combat events and bounty exclusion metadata from Plan 04-02
provides:
  - Auditable aggregate derivation module for legacy counters, relationships, and bounty inputs
  - Namespaced per-replay projections for legacy player results, relationships, game-type metadata, squad inputs, rotation inputs, and bounty inputs
  - Successful parser-core artifacts populated with aggregate contributions and projections
  - Focused aggregate projection fixture and behavior tests covering traceability, bounty exclusions, and duplicate same-name compatibility
affects: [parser-core, aggregate-projection, bounty-inputs, vehicle-score, phase-04-plan-04]

tech-stack:
  added: []
  patterns:
    - Aggregate projections are derived from sorted AggregateContributionRef values, not direct unaudited counters
    - Duplicate-slot same-name compatibility uses explicit legacy_name keys while preserving observed entity IDs

key-files:
  created:
    - crates/parser-core/src/aggregates.rs
    - crates/parser-core/tests/aggregate_projection.rs
    - crates/parser-core/tests/fixtures/aggregate-combat.ocap.json
  modified:
    - crates/parser-core/src/lib.rs
    - crates/parser-core/src/artifact.rs

key-decisions:
  - "Parser-core emits per-replay aggregate contributions/projections only; cross-replay weekly, all-time, squad, and rotation totals remain downstream."
  - "Bounty inputs are emitted only from eligible enemy player kill contributions; excluded combat events remain auditable but do not create bounty inputs."
  - "Legacy duplicate same-name projection rows use legacy_name keys and include all observed source entity IDs instead of canonical player IDs."

patterns-established:
  - "Aggregate contribution IDs use stable aggregate.legacy, aggregate.relationship, and aggregate.bounty namespaces."
  - "Projection rows carry source_contribution_ids back to non-empty source refs and rule IDs."

requirements-completed: [AGG-01, AGG-02, AGG-03, AGG-04, AGG-05, AGG-06, AGG-07]

duration: 11m45s
completed: 2026-04-27
---

# Phase 04 Plan 03: Aggregate Projections Summary

**Auditable per-replay legacy, relationship, game-type, squad, rotation, and bounty projections derived from normalized combat events.**

## Performance

- **Duration:** 11m45s
- **Started:** 2026-04-27T11:25:19Z
- **Completed:** 2026-04-27T11:37:04Z
- **Tasks:** 4
- **Files modified:** 5

## Accomplishments

- Added `derive_aggregate_section` to produce sorted source-backed aggregate contributions using stable legacy, relationship, and bounty contribution IDs.
- Added namespaced projections for `legacy.player_game_results`, `legacy.relationships`, `legacy.game_type_compatibility`, `legacy.squad_inputs`, `legacy.rotation_inputs`, and `bounty.inputs`.
- Wired aggregate derivation into successful parser-core artifact assembly after combat event normalization.
- Added behavior tests proving legacy formulas, relationship summaries, bounty exclusions, duplicate same-name grouping without entity merging, and contribution/source-ref traceability.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement aggregate derivation module and compatibility keys** - `ba27233` (feat)
2. **Task 2: Produce legacy player, relationship, game-type, and bounty projections** - `98a9d54` (feat)
3. **Task 3: Wire aggregate derivation into artifact assembly** - `788012d` (feat)
4. **Task 4: Add aggregate projection fixture and behavior tests** - `ddd3a06` (test)

## Files Created/Modified

- `crates/parser-core/src/aggregates.rs` - Derives aggregate contributions and namespaced projections from normalized combat events.
- `crates/parser-core/src/lib.rs` - Exports the parser-core aggregate module.
- `crates/parser-core/src/artifact.rs` - Populates successful artifacts with derived aggregates.
- `crates/parser-core/tests/aggregate_projection.rs` - Behavior tests for aggregate projections, bounty exclusions, duplicate-name grouping, and traceability.
- `crates/parser-core/tests/fixtures/aggregate-combat.ocap.json` - Focused OCAP fixture for aggregate projection behavior.

## Decisions Made

- Kept aggregate output per replay; no multi-replay all-time, weekly, squad, or rotation totals are computed in parser-core.
- Used observed entity IDs plus `entity:{id}` / `legacy_name:{observed_name}` compatibility keys only; no canonical player IDs were introduced.
- Kept `side_facts` defaulted for Plan 04-05 while aggregate derivation now populates parser-owned aggregate surfaces.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Adjusted projection-map insert handling for repository deny-lints**
- **Found during:** Task 2 (Produce legacy player, relationship, game-type, and bounty projections)
- **Issue:** `cargo check -p parser-core --all-targets` rejects unused `insert` results and unnecessary `serde_json::Value` qualification under the repository's strict lint settings.
- **Fix:** Consumed map insert return values explicitly and used the imported `Value` type.
- **Files modified:** `crates/parser-core/src/aggregates.rs`
- **Verification:** `cargo check -p parser-core --all-targets`
- **Committed in:** `98a9d54`

---

**Total deviations:** 1 auto-fixed (1 blocking).
**Impact on plan:** No scope change; the fix was required for the planned verification gate.

## Issues Encountered

- Local `gsd-sdk query` is unsupported in this checkout, so execution state and metadata updates were handled manually as allowed by the prompt.
- A parallel `git add` attempt in Task 1 briefly hit `.git/index.lock`; the lock cleared immediately, no file changes were lost, and subsequent staging used sequential git commands.

## Authentication Gates

None.

## Known Stubs

None found in created or modified files.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-core aggregate_projection` - passed
- `cargo test -p parser-core deterministic_output` - passed
- `cargo check -p parser-core --all-targets` - passed
- `cargo fmt --all -- --check` - passed
- `git diff --check` - passed

## Next Phase Readiness

Plan 04-04 can consume the aggregate contribution/projection foundation to add issue #13 vehicle score inputs without changing bounty eligibility or canonical identity boundaries. No parser-owned persistence, queue/storage, API, or UI responsibility was introduced.

## Self-Check: PASSED

- Summary file exists.
- Created aggregate module, aggregate projection test file, and aggregate-combat fixture exist.
- Task commits `ba27233`, `98a9d54`, `788012d`, and `ddd3a06` exist in git history.
- No unexpected file deletions were present in task commits.

---
*Phase: 04-event-semantics-and-aggregates*
*Completed: 2026-04-27*
