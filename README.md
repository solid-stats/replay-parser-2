# replay-parser-2

`replay-parser-2` is the Rust replacement for the legacy SolidGames OCAP replay parser.

The parser will turn OCAP JSON replay files into compact, deterministic, versioned parser artifacts that the Solid Stats backend can persist, audit at the statistics-contribution level, compare against golden data, and use for public statistics.

The default v1 output must reduce replay data for `server-2`. A 10-15 MB OCAP replay should not become another 10-15 MB JSON artifact on the ordinary ingestion path.

## Current Status

Phase 6 execution is complete. The repository contains the Rust workspace with `crates/parser-contract`, generated JSON Schema, committed success/failure examples, contract tests, the pure parser core at `crates/parser-core`, the parser harness at `crates/parser-harness`, the CLI adapter binary `replay-parser-2`, and the RabbitMQ/S3 worker adapter at `crates/parser-worker`. Phase 5.2 replaced the earlier compact artifact with a v3 minimal v1 statistics artifact, moved detailed evidence behind an explicit debug sidecar, removed GitHub issue #13 vehicle score from v1, and recorded accepted current performance, hard max-artifact-size, and malformed-file parity evidence before worker integration.

The CLI can parse a local OCAP JSON file into minified minimal JSON by default, export the v3 parser contract schema, compare selected old/new artifacts or a selected replay against a saved old artifact, run `replay-parser-2 worker` for server integration, and write an internal full-detail debug sidecar only when `--debug-artifact <path>` is requested. The ordinary default tables are `players[]`, `weapons[]`, `destroyed_vehicles[]`, and `diagnostics[]`; player-authored enemy/team kills are nested under the killer `players[].kills`, same-name slots are merged into one replay-local player row, and legacy squad tags are split from observed nicknames. Source refs, rule IDs, entity snapshots, and normalized event/entity evidence stay out of ordinary ingestion unless the debug sidecar is explicitly requested. Phase 5.2 performance and size acceptance now treats x3/x10 and artifact percentiles as reported evidence, not blockers; the hard artifact-size blocker is every successful default artifact being <= 100 KB (100,000 bytes), and malformed/non-JSON raw files are acceptable when old/new failure parity matches the accepted evidence. RabbitMQ/S3 worker mode is implemented; PostgreSQL persistence, public APIs, canonical identity handling, replay discovery, public UI, parallel worker hardening, container probe endpoints, and annual/yearly nomination product support remain outside this parser or later-phase scope.

- Current phase: Phase 7, `Parallel and Container Hardening` (ready after Phase 6 worker integration).
- Roadmap: 9 phases.
- v1 requirements: 80 mapped requirements.
- Contract crate: `crates/parser-contract`.
- Current artifact contract version: `3.0.0`.
- Parser-core crate: `crates/parser-core`.
- CLI crate: `crates/parser-cli`.
- Worker crate: `crates/parser-worker`.
- Harness crate: `crates/parser-harness`.
- Contract schema: `schemas/parse-artifact-v3.schema.json`.
- Worker schemas: `schemas/parse-job-v1.schema.json` and `schemas/parse-result-v1.schema.json`.
- Example artifacts: `crates/parser-contract/examples/parse_artifact_success.v3.json` and `crates/parser-contract/examples/parse_failure.v3.json`.
- Worker examples: `crates/parser-contract/examples/parse_job.v1.json`, `crates/parser-contract/examples/parse_completed.v1.json`, and `crates/parser-contract/examples/parse_failed.v1.json`.
- Phase 3 plans: `.planning/phases/03-deterministic-parser-core/03-00-PLAN.md` through `03-05-PLAN.md`.
- Phase 4 plans: `.planning/phases/04-event-semantics-and-aggregates/04-00-PLAN.md` through `04-06-PLAN.md`.
- Phase 5 plans: `.planning/phases/05-cli-golden-parity-benchmarks-and-coverage-gates/05-00-PLAN.md` through `05-05-PLAN.md`.
- Phase 5.1 directory: `.planning/phases/05.1-compact-artifact-and-selective-parser-redesign/` (executed, awaiting benchmark/parity acceptance or remediation).
- Phase 5.2 directory: `.planning/phases/05.2-minimal-artifact-and-performance-acceptance/` (executed; benchmark performance, p95, and known malformed-file gaps accepted on 2026-05-02).
- Phase 6 directory: `.planning/phases/06-rabbitmq-s3-worker-integration/` (executed; worker mode delivered, Phase 7 owns parallel/container hardening).

