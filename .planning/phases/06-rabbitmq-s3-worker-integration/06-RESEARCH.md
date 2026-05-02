---
phase: 06
artifact: research
status: complete
researched_at: 2026-05-02
---

# Phase 06 Research - RabbitMQ/S3 Worker Integration

## User Constraints

### Locked Decisions From CONTEXT.md

- D-01 through D-05: worker request and result contracts live in `parser-contract`, are typed Rust structs, and generate JSON Schema. Incoming jobs contain `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`. Unsupported contract versions publish non-retryable `parse.failed`. Successful results publish an S3 artifact reference plus raw/artifact checksum proof and artifact size.
- D-06 through D-09: worker downloads raw bytes, computes SHA-256 locally, compares against the job checksum, writes deterministic artifacts under a version/replay/checksum key, computes the artifact SHA-256 from exact written bytes, and reuses an existing artifact only when stored bytes match size and checksum.
- D-10 through D-13: manual ack is mandatory. The input job is acked only after durable outcome publication succeeds. Handled failures publish `parse.failed` and ack. RabbitMQ requeue is reserved for cases where the worker cannot publish the outcome. Default prefetch is `1`.
- D-14 through D-17: add a `parser-worker` crate, keep `parser-core` transport-free, expose `replay-parser-2 worker` as a thin CLI delegate, configure AMQP/S3 through env plus flags, avoid logging secrets, defer HTTP health/readiness and parallel hardening to Phase 7, and drain one in-flight job on shutdown.

### Project Constraints

- Parser worker consumes only `server-2` parse jobs and S3 object keys. It must not crawl replay sources, mutate PostgreSQL, match canonical identities, expose public APIs, or own UI behavior.
- Default worker output must be the minimal v3 parser artifact used by the CLI. Debug sidecars and rich event/entity evidence must not be produced by worker ingestion.
- Phase 5.2 accepted current performance, p95 artifact/raw ratio, and the 4 known malformed/non-JSON failures; Phase 6 should not reopen parser-performance or artifact-shape work.
- Strict workspace lints and 100% reachable-code coverage remain release gates. Worker code must avoid `unwrap`, `expect`, hidden panics, secret logging, broad coverage exclusions, and transport logic inside `parser-core`.

## External Technical Findings

| Source | Finding | Planning implication |
|--------|---------|----------------------|
| RabbitMQ acknowledgements and confirms docs, https://www.rabbitmq.com/docs/3.13/confirms | Automatic acknowledgements trade safety for throughput, while manual acknowledgements plus bounded prefetch prevent unbounded in-flight deliveries. Publisher confirms are independent of consumer acknowledgements and confirm broker responsibility for published messages. | Plan must use `no_ack=false`, `basic_qos(prefetch=1)`, result publisher confirms, and input ack only after result confirm. |
| RabbitMQ reliability docs, https://www.rabbitmq.com/docs/reliability | Messaging systems must treat client, broker, network, and channel failures as normal distributed-system failure modes; acknowledgements transfer responsibility. | Plan needs explicit transient publish/connect failure paths and requeue only when no durable outcome can be reported. |
| RabbitMQ DLX docs, https://www.rabbitmq.com/docs/3.13/dlx | Dead-letter behavior is triggered by negative acknowledgements with `requeue=false`, expiry, queue length, or delivery limit; DLX policy is broker/operator configuration. | Phase 6 should not hardcode DLX arguments. `server-2` owns retry scheduling; worker uses `nack(requeue=true)` only when result publication fails. |
| `lapin` docs, https://docs.rs/lapin/latest/lapin/message/struct.Delivery.html and publisher confirm examples | A `Delivery` is acknowledged through `ack`, `nack`, or `reject`; publisher confirms are awaited from `basic_publish(...).await?.await?`. | Worker AMQP adapter should wrap lapin delivery/publisher APIs behind small testable functions and assert confirm `Ack` without returned mandatory message. |
| S3 object key docs, https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html | Object keys are opaque names but path-like `.` and `..` segments can surprise clients and tools. | Artifact key builder should percent-encode replay IDs and never pass raw replay IDs as path segments. |
| S3 object/checksum docs, https://docs.aws.amazon.com/AmazonS3/latest/API/API_Object.html and https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html | S3 supports SHA-256 checksums, but ETag is not always a content MD5 and varies with encryption and multipart uploads. | Worker must compute SHA-256 locally for raw bytes and artifact bytes; ETag is not authoritative. |
| AWS SDK for Rust endpoint docs, https://docs.aws.amazon.com/sdk-for-rust/latest/dg/endpoints.html | SDK clients can use `endpoint_url`, custom endpoint resolvers, and S3 `force_path_style(true)` for local/S3-compatible stores. | Config must include endpoint URL, region, bucket, and path-style settings; tests should not require real AWS. |
| Tokio graceful shutdown docs, https://tokio.rs/tokio/topics/shutdown | Graceful shutdown has three parts: detect shutdown, tell tasks, and wait; `CancellationToken` is a standard way to notify async tasks. | Worker runner should stop consuming, let one in-flight job finish through result publish and ack when possible, then exit. |

