---
phase: 07
artifact: research
status: complete
researched_at: 2026-05-03
---

# Phase 07 Research - Parallel and Container Hardening

## User Constraints

### Locked Decisions From CONTEXT.md

- D-01: Phase 7 proves horizontal worker safety with multiple worker instances or containers, while each instance keeps conservative prefetch `1`. Do not add an in-process task pool or raise default prefetch.
- D-02 and D-03: deterministic artifact writes must be race-safe and idempotent. Prefer conditional create-if-absent writes when supported. Existing matching artifacts are reused; differing bytes are structured conflicts. Redelivered duplicate jobs should republish the durable completed/failed outcome instead of becoming parser failures.
- D-04: proof depth is no-network tests plus a Docker Compose smoke with 2+ worker instances, RabbitMQ, MinIO, duplicate/redelivery, artifact reuse, and artifact conflict behavior. Full-corpus stress is out of scope.
- D-05 through D-08: add configurable HTTP `/livez` and `/readyz` endpoints. Readiness means valid config, active AMQP consumer/publisher path, reachable S3-compatible storage, and no shutdown request. Liveness means process/event-loop health and fatal startup state only. Shutdown flips readiness false immediately and lets the in-flight delivery drain.
- D-09 through D-12: lock a stable, secret-safe JSON log taxonomy with `worker_id`, job/replay/object identifiers, safe AMQP routing/delivery fields, artifact key, stage, outcome, duration, retryability, and low-cardinality error type. Docker remains slim/non-root and gains health wiring. Smoke builds/runs the image with 2 workers.
- D-13: Timeweb Cloud is the actual deployment/storage target. Planner and executor must account for Timeweb endpoint/path-style/signature behavior and must not rely on AWS-only S3 conditional write behavior without a fallback.

### Project Constraints

- `parser-core` and `parser-contract` remain transport-free. RabbitMQ, S3, HTTP probes, Docker, non-deterministic runtime state, and logs stay in `parser-worker`/`parser-cli`.
- Worker output remains the minimal public v3 artifact. Phase 7 must not change parser semantics, artifact fields, RabbitMQ message fields, canonical identity, PostgreSQL persistence, public APIs/UI, replay discovery, bounty payout logic, or annual/yearly statistics.
- Phase 6 ack policy remains mandatory: ack only after confirmed completed/failed outcome publication; nack/requeue only when no durable outcome was published.
- Completed work must update README/planning docs and leave a clean committed git tree.

## External Technical Findings

| Source | Finding | Planning implication |
|--------|---------|----------------------|
| RabbitMQ acknowledgements and publisher confirms, `https://www.rabbitmq.com/docs/3.13/confirms` | Consumer acknowledgements and publisher confirms are independent safety mechanisms. Prefetch controls delivered-but-unacked messages and high/unbounded prefetch increases broker/client memory exposure. | Preserve manual ack, publisher confirms, and default prefetch `1`. Multi-worker proof should be horizontal instances, not in-process concurrency. |
| AWS S3 conditional writes, `https://docs.aws.amazon.com/AmazonS3/latest/userguide/conditional-writes.html` | `If-None-Match: *` makes create-if-absent writes fail with `412 Precondition Failed` when the key already exists. Concurrent writes for the same key let one winner succeed and later writers fail with `412`; `409` can also occur during conflicts. | Add a conditional put path for artifacts. On `412` or race/conflict outcomes, fetch existing bytes and apply existing checksum/size reuse-or-conflict logic. |
| AWS S3 PutObject API, `https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html` | `PutObject` supports `If-None-Match`; `409 ConditionalRequestConflict` should be retried or followed by state inspection. Directory buckets are excluded. | Treat conditional put support as provider capability. Do not assume every S3-compatible store maps errors exactly like AWS. |
| Kubernetes probes, `https://kubernetes.io/docs/concepts/configuration/liveness-readiness-startup-probes/` | Readiness controls whether traffic is routed to a pod; liveness restarts containers and should not be used for ordinary dependency outages. Startup probes can protect slow initialization. | `/readyz` should include dependency/capacity state; `/livez` should not fail merely because RabbitMQ or S3 is down. Docker/Kubernetes examples can wire `/readyz` for readiness and `/livez` for liveness. |
| Dockerfile reference and build best practices, `https://docs.docker.com/reference/dockerfile`, `https://docs.docker.com/build/building/best-practices/` | `HEALTHCHECK` defines container health status. Docker recommends non-root users for services that do not need privileges, minimal packages, explicit working directories, and build/test automation. | Keep the existing slim non-root image, add health wiring, and avoid adding debug tools or secrets to the runtime image. |
| OpenTelemetry RabbitMQ semantic conventions, `https://opentelemetry.io/docs/specs/semconv/messaging/rabbitmq/` | RabbitMQ attributes include destination/routing key, operation name/type, delivery tag, message ID, and low-cardinality error type. The RabbitMQ semconv status is still development. | Use the vocabulary as naming guidance for stable JSON logs, but do not add an OpenTelemetry exporter or claim full OTel instrumentation in Phase 7. |
| Timeweb S3 docs, `https://timeweb.cloud/docs/s3-storage`, `https://timeweb.cloud/docs/s3-storage/manage-storage/s3-guide`, `https://timeweb.cloud/docs/s3-storage/supported-features` | Timeweb describes S3-compatible API access and current AWS Signature V2/V4 support. The public supported-features index lists common features but does not explicitly guarantee conditional write header behavior. | Default Timeweb config should use `REPLAY_PARSER_S3_ENDPOINT`, `AWS_REGION`, `REPLAY_PARSER_S3_FORCE_PATH_STYLE`, and AWS SDK credentials. Conditional writes must have a tested fallback and optional live provider capability smoke when credentials are supplied. |