The implemented developer validation commands are:

```bash
cargo test -p parser-contract
cargo test -p parser-core
cargo check -p parser-cli --all-targets
cargo test -p parser-cli
cargo test -p parser-harness
cargo test -p parser-worker
cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v3.schema.json
cargo run -p parser-contract --example export_worker_schemas -- --output-dir schemas
```

The broader workspace gate is:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
scripts/coverage-gate.sh --check
COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict
scripts/fault-report-gate.sh
scripts/benchmark-phase5.sh --ci
RUN_PHASE5_FULL_CORPUS=1 scripts/benchmark-phase5.sh --ci
```

## Benchmark Acceptance

Phase 5.2 benchmark acceptance is not defined by tiny fixtures. Tiny fixtures are smoke evidence only and do not define acceptance.

Full acceptance requires `scripts/benchmark-phase5.sh --ci` to write `.planning/generated/phase-05/benchmarks/benchmark-report.json` with these gates:

- Selected large replay evidence: the script automatically selects the largest raw replay under `~/sg_stats/raw_replays` by byte size, tie-breaking by lexicographic path, then records selected path, raw bytes, SHA-256, old/new wall times, old-vs-new parity, default artifact bytes, and historical `x3_status`.
- All-raw evidence: every `*.json` file in `~/sg_stats/raw_replays` is attempted sequentially, the old baseline uses direct legacy `parseReplayInfo` with `HOME=<generated-fake-home> WORKER_COUNT=1` and no legacy skip filters, and the new parser writes default artifacts sequentially.
- Performance acceptance: current measured performance is accepted by the product owner; `x3_status` and `x10_status` remain report fields, but no longer block Phase 6 by themselves.
- Failure acceptance: all-raw failures pass when there are zero failed/skipped files, or when failures match a user-approved malformed/non-JSON allowlist and the cached old baseline reports the same failure count.
- Size acceptance: percentile ratios are reported for trend visibility, but p95 > 10% is accepted; the blocking size criterion is `artifact_size_limit_bytes: 100000`, with selected `artifact_bytes <= 100000`, all-raw `max_artifact_bytes <= 100000`, and `oversized_artifact_count == 0`.

Latest Phase 6 final-gate all-raw evidence records cached old wall time `501274.528655ms`, new wall time `272233.457364ms`, speedup `1.8413x`, all-raw old/new attempted files `23473`, new successes/failures/skips `23469/4/0`, p95 artifact/raw ratio `0.12432307336264753`, `max_artifact_bytes: 48270`, and `oversized_artifact_count: 0`. The current performance is accepted, the p95 ratio is accepted, and the 4 malformed/non-JSON failures are accepted through `.planning/benchmarks/phase-05-all-raw-accepted-failures.json` as long as old/new failure parity remains unchanged.

Short cargo aliases are also available:

```bash
cargo fmt-check
cargo lint
cargo quality-check
cargo quality-test
cargo quality-doc
```

Phase 6 worker integration proceeded on top of the accepted Phase 5.2 benchmark policy and is now complete. Phase 7 can proceed with parallel execution proof, container probes, and operations hardening.

## Product Context

Solid Stats is a multi-project SolidGames statistics product:

- `replays-fetcher` owns replay discovery from the external source, raw S3 object writes, source metadata, and ingestion staging/outbox records.
- `replay-parser-2` owns replay parsing, deterministic parse artifacts, parser contract schema, CLI/worker modes, and parity with the old parser.
- `server-2` owns PostgreSQL persistence, public and private APIs, canonical player identity, Steam OAuth, roles, moderation, parse job orchestration, aggregate and bounty calculation, and operations visibility.
- `web` owns the browser UI, public stats pages, authenticated request flows, moderator/admin screens, and typed API consumption from `server-2`.

This project only owns parser behavior and parser output contracts. Replay discovery/fetching, website behavior, authentication, canonical identity matching, moderation workflows, and PostgreSQL business-table writes belong to the adjacent applications.

## What v1 Should Deliver

The first release should provide:

- A Rust parser for historical OCAP JSON replay files.
- A local CLI for parsing a replay file and writing parser output JSON.
- A RabbitMQ/S3 worker mode for `server-2` integration.
- S3 artifact-reference result delivery for successful worker parses.
- A compact deterministic server-facing artifact, not a full replay-shaped JSON dump.
- Minimal contribution/source evidence needed to audit and recalculate statistics.
- Optional debug/parity sidecars only when they are useful and explicitly requested.
- Explicit unknown/null states for missing winner, SteamID, killer, commander, or source fields.
- Legacy-compatible aggregate projections for current SolidGames statistics.
- No issue #13 vehicle score output in v1; v1 keeps ordinary `vehicleKills`, `killsFromVehicle`, weapon, attacker vehicle, and destroyed-vehicle facts and can reprocess raw replays if that statistic is revisited later.
- Golden corpus comparisons against `~/sg_stats`.
- Benchmarks against the pinned legacy parser baseline, recording selected and all-raw old/new wall times, historical x3/x10 target status, artifact-size percentiles, failure parity, and a hard 100 KB (100,000 bytes) maximum default artifact size.
- 100% reachable-code statement, branch, function, and line coverage as a release gate, with behavior-focused tests.

## Out of Scope

The parser will not own:

- Public website or UI.
- Replay discovery or production fetching from the external replay source.
- Steam OAuth.
- Canonical player identity matching.
- PostgreSQL business-table persistence.
- User roles, moderation, or correction request workflows.
- Direct stat editing or correction.
- Replay formats other than OCAP JSON in v1.
- Production Kubernetes deployment.
- Financial reward or payout logic.
- Annual/yearly nomination statistics and nomination pages; these are a separate v2 product surface and should reprocess raw OCAP files when revisited rather than forcing a large v1 default artifact.

## Data and References

The old parser and historical data define the v1 compatibility baseline:

- Legacy parser: `/home/afgan0r/Projects/SolidGames/replays-parser`.
- Historical raw replays: `~/sg_stats/raw_replays`.
- Historical calculated results: `~/sg_stats/results`.
- Legacy annual nomination outputs: `~/sg_stats/year_results`.
- Replay list metadata: `~/sg_stats/lists/replaysList.json`.

Current full-history validation facts:

- `~/sg_stats/raw_replays` contains 23,473 raw replay JSON files.
- `~/sg_stats/lists/replaysList.json` contains 23,456 replay-list rows prepared at `2026-04-25T04:42:54.889Z`.
- `~/sg_stats/results` contains 88,485 existing result files.
- `~/sg_stats/year_results` contains 14 yearly reference files.

The historical archive is for tests, golden validation, and benchmarks. It is not a production import source.
Annual/yearly nomination statistics are a separate legacy surface and are deferred to v2. They should not drive a large default v1 side artifact; raw replay reprocessing remains acceptable for that future product surface.

Phase 1 dossiers:

- `baseline-command-runtime.md`: canonical old-parser command, runtime versions, fake-HOME baseline runs, logs, and output hashes.
- `corpus-manifest.md`: current full-history corpus counts, schema/profile evidence, malformed files, game-type distribution, and fixture seed rationale.
- `legacy-rules-output-surfaces.md`: old parser game-type filters, skip rules, config inputs, identity compatibility rules, ordinary output surfaces, and v2-deferred yearly references.
- `mismatch-taxonomy-interface-notes.md`: old-vs-new mismatch categories plus parser artifact, `server-2`, and `web` impact dimensions.

## Architecture Direction

The expected implementation shape is:

- Rust 2024 Cargo workspace.
- Current contract crate at `crates/parser-contract`.
- Pure parser core at `crates/parser-core`, shared by the CLI, worker, tests, benchmarks, and comparison tools.
- Selective parsing for the v1 hot path, avoiding unnecessary full JSON DOM cloning or full JSON-to-JSON reserialization where practical.
- Thin runtime adapters for CLI and RabbitMQ/S3 worker mode.
- `serde` / `serde_json` for correctness-first OCAP JSON parsing.
- Deterministic contract serialization with stable ordering.
- `schemars` and semantic versioning for machine-readable parser contracts.
- `tracing` and structured `ParseFailure` output for diagnostics.

Parser output must preserve observed replay identity fields only, such as nickname, side, squad/group fields, entity IDs, and SteamID when available. Canonical player matching belongs to `server-2`.

Production raw replay discovery is owned by `replays-fetcher`: it writes raw replay objects under S3 `raw/` and ingestion staging records. `server-2` promotes staged records into canonical `replays` and `parse_jobs`, then passes `object_key` and `checksum` to this parser through RabbitMQ.

`replay-parser-2` owns the parser artifact contract and schema. Successful worker parses write deterministic minimal parser artifacts under S3 `artifacts/` and publish `parse.completed` with an artifact reference. `server-2` remains responsible for validating/storing parser artifacts, mapping them into PostgreSQL and OpenAPI-owned API shapes, and coordinating any API-visible changes with `web`.

## Worker Mode

`replay-parser-2 worker` is implemented for single-worker RabbitMQ/S3 integration.

Worker jobs are JSON messages validated by `schemas/parse-job-v1.schema.json`. Each job contains:

- `job_id`
- `replay_id`
- `object_key`
- `checksum`
- `parser_contract_version`

The worker reads raw replay objects from S3-compatible storage, computes the local SHA-256 digest, and emits a structured non-retryable failure when the supplied checksum does not match the downloaded bytes. Successful parses write the same minimal v3 artifact shape as the default CLI path; the worker does not use the debug sidecar path.

Successful artifacts use deterministic keys in the form `artifacts/v3/{encoded_replay_id}/{source_sha256}.json`, where the replay segment is percent-encoded and the checksum segment is the raw source SHA-256. The worker publishes `parse.completed` with `job_id`, `replay_id`, parser information, artifact bucket/key/checksum/size, and source checksum proof. Parser failures, malformed jobs, checksum mismatches, storage failures, and unsupported contract versions publish `parse.failed` with structured stage, error code, message, retryability, and optional malformed-job fields.

RabbitMQ delivery acknowledgement is manual. The worker acknowledges a job only after confirmed publication of either `parse.completed` or `parse.failed`; inability to publish a durable outcome is nacked for retry. The default prefetch is `1`; horizontal multi-worker safety, container probe endpoints, and runtime hardening are covered by Phase 7.

Runtime configuration is available through `replay-parser-2 worker --help` and matching environment variables. Do not commit broker credentials or cloud secrets; use local shell environment, secret managers, or deployment-time injection.

## User Commands

Implemented local CLI commands:

```bash
# Parse one replay file to minified minimal JSON by default.
replay-parser-2 parse path/to/replay.json --output path/to/artifact.json

