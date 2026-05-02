# Phase 7: Parallel and Container Hardening - Context

**Gathered:** 2026-05-03T00:10:29+07:00
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 7 hardens the already implemented RabbitMQ/S3 worker so operators can run
multiple worker containers safely and tell whether each worker is live and ready
to accept parse jobs.

This phase covers `WORK-08` and `WORK-09`: multi-worker artifact/idempotency
safety, structured worker operations logs, HTTP health/readiness probes, and
container smoke/hardening. It does not change parser semantics, minimal v3
artifact shape, RabbitMQ job/result contract fields, canonical identity,
PostgreSQL persistence, replay discovery, public APIs/UI, final bounty
calculation, production Kubernetes manifests, or annual/yearly statistics.

</domain>

<decisions>
## Implementation Decisions

### Parallel Safety Proof

- **D-01:** The v1 parallel baseline is multiple worker instances or containers,
  each retaining conservative prefetch `1`. Do not introduce an in-process task
  pool or make higher per-instance prefetch the default in this phase.
- **D-02:** Artifact creation should be hardened with conditional create-if-absent
  writes where the S3-compatible provider supports them. If the deterministic
  key already exists, the worker should compare exact artifact checksum and size:
  reuse matching artifacts and emit a structured conflict/failure for differing
  bytes.
- **D-03:** RabbitMQ redelivery of the same job must be idempotent. The worker
  should be able to reuse a matching deterministic artifact and republish the
  durable completed/failed outcome rather than treating harmless duplicates as
  parser failures.
- **D-04:** Phase 7 acceptance should combine no-network unit/integration tests
  with a live Docker Compose smoke test using 2+ worker instances against
  RabbitMQ and MinIO. It does not need a full-corpus load/stress benchmark.

### Readiness and Probes

- **D-05:** Add a small HTTP probe surface to the worker, with configurable bind
  address/port and at least `/livez` and `/readyz` endpoints.
- **D-06:** `ready` means the worker can accept jobs: config is valid, RabbitMQ
  is connected with an active consumer/result publisher path, S3-compatible
  storage is reachable, and shutdown has not been requested. Probe handlers
  should read cached/background state rather than doing expensive blocking I/O
  on every request.
- **D-07:** `live` should represent process/event-loop health and fatal internal
  startup state only. RabbitMQ or S3 outages should make readiness fail without
  forcing liveness restarts.
- **D-08:** During startup the worker is not ready until dependencies are ready.
  During shutdown, readiness flips false immediately, the in-flight delivery is
  allowed to drain through publish and ack/nack, and liveness remains true until
  the process exits cleanly.

### Structured Operations Logs and Container Hardening

- **D-09:** Lock a stable JSON log event taxonomy for operator use. Logs should
  include low-cardinality event names plus fields such as `worker_id`, `job_id`,
  `replay_id`, `object_key`, parser/worker stage, RabbitMQ routing or delivery
  data where safe, artifact key, retryability, outcome, duration, and predictable
  error code/type. Do not log secrets.
- **D-10:** Worker instances should identify themselves in logs and probe state
  with `REPLAY_PARSER_WORKER_ID` when set, otherwise a container/hostname-based
  identity.
- **D-11:** The Docker baseline is a slim non-root runtime image with container
  health wiring, documented probe port/env settings, and no committed secrets or
  credential-bearing examples.
- **D-12:** Extend the smoke path to build/run the container image, run 2 worker
  instances, verify probes, assert useful structured logs, and exercise duplicate
  job/redelivery plus artifact reuse/conflict behavior.

### the agent's Discretion

- Exact HTTP crate, probe response JSON shape, bind defaults, and status-code
  details are planner discretion if `/livez` and `/readyz` preserve the semantics
  above and do not put transport concerns into `parser-core`.
- Exact S3 conditional-write implementation is planner discretion. Prefer AWS SDK
  conditional put support when available; keep a tested S3-compatible fallback
  for providers that cannot enforce the header.
- Exact event names and field spellings are planner discretion if the resulting
  taxonomy is stable, documented, testable, low-cardinality, and secret-safe.
- Exact Docker Compose topology and smoke script layout are planner discretion if
  they build the image, run at least two workers, and verify the behaviors above.
- Resource-limit docs or examples may be added if lightweight, but production
  orchestration manifests remain out of scope.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project and Phase Scope

- `.planning/PROJECT.md` - Current parser scope, Phase 7 readiness, accepted
  Phase 5.2 benchmark policy, worker/container constraints, and cross-app
  ownership boundaries.
