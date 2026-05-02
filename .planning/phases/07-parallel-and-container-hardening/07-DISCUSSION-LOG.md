# Phase 7: Parallel and Container Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-05-03T00:10:29+07:00
**Phase:** 07-parallel-and-container-hardening
**Areas discussed:** Parallel safety proof, Readiness and probes, Structured operations logs and container hardening

---

## Area Selection

| Option | Description | Selected |
|--------|-------------|----------|
| All three | Cover WORK-08 and WORK-09: parallel safety, readiness/probes, and structured ops logs/container hardening. | yes |
| Parallel safety | Focus only on multi-worker/idempotent artifact proof, redelivery, and duplicate/conflict behavior. | |
| Container ops | Focus only on readiness/health probe shape, structured logs, image/runtime hardening, and smoke checks. | |

**User's choice:** All three.
**Notes:** The discussion covered all identified Phase 7 gray areas.

---

## Parallel Safety Proof

### Parallel Execution Model

| Option | Description | Selected |
|--------|-------------|----------|
| Multi-instance only | Multiple worker containers/processes, each with conservative prefetch `1`; directly covers WORK-08 with less runtime change. | yes |
| Instances + prefetch | Prove multiple instances plus per-instance prefetch greater than one; higher throughput, wider in-flight surface. | |
| In-process pool | Add a task pool inside one worker process; broad runtime change and higher scope risk. | |

**User's choice:** Multi-instance only.
**Notes:** Phase 7 should prove horizontal worker instances, not introduce a local task pool or higher default prefetch.

### Duplicate Artifact Race

| Option | Description | Selected |
|--------|-------------|----------|
| Conditional write | Use S3 conditional put/create-if-absent when available, then compare existing checksum/size on AlreadyExists. | yes |
| Compare existing | Keep current get-then-put/reuse-or-conflict logic and prove idempotence in tests. | |
| Server dedupe | Rely on server-2 job de-duplication instead of hardening parser artifact writes. | |

**User's choice:** Conditional write.
**Notes:** Worker safety should not depend on `server-2` de-duplication.

### Redelivery Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Republish idempotently | Reuse matching deterministic artifact and publish the same completed/failed outcome again. | yes |
| Fail duplicates | Treat already-existing artifact/result-like state as structured failure. | |
| Server resolves | Publish outcomes and expect server-2 to dedupe all duplicate outcomes. | |

**User's choice:** Republish idempotently.
**Notes:** Duplicate deliveries should be harmless when the deterministic artifact matches.

### Proof Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Live smoke + unit | No-network race/redelivery tests plus Docker Compose smoke with 2+ workers against RabbitMQ/MinIO and duplicate jobs. | yes |
| Unit only | Fast fake tests only. | |
| Full stress | Multi-worker run over a large corpus/load profile. | |

**User's choice:** Live smoke + unit.
**Notes:** Full stress/load benchmarking is not required for this context.

---

## Readiness and Probes

### Probe Surface

| Option | Description | Selected |
|--------|-------------|----------|
| HTTP probes | Add small configurable `/livez` and `/readyz` server with cached worker state. | yes |
| Exec probe | Add CLI/status-file probe without HTTP port. | |
| Docker only | Use Dockerfile HEALTHCHECK around process/config. | |

**User's choice:** HTTP probes.
**Notes:** The phase should add a real runtime probe surface instead of relying only on Docker process checks.

### Readiness Criteria

| Option | Description | Selected |
|--------|-------------|----------|
| Dependency-ready | Config valid, AMQP connected/consumer active, result publish channel available, S3 reachable, and shutdown not requested. | yes |
| Started-only | Ready once process initialized. | |
| Queue-only | Ready when RabbitMQ consumer is active, ignoring S3 until a job arrives. | |

**User's choice:** Dependency-ready.
**Notes:** Probe handlers should use cached/background state rather than expensive blocking I/O per request.

