---
phase: 06-rabbitmq-s3-worker-integration
plan: 01
subsystem: worker-runtime
tags: [rust, tokio, rabbitmq, s3, clap, config]

requires:
  - phase: 06-rabbitmq-s3-worker-integration
    provides: worker request/result contracts, schemas, and examples from plan 06-00
provides:
  - parser-worker workspace crate for runtime adapter concerns
  - Env-backed and CLI-overridable worker configuration with redacted debug output
  - replay-parser-2 worker subcommand that delegates to parser-worker through a Tokio runtime
affects: [parser-worker, parser-cli, phase-06-worker-integration]

tech-stack:
  added: [aws-config, aws-sdk-s3, futures-util, lapin, percent-encoding, tokio, tokio-util, tracing, tracing-subscriber]
  patterns:
    - Transport/runtime dependencies are confined to parser-worker
    - Worker CLI flags override environment/default configuration
    - WorkerConfig Debug output redacts AMQP credentials

key-files:
  created:
    - crates/parser-worker/Cargo.toml
    - crates/parser-worker/src/lib.rs
    - crates/parser-worker/src/config.rs
    - crates/parser-worker/src/error.rs
    - crates/parser-worker/src/runner.rs
    - crates/parser-worker/tests/config.rs
    - crates/parser-cli/tests/worker_command.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/parser-cli/Cargo.toml
    - crates/parser-cli/src/main.rs

key-decisions:
  - "parser-worker owns RabbitMQ/S3/Tokio runtime dependencies; parser-core and parser-contract remain transport-free."
  - "The worker command validates required S3 bucket configuration before starting the runtime and does not print AMQP credentials in errors."
  - "Phase 6 keeps prefetch defaulted to 1; higher concurrency remains an explicit override and Phase 7 safety work."

patterns-established:
  - "WorkerConfig::from_env_and_overrides provides flag-over-env-over-default merge behavior without mutating process environment in tests."
  - "CLI worker integration builds a Tokio runtime only for the worker subcommand."

requirements-completed: [WORK-01, WORK-02, WORK-07]

duration: 12 min
completed: 2026-05-02
---

# Phase 06 Plan 01: Worker Runtime and Config Summary

**RabbitMQ/S3 worker crate shell with validated env/flag configuration, redacted startup diagnostics, and a thin `replay-parser-2 worker` CLI delegate**

## Performance

- **Duration:** 12 min
- **Started:** 2026-05-02T13:20:03Z
- **Completed:** 2026-05-02T13:31:42Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added the `parser-worker` workspace crate with runtime adapter dependencies and typed worker errors.
- Implemented `WorkerConfig` covering AMQP names, S3 bucket/region/endpoint/path-style, artifact prefix, and prefetch with env defaults and CLI overrides.
- Added `replay-parser-2 worker` as a thin CLI delegate that builds a Tokio runtime only for worker mode.
- Added behavior tests for config defaults/overrides/validation/redaction and worker command help/error output.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add parser-worker crate and runtime shell** - `8e5600c` (feat)
2. **Task 2: Implement worker config and CLI worker subcommand** - `be39f80` (feat)

## Files Created/Modified

- `Cargo.toml` - Adds `crates/parser-worker` as a workspace member.
- `Cargo.lock` - Locks worker runtime dependency graph.
- `crates/parser-worker/Cargo.toml` - Defines worker crate and RabbitMQ/S3/Tokio dependencies.
- `crates/parser-worker/src/lib.rs` - Exports config, error, and runner modules.
- `crates/parser-worker/src/error.rs` - Defines typed `WorkerError` variants for config, AMQP, S3, serialization, checksum, parser metadata, and publish confirmation failures.
- `crates/parser-worker/src/config.rs` - Implements validated env/default/override worker config and redacted debug output.
- `crates/parser-worker/src/runner.rs` - Provides the async worker runtime entrypoint shell with config validation and redacted startup logging.
- `crates/parser-worker/tests/config.rs` - Covers config defaults, env overrides, CLI overrides, missing bucket, zero prefetch, and debug redaction.
- `crates/parser-cli/Cargo.toml` - Adds `parser-worker` and Tokio runtime dependency for worker mode.
- `crates/parser-cli/src/main.rs` - Adds the `Worker` subcommand, worker error handling, and runtime delegation.
- `crates/parser-cli/tests/worker_command.rs` - Covers worker help flags and secret-safe missing-bucket failure.

