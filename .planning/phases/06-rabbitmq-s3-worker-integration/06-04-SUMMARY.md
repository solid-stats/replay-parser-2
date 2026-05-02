---
phase: 06-rabbitmq-s3-worker-integration
plan: 04
subsystem: worker-processor
tags: [rust, tokio, rabbitmq, s3, shutdown, parser-contract]

requires:
  - phase: 06-rabbitmq-s3-worker-integration
    provides: S3 storage boundary from Plan 06-02 and RabbitMQ confirmed publish/ack policy from Plan 06-03
provides:
  - Shared parser-core public minimal artifact path used by CLI and worker output
  - End-to-end worker job processor for parse.completed and parse.failed outcomes
  - SHA-256 verification before parsing raw replay bytes
  - Ack/nack decisions tied to confirmed durable result publication
  - Graceful shutdown drain for one in-flight worker delivery
affects: [parser-worker, parser-cli, parser-core, server-2-parser-integration, phase-07-worker-hardening]

tech-stack:
  added: []
  patterns:
    - Public parse artifact sanitization lives in parser-core to prevent CLI/worker drift
    - Worker processor uses fakeable storage and publisher boundaries for no-network tests
    - Handled failures publish parse.failed and ack only after confirmation; publish failure requeues
    - Shutdown cancellation stops new consumption while allowing the in-flight delivery to finish through ack/nack

key-files:
  created:
    - crates/parser-worker/src/processor.rs
    - crates/parser-worker/tests/processor.rs
    - crates/parser-worker/src/shutdown.rs
    - crates/parser-worker/tests/shutdown.rs
  modified:
    - crates/parser-core/src/lib.rs
    - crates/parser-core/src/artifact.rs
    - crates/parser-core/tests/parser_core_api.rs
    - crates/parser-cli/src/main.rs
    - crates/parser-cli/tests/parse_command.rs
    - crates/parser-worker/src/lib.rs
    - crates/parser-worker/src/amqp.rs
    - crates/parser-worker/src/runner.rs

key-decisions:
  - "CLI default output and worker success output both use parser_core::public_parse_replay, serialized as minified JSON plus a trailing newline."
  - "Worker verifies the job SHA-256 against downloaded raw bytes before parser-core sees the replay payload."
  - "Handled job, checksum, parser, and reportable output failures publish parse.failed and then ack; result publication failure is the NackRequeue path."
  - "Graceful shutdown handles ctrl-c by stopping new delivery polling while draining one in-flight job through result publication and ack/nack."

patterns-established:
  - "ResultPublisher trait keeps durable outcome confirmation testable without a live RabbitMQ broker."
  - "ObjectStore trait keeps raw download and artifact writes testable without S3."
  - "Shutdown drain tests use fake delivery streams to prove cancellation behavior without adding HTTP probe scope."

requirements-completed: [WORK-01, WORK-02, WORK-03, WORK-04, WORK-05, WORK-06, WORK-07]

duration: 18m16s
completed: 2026-05-02
---

# Phase 06 Plan 04: Worker Processor and Shutdown Drain Summary

**Worker job processor with checksum-before-parse, shared CLI artifact bytes, confirmed result publication, and one-job shutdown drain**

## Performance

- **Duration:** 18m16s
- **Started:** 2026-05-02T14:05:10Z
- **Completed:** 2026-05-02T14:23:26Z
- **Tasks:** 3
- **Files modified:** 12

## Accomplishments

- Moved public minimal artifact sanitization into `parser-core` and updated CLI default parsing to use the shared `public_parse_replay` path.
- Added `parser_worker::processor::process_job_body`, covering malformed jobs, unsupported contract versions, checksum mismatch, parser failures, artifact conflicts, S3 output failures, completed results, and publish-failure requeue.
- Connected the worker runner to live S3 and RabbitMQ clients, logs structured lifecycle events, listens for ctrl-c, and drains one in-flight delivery before shutdown completes.

## Task Commits

Each task was committed atomically:

1. **Task 1: Share public minimal artifact bytes between CLI and worker** - `979c3c3` (feat)
2. **Task 2: Implement job processor success and handled-failure paths** - `4d3dc58` (feat)
3. **Task 3: Add graceful shutdown drain and connect runner** - `dcaefeb` (feat)

## Files Created/Modified