### Liveness Criteria

| Option | Description | Selected |
|--------|-------------|----------|
| Process alive | Only event-loop/process health and fatal initialization state; external outages affect readiness. | yes |
| Dependencies too | Fail liveness when RabbitMQ/S3 checks fail. | |
| No liveness | Expose readiness only and rely on container/process exit. | |

**User's choice:** Process alive.
**Notes:** Dependency outages should not force liveness restarts.

### Startup and Shutdown Drain

| Option | Description | Selected |
|--------|-------------|----------|
| Not ready during drain | Startup waits for dependencies; shutdown flips readiness false immediately, drains in-flight job, liveness stays true until clean exit. | yes |
| Ready while draining | Keeps accepting routing longer. | |
| Fast fail | Fail live/readiness immediately and let orchestrator kill. | |

**User's choice:** Not ready during drain.
**Notes:** Preserve Phase 6 ack-after-outcome drain behavior.

---

## Structured Operations Logs and Container Hardening

### Log Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Stable event taxonomy | Lock JSON event names/fields for worker/job/stage/artifact/retryability/duration without secrets. | yes |
| Current logs plus docs | Keep existing worker_starting/job_received/job_completed/job_failed and document them. | |
| OTel spans now | Add OpenTelemetry tracing/metrics export in Phase 7. | |

**User's choice:** Stable event taxonomy.
**Notes:** OpenTelemetry naming can guide fields, but an exporter is not part of this phase.

### Worker Identity

| Option | Description | Selected |
|--------|-------------|----------|
| Env/hostname ID | Use `REPLAY_PARSER_WORKER_ID` when set, otherwise hostname/container identity. | yes |
| No worker ID | Use only job/replay IDs. | |
| Random persisted ID | Generate and persist an instance ID. | |

**User's choice:** Env/hostname ID.
**Notes:** Parallel containers need distinguishable logs/probe state without mutable persisted identity.

### Container Image

| Option | Description | Selected |
|--------|-------------|----------|
| Slim non-root + health | Keep slim runtime, non-root UID/GID, add HEALTHCHECK, document probe port/env, and keep secrets out of image/logs. | yes |
| Distroless/scratch | Smaller attack surface but more TLS/debugging friction. | |
| Debug-friendly | Include curl/shell/tools in runtime image. | |

**User's choice:** Slim non-root + health.
**Notes:** Current Dockerfile already uses a slim non-root runtime foundation.

### Smoke Validation

| Option | Description | Selected |
|--------|-------------|----------|
| Build + 2 workers | Build/run container image, start 2 worker instances, verify probes, logs, duplicate redelivery, artifact reuse/conflict safety. | yes |
| Local binary only | Keep current live smoke against local binary and add unit tests. | |
| Docs only | Document docker run/compose examples without executable smoke. | |

**User's choice:** Build + 2 workers.
**Notes:** The smoke path should become container evidence, not only local binary evidence.

---

## the agent's Discretion

- Exact HTTP crate, probe JSON shape, status-code details, and bind defaults.
- Exact conditional-write implementation and fallback for S3-compatible stores.
- Exact stable log event names and field spellings.
- Exact Docker Compose topology and smoke-script layout.
- Lightweight resource-limit docs/examples, if useful, while keeping production orchestration out of scope.

## Late Captured Context

- User clarified after the initial discussion that the server and S3 are deployed
  in Timeweb Cloud.
- CONTEXT.md now records Timeweb Cloud as the actual deployment/storage target
  for Phase 7 planning.
- Planner should verify Timeweb Cloud S3 endpoint/path-style/signature behavior
  and conditional-write support instead of assuming AWS-only behavior.

## Deferred Ideas

- Full-corpus or sustained multi-worker load/stress benchmarking.
- In-process worker task pool and higher default prefetch.
- OpenTelemetry exporter, metrics stack integration, dashboards, and alerting.
- Production Kubernetes manifests.