# Request human-readable minimal JSON explicitly.
replay-parser-2 parse path/to/replay.json --output path/to/artifact.json --pretty

# Write internal full normalized detail for investigation.
replay-parser-2 parse path/to/replay.json --output path/to/artifact.json --debug-artifact path/to/debug.json

# Emit the v3 parser contract schema.
replay-parser-2 schema --output path/to/schema.json

# Compare new parser output against legacy or golden data.
# Markdown is the default review surface; JSON details are explicit.
replay-parser-2 compare --replay path/to/replay.json --old-artifact path/to/old.json --output path/to/report.md
replay-parser-2 compare --new-artifact path/to/new.json --old-artifact path/to/old.json --output path/to/report.md --detail-output path/to/report-details.json
replay-parser-2 compare --new-artifact path/to/new.json --old-artifact path/to/old.json --format json --output path/to/report.json
```

Coverage and fault gates:

```bash
scripts/coverage-gate.sh --check
COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict
scripts/fault-report-gate.sh
scripts/benchmark-phase5.sh --ci
```

Strict coverage is intentionally opt-in because `cargo llvm-cov --workspace --all-targets`
is resource-heavy. The wrapper defaults to one build job plus `nice`/`ionice`
and timeout limits; use `--check` for routine local verification.

Worker integration command:

```bash
replay-parser-2 worker
```

The command consumes RabbitMQ jobs, reads raw S3 objects, writes parser artifacts to S3-compatible storage, and publishes `parse.completed` or `parse.failed` result messages.

Deployable worker image:

```bash
docker build -t replay-parser-2-worker .
docker run --rm \
  -p 8080:8080 \
  -e REPLAY_PARSER_AMQP_URL \
  -e REPLAY_PARSER_S3_BUCKET='solid-replays' \
  -e AWS_REGION='us-east-1' \
  -e AWS_ACCESS_KEY_ID \
  -e AWS_SECRET_ACCESS_KEY \
  -e REPLAY_PARSER_PROBE_BIND='0.0.0.0' \
  -e REPLAY_PARSER_PROBE_PORT='8080' \
  replay-parser-2-worker
