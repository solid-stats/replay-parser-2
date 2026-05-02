---
phase: 07-parallel-and-container-hardening
status: complete
completed_at: 2026-05-02T19:23:05Z
requirements: [WORK-08, WORK-09]
operator: codex
---

# Phase 07 Human UAT

## Result

Phase 07 is accepted for WORK-08 and WORK-09 with one explicit operator exception: the final `scripts/benchmark-phase5.sh --ci` rerun was stopped by user instruction on 2026-05-02. Phase 7 did not change parser performance paths or artifact shape; benchmark acceptance remains covered by the accepted Phase 05.2/Phase 06 full-corpus evidence recorded in roadmap and state.

## Evidence

| Area | Command / Check | Result |
|------|-----------------|--------|
| Format | `cargo fmt --all -- --check` | Passed |
| Lints | `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| Workspace tests | `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --workspace` | Passed |
| Docs | `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo doc --workspace --no-deps` | Passed |
| Coverage gate | `scripts/coverage-gate.sh --check` | Passed |
| Fault gate | `scripts/fault-report-gate.sh` | Passed with deterministic fault-injection fallback |
| Phase 5 benchmark gate | `scripts/benchmark-phase5.sh --ci` | Skipped after user instruction; accepted Phase 06 full-corpus benchmark evidence remains the reference |
| Worker package | `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p parser-worker` | Passed |
| CLI worker command | `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p parser-cli worker_command` | Passed |
| CLI healthcheck command | `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p parser-cli healthcheck` | Passed |
| Two-worker container smoke | `scripts/worker-smoke.sh` | Passed |
| Core/contract boundary | `! rg -n "lapin|aws_sdk_s3|tokio::signal|axum|hyper|/livez|/readyz|HEALTHCHECK" crates/parser-core crates/parser-contract` | Passed |
| Worker debug boundary | `! rg -n "parse_replay_debug" crates/parser-worker` | Passed |
| Secret boundary | `! rg -n "AWS_SECRET_ACCESS_KEY=.*[^*]|AWS_SESSION_TOKEN=.*[^*]|amqp://[^\\s]*:[^*@\\s]+@" README.md crates/parser-worker crates/parser-cli Dockerfile docker-compose.worker-smoke.yml scripts/worker-smoke.sh` | Passed |
| Operational docs/code markers | `rg -n "worker-a|worker-b|worker_artifact_reused|/livez|/readyz|HEALTHCHECK|Timeweb|s3\\.twcstorage\\.ru" README.md Dockerfile docker-compose.worker-smoke.yml scripts/worker-smoke.sh crates/parser-worker` | Passed |
| Whitespace | `git diff --check` | Passed |

## UAT Checks

### Two-worker Docker Compose smoke

`scripts/worker-smoke.sh` built `replay-parser-2-worker-smoke:latest`, started RabbitMQ, MinIO, `worker-a`, and `worker-b`, waited for both `/livez` and `/readyz`, and ran the live worker smoke through the containerized workers. The smoke also asserted structured log evidence for both worker IDs and the `worker_artifact_reused` event.

### Readiness and liveness probes

`cargo test -p parser-worker` passed the health tests that prove `/readyz` starts unavailable, flips ready only after dependency checks pass, flips unavailable during shutdown drain, and that `/livez` stays available during dependency degradation while returning failure for fatal worker state. `scripts/worker-smoke.sh` proved the same endpoints are reachable from container mode.

### Duplicate artifact reuse and conflict behavior

`cargo test -p parser-worker` passed processor and storage tests for conditional artifact create, duplicate redelivery reuse, matching-object race reuse, conflicting-object failure, and retryable storage failures. The two-worker smoke proved duplicate completed results reuse the same deterministic artifact key and that artifact conflicts publish structured failed results.

### Structured log taxonomy and worker IDs

`cargo test -p parser-worker` passed the log taxonomy tests for stable snake_case low-cardinality event/outcome names. Container smoke grepped logs from `worker-a` and `worker-b` for worker IDs and Phase 7 event names without exposing AMQP passwords or AWS secrets.

### Timeweb S3 compatibility

README and `scripts/worker-smoke.sh` document the Timeweb endpoint `https://s3.twcstorage.ru`, path-style configuration, and `TIMEWEB_S3_SMOKE=1` no-secret capability mode. External Timeweb credentials were not supplied in this session, so live provider validation remains a deployer-run check; local MinIO smoke covered the compare/reuse/conflict fallback behavior used when conditional writes are unsupported or unreliable.

## Out of Scope Preserved

Phase 7 did not add full-corpus multi-worker stress testing, in-process task pools, higher default prefetch, OpenTelemetry exporters, metrics stacks, dashboards, production Kubernetes manifests, public UI/API behavior, PostgreSQL persistence, canonical identity, replay discovery, bounty payout, or yearly stats.
