---
phase: 07-parallel-and-container-hardening
plan: 01
subsystem: worker-observability
tags: [rust, parser-worker, health-checks, axum, tokio, s3, rabbitmq]

requires:
  - phase: 07-parallel-and-container-hardening
    provides: artifact conditional writes and duplicate-redelivery idempotency from 07-00
provides:
  - Worker probe bind/port/enabled configuration and worker identity resolution
  - Cached `/livez` and `/readyz` HTTP probe server in `parser-worker`
  - Readiness lifecycle transitions for startup, dependency-ready, degraded, draining, and fatal states
  - S3 bucket reachability check before worker readiness
affects: [phase-07, parser-worker, parser-cli, worker-health, container-hardening]

tech-stack:
  added: [axum, serde, tower]
  patterns:
    - Cached health state read by HTTP handlers without AMQP/S3 calls per probe
    - Worker identity from explicit flag/env, then `HOSTNAME`, then literal fallback
    - Dependency outages mark readiness false while liveness stays true unless fatal

key-files:
  created:
    - .planning/phases/07-parallel-and-container-hardening/07-01-SUMMARY.md
    - crates/parser-worker/src/health.rs
    - crates/parser-worker/tests/health.rs
  modified:
    - Cargo.lock
    - crates/parser-worker/Cargo.toml
    - crates/parser-worker/src/config.rs
    - crates/parser-worker/src/lib.rs
    - crates/parser-worker/src/runner.rs
    - crates/parser-worker/src/storage.rs
    - crates/parser-worker/tests/config.rs
    - crates/parser-worker/tests/shutdown.rs
    - crates/parser-cli/src/main.rs
    - crates/parser-cli/tests/worker_command.rs

key-decisions:
  - "Probe handlers return deterministic JSON from cached state only; dependency checks run in the worker lifecycle, not per HTTP request."
  - "S3/RabbitMQ startup dependency failures mark readiness degraded rather than fatal so ordinary dependency outages do not become liveness failures."
  - "Plan verification used equivalent single Cargo filters because Cargo accepts only one positional test filter."
  - "STATE.md and ROADMAP.md were intentionally not updated because the wave orchestrator owns shared tracking writes."

patterns-established:
  - "HealthState exposes explicit mark_starting, mark_ready, mark_degraded, mark_draining, and mark_fatal transitions."
  - "run_until_cancelled starts probes before dependency connections, marks ready only after S3 head_bucket and AMQP connect, and flips draining on cancellation."

requirements-completed: [WORK-09]

duration: 14m41s
completed: 2026-05-02
---

# Phase 07 Plan 01: Worker Probe State and Readiness Summary

**Cached container probes with worker identity, `/livez`/`/readyz` JSON, S3 readiness checks, and shutdown readiness draining**

## Performance

- **Duration:** 14m41s
- **Started:** 2026-05-02T17:48:35Z
- **Completed:** 2026-05-02T18:03:16Z
- **Tasks:** 3
- **Files modified:** 13

## Accomplishments

- Added worker probe config/env/CLI flags and worker ID fallback order.
- Added an `axum` probe server with cached `HealthState`, `ReadinessState`, `/livez`, and `/readyz`.
- Wired probe state into worker startup, S3 `head_bucket` readiness, AMQP connection readiness, Ctrl-C/external shutdown, and runtime error paths.
- Added no-network tests for config, probe JSON/status behavior, readiness transitions, and shutdown draining.

## Task Commits

1. **Task 1: Add probe and worker identity config** - `135bb0c` (`feat`)
2. **Task 2: Implement cached /livez and /readyz HTTP probes** - `b0440a5` (`feat`)
3. **Task 3: Wire probe state into worker startup, dependency checks, and shutdown** - `ba60c22` (`feat`)

## Files Created/Modified

- `Cargo.lock` - Locked new `axum`/HTTP stack dependencies.
- `crates/parser-worker/Cargo.toml` - Added `axum`, `serde`, `tower` test utility, and Tokio `net`.
- `crates/parser-worker/src/config.rs` - Added probe env/defaults, worker ID resolution, validation, and redacted debug fields.
- `crates/parser-worker/src/health.rs` - Added cached health state, probe response snapshots, router, and server spawn helper.
- `crates/parser-worker/src/lib.rs` - Exported the `health` module.
- `crates/parser-worker/src/runner.rs` - Started probes, updated lifecycle state, checked dependencies, and marked draining/fatal/degraded states.
- `crates/parser-worker/src/storage.rs` - Added S3 `head_bucket` readiness check.
- `crates/parser-worker/tests/config.rs` - Covered defaults, env/flag overrides, worker ID fallback, probe validation, and redacted debug output.
- `crates/parser-worker/tests/health.rs` - Covered `/livez`, `/readyz`, degraded liveness, fatal liveness, runner readiness, and shutdown readiness transitions.
- `crates/parser-worker/tests/shutdown.rs` - Kept shutdown drain helpers probe-free while allowing probes in runner.
- `crates/parser-cli/src/main.rs` - Added worker probe and worker ID CLI flags.
- `crates/parser-cli/tests/worker_command.rs` - Covered new worker help flags and env isolation.

## Decisions Made

