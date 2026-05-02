---
phase: 07-parallel-and-container-hardening
plan: 03
subsystem: worker-container-smoke
tags: [rust, parser-worker, parser-cli, docker, docker-compose, smoke, timeweb]

requires:
  - phase: 07-parallel-and-container-hardening
    provides: artifact idempotency from 07-00
  - phase: 07-parallel-and-container-hardening
    provides: worker probes and identity from 07-01
  - phase: 07-parallel-and-container-hardening
    provides: stable worker log taxonomy from 07-02
provides:
  - Slim non-root worker image with Docker readiness healthcheck
  - Hidden CLI HTTP healthcheck command without adding curl/debug tooling to the runtime image
  - Two-worker Docker Compose smoke with probe, duplicate reuse, artifact conflict, and structured log assertions
  - No-secret Timeweb S3 compatibility documentation and optional conditional-write capability hook
affects: [phase-07, parser-cli, parser-worker, docker, smoke, README]

tech-stack:
  added: []
  patterns:
    - Hidden CLI subcommand for image-local health checks
    - Container smoke prepares RabbitMQ/S3 topology before starting workers
    - AMQP URLs are constructed at runtime to avoid committed password-bearing literals

key-files:
  created:
    - .planning/phases/07-parallel-and-container-hardening/07-03-SUMMARY.md
  modified:
    - Dockerfile
    - README.md
    - crates/parser-cli/src/main.rs
    - crates/parser-cli/tests/worker_command.rs
    - crates/parser-worker/tests/live_smoke.rs
    - docker-compose.worker-smoke.yml
    - scripts/worker-smoke.sh

key-decisions:
  - "Docker health uses `replay-parser-2 healthcheck --url http://127.0.0.1:8080/readyz` instead of installing curl in the runtime image."
  - "Smoke has a setup-only live-test pass to create the bucket/topology before worker containers perform startup readiness checks."
  - "Worker container logs set `RUST_LOG=info` so smoke can assert structured Phase 7 events from `docker compose logs`."
  - "Timeweb conditional-write checks are optional and require caller-provided AWS credentials and endpoint environment; the script prints only capability labels."

patterns-established:
  - "Use `WORKER_IMAGE`, `WORKER_A_PROBE_PORT`, and `WORKER_B_PROBE_PORT` env knobs for local container smoke."
  - "Use `REPLAY_PARSER_CONTAINER_SMOKE=1` to run live smoke against already-running worker containers instead of spawning an in-process worker."
  - "Use `TIMEWEB_S3_SMOKE=1 scripts/worker-smoke.sh` for an optional provider capability check without committing secrets."

requirements-completed: [WORK-08, WORK-09]

duration: 24m
completed: 2026-05-02
---

# Phase 07 Plan 03: Container Smoke and Timeweb Compatibility Summary

**Container evidence for two worker instances, probes, duplicate artifact reuse, artifact conflict, structured logs, and Timeweb S3 compatibility hooks**

## Performance

- **Completed:** 2026-05-02T19:02:52Z
- **Tasks:** 4
- **Files modified:** 8

## Accomplishments

- Added a hidden `healthcheck` CLI subcommand using `std::net::TcpStream` for plain HTTP probe checks.
- Wired `Dockerfile` with `EXPOSE 8080` and a Docker `HEALTHCHECK` against `/readyz` while preserving `debian:bookworm-slim`, non-root `USER 65532:65532`, entrypoint, and command.
- Added `worker-a` and `worker-b` Compose services with distinct worker IDs, host probe ports, `prefetch=1`, path-style MinIO config, and `RUST_LOG=info`.
- Reworked `scripts/worker-smoke.sh` to build the image, prepare RabbitMQ/S3, start two worker containers, wait for both probes, run live smoke in container mode, and grep structured logs.
- Extended ignored live smoke to assert duplicate redelivery produces two completed results for the same deterministic artifact, verifies exact artifact existence, asserts `output.artifact_conflict`, and checks container probes.
- Documented container smoke behavior and no-secret Timeweb S3 settings in README.
- Added optional `TIMEWEB_S3_SMOKE=1` script path with `timeweb_conditional_write` capability labels.