- `crates/parser-core/src/artifact.rs` - Adds public artifact sanitization that strips replay metadata source provenance for public output.
- `crates/parser-core/src/lib.rs` - Exports `public_parse_artifact` and `public_parse_replay`.
- `crates/parser-core/tests/parser_core_api.rs` - Covers public parser helper behavior.
- `crates/parser-cli/src/main.rs` - Uses shared parser-core public parsing for default parse and compare artifact paths.
- `crates/parser-cli/tests/parse_command.rs` - Proves CLI default bytes match parser-core public minified JSON plus newline.
- `crates/parser-worker/src/processor.rs` - Implements end-to-end job processing, checksum gate, artifact writes, result publication, and ack/nack decisions.
- `crates/parser-worker/tests/processor.rs` - Adds fake storage/publisher tests for success, handled failures, and publish failure requeue.
- `crates/parser-worker/src/amqp.rs` - Exposes the consumer stream to the runner.
- `crates/parser-worker/src/runner.rs` - Builds live clients, consumes deliveries, calls the processor, applies delivery actions, and handles ctrl-c shutdown.
- `crates/parser-worker/src/shutdown.rs` - Adds fakeable drain helpers for cancellation behavior tests.
- `crates/parser-worker/tests/shutdown.rs` - Verifies no second job after cancellation, in-flight drain, NackRequeue during shutdown, and no HTTP probe scope.
- `crates/parser-worker/src/lib.rs` - Exports `processor` and `shutdown`.

## Decisions Made

- Centralized public artifact bytes in parser-core so the CLI and worker cannot drift on replay metadata source stripping.
- Kept debug artifacts CLI-only; the worker calls `public_parse_replay` and never `parse_replay_debug`.
- Treated all reportable processor failures as durable outcomes when `parse.failed` publication is confirmed; only result publication failure requeues the original delivery.
- Kept shutdown scoped to signal handling and in-flight drain only. Phase 7 health/readiness scope was not introduced.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Routed invalid artifact-key failures through parse.failed**
- **Found during:** Task 2 (Implement job processor success and handled-failure paths)
- **Issue:** The planned processor output-failure handling covered artifact conflicts and S3 writes, but an invalid deterministic artifact key would otherwise surface as a raw worker error without a durable result outcome.
- **Fix:** Converted artifact-key construction failures into `parse.failed` with `output.artifact_key`; confirmed publication returns `Ack`, while publication failure returns `NackRequeue`.
- **Files modified:** `crates/parser-worker/src/processor.rs`, `crates/parser-worker/tests/processor.rs`
- **Verification:** `processor_invalid_artifact_key_should_publish_failed` passed as part of `cargo test -p parser-worker processor`.
- **Committed in:** `4d3dc58` (part of Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The auto-fix preserves the plan's durable-outcome ack policy and does not add product scope.

## Issues Encountered

- Rust 2024 workspace lints rejected a redundant `Future` import and unused `tokio::spawn` / `fetch_add` results during Task 3 verification. These were fixed before the Task 3 commit and verified with `cargo check -p parser-worker --all-targets`.

## Known Stubs

None - stub pattern scan found no placeholder/TODO/FIXME or hardcoded empty UI/data stubs in files created or modified by this plan. One scan match was a Rust formatting placeholder string in `amqp.rs`, not a stub.

## User Setup Required

None - no new external service configuration required for this plan. Live worker execution still uses the RabbitMQ/S3 configuration established by earlier Phase 06 plans.

## Verification

- `cargo test -p parser-core public_parse` - passed, including 2 public parser-core tests.
- `cargo test -p parser-cli parse_command` - passed, 12 parse-command tests.
- `cargo test -p parser-worker processor` - passed, 12 processor tests.
- `cargo test -p parser-worker shutdown` - passed, 4 shutdown tests.
- `cargo check -p parser-worker --all-targets` - passed.
- `rg -n "public_parse_replay|public_parse_artifact|parse_command_default_output_should_match_public_parser_core_bytes|processor_worker_artifact_bytes_should_match_cli_default_minified_bytes|checksum\\.mismatch|NackRequeue|worker_shutdown_requested|worker_shutdown_complete|worker_job_received" crates/parser-core crates/parser-cli crates/parser-worker` - passed.
- `rg -n "health|readiness|HEALTHCHECK|/health" crates/parser-worker` - passed with no matches.
- `git diff --check` - passed.

## Next Phase Readiness

Plan 06-05 can run phase-level integration gates and documentation updates on top of a worker that now processes jobs end-to-end. Phase 7 can still own worker health/readiness, retry backoff, and multi-worker hardening without this plan pre-empting that scope.

## Self-Check: PASSED

- Verified summary file exists: `.planning/phases/06-rabbitmq-s3-worker-integration/06-04-SUMMARY.md`.
- Verified key created files exist: `crates/parser-worker/src/processor.rs`, `crates/parser-worker/tests/processor.rs`, `crates/parser-worker/src/shutdown.rs`, `crates/parser-worker/tests/shutdown.rs`.
- Verified task commits exist in git: `979c3c3`, `4d3dc58`, `dcaefeb`.
- Verified `.planning/STATE.md` and `.planning/ROADMAP.md` have no working-tree modifications.
- Verified no accidental tracked-file deletions in task commits.

---
*Phase: 06-rabbitmq-s3-worker-integration*
*Completed: 2026-05-02*