## Existing Code Facts

| Area | Finding | Planning implication |
|------|---------|----------------------|
| Storage | `ObjectStore::write_artifact_if_absent_or_matching` currently performs get-then-put and reuses matching existing bytes. `S3ObjectStore::put_object_bytes` does unconditional `put_object`. | Add conditional create-if-absent to the S3 adapter and keep the existing compare/reuse/conflict behavior as fallback. |
| Processor | `process_job_body` already centralizes job decode, checksum verification, parsing, artifact write, result publication, and ack decision. | Duplicate/redelivery tests can stay no-network by extending fake store/publisher behavior rather than requiring RabbitMQ. |
| Runner | `run_until_cancelled` initializes tracing, builds S3/AMQP clients, consumes one delivery at a time, and logs `worker_starting`, `worker_connected`, `worker_job_received`, and shutdown events. | Add worker ID, probe state, dependency readiness transitions, and duration/log taxonomy here without moving runtime concerns into parser-core. |
| Config/CLI | `WorkerConfig` has env/flag-backed AMQP/S3/prefetch config and redaction; CLI has a thin `worker` subcommand. | Add probe bind/port and worker ID to config/CLI with redaction tests. Keep secret env vars out of config values. |
| AMQP | `RabbitMqClient::connect` uses `basic_qos(config.prefetch)`, `no_ack=false`, publisher confirms, persistent result messages, and mandatory publishes. | Preserve this policy. Logging can include safe routing/delivery metadata, not credentials. |
| Live smoke | `scripts/worker-smoke.sh` starts RabbitMQ/MinIO and runs one ignored live smoke test using the local binary path. | Extend smoke to build/run the container image and start at least two worker instances, then assert probes, logs, duplicate outcome behavior, and artifact conflict safety. |
| Docker | The image is multi-stage, `debian:bookworm-slim`, installs only `ca-certificates`, runs as `65532:65532`, and defaults to `replay-parser-2 worker`. | Keep the non-root slim baseline and add health check/probe port documentation without adding secret-bearing examples. |

## Architecture Recommendations

1. **Artifact race hardening first.** Add a conditional put path to `S3ObjectStore` and a testable trait method such as `put_artifact_bytes_if_absent`. On conditional already-exists/conflict, fetch the existing artifact and reuse only if exact size and SHA-256 match.
2. **Provider fallback is required.** Because Timeweb docs confirm S3 compatibility and SigV2/SigV4 but not conditional write semantics, keep the existing get/compare/reuse/conflict fallback and add a no-secret optional Timeweb capability smoke path.
3. **Probe state should be cached.** Add a `health` module with an `Arc`/`watch` or `Arc<RwLock>` state object. Probe handlers must read cached state and return quickly; dependency checks happen at startup/background intervals, not per request.
4. **HTTP probes belong in `parser-worker`.** Use a small async HTTP server in `parser-worker` and expose config/CLI flags from `parser-cli`. `parser-core` and `parser-contract` remain free of HTTP, RabbitMQ, S3, and signal dependencies.
5. **Readiness transitions are explicit.** Startup: live true, ready false. After AMQP consumer/result channel and S3 check succeed: ready true. Shutdown requested: ready false immediately. Liveness stays true unless a fatal internal startup/runtime state is set.
6. **Stable log taxonomy beats volume.** Emit decision-point logs: worker startup/config, dependency ready/degraded, job received, parse started/finished, artifact write new/reused/conflict, result published, ack/nack applied, readiness transitions, shutdown requested/drained. Include durations and low-cardinality `error_type`; avoid per-event parser hot-loop logs.
7. **Smoke should become container evidence.** Extend Docker Compose or the smoke script to build the image, run two workers with distinct `REPLAY_PARSER_WORKER_ID` values and probe ports, publish duplicate jobs, assert one deterministic artifact and two completed outcomes, assert conflicting artifact behavior, and grep JSON logs for taxonomy fields.