## Existing Code Facts

| Area | Finding | Planning implication |
|------|---------|----------------------|
| Workspace | `Cargo.toml` has members `parser-contract`, `parser-core`, `parser-cli`, and `parser-harness`, with strict workspace lints. | Phase 6 should add `crates/parser-worker` as a workspace member and inherit workspace lints. |
| Parser core | `parser_core::parse_replay(ParserInput)` is pure and accepts caller-provided bytes, `ReplaySource`, parser metadata, and deterministic options. | Worker should feed downloaded bytes into this API and keep RabbitMQ/S3 out of `parser-core`. |
| CLI adapter | `parser-cli/src/main.rs` currently exposes parse/schema/compare and has a private helper that strips metadata source refs before writing public artifacts. | Phase 6 should either share the public-artifact sanitization helper or move it behind a public parser-core helper so CLI and worker use the same minimal path. |
| Contract | `ParseArtifact`, `ParseFailure`, `ReplaySource`, `SourceChecksum`, `ContractVersion`, and `ParserInfo` already model most worker evidence. | Add only worker message envelopes and artifact-reference types; reuse existing checksum/failure/version types. |
| Failure contract | `ErrorCode` accepts `io`, `json`, `schema`, `unsupported`, `internal`, `checksum`, and `output` families. | Worker failures should use existing namespaces such as `unsupported.contract_version`, `checksum.mismatch`, `io.s3_read`, `output.s3_write`, and `output.rabbitmq_publish`. |
| Schema | `parse_artifact_schema()` is the source of truth for committed artifact schema. | Add worker schema functions and tests rather than hand-written schemas. |
| Tests | Current tests use behavior-level Cargo integration tests, `assert_cmd`, JSON schema validation, and explicit grep/boundary checks. | Worker tests should use fake/in-memory object store and AMQP publisher/acker abstractions for deterministic no-network coverage. |

## Architecture Recommendations

1. **Contract first**: add `parser-contract/src/worker.rs` with `ParseJobMessage`, `ParseCompletedMessage`, `ParseFailedMessage`, `ParseResultMessage`, and `ArtifactReference`. `parse.completed` should carry concrete job/replay fields; `parse.failed` should use `FieldPresence` for job/replay/object fields so invalid JSON and missing-field jobs can still be reported.
2. **Schema-backed messages**: add `worker_message_schema()` helpers and committed schema/examples for request/result messages. Tests must validate examples and schema freshness.
3. **Worker crate split**: add `parser-worker` with modules `config`, `amqp`, `storage`, `artifact_key`, `processor`, `shutdown`, and `error`. Keep production dependencies out of `parser-core`; `parser-cli` only depends on `parser-worker` for the worker subcommand.
4. **Testable transport boundary**: make job processing generic over small object-store and result-publisher traits or function traits so unit tests can simulate S3 and RabbitMQ without networked RabbitMQ, LocalStack, or credentials.
5. **Artifact key policy**: use `artifacts/v3/{percent_encoded_replay_id}/{source_sha256}.json` by default. Percent-encode every byte outside ASCII alphanumeric, `-`, `_`, and `.`; reject or encode path separators so no raw replay ID can create path traversal or ambiguous S3 segments.
6. **Outcome-first ack policy**: `process_delivery` should return one of: `Ack` after confirmed `parse.completed`, `Ack` after confirmed `parse.failed`, or `NackRequeue` when no durable outcome could be published. Handled input/parser/storage failures should not requeue directly.
7. **Checksum proof**: raw checksum mismatch is a non-retryable `checksum.mismatch`; artifact object conflicts are retryability `unknown` or retryable output/internal failures depending on whether the store state is observable.
8. **No Phase 7 scope creep**: structured logs and process exit behavior are in scope. HTTP health endpoints, multi-worker proof, default concurrency above one, container image work, and readiness probes stay deferred.