```

The worker image runs as `USER 65532:65532`, exposes port `8080`, and uses the hidden `replay-parser-2 healthcheck --url http://127.0.0.1:8080/readyz` command for Docker readiness without adding curl or debug tooling to the runtime image.

Runtime probes are configured with:

- `REPLAY_PARSER_PROBES_ENABLED` enables or disables the HTTP probe server.
- `REPLAY_PARSER_PROBE_BIND` sets the probe bind host; containers should use `0.0.0.0`.
- `REPLAY_PARSER_PROBE_PORT` sets the probe port.
- `REPLAY_PARSER_WORKER_ID` sets the stable worker identifier included in probe bodies and structured logs.

`/readyz` returns ready only after RabbitMQ and S3 dependency checks pass and flips unavailable during shutdown drain. `/livez` reports process liveness and remains available during dependency degradation, but returns failure for fatal worker state. The worker keeps RabbitMQ prefetch at `1`; horizontal scale is provided by running multiple worker instances, not by increasing in-process concurrency by default.

Multiple workers are safe because artifact writes use deterministic keys plus conditional create with compare/reuse/conflict fallback. Duplicate or redelivered jobs can complete idempotently by reusing byte-identical artifacts, while conflicting existing artifacts produce structured `parse.failed` results instead of overwriting data.

