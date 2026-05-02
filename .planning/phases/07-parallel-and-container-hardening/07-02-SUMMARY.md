---
phase: 07-parallel-and-container-hardening
plan: 02
subsystem: worker-observability
tags: [rust, parser-worker, structured-logs, telemetry, redaction, rabbitmq, s3]

requires:
  - phase: 07-parallel-and-container-hardening
    provides: worker identity and cached readiness probes from 07-01
provides:
  - Stable worker log event taxonomy and low-cardinality outcome constants
  - Structured worker lifecycle, dependency, job, parse, artifact, result, and ack/nack logs
  - Secret-safe config/log boundary checks for AMQP and AWS credential surfaces
affects: [phase-07, parser-worker, worker-observability, container-smoke]

tech-stack:
  added: []
  patterns:
    - Event names live in `parser_worker::logging` and dynamic identifiers stay in structured fields
    - Worker logs include `worker_id` at runtime boundaries
    - Duration fields are emitted as bounded milliseconds using a shared helper

key-files:
  created:
    - .planning/phases/07-parallel-and-container-hardening/07-02-SUMMARY.md
    - crates/parser-worker/src/logging.rs
    - crates/parser-worker/tests/log_taxonomy.rs
  modified:
    - README.md
    - crates/parser-worker/src/amqp.rs
    - crates/parser-worker/src/health.rs
    - crates/parser-worker/src/lib.rs
    - crates/parser-worker/src/processor.rs
    - crates/parser-worker/src/runner.rs
    - crates/parser-worker/src/storage.rs
    - crates/parser-worker/tests/config.rs

key-decisions:
  - "Result publication logs are emitted only after broker publish confirms succeed."
  - "Ack/nack logs are emitted only after the RabbitMQ delivery action succeeds."
  - "Cargo multi-filter commands in the plan were executed as equivalent single-filter or per-test-target commands."
  - "README's worker Docker example now expects `REPLAY_PARSER_AMQP_URL` from the caller environment instead of showing password-bearing AMQP userinfo."

patterns-established:
  - "Use `event = WORKER_*` constants with low-cardinality `outcome` and `error_type` fields."
  - "Use `worker_id`, `job_id`, `replay_id`, `object_key`, `artifact_key`, and safe AMQP delivery/routing fields as structured fields, not event-name fragments."
  - "Use `config.redacted()` in startup logs and tests to keep probe/worker fields visible while hiding AMQP credentials."

requirements-completed: [WORK-08, WORK-09]

duration: interrupted; completed inline after subagent stop
completed: 2026-05-02
---

# Phase 07 Plan 02: Worker Log Taxonomy and Secret-Safe Observability Summary

**Stable worker operations logs with worker identity, decision-point fields, durations, and redaction boundaries**

## Performance

- **Completed:** 2026-05-02T18:39:36Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Added `parser_worker::logging` with stable event constants and outcome constants.
- Instrumented worker startup, dependency readiness/degraded transitions, readiness changes, job receipt, parse start/finish, artifact write/reuse/conflict, result publish, job completion/failure, delivery ack/nack, and shutdown logs.
- Added `worker_id` to runtime logs and safe AMQP delivery/routing fields to job/ack logs.
- Added duration logging for parse, artifact write, and result publish decisions.
- Added redaction tests proving worker/probe fields remain visible while AMQP credentials stay hidden.
- Removed a password-bearing AMQP URL literal from README so secret grep can enforce the boundary.

## Task Commits

1. **Task 1: Define stable worker log event constants and tests** - `73cc686` (`feat`)
2. **Task 2: Instrument job, artifact, result, ack, and readiness decisions** - `e09d5aa` (`feat`)
3. **Task 3: Add secret-safe log and config boundary checks** - `f92933c` (`test`)

## Files Created/Modified

- `README.md` - Removed the sample password-bearing AMQP URL from the worker Docker example.
- `crates/parser-worker/src/amqp.rs` - Added result publish and delivery ack/nack structured logs.
- `crates/parser-worker/src/health.rs` - Exposed worker identity for shutdown logging.
- `crates/parser-worker/src/lib.rs` - Exported the logging taxonomy module.
- `crates/parser-worker/src/logging.rs` - Added stable event/outcome constants and `duration_ms`.
- `crates/parser-worker/src/processor.rs` - Added parse, artifact, publish outcome, and failure logs.
- `crates/parser-worker/src/runner.rs` - Added startup, dependency, readiness, job receipt, and shutdown logs.
- `crates/parser-worker/src/storage.rs` - Added artifact write/reuse log event selection.
- `crates/parser-worker/tests/config.rs` - Added worker/probe redaction boundary coverage.
- `crates/parser-worker/tests/log_taxonomy.rs` - Added low-cardinality taxonomy checks.