## Do Not Hand-Roll

- Do not parse HTTP requests manually if a small maintained Rust HTTP stack is available.
- Do not add OpenTelemetry exporter, Prometheus, dashboards, or Kubernetes manifests in Phase 7.
- Do not rely on S3 ETag as artifact integrity proof; continue local SHA-256.
- Do not log AMQP passwords, AWS credentials, full secret env values, or replay artifact contents.
- Do not add higher default prefetch or in-process worker pools.

## Common Pitfalls

| Pitfall | Mitigation |
|---------|------------|
| Two workers overwrite the same deterministic key | Conditional create-if-absent plus post-race existing-object checksum/size comparison. |
| S3-compatible provider ignores or rejects conditional headers | Treat conditional support as optimization; fall back to compare/reuse/conflict and document Timeweb capability evidence. |
| Readiness performs blocking network calls per probe | Cache dependency state and update it in startup/background checks. |
| Liveness restarts workers during RabbitMQ/S3 outage | Keep dependency failures in readiness, not liveness. |
| Shutdown keeps pod ready while draining | Flip readiness false as soon as cancellation is requested, then drain the one in-flight delivery. |
| Logs become high-cardinality/noisy | Use stable event names and low-cardinality error types; put job/replay IDs in fields, not event names. |
| Container smoke only tests the local binary | Build/run the image and assert worker IDs, probes, logs, duplicate job behavior, and artifact safety. |

## Validation Architecture

| Dimension | Required gate |
|-----------|---------------|
| Artifact race/idempotency | `cargo test -p parser-worker storage conditional_put artifact_write_existing_match artifact_write_existing_conflict processor_duplicate_redelivery processor_artifact_conflict` proves conditional create-if-absent fallback, matching-object reuse, differing-object conflict, and duplicate job completion behavior. |
| Probe config and state | `cargo test -p parser-worker config health runner_readiness` and `cargo test -p parser-cli worker_command` prove worker ID/probe env+flags, redacted config, `/livez` and `/readyz` status codes/bodies, startup ready=false, dependency ready=true, and shutdown ready=false. |
| Structured logs | `cargo test -p parser-worker log_taxonomy` or equivalent assertions prove stable event names and fields for worker/job/artifact/result/ack/probe transitions, plus no secrets in redacted config/log payloads. |
| Container smoke | `scripts/worker-smoke.sh` builds the Docker image, runs RabbitMQ/MinIO plus two worker containers, verifies probes, publishes duplicate/redelivered jobs, asserts deterministic artifact reuse/conflict behavior, and validates structured logs. |
| Timeweb compatibility | A no-secret documented command or smoke mode verifies endpoint, region/signature/path-style config and conditional-write capability when Timeweb credentials are supplied. Without credentials, final docs must state conditional writes are attempted but compare/reuse/conflict fallback remains required. |
| Boundary and security | Boundary greps prove `parser-core`/`parser-contract` contain no `lapin`, `aws_sdk_s3`, HTTP server crate, `tokio::signal`, `/livez`, `/readyz`, or `HEALTHCHECK`; secret grep rejects credential-bearing examples. |
| Final quality | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps`, `scripts/coverage-gate.sh --check`, `scripts/fault-report-gate.sh`, `scripts/benchmark-phase5.sh --ci`, `scripts/worker-smoke.sh`, schema freshness, boundary greps, secret greps, and `git diff --check`. |

## Research Complete

Phase 7 should be planned as five executable plans: S3 artifact race/idempotency hardening, HTTP probe state and config, structured worker log taxonomy and worker identity, container/two-worker smoke hardening with Timeweb compatibility hooks, and final quality/docs handoff.
