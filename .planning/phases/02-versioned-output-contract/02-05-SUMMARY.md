---
phase: 02-versioned-output-contract
plan: 05
subsystem: contract
tags: [rust, serde, schemars, json-schema, parser-contract]
requires:
  - phase: 02-versioned-output-contract
    provides: ParseArtifact envelope, source references, failures, schema export, and examples from plans 02-00 through 02-04
provides:
  - Enforceable checksum identity and checksum-unavailable states
  - Status/failure payload invariants in Rust and JSON Schema
  - Non-empty, non-hollow source-reference validation for auditable events, diagnostics, failures, and aggregate contributions
  - Bounded inferred confidence values in Rust and JSON Schema
affects: [phase-03-deterministic-parser-core, server-2-parser-integration, web-generated-api-types]
tech-stack:
  added: []
  patterns:
    - Transparent Rust newtypes for contract scalars that need schema-visible validation
    - Generated schema post-processing for invariants schemars derives cannot express alone
key-files:
  created:
    - .planning/phases/02-versioned-output-contract/02-05-SUMMARY.md
  modified:
    - crates/parser-contract/src/source_ref.rs
    - crates/parser-contract/src/artifact.rs
    - crates/parser-contract/src/failure.rs
    - crates/parser-contract/src/presence.rs
    - crates/parser-contract/src/schema.rs
    - crates/parser-contract/tests/schema_contract.rs
    - schemas/parse-artifact-v1.schema.json
key-decisions:
  - "Checksum identity is sha256-only in v1 and represented through SourceChecksum plus ChecksumValue."
  - "ReplaySource.checksum and ParseFailure checksum/context fields use FieldPresence so input failures can avoid fabricating unavailable facts."
  - "Schema conditionals are injected by parse_artifact_schema() so committed schema remains generated while enforcing status/failure and source-reference invariants."
patterns-established:
  - "Use FieldPresence for contract facts that can be present, unavailable, inferred, explicitly null, or not applicable."
  - "Use SourceRefs for non-empty source-reference arrays where auditability is mandatory."
requirements-completed: [OUT-01, OUT-04, OUT-05, OUT-06, OUT-07]
duration: 7m24s
completed: 2026-04-26
---

# Phase 02 Plan 05: Gap Closure for Contract Invariants Summary

**Machine-checkable parser contract invariants for checksums, failed artifacts, source references, error codes, and inferred confidence**

## Performance

- **Duration:** 7m24s
- **Started:** 2026-04-26T06:23:39Z
- **Completed:** 2026-04-26T06:31:03Z
- **Tasks:** 4
- **Files modified:** 16

## Accomplishments

- Added sha256-only checksum types and explicit checksum availability through `FieldPresence<SourceChecksum>`.
- Added `ParseArtifact::validate_status_payload()` and generated JSON Schema conditionals for failed/non-failed payload rules.
- Replaced auditable source-reference arrays with `SourceRefs`, rejecting empty arrays and hollow source-reference objects in Rust and schema validation.
- Added stricter error-code families for checksum/output stages and bounded inferred confidence values to `0.0..=1.0`.
- Regenerated `schemas/parse-artifact-v1.schema.json` and updated success/failure examples to validate against the hardened contract.

## Task Commits

Implementation was committed as one cohesive gap-closure commit because the four planned invariants share the same contract types, examples, and generated schema:

1. **Tasks 1-4: Contract gap invariants** - `533a8fb` (`feat(02-05)`)

**Plan metadata:** pending docs commit.

## Files Created/Modified

- `crates/parser-contract/src/source_ref.rs` - Adds checksum newtypes, source-reference evidence validation, and `SourceRefs`.
- `crates/parser-contract/src/artifact.rs` - Adds status/failure payload validation.
- `crates/parser-contract/src/failure.rs` - Uses explicit failure context presence and stricter error-code validation.
- `crates/parser-contract/src/presence.rs` - Adds checksum-unavailable unknown reason and bounded `Confidence`.
- `crates/parser-contract/src/schema.rs` - Keeps schema generated from Rust while injecting status/failure and source-reference invariants.
- `crates/parser-contract/tests/*.rs` - Adds Rust and schema regression coverage for the Phase 2 gaps.
- `crates/parser-contract/examples/*.json` - Updates committed success/failure examples to the hardened contract shape.
- `schemas/parse-artifact-v1.schema.json` - Regenerated schema matching `parse_artifact_schema()`.

## Decisions Made

- Kept a single unified `ParseArtifact` envelope and made success/failure validity enforceable instead of splitting artifact shapes.
- Used explicit presence states for checksum and failure context so missing input bytes or source cause are represented truthfully.
- Required non-empty source-reference collections only on fields that are audit-critical in this phase; entity source refs remain a plain vector for now.
- Used generated-schema post-processing rather than hand-maintaining schema constraints outside Rust.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Coupled task boundary] Consolidated implementation commit**
- **Found during:** Tasks 1-4
- **Issue:** The planned task boundaries all touched the same contract types and generated schema, so splitting by task would create intermediate states with broken examples or schema tests.
- **Fix:** Implemented and verified the coupled changes as one cohesive commit while preserving all task-level acceptance criteria.
- **Files modified:** Contract source, examples, tests, and schema files listed above.
- **Verification:** Full plan validation passed.
- **Committed in:** `533a8fb`

---

**Total deviations:** 1 auto-fixed (1 coupled task boundary)
**Impact on plan:** No product or contract scope changed. The commit shape differs from the requested per-task granularity, but all planned invariants and verification gates passed.

## Issues Encountered

None.

## Verification

Passed:

- `cargo test -p parser-contract source_ref_contract_checksum`
- `cargo test -p parser-contract artifact_envelope`
- `cargo test -p parser-contract failure_contract`
- `cargo test -p parser-contract source_ref_contract`
- `cargo test -p parser-contract field_presence_inferred_confidence`
- `cargo test -p parser-contract schema_contract_gap_regression`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p parser-contract --example export_schema > /tmp/parse-artifact-v1.schema.json`
- `cmp /tmp/parse-artifact-v1.schema.json schemas/parse-artifact-v1.schema.json`
- Success and failure example `jq` checks
- `git diff --check`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 2 contract artifacts are hardened enough for phase-level verification. Phase 3 can build parser-core output against these contract invariants without inventing checksums, emitting hollow source references, or producing ambiguous failed artifacts.

---
*Phase: 02-versioned-output-contract*
*Completed: 2026-04-26*