## Don't Hand-Roll

- Do not write ad hoc JSON string parsing for RabbitMQ payloads. Use `serde_json` into contract structs and structured failure on decode/schema errors.
- Do not rely on S3 ETag for integrity. Compute local SHA-256 from bytes.
- Do not embed credentials in CLI examples, docs, logs, errors, or planning artifacts.
- Do not hardcode production RabbitMQ topology. Provide safe defaults and env/flag overrides.
- Do not build a real RabbitMQ/S3 dependency into ordinary tests when fakes can prove ack/order/checksum semantics deterministically.

## Common Pitfalls

| Pitfall | Mitigation |
|---------|------------|
| Acking before result publish confirm | Make ack/nack a return value from the processor and assert in tests that ack occurs only after successful publisher fake confirms. |
| Invalid job without `job_id` cannot be reported | Model failed result identifiers as `FieldPresence<String>` and route missing fields through `ParseFailure`. |
| Existing S3 object hides nondeterminism | On deterministic key collision, compare stored bytes checksum/size to new artifact bytes; mismatch publishes structured output failure. |
| Raw replay ID creates bad S3 key | Add artifact-key tests for `/`, `..`, `.`, spaces, Unicode, and empty replay IDs. |
| Worker bloats CLI/core dependencies | Put AMQP/S3/Tokio in `parser-worker`, leave `parser-core` pure, and keep CLI worker subcommand thin. |
| Shutdown drops in-flight result | Use cancellation to stop new deliveries but complete one in-flight process through publish confirm and ack when possible. |
| Secret disclosure | Add tests/log review criteria that config debug output and error messages do not include `AWS_SECRET_ACCESS_KEY`, passwords, or full AMQP URL credentials. |

## Validation Architecture

| Dimension | Required gate |
|-----------|---------------|
| Worker message contract | `cargo test -p parser-contract worker_message_contract schema_contract` validates request/result structs, examples, unsupported-version failure payloads, and committed worker schemas. |
| Worker crate boundary | `cargo test -p parser-worker config artifact_key` proves config defaults/env overrides, secret redaction, artifact prefix defaults, and safe key encoding. |
| S3/checksum behavior | `cargo test -p parser-worker storage processor_checksum` proves raw SHA-256 validation, checksum mismatch failure, deterministic artifact write, existing object reuse, and conflict failure using a fake object store. |
| RabbitMQ ack/publish behavior | `cargo test -p parser-worker amqp_ack processor_ack_order` proves prefetch `1`, manual `ack` only after confirmed outcome publish, `parse.failed` for handled failures, and `nack(requeue=true)` only when publish confirmation fails. |
| Parser integration | `cargo test -p parser-worker processor_success processor_failure` proves worker success uses the same minimal public artifact bytes as CLI default and parser failures publish `parse.failed` without S3 artifact upload. |
| Shutdown | `cargo test -p parser-worker shutdown` proves cancellation stops new deliveries and drains one in-flight job through publish confirm and ack. |
| CLI worker command | `cargo test -p parser-cli worker_command` proves `replay-parser-2 worker --help` exposes required config flags and missing required config exits non-zero without printing secrets. |
| Quality gates | Final plan runs `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps`, `scripts/coverage-gate.sh --check`, `scripts/fault-report-gate.sh`, `scripts/benchmark-phase5.sh --ci`, worker schema freshness, secret grep, and `git diff --check`. |

## Research Complete

Phase 6 should be planned as six executable plans: worker message contracts, worker crate/config/CLI foundation, S3/checksum/artifact storage, RabbitMQ consumer/publisher/ack policy, end-to-end processor/shutdown integration, and final verification/docs handoff.
