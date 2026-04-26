---
phase: 02-versioned-output-contract
plan: 02
subsystem: contract
tags: [rust, serde, parser-contract, presence, metadata, identity]
requires:
  - phase: 02-01
    provides: ParseArtifact envelope, source references, RuleId, and temporary metadata/identity sections
provides:
  - Reusable FieldPresence tagged union for present, explicit null, unknown, inferred, and not applicable field states
  - ReplayMetadata contract for observed OCAP top-level metadata fields
  - ObservedEntity and ObservedIdentity contract types without canonical identity fields
affects: [phase-02, phase-03, phase-04, phase-05, server-2-integration]
tech-stack:
  added: []
  patterns: [serde internally tagged presence enum, FieldPresence optional field semantics, observed identity boundary]
key-files:
  created:
    - crates/parser-contract/tests/metadata_identity_contract.rs
  modified:
    - crates/parser-contract/src/presence.rs
    - crates/parser-contract/src/metadata.rs
    - crates/parser-contract/src/identity.rs
key-decisions:
  - "Implemented D-05 and D-06 as one reusable `FieldPresence<T>` internally tagged union for optional contract fields."
  - "Implemented D-07 with explicit inferred values carrying reason, confidence, source, and `rule_id` metadata."
  - "Preserved the parser/server identity boundary by exposing observed replay identity only and excluding canonical player/account fields."
patterns-established:
  - "Optional contract facts use `FieldPresence<T>` rather than bare nullable fields."
  - "Observed identity fields serialize in `snake_case` and remain replay-owned facts."
requirements-completed: [OUT-02, OUT-03, OUT-04]
duration: 3m51s
completed: 2026-04-26
---

# Phase 02 Plan 02: Replay Metadata, Observed Identity, and Presence Semantics Summary

**Explicit presence-state contract for replay metadata and observed identity without canonical matching**

## Performance

- **Duration:** 3m51s
- **Started:** 2026-04-26T05:15:47Z
- **Completed:** 2026-04-26T05:19:38Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Added `FieldPresence<T>` with `present`, `explicit_null`, `unknown`, `inferred`, and `not_applicable` JSON states.
- Added `ReplayMetadata` fields for mission/world/author/player-count/capture/end-frame/time/frame metadata using explicit presence semantics.
- Added observed entity and identity contract types with source entity IDs, side/kind enums, source refs, and no canonical identity fields.
- Added contract tests for missing SteamID, null killer, inferred values, metadata snake_case keys, and observed identity preservation.

## Task Commits

1. **Task 1: Implement reusable explicit presence semantics** - `4e0b5d9` (feat)
2. **Task 2: Define replay metadata from observed OCAP top-level keys** - `002d16f` (feat)
3. **Task 3: Define observed identity without canonical matching** - `5e5fe79` (feat)

## Files Created/Modified

- `crates/parser-contract/src/presence.rs` - Defines `FieldPresence<T>`, `NullReason`, and `UnknownReason`.
- `crates/parser-contract/src/metadata.rs` - Defines `ReplayMetadata`, `ReplayTimeBounds`, and `FrameBounds`.
- `crates/parser-contract/src/identity.rs` - Defines `ObservedEntity`, `ObservedIdentity`, `EntityKind`, and `EntitySide`.
- `crates/parser-contract/tests/metadata_identity_contract.rs` - Tests presence, metadata, and observed identity serialization behavior.

## Decisions Made

- Used one compact internally tagged serde enum for all optional/null/unknown/inferred field states.
- Kept inferred values distinct from observed values by requiring reason, optional confidence, source, and `RuleId`.
- Kept identity contract scoped to observed replay facts; canonical player/account matching remains owned by `server-2`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None - the Plan 01 metadata and identity placeholders were replaced for this plan's scope.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-contract field_presence`
- `cargo test -p parser-contract replay_metadata`
- `cargo test -p parser-contract observed_identity`
- `! rg -n "canonical_player|canonical_id|account_id" crates/parser-contract/src/identity.rs`
- `git diff --check`
- `cargo test -p parser-contract`
- `cargo fmt --all -- --check`

## Next Phase Readiness

Plan 03 can build normalized event and aggregate contribution skeletons on top of the shared `FieldPresence<T>`, `SourceRef`, and `RuleId` model. `.planning/STATE.md` and `.planning/ROADMAP.md` were left for the orchestrator to update after the wave.

## Self-Check: PASSED

- Created summary file exists.
- Modified contract files and the new metadata/identity contract test file exist.
- Task commits `4e0b5d9`, `002d16f`, and `5e5fe79` exist in git history.
- `.planning/STATE.md` and `.planning/ROADMAP.md` remain unmodified by this executor.

---
*Phase: 02-versioned-output-contract*
*Completed: 2026-04-26*