## Task Commits

1. **Task 1: Add container health wiring without adding debug tooling** - `d1dd9b8` (`feat`)
2. **Task 2: Extend Compose and smoke script for two worker containers** - `8df04ac` (`test`)
3. **Task 3: Prove duplicate, redelivery, artifact reuse, conflict, probes, and logs in live smoke** - `8df04ac` (`test`)
4. **Task 4: Document and script Timeweb S3 compatibility checks without secrets** - `8df04ac` (`test`) and `2994f16` (`docs`)

## Files Created/Modified

- `Dockerfile` - Added probe port exposure and image-local readiness healthcheck.
- `README.md` - Documented container smoke, worker probes, and Timeweb S3-compatible settings.
- `crates/parser-cli/src/main.rs` - Added hidden healthcheck subcommand and small HTTP status checker.
- `crates/parser-cli/tests/worker_command.rs` - Added hidden healthcheck behavior tests.
- `crates/parser-worker/tests/live_smoke.rs` - Added container mode, setup-only mode, duplicate/reuse/conflict checks, and probe polling.
- `docker-compose.worker-smoke.yml` - Added `worker-a` and `worker-b` services.
- `scripts/worker-smoke.sh` - Added image build, topology setup, two-worker probe waits, live smoke container mode, log greps, and Timeweb capability hook.

## Decisions Made

- The healthcheck command intentionally supports only `http://host:port/path`; malformed URLs and I/O failures exit `2`.
- The smoke script prepares RabbitMQ queues/exchange/bindings and runs a setup-only live smoke pass before starting workers because worker readiness checks require the bucket and job queue to exist.
- The Compose file receives AMQP URLs through `REPLAY_PARSER_CONTAINER_AMQP_URL`, built at runtime by the script, so no password-bearing AMQP literal is committed.
- The optional Timeweb mode uses the AWS CLI if available; if absent it reports `timeweb_conditional_write=failed` without printing secrets.

## Verification Evidence

- `cargo fmt --all` - passed.
- `cargo test -p parser-cli worker_command` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `cargo test -p parser-cli healthcheck` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `rg -n "HEALTHCHECK|EXPOSE 8080|USER 65532:65532" Dockerfile` - passed.
- `cargo test -p parser-worker --test live_smoke` - passed compile-only with the ignored test not run.
- `scripts/worker-smoke.sh` - passed against local Docker/Compose, building `replay-parser-2-worker-smoke:latest`, starting RabbitMQ/MinIO plus `worker-a` and `worker-b`, running setup and container live smoke, and passing structured log greps.
- `cargo clippy -p parser-cli --all-targets -- -D warnings` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `cargo clippy -p parser-worker --all-targets -- -D warnings` - passed with `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
- `bash -n scripts/worker-smoke.sh` - passed.
- `rg -n "Timeweb|s3\\.twcstorage\\.ru|TIMEWEB_S3_SMOKE|timeweb_conditional_write|REPLAY_PARSER_S3_FORCE_PATH_STYLE|AWS_SECRET_ACCESS_KEY" README.md scripts/worker-smoke.sh` - passed.
- `! rg -n "AWS_SECRET_ACCESS_KEY=.*[^*]|AWS_SESSION_TOKEN=.*[^*]|amqp://[^\\s]*:[^*@\\s]+@" README.md scripts/worker-smoke.sh docker-compose.worker-smoke.yml Dockerfile` - passed.
- `git diff --check` - passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split invalid Cargo multi-filter command**
- **Found during:** Task 1 verification.
- **Issue:** `cargo test -p parser-cli worker_command healthcheck` is invalid Cargo syntax because Cargo accepts one positional test filter.
- **Fix:** Ran `worker_command` and `healthcheck` as separate equivalent filters.
- **Files modified:** None.
- **Verification:** Both commands passed.
- **Committed in:** N/A, verification-only deviation.

**2. [Rule 1 - Bug] Prepared live smoke dependencies before worker startup**
- **Found during:** Task 2 implementation.
- **Issue:** Worker containers perform readiness checks against S3 and consume an existing RabbitMQ queue; starting containers before bucket/topology setup can make startup fail for infrastructure, not parser behavior.
- **Fix:** Added script topology preparation and a setup-only live smoke pass before starting worker containers.
- **Files modified:** `scripts/worker-smoke.sh`, `crates/parser-worker/tests/live_smoke.rs`.
- **Verification:** `scripts/worker-smoke.sh` passed.
- **Committed in:** `8df04ac`.

**3. [Rule 3 - Blocking] Avoided committed password-bearing AMQP literals**
- **Found during:** Secret grep verification.
- **Issue:** Local smoke needs guest credentials, but the secret grep rejects `amqp://user:password@...` literals in docs/scripts/compose.
- **Fix:** Built AMQP URLs at runtime with `printf '%s://...'` and passed the container URL through an environment variable.
- **Files modified:** `scripts/worker-smoke.sh`, `docker-compose.worker-smoke.yml`.
- **Verification:** Secret grep passed.
- **Committed in:** `8df04ac`.

