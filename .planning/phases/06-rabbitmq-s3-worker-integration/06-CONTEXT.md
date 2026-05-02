# Phase 6: RabbitMQ/S3 Worker Integration - Context

**Gathered:** 2026-05-02T19:47:56+07:00
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 6 exposes the proven parser core through a RabbitMQ/S3 worker adapter.
`server-2` publishes parse jobs with replay identity, object key, checksum, and
contract version; the worker downloads the raw object, verifies checksum,
parses with the same minimal v3 parser path used by the CLI, writes a durable
S3 artifact, publishes `parse.completed` or `parse.failed`, and acknowledges
the input job only after the durable result path succeeds.

This phase does not change the minimal v3 artifact shape, rebuild parser
semantics, add PostgreSQL persistence, own replay discovery, perform canonical
identity matching, implement public APIs/UI, calculate final bounty payouts, or
prove parallel/container readiness. Phase 7 owns multi-worker safety and
health/readiness hardening.

</domain>

<decisions>
## Implementation Decisions

### RabbitMQ Message Contract

- **D-01:** Define the RabbitMQ request/result JSON contracts in
  `parser-contract`, with typed Rust structs and generated JSON Schema. Do not
  leave worker messages as docs-only or worker-local structs.
- **D-02:** The incoming parse job contract is the Phase 6 roadmap shape:
  `job_id`, `replay_id`, `object_key`, `checksum`, and
  `parser_contract_version`.
- **D-03:** If `parser_contract_version` is not exactly the current supported
  contract version `3.0.0`, the worker publishes non-retryable `parse.failed`
  with a structured schema/unsupported-version error. Retrying the same job is
  not expected to succeed.
- **D-04:** `parse.completed` carries an artifact reference plus proof:
  `job_id`, `replay_id`, parser contract version, source checksum, artifact
  bucket/key, artifact SHA-256, artifact byte size, and parser info. The
  minimal parse artifact itself is not sent inline over RabbitMQ.
- **D-05:** RabbitMQ exchange, queue, and routing key names are worker config,
  not hardcoded deployment truth. Provide safe env-backed defaults including
  `parse.completed` and `parse.failed`, but allow `server-2` deployments to
  override names.

### S3 Objects, Artifact Keys, and Checksums

- **D-06:** The worker verifies raw replay integrity by downloading object
  bytes, computing SHA-256 locally, and comparing against the checksum supplied
  in the parse job. Do not rely on S3 ETag as the authoritative source
  checksum.
- **D-07:** Successful artifacts use a deterministic version/replay/checksum
  key pattern, for example `artifacts/v3/{replay_id}/{sha256}.json`. The
  planner must specify safe key encoding for `replay_id` and keep the result
  compatible with `server-2`.
- **D-08:** The worker computes SHA-256 of the exact minimal artifact bytes it
  writes and includes that checksum plus byte size in `parse.completed`.
- **D-09:** If the deterministic artifact key already exists, the worker may
  reuse it only when stored size/checksum match the newly produced artifact. If
  the existing object differs, emit a structured output/internal failure and do
  not silently overwrite it.

### Ack, Nack, and Retry Behavior

- **D-10:** The worker may `ack` an incoming parse job only after the durable
  outcome path completes: successful artifact write plus confirmed
  `parse.completed`, or confirmed `parse.failed`.
- **D-11:** `server-2` owns retry scheduling for handled failures. The worker
  publishes `parse.failed` with retryability and then acks the input job.
  RabbitMQ requeue is reserved for cases where the worker could not durably
  publish the result/outcome.
- **D-12:** Bad input jobs, including invalid message JSON, missing required
  fields, unsupported contract version, and checksum mismatch, go through the
  normal structured failure path: publish `parse.failed`, wait for publisher
  confirm, then ack.
- **D-13:** Phase 6 defaults to one in-flight job: prefetch `1`. An env override
  may exist, but Phase 7 owns proving multi-worker and higher-concurrency
  safety.

### Worker Runtime and Configuration

- **D-14:** Add a `parser-worker` crate for runtime/config/RabbitMQ/S3 logic.
  `parser-cli` should expose a thin `replay-parser-2 worker` subcommand that
  delegates to the worker crate.
- **D-15:** Worker config is env variables plus CLI flags. Required config
  should cover AMQP URL/names, S3 endpoint, bucket, region, path-style mode,
  credentials, artifact prefix, and prefetch. Do not record secrets in planning
  docs, logs, commits, or examples.
- **D-16:** Phase 6 does not implement HTTP health/readiness endpoints. It
  should provide structured logs and clear process exit behavior. Health,
  readiness, and container probes remain Phase 7 scope.