## Decisions Made

- Dynamic identifiers such as job/replay IDs, object keys, artifact keys, routing keys, and delivery tags are structured fields only.
- Log event constants are intentionally stable strings so container smoke tests can assert exact lifecycle events.
- Publish and delivery logs are success-side logs; publish/ack failures remain failure logs with low-cardinality `error_type`.
- README examples avoid credential-bearing AMQP literals to keep documentation inside the same secret boundary as code.

## Verification Evidence

- `cargo fmt --all` - passed.
- `cargo test -p parser-worker --test log_taxonomy` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `cargo test -p parser-worker processor` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, including 15/15 processor tests.
- `cargo test -p parser-worker storage` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, including 9/9 storage tests.
- `cargo test -p parser-worker amqp` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, including AMQP publish and redaction-filtered config tests.
- `cargo test -p parser-worker --test config` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, including 13/13 config tests.
- `cargo clippy -p parser-worker --all-targets -- -D warnings` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `! rg -n "AWS_SECRET_ACCESS_KEY=.*[^*]|AWS_SESSION_TOKEN=.*[^*]|amqp://[^\\s]*:[^*@\\s]+@" README.md crates/parser-worker crates/parser-cli Dockerfile docker-compose.worker-smoke.yml` - passed.
- `git diff --check` - passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Continued inline after subagent interruption**
- **Found during:** Wave 3 execution.
- **Issue:** The delegated 07-02 subagent stopped after Task 1.
- **Fix:** Closed the stopped subagent and completed Tasks 2 and 3 inline.
- **Files modified:** All Task 2/3 files listed above.
- **Verification:** All plan-level gates listed in verification evidence passed.
- **Committed in:** `e09d5aa`, `f92933c`.

**2. [Rule 3 - Blocking] Split invalid Cargo multi-filter commands**
- **Found during:** Task verification.
- **Issue:** Plan commands such as `cargo test -p parser-worker log_taxonomy processor storage amqp config` are invalid Cargo syntax because Cargo accepts one positional test filter.
- **Fix:** Ran equivalent single-filter and per-test-target commands.
- **Files modified:** None.
- **Verification:** Every requested test area passed.
- **Committed in:** N/A, verification-only deviation.

**3. [Rule 3 - Blocking] Fixed pedantic clippy line-count regressions**
- **Found during:** `cargo clippy -p parser-worker --all-targets -- -D warnings`.
- **Issue:** New tracing blocks pushed `process_decoded_job` and `run_with_shutdown` over the local `too_many_lines` threshold.
- **Fix:** Extracted repeated tracing blocks into small helper functions.
- **Files modified:** `crates/parser-worker/src/processor.rs`, `crates/parser-worker/src/runner.rs`.
- **Verification:** `cargo clippy -p parser-worker --all-targets -- -D warnings` passed.
- **Committed in:** `e09d5aa`.

**4. [Rule 2 - Maintainability] Removed password-bearing README AMQP literal**
- **Found during:** Secret grep.
- **Issue:** README contained `amqp://user:pass@...`, which violated the planned secret boundary gate.
- **Fix:** Switched the Docker example to pass `REPLAY_PARSER_AMQP_URL` from the caller environment.
- **Files modified:** `README.md`.
- **Verification:** Secret grep passed.
- **Committed in:** `f92933c`.

---

**Total deviations:** 4 handled (4 blocking/maintainability issues).
**Impact on plan:** No scope expansion beyond WORK-08/WORK-09. Changes remain inside worker observability, config redaction, and documentation examples.

## Issues Encountered

- The plan inherited invalid Cargo multi-filter verification commands; equivalent commands were used and recorded.
- Prior resource pressure remained a risk, so verification used `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.

## Known Stubs

None. No `TODO`, `FIXME`, placeholder, raw replay bytes, or raw artifact JSON logging was added.

## Authentication Gates

None.

## User Setup Required

None.

## Next Phase Readiness

Plan 07-03 can now assert worker logs and probe behavior across multiple containers using stable event names and visible worker identities.

## Self-Check: PASSED

- `07-02-SUMMARY.md` exists.
- `crates/parser-worker/src/logging.rs` exists.
- `crates/parser-worker/tests/log_taxonomy.rs` exists.
- Task commit `73cc686` exists.
- Task commit `e09d5aa` exists.
- Task commit `f92933c` exists.
- No missing created files or commit references found.

---
*Phase: 07-parallel-and-container-hardening*
*Completed: 2026-05-02*
