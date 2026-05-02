---
phase: 06
artifact: patterns
status: complete
created: 2026-05-02
---

# Phase 06 Pattern Map

## Purpose

Map RabbitMQ/S3 worker integration to existing project patterns so execution adds transport adapters without weakening the pure parser core, minimal artifact contract, or cross-app boundaries.

## Planned File Families

| Planned file family | Role | Closest existing analog | Pattern to preserve |
|---------------------|------|-------------------------|---------------------|
| `crates/parser-contract/src/worker.rs` | Worker request/result message contract | `artifact.rs`, `failure.rs`, `source_ref.rs`, `version.rs` | Typed Serde/Schemars structs, stable field order, reuse `SourceChecksum`, `ContractVersion`, `ParserInfo`, `ParseFailure`. |
| `crates/parser-contract/src/schema.rs` | Worker schema generation | Current `parse_artifact_schema()` | Generate schemas from Rust types and enforce tests against committed schema files. |
| `crates/parser-contract/tests/worker_message_contract.rs` | Message behavior tests | `failure_contract.rs`, `schema_contract.rs` | Behavior-first JSON assertions, examples deserialize and validate, no docs-only contract. |
| `schemas/parse-job-v1.schema.json`, `schemas/parse-result-v1.schema.json` | Committed worker schemas | `schemas/parse-artifact-v3.schema.json` | Fresh generation must byte-match committed schemas. |
| `crates/parser-worker/Cargo.toml` | New runtime adapter crate | Existing crate manifests | Inherit workspace package/lints; keep AMQP/S3/Tokio dependencies out of `parser-core`. |
| `crates/parser-worker/src/config.rs` | Env/flag-backed worker config | CLI clap pattern in `parser-cli/src/main.rs` | Explicit defaults, required fields, secret redaction, no secret logging. |
| `crates/parser-worker/src/artifact_key.rs` | Deterministic S3 key construction | Minimal artifact deterministic serialization tests | Stable `artifacts/v3/{encoded_replay_id}/{source_sha256}.json`, portable encoding, no raw path segments. |
| `crates/parser-worker/src/storage.rs` | S3-compatible object access | CLI file I/O boundary in `parser-cli/src/main.rs` | Adapter owns I/O and checksums; parser core receives bytes and source metadata only. |
| `crates/parser-worker/src/amqp.rs` | RabbitMQ connection/consumer/publisher | Existing adapter separation in CLI/harness | Explicit manual ack, `basic_qos`, publisher confirms, no hardcoded production topology. |
| `crates/parser-worker/src/processor.rs` | Job lifecycle orchestration | CLI `parse_command`, parser-core `parse_replay` | Call the same minimal public parse path, write S3 artifacts only on success, publish structured failed results otherwise. |
| `crates/parser-worker/src/shutdown.rs` | Graceful drain | Tokio cancellation pattern from official docs | Stop consuming new jobs, drain one in-flight job, then exit. |
| `crates/parser-cli/src/main.rs` | Public worker subcommand | Current parse/schema/compare subcommands | Thin command adapter; worker implementation remains in `parser-worker`. |
| `crates/parser-worker/tests/*.rs` | Worker behavior tests | Existing CLI/core/harness integration tests | No-network fakes for object store, publisher, and acker; assert observable ack/order/result behavior. |
| `README.md`, `.planning/ROADMAP.md`, `.planning/STATE.md` | Handoff docs | Phase 5.2 final handoff docs | Keep AI+GSD workflow visible, document worker command/config, and keep Phase 7 boundary clear. |

## Existing Interfaces To Reuse

- `parser_core::parse_replay(ParserInput)` remains the parser entrypoint.
- `ParserInput` carries bytes, `ReplaySource`, parser metadata, and deterministic options.
- `ParseArtifact`, `ParseStatus`, and `ParseFailure` already represent success/failure output.
- `SourceChecksum::sha256` validates lowercase SHA-256 hex and should be used for raw and artifact checksums.
- `ContractVersion::current()` is the exact supported parser contract version.
- `ParserInfo` is already embedded in artifacts and should be included in result messages.
- CLI command tests already use `assert_cmd` and temporary output files.

## Existing Test Style

Current tests prefer public API behavior, deterministic JSON assertions, schema freshness, and grep-verifiable boundaries. Phase 6 should continue that style with:

- contract tests for valid parse jobs and completed/failed result JSON;
- config tests for env/flag precedence and redacted debug output;
- artifact key tests for slash, dot-segment, space, Unicode, and empty values;
- fake object-store tests for raw checksum mismatch, successful write, existing-object reuse, and conflict;
- fake publisher/acker tests for `parse.completed`, `parse.failed`, ack-after-confirm, and requeue on publish failure;
- CLI tests for `replay-parser-2 worker --help` and missing config behavior;
- final grep checks that secrets are not logged and Phase 7 health/readiness endpoints were not added.

## Boundary Constraints

Do not add these concerns during Phase 6:

- HTTP health/readiness endpoints or container probes;
- default prefetch/concurrency above `1`;
- multi-worker duplicate-artifact proof beyond single-worker idempotent key reuse;
- PostgreSQL writes, OpenAPI/server APIs, canonical identity, or web UI behavior;
- replay discovery or raw S3 staging;
- debug sidecar generation during worker ingestion;
- parser artifact shape changes beyond worker message/reference contracts.

## Pattern Mapping Complete

The plan can proceed with typed worker contracts, a new runtime adapter crate, S3 checksum/artifact behavior, RabbitMQ manual ack and publisher confirms, end-to-end orchestration with graceful drain, and final docs/quality gates.
