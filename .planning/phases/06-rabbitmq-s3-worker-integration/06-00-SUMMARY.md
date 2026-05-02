---
phase: 06-rabbitmq-s3-worker-integration
plan: 00
subsystem: worker-contract
tags: [rust, serde, schemars, rabbitmq, s3, json-schema]

requires:
  - phase: 05.2-minimal-artifact-and-performance-acceptance
    provides: minimal v3 parser artifact and accepted Phase 6 readiness policy
provides:
  - Typed RabbitMQ parse job contract in parser-contract
  - Typed parse.completed and parse.failed result contracts
  - Generated parse job and parse result JSON Schemas
  - Valid request/completed/failed worker message examples
affects: [parser-contract, phase-06-worker-integration, server-2-parser-integration]

tech-stack:
  added: []
  patterns:
    - Rust-type-driven JSON Schema generation for worker message contracts
    - FieldPresence-backed failed-message identifiers for malformed jobs

key-files:
  created:
    - crates/parser-contract/src/worker.rs
    - crates/parser-contract/tests/worker_message_contract.rs
    - crates/parser-contract/examples/export_worker_schemas.rs
    - crates/parser-contract/examples/parse_job.v1.json
    - crates/parser-contract/examples/parse_completed.v1.json
    - crates/parser-contract/examples/parse_failed.v1.json
    - schemas/parse-job-v1.schema.json
    - schemas/parse-result-v1.schema.json
  modified:
    - crates/parser-contract/src/lib.rs
    - crates/parser-contract/src/schema.rs
    - crates/parser-contract/tests/schema_contract.rs

key-decisions:
  - "Worker request/result JSON contracts live in parser-contract as typed Rust structs with generated JSON Schema."
  - "parse.completed carries artifact references and checksum/size proof, not inline parse artifacts."
  - "parse.failed uses FieldPresence for job/replay/object identifiers so malformed jobs can still produce structured failures."

patterns-established:
  - "Worker schema freshness: committed schemas must byte-match schema generation from Rust types."
  - "Unsupported contract versions produce schema-stage, not-retryable unsupported.contract_version failures."

requirements-completed: [WORK-01, WORK-05, WORK-06]

duration: 9 min
completed: 2026-05-02
---

# Phase 06 Plan 00: Worker Request/Result Contract Summary

**Schema-backed RabbitMQ worker job/result envelopes with artifact-reference success messages and structured failed-message identifiers**

## Performance

- **Duration:** 9 min
- **Started:** 2026-05-02T13:06:51Z
- **Completed:** 2026-05-02T13:15:28Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added `ParseJobMessage`, `ParseCompletedMessage`, `ParseFailedMessage`, `ParseResultMessage`, and `ArtifactReference` under `parser-contract`.
- Added schema helpers plus an exporter for `parse-job-v1.schema.json` and `parse-result-v1.schema.json`.
- Added examples and schema tests proving worker messages deserialize, validate, reject missing job fields, preserve malformed-job unknown states, and reject inline artifact payloads.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add typed worker message structs** - `0935438` (feat)
2. **Task 2: Generate and validate worker message schemas and examples** - `6641a7a` (feat)

## Files Created/Modified

- `crates/parser-contract/src/worker.rs` - Worker request/result message structs, constructors, result kind, and unsupported-version helper.
- `crates/parser-contract/src/lib.rs` - Exports the worker contract module.
- `crates/parser-contract/src/schema.rs` - Adds parse job/result schema generation.
- `crates/parser-contract/tests/worker_message_contract.rs` - Covers required fields, serialization, failed-message FieldPresence states, and unsupported-version failure semantics.
- `crates/parser-contract/tests/schema_contract.rs` - Adds worker schema freshness, example validation, and inline artifact rejection tests.
- `crates/parser-contract/examples/export_worker_schemas.rs` - Writes committed worker schemas to a requested output directory.
- `crates/parser-contract/examples/parse_job.v1.json` - Valid parse job example.
- `crates/parser-contract/examples/parse_completed.v1.json` - Valid completed result example with artifact bucket/key, checksum, size, and parser info.
- `crates/parser-contract/examples/parse_failed.v1.json` - Valid failed result example for unsupported contract version.
- `schemas/parse-job-v1.schema.json` - Committed request schema.
- `schemas/parse-result-v1.schema.json` - Committed result schema.

## Decisions Made

- Kept worker message contracts in `parser-contract` only; no worker runtime, S3 client, RabbitMQ client, PostgreSQL, replay discovery, canonical identity, API, or UI behavior was added.
- Made the unsupported-version helper fallible instead of using `expect` in production code, preserving workspace lint rules.
- Left `.planning/STATE.md` and `.planning/ROADMAP.md` untouched for this executor; the orchestrator owns shared tracking writes after wave completion.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split invalid combined Cargo test filter**
- **Found during:** Task 2 verification
- **Issue:** `cargo test -p parser-contract schema_contract worker_message_contract` is not valid Cargo syntax because Cargo accepts only one `TESTNAME` filter.
- **Fix:** Ran equivalent targeted checks separately with `cargo test -p parser-contract schema_contract` and `cargo test -p parser-contract worker_message_contract`, then ran `cargo test -p parser-contract` as an additional package-level verification.
- **Files modified:** None
- **Verification:** Both targeted tests and the full parser-contract suite passed.
- **Committed in:** N/A - verification-only adjustment

---

**Total deviations:** 1 auto-fixed (1 blocking verification adjustment)
**Impact on plan:** No code scope change. The requested behavior and acceptance criteria were verified with equivalent Cargo commands.

## Issues Encountered

- A parallel `git add` attempt briefly hit `.git/index.lock`; the lock disappeared after the concurrent commands exited, and staging continued sequentially. No repository files were changed by this issue.
- Existing `.planning/STATE.md` was already dirty from orchestration state before implementation. It was intentionally not staged or committed.

## Known Stubs

None - stub pattern scan found no placeholder/TODO/FIXME or hardcoded empty UI/data stubs in files created or modified by this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 06-01 can build the worker crate and CLI worker subcommand against these typed request/result contracts and committed schemas.

## Self-Check: PASSED

- Verified summary file exists: `.planning/phases/06-rabbitmq-s3-worker-integration/06-00-SUMMARY.md`.
- Verified key created files exist: `crates/parser-contract/src/worker.rs`, `crates/parser-contract/tests/worker_message_contract.rs`, `schemas/parse-job-v1.schema.json`, `schemas/parse-result-v1.schema.json`.
- Verified task commits exist in git log: `0935438`, `6641a7a`.
- Verified no accidental tracked-file deletions in task commits.

---
*Phase: 06-rabbitmq-s3-worker-integration*
*Completed: 2026-05-02*
