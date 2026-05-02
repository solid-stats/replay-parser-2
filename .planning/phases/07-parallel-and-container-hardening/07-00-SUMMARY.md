---
phase: 07-parallel-and-container-hardening
plan: 00
subsystem: worker-storage
tags: [rust, parser-worker, s3, idempotency, rabbitmq]

requires:
  - phase: 06-rabbitmq-s3-worker-integration
    provides: worker processor, S3 object-store boundary, deterministic artifact keys, RabbitMQ result publication
provides:
  - Conditional create-if-absent artifact write path for S3-compatible storage
  - Compare/reuse/conflict fallback for existing deterministic artifact keys
  - Processor duplicate-redelivery idempotency tests
  - Processor artifact-conflict failed-outcome tests
affects: [phase-07, parser-worker, worker-storage, worker-processor]

tech-stack:
  added: []
  patterns:
    - ObjectStore conditional artifact create outcome with provider fallback
    - Exact SHA-256 plus byte-size artifact reuse checks
    - Processor redelivery proof through fake storage and publisher boundaries

key-files:
  created:
    - .planning/phases/07-parallel-and-container-hardening/07-00-SUMMARY.md
  modified:
    - crates/parser-worker/src/storage.rs
    - crates/parser-worker/tests/storage.rs
    - crates/parser-worker/tests/processor.rs

key-decisions:
  - "S3 artifact writes now attempt If-None-Match create-if-absent before comparing existing artifact bytes."
  - "Provider conditional-write unsupported responses fall back to get-then-put while preserving existing-object compare/reuse/conflict behavior."
  - "Duplicate redelivery republishes a normal parse.completed result when the existing deterministic artifact matches."
  - "STATE.md and ROADMAP.md were intentionally not updated because the wave orchestrator owns shared tracking writes."

patterns-established:
  - "ArtifactPutOutcome maps storage races into Created, AlreadyExists, and UnsupportedConditionalWrite before artifact comparison."
  - "Processor tests assert observable completed/failed messages and DeliveryAction, not private processor internals."

requirements-completed: [WORK-08]

duration: 9m54s
completed: 2026-05-02
---

# Phase 07 Plan 00: Artifact Race and Duplicate Redelivery Summary

**Race-safe deterministic artifact writes using S3 conditional create, exact checksum/size reuse, and processor-level duplicate redelivery proof**

## Performance

- **Duration:** 9m54s
- **Started:** 2026-05-02T17:33:09Z
- **Completed:** 2026-05-02T17:43:03Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `ObjectStore::put_artifact_bytes_if_absent` and `ArtifactPutOutcome` so artifact writes try conditional create before falling back.
- Implemented S3 `if_none_match("*")` artifact puts and mapped 412/409 existing-key races to compare/reuse/conflict behavior.
- Kept Timeweb/S3-compatible uncertainty covered by an unsupported-conditional fallback path that still compares existing bytes.
- Proved duplicate redelivery publishes a second `parse.completed` with matching artifact key/checksum/size.
- Proved deterministic artifact conflicts publish `parse.failed` with `output.artifact_conflict` at `ParseStage::Output` and still ack.

## Task Commits

1. **Task 1: Add conditional artifact create and provider fallback** - `e0e5ff1` (`feat`)
2. **Task 2: Prove duplicate redelivery idempotency through the processor** - `a1f5bb2` (`test`)

## Files Created/Modified

- `crates/parser-worker/src/storage.rs` - Added conditional artifact write outcome, S3 `If-None-Match` put path, provider fallback classification, and shared exact existing-artifact comparison.
- `crates/parser-worker/tests/storage.rs` - Added fake conditional put behavior and tests for new writes, matching races, conflicting races, and unsupported-provider fallback.
- `crates/parser-worker/tests/processor.rs` - Added fake artifact write outcome recording plus duplicate-redelivery and artifact-conflict processor tests.
- `.planning/phases/07-parallel-and-container-hardening/07-00-SUMMARY.md` - Execution summary and verification record.

## Decisions Made

- Reused the existing worker message contract unchanged; no `ParseJobMessage`, `ParseCompletedMessage`, or `ParseFailedMessage` fields changed.
- Kept the deterministic artifact key format unchanged: `artifacts/v3/{encoded_replay_id}/{source_sha256}.json`.
- Treated Cargo multi-filter commands in the plan as intent and executed equivalent single-filter commands because Cargo accepts only one positional test filter.
- Did not update shared orchestrator artifacts (`STATE.md`, `ROADMAP.md`, `REQUIREMENTS.md`) per delegated execution instructions.

## Verification Evidence

- `cargo test -p parser-worker storage` - passed, including 9/9 storage tests.
- `cargo test -p parser-worker conditional_put` - passed, 1/1 targeted storage test.
- `cargo test -p parser-worker artifact_write_existing_match` - passed, 1/1 targeted storage test.
- `cargo test -p parser-worker artifact_write_existing_conflict` - passed, 1/1 targeted storage test.
- `cargo test -p parser-worker processor_duplicate_redelivery` - passed, 1/1 targeted processor test.
- `cargo test -p parser-worker processor_artifact_conflict` - passed, 1/1 targeted processor test.
- `cargo test -p parser-worker processor` - passed, 15/15 processor tests.
- `git diff --check` - passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split invalid multi-filter Cargo verification commands**
- **Found during:** Task 1, Task 2, and plan-level verification.
- **Issue:** Commands such as `cargo test -p parser-worker storage conditional_put artifact_write_existing_match artifact_write_existing_conflict` are invalid Cargo syntax because Cargo accepts one positional test filter.
- **Fix:** Ran equivalent single-filter commands for each requested filter and also ran broad `storage`/`processor` filters.
- **Files modified:** None.
- **Verification:** All equivalent targeted and broad test commands listed above passed.
- **Committed in:** N/A, verification-only deviation.

---

**Total deviations:** 1 handled blocking verification issue.
**Impact on plan:** No behavioral scope change. The intended test coverage was executed despite the invalid literal command syntax.

## Issues Encountered

- Cargo rejected the plan's multi-filter test commands with `unexpected argument`; equivalent single-filter commands were used.

## Known Stubs

None. Stub scan of modified files found no `TODO`, `FIXME`, placeholder text, or hardcoded empty UI/data stubs.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 07-00 leaves the worker ready for Phase 7 follow-up plans on HTTP probes, structured worker logs, and container smoke hardening. Multi-instance artifact races now have no-network coverage at the storage and processor boundaries.

## Self-Check: PASSED

- `07-00-SUMMARY.md` exists.
- Task commit `e0e5ff1` exists.
- Task commit `a1f5bb2` exists.
- No missing created files or commit references found.

---
*Phase: 07-parallel-and-container-hardening*
*Completed: 2026-05-02*