- **D-17:** On shutdown signal, the worker stops consuming new deliveries,
  drains the in-flight job through result confirmation and ack when possible,
  then exits.

### the agent's Discretion

- Exact Rust type names for worker request/result messages are planner
  discretion if they live in `parser-contract` and generate schemas.
- Exact default exchange/queue/routing key names are planner discretion if they
  are env-overridable and include `parse.completed`/`parse.failed`.
- Exact error-code strings are planner discretion if they fit the existing
  `ErrorCode` namespace rules and preserve retryability semantics.
- Exact `replay_id` key-encoding strategy is planner discretion if artifact
  keys remain deterministic, portable across S3-compatible stores, and
  compatible with `server-2`.
- Exact integration test shape is planner discretion. Prefer the lightest test
  harness that can prove ack/order/checksum/S3/result behavior without
  weakening production boundaries.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project and Phase Scope

- `.planning/PROJECT.md` - Current parser scope, Phase 6 readiness, accepted
  Phase 5.2 benchmark policy, worker integration flow, and cross-app ownership
  boundaries.
- `.planning/REQUIREMENTS.md` - Pending Phase 6 requirements `WORK-01` through
  `WORK-07`; Phase 7 requirements `WORK-08` and `WORK-09` are explicitly not
  Phase 6 scope.
- `.planning/ROADMAP.md` - Phase 6 goal, success criteria, dependency on Phase
  5.2, and Phase 7 boundary.
- `.planning/STATE.md` - Current focus, accepted Phase 5.2 gaps, and recent
  parity/performance context.
- `.planning/research/SUMMARY.md` - Research rationale for worker mode,
  RabbitMQ/S3, durable artifacts, checksum validation, and safe ack behavior.
- `README.md` - Current public command surface, reserved `replay-parser-2
  worker` slot, minimal artifact contract, and AI/GSD workflow rules.

### Prior Phase Decisions

- `.planning/phases/05.2-minimal-artifact-and-performance-acceptance/05.2-CONTEXT.md`
  - Minimal v3 artifact, debug sidecar, accepted performance/size/failure
  policy, and server compatibility decision.
- `.planning/phases/05.1-compact-artifact-and-selective-parser-redesign/05.1-CONTEXT.md`
  - Compact/server-facing artifact boundary, selective parser path, and
  server compatibility gate that Phase 5.2 tightened.
- `.planning/phases/05-cli-golden-parity-benchmarks-and-coverage-gates/05-CONTEXT.md`
  - CLI boundary, local checksum behavior, coverage/fault gates, and benchmark
  evidence decisions.

### Cross-Application Boundaries

- `gsd-briefs/replay-parser-2.md` - Parser-owned parsing, contract, CLI/worker,
  S3 artifact, and result-publication responsibilities.
- `gsd-briefs/server-2.md` - `server-2` ownership of parse jobs, PostgreSQL,
  canonical identity, API mapping, aggregate calculation, and retry/job state.
- `gsd-briefs/replays-fetcher.md` - Raw S3 object and SHA-256 checksum
  ownership; parser worker consumes only `object_key`, checksum, and job
  metadata.
- `gsd-briefs/web.md` - Public UI/API type boundary; parser worker does not own
  UI-visible behavior directly.

### Current Code

- `Cargo.toml` - Workspace members, Rust edition/MSRV, and strict lint policy
  that a new worker crate must inherit.
- `crates/parser-cli/src/main.rs` - Current CLI has parse/schema/compare only;
  Phase 6 should add a thin `worker` subcommand.
- `crates/parser-cli/Cargo.toml` - Current CLI dependencies; worker-specific
  async/RabbitMQ/S3 dependencies should not bloat parser-core.
- `crates/parser-core/src/lib.rs` - Pure `parse_replay` and
  `parse_replay_debug` boundaries that runtime adapters must call instead of
  adding transport logic to parser-core.
- `crates/parser-core/src/input.rs` - `ParserInput` carries bytes, source
  metadata, parser info, and deterministic options for adapter-provided input.
- `crates/parser-contract/src/artifact.rs` - Minimal v3 `ParseArtifact`
  envelope consumed by `server-2` through S3 artifact references.
- `crates/parser-contract/src/failure.rs` - Existing structured
  `ParseFailure`, `ParseStage`, `ErrorCode`, and `Retryability` types to reuse
  in worker failure/result contracts.
- `crates/parser-contract/src/source_ref.rs` - `ReplaySource` and
  `SourceChecksum` SHA-256 validation types for raw source and artifact proof.
- `crates/parser-contract/src/version.rs` - Current contract version `3.0.0`
  and parser metadata types.

### Official Technical References

