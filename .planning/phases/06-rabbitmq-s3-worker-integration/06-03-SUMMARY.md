---
phase: 06-rabbitmq-s3-worker-integration
plan: 03
subsystem: worker-amqp
tags: [rust, rabbitmq, lapin, publisher-confirms, manual-ack]

requires:
  - phase: 06-rabbitmq-s3-worker-integration
    provides: worker message contracts, worker config, and storage policy from plans 06-00 through 06-02
provides:
  - RabbitMQ client setup with separate consume and publish channels
  - Manual consumer configuration with default prefetch 1
  - Confirmed result publishing for parse.completed and parse.failed
  - Explicit ack-after-confirm delivery policy with publish-failure requeue
affects: [parser-worker, phase-06-worker-integration, server-2-parser-integration]

tech-stack:
  added: []
  patterns:
    - RabbitMQ result publication requires publisher confirms and mandatory routing
    - Delivery acknowledgements are modeled as explicit actions derived from confirmed durable outcomes
    - AMQP behavior is covered by no-network fake confirm and fake acker tests

key-files:
  created:
    - crates/parser-worker/src/amqp.rs
    - crates/parser-worker/tests/amqp.rs
  modified:
    - crates/parser-worker/src/lib.rs
    - crates/parser-worker/src/error.rs

key-decisions:
  - "RabbitMQ consumption uses no_ack=false and basic_qos with WorkerConfig prefetch defaulting to 1."
  - "Result publication uses config-backed result exchange/routing keys, mandatory publishes, JSON content type, and accepts only Ack(None) confirms."
  - "Confirmed parse.completed and parse.failed outcomes ack the input delivery; result publish failure maps to NackRequeue."

patterns-established:
  - "PreparedResultPublish allows routing/body/content-type behavior to be tested without a live RabbitMQ broker."
  - "DeliveryAction and PublishedOutcome keep ack policy separate from job-processing logic."

requirements-completed: [WORK-01, WORK-05, WORK-06, WORK-07]

duration: 8m47s
completed: 2026-05-02
---

# Phase 06 Plan 03: RabbitMQ Consumer, Confirms, and Ack Policy Summary

**RabbitMQ adapter with manual prefetch-1 consumption, confirmed parse result publishing, and ack-after-confirm delivery policy**

## Performance

- **Duration:** 8m47s
- **Started:** 2026-05-02T13:50:44Z
- **Completed:** 2026-05-02T13:59:31Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `parser_worker::amqp` with `RabbitMqClient::connect`, separate consume/publish channels, `basic_qos`, `basic_consume` with `no_ack: false`, and `confirm_select`.
- Added `publish_completed` and `publish_failed` paths that serialize JSON result messages, use config-backed routing keys, set `application/json`, publish with `mandatory: true`, and require broker `Ack(None)` confirms.
- Added `DeliveryAction`, `PublishedOutcome`, and a testable `DeliveryAcker` adapter so confirmed completed/failed outcomes ack, while publish failures requeue.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add RabbitMQ connection, consume, and result publish helpers** - `1a771b3` (feat)
2. **Task 2: Model ack decisions for confirmed outcomes and publish failures** - `ab1ed3b` (feat)

## Files Created/Modified

- `crates/parser-worker/src/amqp.rs` - RabbitMQ connection setup, publish preparation, confirm validation, delivery action model, and acker adapters.
- `crates/parser-worker/tests/amqp.rs` - No-network AMQP tests for default routing keys, confirm failures, mandatory returns, prefetch default, and ack policy.
- `crates/parser-worker/src/error.rs` - Adds `WorkerFailureKind::RabbitMqPublish` with `output.rabbitmq_publish` output-stage classification.
- `crates/parser-worker/src/lib.rs` - Exports the new `amqp` module.

## Decisions Made

- Kept AMQP behavior in `parser-worker`; `parser-core` and `parser-contract` remain transport-free.
- Treated `Confirmation::NotRequested`, `Nack`, and mandatory returned messages as `output.rabbitmq_publish` failures.
- Modeled invalid job JSON as a handled failed-result path for ack policy: confirmed `parse.failed` acks, while failure to publish that outcome requeues.
- Left `.planning/STATE.md` and `.planning/ROADMAP.md` untouched because the orchestrator owns shared tracking writes after wave completion.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Workspace lints required `RabbitMqClient` to implement `Debug`, integration test crates to have crate-level docs, and `lapin` ack/nack boolean results to be handled explicitly. These were fixed before task commits.
- A concurrent targeted Cargo test briefly waited on the shared target directory lock; both plan-level test commands completed successfully.

## Known Stubs

None - stub pattern scan found no placeholder/TODO/FIXME or hardcoded empty UI/data stubs in files created or modified by this plan. One scan match was a Rust `format!` placeholder string in `amqp.rs`, not a stub.

## User Setup Required

None - no external service configuration required for this plan. Live RabbitMQ usage still depends on worker config values established in Plan 06-01.

## Verification

- `cargo test -p parser-worker amqp` - passed, including 5 AMQP-filtered tests plus the existing AMQP-redaction config test.
- `cargo test -p parser-worker ack_policy` - passed, 6 ack-policy tests.
- `rg -n "basic_qos|confirm_select|mandatory: true|DeliveryAction" crates/parser-worker/src crates/parser-worker/tests` - passed.
- `rg -n "DeliveryAction|NackRequeue|BasicNackOptions|requeue: true|BasicAckOptions" crates/parser-worker/src/amqp.rs crates/parser-worker/tests/amqp.rs` - passed.
- `git diff --check` - passed.

## Next Phase Readiness

Plan 06-04 can wire job processing on top of the storage boundary and this AMQP adapter. It can publish structured `parse.completed`/`parse.failed` outcomes, then apply `DeliveryAction::Ack` only after confirmed publication or `DeliveryAction::NackRequeue` when outcome publication fails.

## Self-Check: PASSED

- Verified summary file exists: `.planning/phases/06-rabbitmq-s3-worker-integration/06-03-SUMMARY.md`.
- Verified key created files exist: `crates/parser-worker/src/amqp.rs`, `crates/parser-worker/tests/amqp.rs`.
- Verified task commits exist in git: `1a771b3`, `ab1ed3b`.
- Verified `.planning/STATE.md` and `.planning/ROADMAP.md` have no working-tree modifications.
- Verified no accidental tracked-file deletions in task commits.

---
*Phase: 06-rabbitmq-s3-worker-integration*
*Completed: 2026-05-02*
