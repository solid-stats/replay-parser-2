# Phase 6: RabbitMQ/S3 Worker Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-05-02T19:47:56+07:00
**Phase:** 6-RabbitMQ/S3 Worker Integration
**Areas discussed:** RabbitMQ message contract, S3 object/artifact keys and checksum, Ack/nack/retry behavior, Worker config/runtime boundary

---

## RabbitMQ Message Contract

### Message Contract Home

| Option | Description | Selected |
|--------|-------------|----------|
| parser-contract | Typed structs + JSON Schema beside `ParseArtifact`, keeping worker and `server-2` aligned. | yes |
| worker-local | Faster start, but weaker contract and higher drift risk. | |
| docs-only | Minimal code now, but downstream agents would guess fields and validation. | |

**User's choice:** parser-contract.
**Notes:** Message schemas should be code-backed and schema-backed.

### Contract Version Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Fail not-retry | Publish `parse.failed` with unsupported contract version; retrying same job will not help. | yes |
| Same major OK | Accept compatible `3.x` only after backward compatibility is proven. | |
| Worker decides | Leave exact policy to planner while preserving structured failure. | |

**User's choice:** Fail not-retry.
**Notes:** Current supported contract version is `3.0.0`.

### Completed Payload

| Option | Description | Selected |
|--------|-------------|----------|
| Ref + proof | Include identifiers, contract version, source checksum, artifact bucket/key, artifact checksum/size, and parser info. | yes |
| Ref only | Smaller payload but weaker audit evidence. | |
| Add counts | Adds row counters for monitoring, but expands result contract. | |

**User's choice:** Ref + proof.
**Notes:** The artifact itself remains in S3, not RabbitMQ.

### Routing Names

| Option | Description | Selected |
|--------|-------------|----------|
| Env defaults | Safe defaults for local use, all overridable by env config. | yes |
| No defaults | Cleaner production contract, weaker local verification. | |
| Hardcode names | Simple but brittle against real `server-2` settings. | |

**User's choice:** Env defaults.
**Notes:** Defaults should include `parse.completed` and `parse.failed`.

---

## S3 Object/Artifact Keys and Checksum

### Raw Checksum Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Compute SHA-256 | Download bytes, compute SHA-256, compare to job checksum; do not rely on ETag. | yes |
| Trust S3 checksum | Use S3 checksum metadata; weaker for S3-compatible providers. | |
| Worker decides | Planner chooses implementation while preserving structured checksum mismatch failure. | |

**User's choice:** Compute SHA-256.
**Notes:** Aligns with `replays-fetcher` SHA-256 evidence.

### Artifact Key Format

| Option | Description | Selected |
|--------|-------------|----------|
| Version/replay/checksum | Example `artifacts/v3/{replay_id}/{sha256}.json`; readable and stable. | yes |
| Version/checksum | Simpler and deduplicates identical bytes. | |
| Job-based | Easier job trace, weaker deterministic reprocessing. | |

**User's choice:** Version/replay/checksum.
**Notes:** Planner must define safe key encoding for `replay_id`.

### Artifact Checksum

| Option | Description | Selected |
|--------|-------------|----------|
| Yes SHA-256 | Compute artifact SHA-256 and include it in `parse.completed`. | yes |
| Size only | Less work, weaker durable verification. | |
| S3 metadata only | Less portable across S3-compatible storage. | |

**User's choice:** Yes SHA-256.
**Notes:** Include artifact byte size too.

### Existing Artifact Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse if match | Existing object is success only when size/checksum match; mismatch is failure. | yes |
| Always overwrite | Simpler but can hide nondeterminism or corruption. | |
| Always fail | Strict but breaks idempotent redelivery/reprocess. | |

**User's choice:** Reuse if match.
**Notes:** Do not silently overwrite mismatched deterministic keys.

---

## Ack/Nack/Retry Behavior

### Ack Order

| Option | Description | Selected |
|--------|-------------|----------|
| After result confirm | Ack only after artifact write and broker-confirmed result publication. | yes |
| After artifact write | Can lose job state if result publish fails. | |
| After parse done | Unsafe if S3 write or result publish later fails. | |

**User's choice:** After result confirm.
**Notes:** Applies to both completed and failed results.

### Retry Owner

| Option | Description | Selected |
|--------|-------------|----------|
| server-2 | Publish `parse.failed` with retryability and ack; Rabbit requeue only when outcome cannot be published. | yes |
| RabbitMQ requeue | Simpler broker retry but risks loops outside server job state. | |
| DLX first | Pushes more lifecycle into broker routing and away from server state. | |

**User's choice:** server-2.
**Notes:** Worker reports retryability; `server-2` schedules retry.

### Bad Job Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Failed + ack | Publish structured `parse.failed`, wait for confirm, then ack. | yes |
| Reject to DLX | Server may lose unified failure path. | |
| Worker decides | Planner splits cases while keeping manual ack safety. | |

**User's choice:** Failed + ack.
**Notes:** Covers invalid body, missing fields, unsupported version, and checksum mismatch.

### Prefetch Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Prefetch 1 | One in-flight job by default; Phase 7 proves parallel safety. | yes |
| Bounded >1 | Higher throughput now, larger idempotency surface. | |
| Planner decides | Planner sets exact value while proving bounded prefetch/manual ack. | |

**User's choice:** Prefetch 1.
**Notes:** An env override may exist, but default is conservative.

---

## Worker Config/Runtime Boundary

### Worker Crate Split

| Option | Description | Selected |
|--------|-------------|----------|
| parser-worker crate | New crate for runtime/config/RabbitMQ/S3, called by CLI worker subcommand. | yes |
| Inside parser-cli | Fewer files, but CLI becomes a heavy transport adapter. | |
| Separate binary only | Isolated runtime, weaker test/helper reuse. | |

**User's choice:** parser-worker crate.
**Notes:** `parser-core` remains pure and transport-free.

### Config Surface

| Option | Description | Selected |
|--------|-------------|----------|
| Env + flags | AMQP URL/names, S3 endpoint/bucket/region/path-style/credentials, artifact prefix, prefetch. | yes |
| Env only | Good for containers, worse for local tests and examples. | |
| Config file | Convenient locally, higher secret-leak and format-surface risk. | |

**User's choice:** Env + flags.
**Notes:** Secrets must not be committed or logged.

### Health Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 7 | Phase 6 uses structured logs and exit codes; health/readiness stay in hardening. | yes |
| Basic now | Adds a minimal endpoint but expands Phase 6 beyond WORK-01..07. | |
| Planner decides | Add only if needed for tests without scope creep. | |

**User's choice:** Phase 7.
**Notes:** `WORK-09` belongs to Phase 7.

### Shutdown Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Drain in-flight | Stop consuming, finish current job through result confirm and ack, then exit. | yes |
| Fast cancel | Leaves job unacked/requeued, causing extra redelivery and possible partial writes. | |
| Configurable | Adds timeout/fallback behavior; useful but increases scope. | |

**User's choice:** Drain in-flight.
**Notes:** Graceful shutdown is best-effort; crash safety still depends on idempotent S3/result behavior.

---

## the agent's Discretion

- Exact worker message type names.
- Exact default exchange/queue/routing key names.
- Exact error-code strings within existing parser-contract namespaces.
- Exact `replay_id` escaping/encoding strategy for artifact keys.
- Exact integration test harness shape.

## Deferred Ideas

- HTTP health/readiness endpoints, container probes, and operator readiness checks remain Phase 7.
- Multi-worker safety and higher default concurrency remain Phase 7.
- DLX-first poison-message workflows may be revisited later if `server-2` retry state is insufficient.
