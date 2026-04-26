---
phase: 02-versioned-output-contract
plan: 04
subsystem: contract
tags: [rust, serde, schemars, jsonschema, parser-contract, parse-failure, schema]
requires:
  - phase: 02-03
    provides: SourceRef, RuleId, normalized event skeleton, and aggregate contribution references
provides:
  - Structured ParseFailure contract with stage, retryability, namespaced error codes, and source evidence
  - Generated parse artifact JSON Schema committed at schemas/parse-artifact-v1.schema.json
  - Success and failure ParseArtifact examples validated against the committed schema
  - README handoff documenting the contract crate and validation commands
affects: [phase-02, phase-03, phase-05, phase-06, server-2-integration, web-api-coordination]
tech-stack:
  added: [jsonschema 0.46.2]
  patterns: [schemars generated schema export, draft2020-12 example validation, unified success/failure artifact envelope]
key-files:
  created:
    - crates/parser-contract/examples/export_schema.rs
    - crates/parser-contract/examples/parse_failure.v1.json
    - crates/parser-contract/tests/failure_contract.rs
    - crates/parser-contract/tests/schema_contract.rs
    - schemas/parse-artifact-v1.schema.json
  modified:
    - Cargo.lock
    - README.md
    - crates/parser-contract/Cargo.toml
    - crates/parser-contract/examples/parse_artifact_success.v1.json
    - crates/parser-contract/src/failure.rs
    - crates/parser-contract/src/schema.rs
key-decisions:
  - "Schemars 1.2.1 exposes generated root schemas as schemars::Schema, so parse_artifact_schema returns that public type."
  - "jsonschema 0.46.2 is used as a dev-dependency with default features disabled because tests validate local committed artifacts only."
  - "Failure examples remain full ParseArtifact envelopes with empty data sections and a populated failure object."
patterns-established:
  - "Schema drift tests compare committed schema bytes against freshly generated export_schema output."
  - "Contract example tests deserialize examples into ParseArtifact and validate them against the committed JSON Schema."
requirements-completed: [OUT-01, OUT-02, OUT-03, OUT-04, OUT-05, OUT-06, OUT-07]
duration: 8m47s
completed: 2026-04-26
---

# Phase 02 Plan 04: Structured Failures and Schema Validation Summary

**Machine-checkable ParseArtifact success/failure contract generated from Rust types**

## Performance

- **Duration:** 8m47s
- **Started:** 2026-04-26T05:32:59Z
- **Completed:** 2026-04-26T05:41:46Z
- **Tasks:** 4
- **Files modified:** 11

## Accomplishments

- Replaced the placeholder failure type with `ParseFailure`, `ParseStage`, `Retryability`, and validated namespaced `ErrorCode` values.
- Added `parse_artifact_schema()` and `export_schema` so `schemas/parse-artifact-v1.schema.json` is generated from Rust contract types and checked for drift.
- Populated success and failure examples, deserialized both into `ParseArtifact`, and validated both against the committed schema.
- Updated README to document `crates/parser-contract`, schema export, `cargo test -p parser-contract`, AI+GSD workflow, and `server-2`/`web` boundaries.

## Task Commits

1. **Task 1: Implement structured ParseFailure and retryability** - `83d5c94` (feat)
2. **Task 2: Generate JSON Schema from Rust contract types** - `59c2b29` (feat)
3. **Task 3: Validate committed success and failure examples against schema** - `0dad181` (feat)
4. **Task 4: Update README and run final contract validation** - `157d9ce` (docs)

## Files Created/Modified

- `crates/parser-contract/src/failure.rs` - Defines parse stages, retryability, validated error codes, and structured failure payloads.
- `crates/parser-contract/src/schema.rs` - Exposes `parse_artifact_schema()` using `schemars::schema_for!(ParseArtifact)`.
- `crates/parser-contract/examples/export_schema.rs` - Prints pretty JSON schema to stdout.
- `schemas/parse-artifact-v1.schema.json` - Committed generated ParseArtifact schema.
- `crates/parser-contract/examples/parse_artifact_success.v1.json` - Concrete success artifact with replay metadata, one observed entity, one event, and one aggregate contribution.
- `crates/parser-contract/examples/parse_failure.v1.json` - Full failed artifact with structured `json.decode` failure details.
- `crates/parser-contract/tests/failure_contract.rs` - Covers failure serialization, retryability/stage values, and error code validation.
- `crates/parser-contract/tests/schema_contract.rs` - Covers schema existence/drift, example deserialization, and schema validation.
- `crates/parser-contract/Cargo.toml` and `Cargo.lock` - Add the local-only `jsonschema` dev-dependency.
- `README.md` - Documents the implemented contract crate, schema commands, workflow, and ownership boundaries.

## Decisions Made

- Used `schemars::Schema` instead of the older `schemars::schema::RootSchema` because the installed `schemars` 1.2.1 public API returns `Schema`.
- Disabled default `jsonschema` features to avoid HTTP/file resolver and TLS dependencies for local committed-example validation.
- Kept parser-level `failed` examples distinct from parser-level `skipped`; legacy aggregate skip reasons remain diagnostics for later phases.

## Pre-Wave Note

The orchestrator's `verify.key-links` miss for `schemas/parse-artifact-v1.schema.json` before this plan was expected. Task 2 generated and committed the schema from Rust types, and the final verification confirmed fresh export output is byte-identical to the committed schema.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None - empty arrays/objects in the failure example are intentional data-section values for a failed `ParseArtifact`, and empty diagnostics/extensions in examples are valid contract states.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-contract failure_contract`
- `cargo run -p parser-contract --example export_schema > /tmp/parse-artifact-v1.schema.json`
- `cmp /tmp/parse-artifact-v1.schema.json schemas/parse-artifact-v1.schema.json`
- `cargo test -p parser-contract schema_contract`
- `jq -e '.status == "success" and (.entities | length) == 1 and (.events | length) == 1 and (.aggregates.contributions | length) == 1' crates/parser-contract/examples/parse_artifact_success.v1.json`
- `jq -e '.status == "failed" and .failure.error_code == "json.decode" and .failure.retryability == "not_retryable"' crates/parser-contract/examples/parse_failure.v1.json`
- `rg -n "crates/parser-contract|cargo test -p parser-contract|export_schema|AI agents using the GSD workflow|server-2|web" README.md`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `git diff --check`

## Next Phase Readiness

Phase 3 can consume the stable `ParseArtifact` skeleton, explicit presence model, source refs, aggregate contribution refs, structured failures, and generated schema. Phase 5 can build CLI schema export around the existing `export_schema` example, and Phase 6 can map worker `parse.failed` messages to `ParseFailure` without changing the parser-owned contract.

Orchestrator-owned `.planning/STATE.md` and `.planning/ROADMAP.md` were not modified by this executor.

## Self-Check: PASSED

- Created summary file exists.
- Generated schema, failure example, and contract test files exist.
- Task commits `83d5c94`, `59c2b29`, `0dad181`, and `157d9ce` exist in git history.
- `.planning/STATE.md` and `.planning/ROADMAP.md` remain unmodified by this executor.

---
*Phase: 02-versioned-output-contract*
*Completed: 2026-04-26*
