---
phase: 06-rabbitmq-s3-worker-integration
plan: 02
subsystem: worker-storage
tags: [rust, s3, sha256, worker, artifact-key]

requires:
  - phase: 06-rabbitmq-s3-worker-integration
    provides: worker runtime/config crate and worker message contracts from plans 06-00 and 06-01
provides:
  - Local SHA-256 helpers for raw replay and artifact bytes
  - Deterministic encoded artifact key construction
  - Testable object-store boundary with S3-compatible implementation
  - Artifact write/reuse policy with checksum/size proof and conflict failure
affects: [parser-worker, phase-06-worker-integration, server-2-parser-integration]

tech-stack:
  added: []
  patterns:
    - Local checksum computation is authoritative for raw and artifact bytes
    - Object storage policy is tested through no-network fake stores
    - Deterministic artifact reuse requires exact checksum and size match

key-files:
  created:
    - crates/parser-worker/src/artifact_key.rs
    - crates/parser-worker/src/checksum.rs
    - crates/parser-worker/src/storage.rs
    - crates/parser-worker/tests/artifact_key.rs
    - crates/parser-worker/tests/storage.rs
  modified:
    - crates/parser-worker/src/lib.rs
    - crates/parser-worker/src/error.rs

key-decisions:
  - "Raw replay integrity is verified by computing local SHA-256 from downloaded bytes; S3 object metadata is not trusted as the checksum source."
  - "Artifact keys use normalized prefix, percent-encoded replay_id, and source SHA-256: artifacts/v3/{encoded_replay_id}/{source_sha256}.json."
  - "Existing deterministic artifact objects are reused only when stored byte length and local SHA-256 match the new artifact bytes."

patterns-established:
  - "ObjectStore default methods hold checksum/write policy while concrete stores provide get/put byte primitives."
  - "WorkerFailureKind carries checksum.mismatch and output.artifact_conflict codes for later parse.failed mapping."

requirements-completed: [WORK-02, WORK-03, WORK-04]

duration: 10m08s
completed: 2026-05-02
---

# Phase 06 Plan 02: S3 Storage and Artifact Policy Summary

**S3-compatible worker storage boundary with local SHA-256 verification, encoded deterministic artifact keys, and checksum/size-guarded artifact reuse**

## Performance

- **Duration:** 10m08s
- **Started:** 2026-05-02T13:37:06Z
- **Completed:** 2026-05-02T13:47:14Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added local SHA-256 helpers and source checksum verification for worker-downloaded raw bytes.
- Added deterministic artifact key construction with prefix normalization and safe replay ID encoding.
- Added a testable `ObjectStore` boundary plus `S3ObjectStore` using `endpoint_url`, region, and `force_path_style` from `WorkerConfig`.
- Implemented artifact write/reuse behavior that returns artifact reference, checksum, byte size, and `reused_existing`, and reports mismatched existing objects as `output.artifact_conflict`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add checksum helpers and deterministic artifact key encoding** - `e7dbcde` (feat)
2. **Task 2: Add S3-compatible object storage adapter and artifact write policy** - `785c799` (feat)

## Files Created/Modified

- `crates/parser-worker/src/artifact_key.rs` - Deterministic artifact key builder with encoded replay IDs.
- `crates/parser-worker/src/checksum.rs` - Local SHA-256 checksum and source checksum verification helpers.
- `crates/parser-worker/src/storage.rs` - Object store trait, S3 implementation, raw download checksum, and artifact write/reuse policy.
- `crates/parser-worker/src/error.rs` - Structured worker failure kinds plus storage error context.
- `crates/parser-worker/src/lib.rs` - Exports artifact key, checksum, and storage modules.
- `crates/parser-worker/tests/artifact_key.rs` - Artifact key safety and checksum helper tests.
- `crates/parser-worker/tests/storage.rs` - No-network fake store tests for raw download, checksum mismatch, artifact write/reuse/conflict, and storage errors.

## Decisions Made

- Kept RabbitMQ, processing, and parse.failed message assembly out of this plan; those remain 06-03 and 06-04 scope.
- Used local SHA-256 of exact bytes for raw and artifact checksums; no storage metadata checksum shortcuts were introduced.
- Reused existing artifacts only after reading stored bytes and comparing both byte length and SHA-256.
- Left `.planning/STATE.md` and `.planning/ROADMAP.md` untouched because the orchestrator owns shared tracking writes after wave completion.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split invalid combined Cargo test filter**
- **Found during:** Task 1 verification and final verification
- **Issue:** `cargo test -p parser-worker artifact_key checksum` is not valid Cargo syntax because Cargo accepts only one test filter.
- **Fix:** Ran equivalent targeted checks separately with `cargo test -p parser-worker artifact_key` and `cargo test -p parser-worker checksum`.
- **Files modified:** None
- **Verification:** Both targeted filters passed after Task 1 and again during final verification.
- **Committed in:** N/A - verification-only adjustment

---

**Total deviations:** 1 auto-fixed (1 blocking verification adjustment)
**Impact on plan:** No behavior or architecture change. The requested checksum and artifact-key acceptance criteria were verified with equivalent Cargo commands.

## Issues Encountered

- An extra non-plan `cargo clippy -p parser-worker --all-targets -- -D warnings` attempt was not used as an acceptance gate. It tried to download `system-configuration-sys` under restricted network access and also surfaced existing `parser-contract` clippy findings from prior plan code, outside this plan's modified files. The required plan tests and grep gates passed.

## Known Stubs

None - stub pattern scan found no placeholder/TODO/FIXME or hardcoded empty UI/data stubs in files created or modified by this plan.

## User Setup Required

None - no external service configuration required for this plan. Live S3-compatible runtime still depends on the worker config and AWS credential provider chain from plan 06-01.

## Verification

- `cargo test -p parser-worker artifact_key` - passed, 4 artifact key tests.
- `cargo test -p parser-worker checksum` - passed, 4 checksum-filtered tests.
- `cargo test -p parser-worker storage` - passed, 8 storage tests.
- `! rg -n "ETag|etag" crates/parser-worker/src/storage.rs crates/parser-worker/tests/storage.rs` - passed, no matches.
- `git diff --check` - passed.

## Next Phase Readiness

Plan 06-03 can add RabbitMQ consumer/publisher behavior against the existing worker config and error model. Plan 06-04 can call the storage boundary to download raw bytes, verify job checksums, write deterministic artifacts, and assemble `parse.completed` or `parse.failed` messages.

## Self-Check: PASSED

- Verified summary file exists: `.planning/phases/06-rabbitmq-s3-worker-integration/06-02-SUMMARY.md`.
- Verified key created files exist: `crates/parser-worker/src/artifact_key.rs`, `crates/parser-worker/src/checksum.rs`, `crates/parser-worker/src/storage.rs`, `crates/parser-worker/tests/artifact_key.rs`, `crates/parser-worker/tests/storage.rs`.
- Verified task commits exist in git log: `e7dbcde`, `785c799`.
- Verified `.planning/STATE.md` and `.planning/ROADMAP.md` were not modified by this executor.

---
*Phase: 06-rabbitmq-s3-worker-integration*
*Completed: 2026-05-02*