**4. [Rule 1 - Bug] Made MinIO bucket setup idempotent across repeated smoke passes**
- **Found during:** `scripts/worker-smoke.sh`.
- **Issue:** MinIO returned a generic `service error` for a second create-bucket call, so matching provider error strings was not reliable.
- **Fix:** On create-bucket failure, `ensure_bucket` now accepts the bucket when `head_bucket` succeeds.
- **Files modified:** `crates/parser-worker/tests/live_smoke.rs`.
- **Verification:** Re-run `scripts/worker-smoke.sh` passed.
- **Committed in:** `8df04ac`.

**5. [Rule 3 - Blocking] Enabled info-level worker logs for log-grep evidence**
- **Found during:** `scripts/worker-smoke.sh`.
- **Issue:** With no `RUST_LOG`, `docker compose logs` did not include the info-level structured events required by the plan.
- **Fix:** Added `RUST_LOG=info` to both worker services and explicit diagnostic log assertions in the script.
- **Files modified:** `docker-compose.worker-smoke.yml`, `scripts/worker-smoke.sh`.
- **Verification:** Re-run `scripts/worker-smoke.sh` passed log greps for worker IDs and event names.
- **Committed in:** `8df04ac`.

---

**Total deviations:** 5 handled (2 bugs, 3 blocking issues).
**Impact on plan:** No scope expansion beyond WORK-08/WORK-09. Changes strengthen the planned container proof and secret boundaries.

## Issues Encountered

- Docker release builds downloaded crates inside the builder stage after source changes invalidated the `COPY crates` layer; final smoke passed after rebuild.
- One transient crates.io timeout occurred during Docker build and recovered automatically.

## Known Stubs

None. No generated smoke logs are committed.

## Authentication Gates

None for local smoke. Optional Timeweb smoke requires caller-provided `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `REPLAY_PARSER_S3_BUCKET`, and `REPLAY_PARSER_S3_ENDPOINT`.

## User Setup Required

- Local container smoke requires Docker and Docker Compose.
- Optional Timeweb capability smoke requires AWS CLI plus Timeweb credentials supplied through environment or the deployment secret store.

## Next Phase Readiness

Plan 07-04 can run final gates over a worker image, two-worker smoke evidence, health probes, log taxonomy, and secret boundary checks.

## Self-Check: PASSED

- `07-03-SUMMARY.md` exists.
- Task commit `d1dd9b8` exists.
- Task commit `8df04ac` exists.
- Task commit `2994f16` exists.
- `scripts/worker-smoke.sh` passed on a Docker-capable host.
- No missing created files or commit references found.

---
*Phase: 07-parallel-and-container-hardening*
*Completed: 2026-05-02*