## Decisions Made

- Kept AMQP/S3/Tokio runtime concerns outside `parser-core` and `parser-contract`; boundary greps confirm no transport references were added there.
- Used a custom `Debug` implementation for `WorkerConfig` so ordinary debug output is redacted by default.
- Kept HTTP health/readiness endpoints out of scope for this plan and Phase 6 plan 01.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added minimal config module during runtime shell task**
- **Found during:** Task 1 (Add parser-worker crate and runtime shell)
- **Issue:** The task required `lib.rs` to export `config` and `runner::run(config: WorkerConfig)`, which cannot compile without a `config.rs` module even though Task 1's file list omitted it.
- **Fix:** Added a minimal `config.rs` shell in Task 1, then expanded it fully in Task 2.
- **Files modified:** `crates/parser-worker/src/config.rs`, `crates/parser-worker/src/runner.rs`
- **Verification:** `cargo check -p parser-worker --all-targets` passed.
- **Committed in:** `8e5600c`

**2. [Rule 3 - Blocking] Made worker CLI tests match the plan's Cargo filter**
- **Found during:** Task 2 (Implement worker config and CLI worker subcommand)
- **Issue:** `cargo test -p parser-cli worker_command` filters by test name. The initial test file name alone was insufficient, causing the plan command to run zero worker tests.
- **Fix:** Named both tests with `worker_command_` so the exact plan command executes the intended assertions.
- **Files modified:** `crates/parser-cli/tests/worker_command.rs`
- **Verification:** `cargo test -p parser-cli worker_command` now runs and passes 2 tests.
- **Committed in:** `be39f80`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes made the planned build and verification commands meaningful without changing architecture or adding Phase 7 scope.

## Issues Encountered

- Initial `cargo check` needed to download new worker dependencies and failed under the network-restricted sandbox. It was rerun with approved Cargo network access and then passed.

## Known Stubs

None - stub pattern scan found no placeholder/TODO/FIXME or hardcoded empty UI/data stubs in files created or modified by this plan.

## User Setup Required

None for this plan. Runtime deployments will need `REPLAY_PARSER_S3_BUCKET` or `--s3-bucket` once the worker is used against real storage.

## Next Phase Readiness

Plan 06-02 can add S3 checksum/artifact storage behavior inside `parser-worker` using the validated configuration and preserving the parser-core/parser-contract transport boundary. Phase 7 health/readiness scope remains unimplemented.

## Verification

- `cargo check -p parser-worker --all-targets` - passed
- `cargo test -p parser-worker config` - passed, 6 config tests
- `cargo test -p parser-cli worker_command` - passed, 2 worker CLI tests
- `rg -n "health|readiness|HEALTHCHECK|/health" crates/parser-worker crates/parser-cli` - no matches
- `rg -n "lapin|aws_sdk_s3|tokio::signal" crates/parser-core crates/parser-contract` - no matches
- `git diff --check` - passed

## Self-Check: PASSED

- Verified summary file exists: `.planning/phases/06-rabbitmq-s3-worker-integration/06-01-SUMMARY.md`.
- Verified key created files exist: `crates/parser-worker/src/config.rs`, `crates/parser-worker/src/error.rs`, `crates/parser-worker/src/runner.rs`, `crates/parser-worker/tests/config.rs`, `crates/parser-cli/tests/worker_command.rs`.
- Verified task commits exist in git log: `8e5600c`, `be39f80`.
- Verified `.planning/STATE.md` and `.planning/ROADMAP.md` were not modified by this executor.

---
*Phase: 06-rabbitmq-s3-worker-integration*
*Completed: 2026-05-02*