- `https://www.rabbitmq.com/docs/3.13/confirms` - Consumer acknowledgements,
  publisher confirms, prefetch, and negative acknowledgements.
- `https://www.rabbitmq.com/docs/reliability` - Data safety, duplicates, and
  acknowledgement reliability model.
- `https://www.rabbitmq.com/docs/3.13/dlx` - Dead-letter behavior for rejected
  or nacked messages when `requeue=false`.
- `https://docs.rs/lapin/latest/lapin/struct.Consumer.html` - Rust `lapin`
  consumer stream, explicit ack/nack/reject behavior, and `basic_qos`.
- `https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html` -
  S3 object key naming and prefix behavior.
- `https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html`
  - S3 checksum support, including SHA-256.
- `https://docs.aws.amazon.com/AmazonS3/latest/API/API_Object.html` - S3 object
  metadata and why ETag is not a universal content checksum.
- `https://docs.aws.amazon.com/sdk-for-rust/latest/dg/endpoints.html` - AWS SDK
  for Rust custom endpoints and S3 `force_path_style`.
- `https://tokio.rs/tokio/topics/shutdown` - Tokio graceful shutdown pattern.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `parser_core::parse_replay(ParserInput)` is already the pure parser entry
  point. Worker mode should feed S3 bytes into this API rather than introducing
  parser semantics in the transport adapter.
- `ParserInput` already separates replay bytes, `ReplaySource`, parser info,
  and deterministic options. Worker mode can set `ReplaySource.source_file` to
  the S3 `object_key`, preserve `replay_id`, and attach the verified SHA-256.
- `ParseFailure` already includes job/replay/source/checksum/stage/code/message
  and retryability fields. Worker result contracts can reuse or wrap this
  instead of inventing a parallel failure model.
- `SourceChecksum::sha256` validates lowercase SHA-256 hex, matching the
  current `replays-fetcher` compatibility rule.

### Established Patterns

- Parser-core is transport-free. File I/O, RabbitMQ, S3, non-deterministic
  timestamps, publisher confirms, and shutdown behavior belong in adapters.
- Contract/schema generation is Rust-type driven from `parser-contract`; worker
  message schemas should follow the same pattern as artifact schemas.
- CLI writes minimal v3 JSON by default and only builds debug detail when
  explicitly requested. Worker mode must deliver the same minimal default
  artifact, not debug sidecar output.
- The workspace enforces strict Rust/clippy/rustdoc lints, forbids unsafe code,
  and requires coverage allowlists for reachable production exclusions.

### Integration Points

- Add a new `parser-worker` workspace crate with `tokio`, `lapin`,
  `aws-config`, `aws-sdk-s3`, `tracing`, and typed worker config.
- Add a `worker` subcommand to `crates/parser-cli/src/main.rs` that delegates
  to the worker crate and preserves the public binary name
  `replay-parser-2`.
- Extend `parser-contract` with parse job and parse result message types plus
  schema generation/tests.
- Implement S3 download, raw checksum verification, parse invocation, artifact
  serialization, deterministic S3 write/reuse policy, artifact checksum proof,
  result publish with confirms, and manual ack/nack behavior around those
  steps.
- Keep Phase 7-only concerns out of Phase 6 unless a minimal hook is required
  for tests. In particular, HTTP health/readiness and multi-worker/container
  hardening stay deferred.

</code_context>

<specifics>
## Specific Ideas

- Example artifact key shape: `artifacts/v3/{replay_id}/{sha256}.json`.
  Planner must choose safe escaping/encoding for `replay_id`.
- `parse.completed` should be small and audit-friendly: artifact reference,
  raw checksum, artifact checksum, artifact size, parser info, and identifiers.
- `parse.failed` should be the normal lifecycle path for handled failures,
  including invalid job body, unsupported version, checksum mismatch, parse
  failure, S3 input/output failure that can be reported, and resultable
  structured errors.
- RabbitMQ requeue is not the main retry scheduler. Use it only when the worker
  cannot durably publish the outcome that would let `server-2` own job state.
- Phase 6 should be conservative by default: prefetch `1`, one in-flight job,
  and graceful drain on shutdown.

</specifics>

<deferred>
## Deferred Ideas

- HTTP health/readiness endpoints, container probes, and operator readiness
  checks remain Phase 7.
- Multi-worker safety, higher default concurrency, and duplicate artifact
  corruption proof remain Phase 7.
- Broader DLX-first poison-message workflows may be considered later if
  `server-2` job-state retries are insufficient.

</deferred>

---

*Phase: 06-rabbitmq-s3-worker-integration*
*Context gathered: 2026-05-02T19:47:56+07:00*