Local RabbitMQ/S3 smoke infrastructure:

```bash
scripts/worker-smoke.sh
```

The smoke script builds the worker image, starts RabbitMQ and MinIO with Docker Compose, prepares the bucket/topology, runs two worker containers (`worker-a` and `worker-b`) with `/livez` and `/readyz` probes, verifies duplicate redelivery artifact reuse, verifies an artifact-conflict `parse.failed` result, and greps structured worker logs for worker IDs and stable event names. Set `KEEP_SMOKE_INFRA=1` to leave the local services running for manual inspection.

Timeweb S3-compatible deployments should set:

```bash
REPLAY_PARSER_S3_ENDPOINT=https://s3.twcstorage.ru
REPLAY_PARSER_S3_FORCE_PATH_STYLE=true
AWS_REGION=ru-1
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
```

Supply credentials through the deployment secret store; do not commit them. To run the optional no-secret capability probe against an already configured Timeweb bucket, use:

```bash
TIMEWEB_S3_SMOKE=1 scripts/worker-smoke.sh
```

This mode prints only capability labels such as `timeweb_conditional_write=pass`, `timeweb_conditional_write=unsupported_fallback_required`, or `timeweb_conditional_write=failed`. If Timeweb conditional writes are unsupported or unreliable, the worker still uses the tested compare/reuse/conflict fallback before accepting or rejecting existing artifact objects.

## Development Workflow

Project development is performed only by AI agents using the GSD workflow. Direct non-GSD development is out of process for this repository.
This is an AI agents plus GSD-only workflow: project-changing work must be captured in GSD planning, phase execution, or quick-task artifacts.

For project-changing work:

- Use GSD planning, phase execution, or quick-task artifacts under `.planning/`.
- Keep README and planning docs current when scope, commands, architecture, validation data, benchmark expectations, integration workflow, or development workflow changes.
- End completed work with a clean git working tree by committing intended results.
- Do not delete completed work just to make `git status` clean.
- Ask the user when change ownership, commit intent, or cross-project compatibility is unclear.
- Challenge requests that conflict with current architecture, accepted decisions, quality standards, maintainability, or proportional scope; explain the risk and propose safer alternatives.

## Documentation Map

- `.planning/PROJECT.md`: full project context, active requirements, constraints, and decisions.
- `.planning/REQUIREMENTS.md`: v1 requirements and phase traceability.
- `.planning/ROADMAP.md`: milestone phase plan.
- `.planning/STATE.md`: current GSD state and completed quick tasks.
- `.planning/research/SUMMARY.md`: technical research and architecture rationale.
- `.planning/phases/04-event-semantics-and-aggregates/04-VERIFICATION.md`: Phase 4 verification result.
- `.planning/phases/04-event-semantics-and-aggregates/04-SECURITY.md`: Phase 4 threat mitigation verification.
- `.planning/phases/04-event-semantics-and-aggregates/04-UAT.md`: Phase 4 acceptance evidence.
- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md`: Phase 1 legacy parser command/runtime baseline.
- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md`: Phase 1 full-history corpus profile summary.
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md`: Phase 1 legacy filters, identity, and output-surface inventory.
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md`: Phase 1 mismatch taxonomy and cross-app interface notes.
- `gsd-briefs/`: project briefs for `replays-fetcher`, `replay-parser-2`, `server-2`, and `web`.
- `AGENTS.md`: repository-specific instructions for AI agents.

## README Maintenance

This README is the human-facing entry point for the repository. Keep it useful for SolidGames maintainers, product reviewers, and developers who are not already familiar with the GSD planning history.

Last updated: 2026-05-02 during Phase 7 parallel and container hardening.