- `.planning/REQUIREMENTS.md` - Phase 7 requirements `WORK-08` and `WORK-09`.
- `.planning/ROADMAP.md` - Phase 7 goal and success criteria.
- `.planning/STATE.md` - Current focus, Phase 6 completion, and Phase 7 pending
  scope.
- `.planning/research/SUMMARY.md` - Research rationale for RabbitMQ/S3 worker
  mode, operational hardening, and container readiness.
- `README.md` - Public worker command surface, current artifact contract, worker
  integration behavior, Docker/smoke commands, and AI/GSD workflow rules.

### Prior Phase Decisions

- `.planning/phases/06-rabbitmq-s3-worker-integration/06-CONTEXT.md` - Worker
  request/result contract, deterministic artifact key policy, ack/nack rules,
  prefetch `1`, and Phase 7 deferrals.
- `.planning/phases/06-rabbitmq-s3-worker-integration/06-HUMAN-UAT.md` - Live
  RabbitMQ/MinIO smoke evidence from Phase 6.
- `.planning/phases/06-rabbitmq-s3-worker-integration/06-PATTERNS.md` - Worker
  code/test pattern map and explicit Phase 7 boundary constraints.
- `.planning/phases/06-rabbitmq-s3-worker-integration/06-05-SUMMARY.md` - Final
  Phase 6 handoff: single-worker integration complete; Phase 7 owns multi-worker
  safety, probes, and runtime hardening.
- `.planning/phases/05.2-minimal-artifact-and-performance-acceptance/05.2-CONTEXT.md`
  - Minimal v3 artifact, debug-sidecar boundary, and accepted performance/size
  policy that worker output must preserve.

### Cross-Application Boundaries

- `gsd-briefs/replay-parser-2.md` - Parser-owned parsing, contract, CLI/worker,
  S3 artifact, and result-publication responsibilities.
- `gsd-briefs/server-2.md` - `server-2` ownership of parse jobs, retry/job state,
  PostgreSQL persistence, canonical identity, API mapping, and aggregate
  calculation.
- `gsd-briefs/replays-fetcher.md` - Raw S3 object and SHA-256 checksum ownership;
  parser worker consumes only `object_key`, checksum, and job metadata.
- `gsd-briefs/web.md` - UI/API type boundary; parser worker hardening does not
  directly own web behavior.

### Current Code

- `Cargo.toml` - Workspace members and lint/edition policy.
- `crates/parser-cli/src/main.rs` - Thin `worker` subcommand and CLI config
  override wiring.
- `crates/parser-contract/src/worker.rs` - Typed parse job/result message
  contract.
- `crates/parser-worker/src/config.rs` - Worker config defaults, env variables,
  prefetch validation, and secret redaction.
- `crates/parser-worker/src/runner.rs` - Runtime entrypoint, JSON tracing setup,
  consumer loop, cancellation, and job-received logging.
- `crates/parser-worker/src/processor.rs` - End-to-end job lifecycle, parse path,
  result publication, and delivery action selection.
- `crates/parser-worker/src/amqp.rs` - RabbitMQ manual ack/nack, consumer
  prefetch, publisher confirms, mandatory result publication, and routing keys.
- `crates/parser-worker/src/storage.rs` - S3-compatible object storage, checksum
  calculation, artifact write/reuse/conflict policy.
- `crates/parser-worker/src/artifact_key.rs` - Deterministic
  `artifacts/v3/{encoded_replay_id}/{source_sha256}.json` key construction.
- `crates/parser-worker/src/shutdown.rs` - In-flight delivery drain helper and
  cancellation behavior.
- `crates/parser-worker/tests/processor.rs` - No-network processor tests for
  artifact writes, failures, ack actions, and publish-failure requeue.
- `crates/parser-worker/tests/amqp.rs` - AMQP publish/ack policy tests.
- `crates/parser-worker/tests/storage.rs` - Object-store checksum, reuse,
  conflict, and failure tests.
- `crates/parser-worker/tests/live_smoke.rs` - Current single-worker live
  RabbitMQ/MinIO smoke harness to extend.
- `crates/parser-worker/tests/shutdown.rs` - Shutdown drain behavior tests.
- `Dockerfile` - Current multi-stage image with slim runtime and non-root user.
- `docker-compose.worker-smoke.yml` - Local RabbitMQ/MinIO smoke infrastructure.
- `scripts/worker-smoke.sh` - Current live smoke entrypoint.

### Official Technical References

- `https://www.rabbitmq.com/docs/3.13/confirms` - Consumer acknowledgements,
  publisher confirms, prefetch behavior, and redelivery/idempotency guidance.