- Dependency outages use `degraded` readiness, preserving D-07 liveness semantics for ordinary RabbitMQ/S3 outages.
- Probe responses expose only `status`, `ready`, `worker_id`, `state`, and `reason`; no credentials, AMQP URLs, or S3 secret material are returned.
- `run_until_cancelled` owns probe lifecycle cleanup by cancelling the shared `CancellationToken` and awaiting the probe task before returning.
- Shared orchestrator artifacts were left untouched per delegated wave execution instructions.

## Verification Evidence

- `cargo test -p parser-worker config` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `cargo test -p parser-worker worker_identity` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `cargo test -p parser-worker probe` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `cargo test -p parser-worker health` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, including 7/7 health tests.
- `cargo test -p parser-worker runner_readiness` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, including 2/2 runner readiness tests.
- `cargo test -p parser-worker shutdown` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, including the readiness shutdown test and 4/4 shutdown drain tests.
- `cargo test -p parser-cli worker_command` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `cargo clippy -p parser-worker --all-targets -- -D warnings` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `git diff --check` - passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed probe port parser return type**
- **Found during:** Task 1.
- **Issue:** `parse_probe_port` returned `Result<(), WorkerError>` from the validator while the function contract required `Result<u16, WorkerError>`.
- **Fix:** Validate first, then return the parsed `u16`.
- **Files modified:** `crates/parser-worker/src/config.rs`.
- **Verification:** `cargo test -p parser-worker config`, `worker_identity`, and `probe` passed.
- **Committed in:** `135bb0c`.

**2. [Rule 3 - Blocking] Split invalid multi-filter Cargo verification commands**
- **Found during:** Task and plan verification.
- **Issue:** Plan commands such as `cargo test -p parser-worker config worker_identity probe` are invalid Cargo syntax because Cargo accepts one positional test filter.
- **Fix:** Ran equivalent single-filter commands for every requested filter.
- **Files modified:** None.
- **Verification:** All listed single-filter commands passed.
- **Committed in:** N/A, verification-only deviation.

**3. [Rule 3 - Blocking] Reduced Cargo build resource pressure**
- **Found during:** Task 1 and Task 2 verification.
- **Issue:** Default parallel test builds triggered linker `Bus error` and then Cargo failed with `No space left on device` while writing `target` fingerprints.
- **Fix:** Removed only generated Cargo incremental artifacts under `target/*/incremental`, then ran verification with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- **Files modified:** None.
- **Verification:** All plan-level test commands and clippy passed under the constrained build settings.
- **Committed in:** N/A, environment-only deviation.

**4. [Rule 3 - Blocking] Fixed clippy issues surfaced by the parser-worker scoped gate**
- **Found during:** Task 3.
- **Issue:** New health code had redundant closure/double-must-use findings, `check_ready` needed `#[must_use]`, and the touched storage module had an `option_if_let_else` clippy finding.
- **Fix:** Applied clippy-suggested idioms and added the required `#[must_use]`.
- **Files modified:** `crates/parser-worker/src/health.rs`, `crates/parser-worker/src/storage.rs`.
- **Verification:** `cargo clippy -p parser-worker --all-targets -- -D warnings` passed.
- **Committed in:** `ba60c22`.

**5. [Rule 1 - Bug] Updated obsolete shutdown boundary assertion**
- **Found during:** Task 3.
- **Issue:** The previous shutdown test asserted that `runner.rs` must not contain probe/readiness text, which contradicts this plan's requirement to wire probes into the runner.
- **Fix:** Narrowed the assertion to keep the reusable shutdown drain helper module probe-free while allowing runner-owned probe lifecycle code.
- **Files modified:** `crates/parser-worker/tests/shutdown.rs`.
- **Verification:** `cargo test -p parser-worker shutdown` passed.
- **Committed in:** `ba60c22`.

---

**Total deviations:** 5 handled (2 bugs, 3 blocking issues).
**Impact on plan:** No scope expansion beyond WORK-09. Fixes were required to make planned probe behavior compile, verify, and align stale tests with the accepted Phase 7 design.

## Issues Encountered

- Disk space in `/` was exhausted by generated Cargo build artifacts during the new dependency build. Only `target/*/incremental` generated artifacts were removed; no source or planning files were deleted.
- The plan inherited invalid Cargo multi-filter commands; equivalent single-filter commands were used as in 07-00.

## Known Stubs

None. Stub scan of modified files found no `TODO`, `FIXME`, placeholder text, or hardcoded empty UI/data stubs.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required by this plan.

## Next Phase Readiness

Plan 07-02 can build structured worker log taxonomy on top of stable `worker_id` config and cached health state. Plan 07-03 can wire Docker/container smoke to `/livez` and `/readyz` without changing parser-core or parser-contract.

## Self-Check: PASSED

- `07-01-SUMMARY.md` exists.
- `crates/parser-worker/src/health.rs` exists.
- `crates/parser-worker/tests/health.rs` exists.
- Task commit `135bb0c` exists.
- Task commit `b0440a5` exists.
- Task commit `ba60c22` exists.
- No missing created files or commit references found.

---
*Phase: 07-parallel-and-container-hardening*
*Completed: 2026-05-02*
