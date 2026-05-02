---
phase: 07
artifact: patterns
status: complete
created: 2026-05-03
---

# Phase 07 Pattern Map

## Purpose

Map Phase 7 parallel/container hardening onto existing worker patterns so execution adds race-safety, probes, logs, and smoke evidence without changing parser semantics or the worker contract shape.

## Planned File Families

| Planned file family | Role | Closest existing analog | Pattern to preserve |
|---------------------|------|-------------------------|---------------------|
| `crates/parser-worker/src/storage.rs` | S3 artifact write race hardening | Existing `ObjectStore::write_artifact_if_absent_or_matching` | Compute local SHA-256, compare exact bytes/size, return structured `ArtifactConflict`, map storage failures to `ParseStage::Output`. |
| `crates/parser-worker/tests/storage.rs` | Conditional write/reuse/conflict tests | Existing fake `ObjectStore` tests | Behavior-first async tests with fake storage state; no real S3 in ordinary tests. |
| `crates/parser-worker/src/processor.rs` | Duplicate/redelivery idempotency | Existing job lifecycle and `ResultPublisher` abstraction | Keep processor generic over fakeable store/publisher, return `DeliveryAction`, publish completed/failed before ack. |
| `crates/parser-worker/tests/processor.rs` | Duplicate job and conflict behavior tests | Existing success/failure ack policy tests | Assert observable result messages, artifact refs, and ack/nack decisions rather than private internals. |
| `crates/parser-worker/src/health.rs` | Cached probe state and HTTP handlers | Existing `shutdown.rs` and `runner.rs` cancellation state | Runtime-only module, cheap cached reads, explicit startup/ready/draining/fatal states, no parser-core dependency. |
| `crates/parser-worker/src/config.rs` | Probe bind/port and worker ID config | Existing env/flag config with redacted debug output | Constants for env names, defaults, validation helpers, no secret storage, redacted `Debug`. |
| `crates/parser-cli/src/main.rs` | Worker CLI probe flags | Existing thin worker subcommand | CLI only builds `WorkerConfigOverrides`; runtime logic stays in `parser-worker`. |
| `crates/parser-worker/src/runner.rs` | Probe server lifecycle, dependency readiness, log fields | Existing `run_until_cancelled`, `CancellationToken`, JSON tracing | Startup validates config, starts probes, connects dependencies, flips readiness, consumes until shutdown, drains before exit. |
| `crates/parser-worker/src/amqp.rs` | Safe delivery/routing log fields | Existing manual ack and publisher confirm helpers | Preserve `no_ack=false`, persistent results, mandatory publishes, and publisher-confirm enforcement. |
| `crates/parser-worker/tests/live_smoke.rs` | Two-worker live smoke assertions | Existing ignored RabbitMQ/MinIO smoke | Live-only test behind `REPLAY_PARSER_LIVE_SMOKE=1`; use real broker/storage only for container proof. |
| `Dockerfile` | Container health wiring | Existing multi-stage slim non-root image | Keep non-root `USER 65532:65532`, no debug packages/secrets, default `replay-parser-2 worker`, add health check. |
| `docker-compose.worker-smoke.yml` | Local RabbitMQ/MinIO/two-worker topology | Existing smoke infrastructure | Compose remains local-only, parameterized ports/env, no committed credentials beyond local MinIO defaults. |
| `scripts/worker-smoke.sh` | Executable container smoke | Existing live smoke entrypoint | `set -euo pipefail`, cleanup trap, dependency checks, generated/live evidence only, no secret output. |
| `README.md`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md` | Final handoff docs | Phase 6 final handoff | Keep AI+GSD workflow visible, document worker probes/container behavior, preserve cross-app boundaries. |

## Existing Interfaces To Reuse

- `parser_worker::processor::process_job_body` stays the job lifecycle entrypoint for fake/no-network tests.
- `parser_worker::storage::ObjectStore` remains the testable object-store boundary.
- `parser_worker::amqp::DeliveryAction` and `delivery_action_after_publish` remain the ack policy boundary.
- `parser_worker::runner::run_until_cancelled` remains the testable runtime entrypoint with caller-supplied cancellation.
- `WorkerConfig::from_env_and_overrides`, `WorkerConfig::validate`, and `WorkerConfig::redacted` are the config pattern to extend.
- `scripts/worker-smoke.sh` remains the single user-facing smoke command.

## Boundary Constraints

Do not add these concerns during Phase 7:

- Parser artifact shape or parser semantic changes.
- RabbitMQ job/result message field changes unless they are strictly additive local log/probe fields outside the message contract.
- Transport, HTTP, Docker, signal, or S3 dependencies in `parser-core` or `parser-contract`.
- PostgreSQL writes, OpenAPI/server APIs, canonical identity, replay discovery, web UI, payout logic, or yearly statistics.
- OpenTelemetry exporter, metrics stack, dashboards, production Kubernetes manifests, in-process task pool, higher default prefetch, or full-corpus stress testing.

## Pattern Mapping Complete

The plan can proceed with S3 conditional-write hardening, cached HTTP probes, stable worker logs and identity, two-worker container smoke, Timeweb compatibility hooks, and final docs/quality gates while preserving the Phase 6 worker contract and parser-core boundary.