- `https://www.rabbitmq.com/docs/reliability` - RabbitMQ reliability model,
  connection loss, confirms, and redelivered messages.
- `https://docs.aws.amazon.com/AmazonS3/latest/userguide/conditional-writes.html`
  - S3 conditional writes with `If-None-Match`/`If-Match` to prevent overwrites.
- `https://kubernetes.io/docs/concepts/configuration/liveness-readiness-startup-probes/`
  - Liveness, readiness, startup probe semantics and check mechanisms.
- `https://docs.docker.com/reference/builder` - Dockerfile `HEALTHCHECK`
  reference.
- `https://docs.docker.com/build/building/best-practices/` - Docker image build
  best practices including non-root `USER` guidance.
- `https://opentelemetry.io/docs/specs/semconv/messaging/rabbitmq/` - RabbitMQ
  messaging field vocabulary useful for stable log-field naming without adding
  an OpenTelemetry exporter in this phase.
- `https://tokio.rs/tokio/topics/shutdown` - Tokio graceful shutdown pattern.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `parser_worker::processor::process_job_body` already isolates the job
  lifecycle behind fakeable `ObjectStore` and `ResultPublisher` traits. This is
  the right seam for duplicate/redelivery and publish-failure tests.
- `ObjectStore::write_artifact_if_absent_or_matching` already implements
  matching-object reuse and conflicting-object failure. Phase 7 should harden
  the first-write race with conditional put semantics.
- `RabbitMqClient` already uses manual consumer ack/nack, configurable prefetch,
  and publisher confirms. Phase 7 should preserve ack-after-outcome semantics.
- `runner::run_until_cancelled` and `shutdown::drain_until_cancelled` already use
  `CancellationToken` for graceful shutdown. Probe readiness can hook into this
  state without changing parser-core.
- `WorkerConfig::redacted()` already avoids logging secrets. Add probe bind/port
  and worker-id config with the same redaction discipline.
- `Dockerfile`, `docker-compose.worker-smoke.yml`, and `scripts/worker-smoke.sh`
  already provide a container/smoke foundation to extend for two workers and
  probe checks.

### Established Patterns

- Parser-core and parser-contract stay transport-free. RabbitMQ, S3, HTTP probe
  server, Docker, timestamps, and non-deterministic runtime state belong in
  `parser-worker`/`parser-cli` adapters only.
- Contracts are typed in Rust and backed by schema/tests rather than docs-only
  message shapes.
- Tests use public behavior, fake adapters, live smoke only where needed, and
  grep-backed boundary checks.
- Worker output uses the minimal public v3 artifact path and must not invoke the
  debug sidecar path.
- Generated/heavy smoke evidence belongs under generated/ignored paths, not
  committed as large artifacts.

### Integration Points

- Add probe runtime state and HTTP serving to `crates/parser-worker/src/runner.rs`
  or a new `parser-worker` module without touching parser-core.
- Extend `WorkerConfig` and CLI worker flags/env variables for probe bind/port
  and worker identity.
- Update `storage.rs` to use conditional create-if-absent semantics and map
  already-exists/race responses into existing compare/reuse/conflict behavior.
- Extend `amqp.rs`/`runner.rs` logging with safe delivery/routing fields and
  durations.
- Extend `Dockerfile` with health wiring and update `scripts/worker-smoke.sh` to
  build/run the image and assert two-worker behavior.

</code_context>

<specifics>
## Specific Ideas

- Prefer `/livez` for process health and `/readyz` for dependency/capacity
  readiness.
- Readiness should flip false immediately after shutdown is requested, before
  the worker finishes draining its current delivery.
- A repeated delivery for an already written matching artifact should be a
  normal idempotent completion path, not a special operator error.
- Do not add an OpenTelemetry exporter in Phase 7. Use OpenTelemetry RabbitMQ
  naming only as a reference for stable log field vocabulary.

</specifics>

<deferred>
## Deferred Ideas

- Full-corpus or sustained load/stress benchmarking for multi-worker throughput
  is deferred unless a later phase explicitly targets performance/scaling.
- An in-process worker task pool and higher default prefetch are deferred until
  a workload-backed need exists.
- OpenTelemetry exporter, metrics stack integration, dashboards, and alerting
  can be future operational work after stable JSON logs and probes exist.
- Production Kubernetes manifests remain out of scope; this phase should be
  Kubernetes-ready, not a Kubernetes deployment phase.

</deferred>

---

*Phase: 07-parallel-and-container-hardening*
*Context gathered: 2026-05-03T00:10:29+07:00*
