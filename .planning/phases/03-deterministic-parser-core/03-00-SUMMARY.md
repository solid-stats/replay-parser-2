---
phase: 03-deterministic-parser-core
plan: 00
subsystem: parser-contract
tags: [rust, serde, schemars, parser-contract, observed-identity, source-refs]

requires:
  - phase: 02-versioned-output-contract
    provides: Versioned ParseArtifact, FieldPresence states, RuleId, SourceRef, and SourceRefs
provides:
  - Typed observed entity name and class fields
  - Auditable entity compatibility hints for connected-player backfill and duplicate-slot same-name behavior
  - Non-empty SourceRefs on ObservedEntity in Rust and JSON Schema
  - Updated success example and schema regression tests for entity source evidence
affects: [03-deterministic-parser-core, parser-core, server-2-parser-contract-consumers]

tech-stack:
  added: []
  patterns:
    - Typed parser contract extension instead of unstructured extension payloads
    - SourceRefs for mandatory entity provenance
    - Compatibility behavior represented as hints, not merged observed identity

key-files:
  created:
    - .planning/phases/03-deterministic-parser-core/03-00-SUMMARY.md
  modified:
    - crates/parser-contract/src/identity.rs
    - crates/parser-contract/tests/metadata_identity_contract.rs
    - crates/parser-contract/tests/schema_contract.rs
    - crates/parser-contract/examples/parse_artifact_success.v1.json
    - schemas/parse-artifact-v1.schema.json

key-decisions:
  - "ObservedEntity now carries observed_name and observed_class as FieldPresence<String> so parser-core can populate unit, vehicle, and static weapon facts without generic extensions."
  - "ObservedEntity.source_refs now uses SourceRefs, making empty entity provenance invalid in Rust and schema validation."
  - "Legacy connected-player and duplicate-slot behavior is represented as typed EntityCompatibilityHint data without canonical identity or entity merging."

patterns-established:
  - "Contract fields for observed facts stay typed and auditable."
  - "Legacy aggregate-compatibility behavior is preserved as provenance hints for later parser-core and aggregate phases."

requirements-completed: [OUT-08, PARS-04, PARS-05, PARS-06, PARS-07]

duration: 11m44s
completed: 2026-04-27
---

# Phase 3 Plan 00: Contract Extension for Observed Entity Facts Summary

**Typed observed entity name/class fields with non-empty provenance and compatibility hints for Phase 3 parser-core population**

## Performance

- **Duration:** 11m44s
- **Started:** 2026-04-27T04:39:09Z
- **Completed:** 2026-04-27T04:50:53Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Extended `ObservedEntity` with `observed_name`, `observed_class`, `compatibility_hints`, and non-empty `SourceRefs`.
- Added typed `EntityCompatibilityHint` and `EntityCompatibilityHintKind` for connected-player backfill and duplicate-slot same-name compatibility.
- Updated contract tests, success example, and generated schema so empty entity source refs are rejected.
- Confirmed no canonical player, canonical ID, account ID, or user ID field was added to the contract.

## Task Commits

1. **Task 1: Extend observed entity contract fields and compatibility hints** - `431a162` (feat)
2. **Task 2: Update contract tests, schema example, and generated schema** - `cf18985` (test)

## Files Created/Modified

- `crates/parser-contract/src/identity.rs` - Adds typed observed entity fields, compatibility hints, and `SourceRefs`.
- `crates/parser-contract/tests/metadata_identity_contract.rs` - Covers entity name/class serialization and duplicate-slot hint serialization.
- `crates/parser-contract/tests/schema_contract.rs` - Adds schema regressions for entity source refs and compatibility hint shape.
- `crates/parser-contract/examples/parse_artifact_success.v1.json` - Includes new entity fields in the committed success example.
- `schemas/parse-artifact-v1.schema.json` - Regenerated schema for the updated contract shape.

## Verification

| Command | Result |
| --- | --- |
| `rg -n "observed_name|observed_class|EntityCompatibilityHint|SourceRefs" crates/parser-contract/src/identity.rs` | PASS |
| `rg -n "canonical_player|canonical_id|account_id|user_id" crates/parser-contract/src/identity.rs` | PASS - no matches |
| `cargo test -p parser-contract metadata_identity_contract` | PASS - plan command exits 0; Cargo filter runs 0 tests |
| `cargo test -p parser-contract --test metadata_identity_contract` | PASS - 11 tests passed |
| `cargo test -p parser-contract schema_contract` | PASS - 14 schema tests passed |
| `cargo run -p parser-contract --example export_schema > /tmp/parse-artifact-v1.schema.json` | PASS |
| `cmp /tmp/parse-artifact-v1.schema.json schemas/parse-artifact-v1.schema.json` | PASS |
| `jq -e '.entities[0].observed_name.state == "present" and .entities[0].observed_class.state == "unknown" and (.entities[0].compatibility_hints \| type == "array")' crates/parser-contract/examples/parse_artifact_success.v1.json` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |

## Decisions Made

- Followed the Phase 3 plan contract shape exactly and kept parser-core population for later plans.
- Used typed compatibility hints rather than merging duplicate-slot same-name entities, preserving raw observed entity IDs for audit and later aggregate projection.
- Treated the actual legacy parser path on this machine, `/home/alexandr/Projects/SolidGames/sg-replay-parser`, as the read-first equivalent for missing `/home/afgan0r/Projects/SolidGames/replays-parser`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired broken pinned Rust 1.95.0 toolchain components**
- **Found during:** Task 2 verification
- **Issue:** `cargo`, host `rust-std`, and `cargo-clippy` were missing or unusable in the pinned `1.95.0-x86_64-unknown-linux-gnu` toolchain, blocking the required cargo gates.
- **Fix:** Reinstalled the local rustup `cargo`, `rust-std`, and `clippy` components for the pinned toolchain.
- **Files modified:** None in repository; local rustup toolchain only.
- **Verification:** Cargo tests, schema export, fmt check, and clippy gate passed afterward.
- **Committed in:** N/A - environment repair only.

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Verification was unblocked without changing repository scope or parser behavior.

## Issues Encountered

- The read-first legacy paths under `/home/afgan0r/Projects/SolidGames/replays-parser` were absent in this environment. The equivalent old parser files were read from `/home/alexandr/Projects/SolidGames/sg-replay-parser`.
- The plan command `cargo test -p parser-contract metadata_identity_contract` exits successfully but filters out all tests. To preserve the plan gate and still verify behavior, the real integration target was also run with `cargo test -p parser-contract --test metadata_identity_contract`.

## Known Stubs

None. Stub scan found only intentional contract/example/schema `null`, `[]`, and `{}` values used to represent explicit field states, empty diagnostics/projections, or negative schema fixtures.

## Threat Flags

None. The plan changed parser contract shape only; it added no network endpoints, auth paths, file access patterns, persistence, queue messages, or canonical identity fields beyond the plan's threat model.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03-01 can build parser-core against a contract that now supports observed entity names/classes, mandatory entity source evidence, and typed legacy compatibility hints.

## Self-Check: PASSED

- Found `.planning/phases/03-deterministic-parser-core/03-00-SUMMARY.md`.
- Found `crates/parser-contract/src/identity.rs`.
- Found `schemas/parse-artifact-v1.schema.json`.
- Found task commits `431a162` and `cf18985` in git history.

---
*Phase: 03-deterministic-parser-core*
*Completed: 2026-04-27*
