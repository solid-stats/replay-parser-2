# Parser reference

Technical reference for `replay-parser-2`, the Rust OCAP replay parser for Solid Stats.
This document holds the depth that does not belong in the front-door README: artifact
contract, CLI and worker behavior, deployment, quality gates, validation data, and the
historical v1.0 acceptance record.

For the platform-wide ownership boundaries this repo operates under, see the cross-app
boundary map in the `solid-stats/skills` repo (`solidstats-shared-project-standards`).

## Contents

- [What v1 delivers](#what-v1-delivers)
- [Status and milestones](#status-and-milestones)
- [Artifact contract](#artifact-contract)
- [Architecture](#architecture)
- [CLI commands](#cli-commands)
- [Worker mode](#worker-mode)
- [Deployment](#deployment)
- [Quality gates and build budgets](#quality-gates-and-build-budgets)
- [Validation data and references](#validation-data-and-references)
- [Out of scope](#out-of-scope)
- [Historical v1.0 acceptance](#historical-v10-acceptance)
- [Planning and documentation map](#planning-and-documentation-map)

## What v1 delivers

The first release provides:

- A Rust parser for historical OCAP JSON replay files.
- A local CLI for parsing a replay file and writing parser output JSON.
- A RabbitMQ/S3 worker mode for `server-2` integration.
- S3 artifact-reference result delivery for successful worker parses.
- A compact deterministic server-facing artifact, not a full replay-shaped JSON dump.
- Minimal contribution/source evidence needed to audit and recalculate statistics.
- Optional internal debug sidecars only when they are explicitly requested.
- Explicit unknown/null states for missing winner, SteamID, killer, commander, or source
  fields.
- Legacy-compatible aggregate projections for current SolidGames statistics.
- No issue #13 vehicle score output in v1; v1 keeps ordinary `vehicleKills`,
  `killsFromVehicle`, weapon, attacker vehicle, and destroyed-vehicle facts and can
  reprocess raw replays if that statistic is revisited later.
- Curated regression fixtures derived from `~/sg_stats`.
- Historical v1 acceptance evidence for performance, artifact size, and malformed-file
  behavior.
- 100% reachable-code statement, branch, function, and line coverage as a release gate,
  with behavior-focused tests.

A design constraint runs through all of this: the default v1 output must *reduce* replay
data for `server-2`. A 10–15 MB OCAP replay should not become another 10–15 MB JSON
artifact on the ordinary ingestion path.

## Status and milestones

The v1.0 milestone is complete and archived. The repository contains the Rust workspace
with `crates/parser-contract`, generated JSON Schema, committed success/failure examples,
contract tests, the pure parser core at `crates/parser-core`, the CLI adapter binary
`replay-parser-2`, the RabbitMQ/S3 worker adapter at `crates/parser-worker`, and strict
quality gate helpers at `crates/parser-quality`.

Phase 5.2 replaced the earlier compact artifact with a v3 minimal v1 statistics artifact,
moved detailed evidence behind an explicit debug sidecar, removed GitHub issue #13 vehicle
score from v1, and recorded accepted current performance, hard max-artifact-size, and
malformed-file parity evidence before worker integration. Phase 6 worker integration
proceeded on top of the accepted Phase 5.2 policy. Phase 7 completed parallel safety,
container probes, and operations hardening.

Quick reference:

- Current focus: awaiting the next milestone definition after v1.0 archival.
- Roadmap: 9 phases.
- v1 requirements: 80 mapped requirements.
- Current artifact contract version: `3.0.0`.
- Crates: `crates/parser-contract`, `crates/parser-core`, `crates/parser-cli`,
  `crates/parser-worker`, `crates/parser-quality`.
- Contract schema: `schemas/parse-artifact-v3.schema.json`.
- Worker schemas: `schemas/parse-job-v1.schema.json` and
  `schemas/parse-result-v1.schema.json`.
- Example artifacts: `crates/parser-contract/examples/parse_artifact_success.v3.json` and
  `crates/parser-contract/examples/parse_failure.v3.json`.
- Worker examples: `crates/parser-contract/examples/parse_job.v1.json`,
  `crates/parser-contract/examples/parse_completed.v1.json`, and
  `crates/parser-contract/examples/parse_failed.v1.json`.
- Phase 3 plans: `.planning/phases/03-deterministic-parser-core/03-00-PLAN.md` through
  `03-05-PLAN.md`.
- Phase 4 plans: `.planning/phases/04-event-semantics-and-aggregates/04-00-PLAN.md`
  through `04-06-PLAN.md`.
- Phase 5 plans:
  `.planning/phases/05-cli-golden-parity-benchmarks-and-coverage-gates/05-00-PLAN.md`
  through `05-05-PLAN.md`.
- Phase 5.1 directory:
  `.planning/phases/05.1-compact-artifact-and-selective-parser-redesign/` (executed;
  superseded by Phase 5.2 acceptance).
- Phase 5.2 directory:
  `.planning/phases/05.2-minimal-artifact-and-performance-acceptance/` (executed;
  benchmark performance, p95, and known malformed-file gaps accepted on 2026-05-02; active
  benchmark tooling retired after v1.0).
- Phase 6 directory: `.planning/phases/06-rabbitmq-s3-worker-integration/` (executed;
  worker mode delivered, Phase 7 owns parallel/container hardening).

## Artifact contract

The default v3 artifact is a minimal, deterministic, server-facing shape — not a
replay-shaped JSON dump. The ordinary default tables are `players[]`, `weapons[]`,
`destroyed_vehicles[]`, and `diagnostics[]`.

- Player-authored enemy/team kills are nested under the killer at `players[].kills`.
- Same-name slots are merged into one replay-local player row.
- Legacy squad tags are split from observed nicknames.
- Source refs, rule IDs, entity snapshots, and normalized event/entity evidence stay out
  of ordinary ingestion unless the debug sidecar is explicitly requested.

Parser output preserves observed replay identity fields only — nickname, side, squad/group
fields, entity IDs, and SteamID when available. Canonical player matching belongs to
`server-2`; the parser must not collapse observed identity into canonical identity.

The minimal v3 artifact is the primary server-facing output. Full normalized evidence
(source refs, rule IDs, entity snapshots, normalized event/entity records) belongs only to
the explicit debug sidecar, requested per-parse with `--debug-artifact <path>` on the CLI.
The worker never uses the debug sidecar path.

## Architecture

The implementation shape is:

- Rust 2024 Cargo workspace (Rust 1.95).
- Current contract crate at `crates/parser-contract`.
- Pure parser core at `crates/parser-core`, shared by the CLI, worker, and tests.
- Selective parsing for the v1 hot path, avoiding unnecessary full JSON DOM cloning or
  full JSON-to-JSON reserialization where practical.
- Thin runtime adapters for CLI and RabbitMQ/S3 worker mode.
- `serde` / `serde_json` for correctness-first OCAP JSON parsing.
- Deterministic contract serialization with stable ordering.
- `schemars` and semantic versioning for machine-readable parser contracts.
- `tracing` and structured `ParseFailure` output for diagnostics.

### Place in the ingestion pipeline

- Production raw replay discovery is owned by `replays-fetcher`: it writes raw replay
  objects under S3 `raw/` and ingestion staging records.
- `server-2` promotes staged records into canonical `replays` and `parse_jobs`, then passes
  `object_key` and `checksum` to this parser through RabbitMQ.
- `replay-parser-2` owns the parser artifact contract and schema. Successful worker parses
  write deterministic minimal parser artifacts under S3 `artifacts/` and publish
  `parse.completed` with an artifact reference.
- `server-2` remains responsible for validating/storing parser artifacts, mapping them into
  PostgreSQL and OpenAPI-owned API shapes, and coordinating any API-visible changes with
  `web`.

## CLI commands

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
```

Contract and worker schema export (used to regenerate committed schemas):

```bash
cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v3.schema.json
cargo run -p parser-contract --example export_worker_schemas -- --output-dir schemas
```

## Worker mode

`replay-parser-2 worker` is implemented for single-worker RabbitMQ/S3 integration.

```bash
replay-parser-2 worker
```

The command consumes RabbitMQ jobs, reads raw S3 objects, writes parser artifacts to
S3-compatible storage, and publishes `parse.completed` or `parse.failed` result messages.

### Jobs

Worker jobs are JSON messages validated by `schemas/parse-job-v1.schema.json`. Each job
contains:

- `job_id`
- `replay_id`
- `object_key`
- `checksum`
- `parser_contract_version`

The worker reads raw replay objects from S3-compatible storage, computes the local SHA-256
digest, and emits a structured non-retryable failure when the supplied checksum does not
match the downloaded bytes. Successful parses write the same minimal v3 artifact shape as
the default CLI path; the worker does not use the debug sidecar path.

### Results

Successful artifacts use deterministic keys in the form
`artifacts/v3/{encoded_replay_id}/{source_sha256}.json`, where the replay segment is
percent-encoded and the checksum segment is the raw source SHA-256. The worker publishes
`parse.completed` with `job_id`, `replay_id`, parser information, artifact
bucket/key/checksum/size, and source checksum proof.

Parser failures, malformed jobs, checksum mismatches, storage failures, and unsupported
contract versions publish `parse.failed` with structured stage, error code, message,
retryability, and optional malformed-job fields.

### Delivery and acknowledgement

RabbitMQ delivery acknowledgement is manual. The worker acknowledges a job only after
confirmed publication of either `parse.completed` or `parse.failed`; inability to publish a
durable outcome is nacked for retry. The default prefetch is `1`.

Multiple workers are safe because artifact writes use deterministic keys plus conditional
create with compare/reuse/conflict fallback. Duplicate or redelivered jobs can complete
idempotently by reusing byte-identical artifacts, while conflicting existing artifacts
produce structured `parse.failed` results instead of overwriting data. Horizontal scale is
provided by running multiple worker instances, not by increasing in-process concurrency by
default.

### RabbitMQ topology

The shared staging RabbitMQ topology is owned by `server-2`: parser jobs are published to
`solid_stats.parser` with routing key `parse.requested` and routed to durable queue
`server2.parse.requested`. The worker consumes `server2.parse.requested` and publishes
`parse.completed` / `parse.failed` results back to the same `solid_stats.parser` exchange.

### Runtime configuration

Runtime configuration is available through `replay-parser-2 worker --help` and matching
environment variables. Do not commit broker credentials or cloud secrets; use local shell
environment, secret managers, or deployment-time injection.

Runtime probes are configured with:

- `REPLAY_PARSER_PROBES_ENABLED` enables or disables the HTTP probe server.
- `REPLAY_PARSER_PROBE_BIND` sets the probe bind host; containers should use `0.0.0.0`.
- `REPLAY_PARSER_PROBE_PORT` sets the probe port.
- `REPLAY_PARSER_WORKER_ID` sets the stable worker identifier included in probe bodies and
  structured logs.

`/readyz` returns ready only after RabbitMQ and S3 dependency checks pass and flips
unavailable during shutdown drain. `/livez` reports process liveness and remains available
during dependency degradation, but returns failure for fatal worker state.

## Deployment

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

The worker image runs as `USER 65532:65532`, exposes port `8080`, and uses the hidden
`replay-parser-2 healthcheck --url http://127.0.0.1:8080/readyz` command for Docker
readiness without adding curl or debug tooling to the runtime image.

The v1.0 local Docker Compose smoke deployment was retired after milestone acceptance;
worker behavior remains covered by focused worker tests and deployment-time checks.

### Timeweb S3-compatible deployments

Timeweb S3-compatible deployments should set:

```bash
REPLAY_PARSER_S3_ENDPOINT=https://s3.twcstorage.ru
REPLAY_PARSER_S3_FORCE_PATH_STYLE=true
AWS_REGION=ru-1
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
```

Supply credentials through the deployment secret store; do not commit them. If Timeweb
conditional writes are unsupported or unreliable, the worker still uses the tested
compare/reuse/conflict fallback before accepting or rejecting existing artifact objects.

## Quality gates and build budgets

Per-crate developer validation commands:

```bash
cargo test -p parser-contract
cargo test -p parser-core
cargo check -p parser-cli --all-targets
cargo test -p parser-cli
cargo test -p parser-quality
cargo test -p parser-worker
cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v3.schema.json
cargo run -p parser-contract --example export_worker_schemas -- --output-dir schemas
```

The broader workspace gate:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
scripts/coverage-gate.sh --check
COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict
scripts/fault-report-gate.sh
```

Short cargo aliases are also available:

```bash
cargo fmt-check
cargo lint
cargo quality-check
cargo quality-test
cargo quality-doc
```

### Coverage and fault gates

```bash
scripts/coverage-gate.sh --check
COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict
scripts/fault-report-gate.sh
```

Strict coverage is intentionally opt-in because instrumented workspace coverage is
resource-heavy. The wrapper runs bins, tests, and examples, but excludes benchmark targets.
It also passes `--no-cfg-coverage` so source-level unit tests guarded by `not(coverage)`
stay active. It defaults to one build job, `nice`/`ionice`, timeout limits, a 10 GiB
`.coverage/target` size budget, and `--no-clean` so repeated local coverage runs can reuse
build artifacts. Each run still clears stale `.profraw` profile data so reports do not
accumulate old executions. Automatic build-artifact cleanup now only removes
`.coverage/target/llvm-cov-target` when `.coverage/target` exceeds
`COVERAGE_MAX_TARGET_MIB`; set `COVERAGE_AUTO_CLEAN=0` to disable even that over-budget
cleanup.

### Build budgets

Local Cargo builds are configured with smaller debug/test artifacts and incremental
compilation disabled to keep `target/` from growing unbounded during repeated worker test
builds. For heavy local runs, use the budget wrapper:

```bash
scripts/cargo-budget.sh test -p parser-worker
CARGO_TARGET_MAX_MIB=6144 CARGO_TARGET_KEEP_MIB=4915 scripts/cargo-budget.sh check --workspace --all-targets
```

The wrapper runs Cargo normally, then prunes coverage artifacts, incremental state, stale
hashed test binaries, and oldest dependency artifacts only when `target/` exceeds the
configured budget. Run `scripts/prune-cargo-target.sh` directly to prune without starting
another build.

## Validation data and references

The old parser and historical data defined the v1 compatibility baseline:

- Legacy parser: `replays-parser`.
- Historical raw replays: `~/sg_stats/raw_replays`.
- Historical calculated results: `~/sg_stats/results`.
- Legacy annual nomination outputs: `~/sg_stats/year_results`.
- Replay list metadata: `~/sg_stats/lists/replaysList.json`.

Current full-history validation facts:

- `~/sg_stats/raw_replays` contains 23,473 raw replay JSON files.
- `~/sg_stats/lists/replaysList.json` contains 23,456 replay-list rows prepared at
  `2026-04-25T04:42:54.889Z`.
- `~/sg_stats/results` contains 88,485 existing result files.
- `~/sg_stats/year_results` contains 14 yearly reference files.

The historical archive is for curated regression tests and future investigative work. It is
not a production import source. Annual/yearly nomination statistics are a separate legacy
surface and are deferred to v2; they should not drive a large default v1 side artifact, and
raw replay reprocessing remains acceptable for that future product surface.

Phase 1 dossiers:

- `baseline-command-runtime.md`: canonical old-parser command, runtime versions, fake-HOME
  baseline runs, logs, and output hashes.
- `corpus-manifest.md`: current full-history corpus counts, schema/profile evidence,
  malformed files, game-type distribution, and fixture seed rationale.
- `legacy-rules-output-surfaces.md`: old parser game-type filters, skip rules, config
  inputs, identity compatibility rules, ordinary output surfaces, and v2-deferred yearly
  references.
- `mismatch-taxonomy-interface-notes.md`: historical v1 old-vs-new mismatch categories plus
  parser artifact, `server-2`, and `web` impact dimensions.

## Out of scope

The parser does not own:

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
- Annual/yearly nomination statistics and nomination pages; these are a separate v2 product
  surface and should reprocess raw OCAP files when revisited rather than forcing a large v1
  default artifact.

This project only owns parser behavior and parser output contracts. Replay
discovery/fetching, website behavior, authentication, canonical identity matching,
moderation workflows, and PostgreSQL business-table writes belong to the adjacent
applications.

## Historical v1.0 acceptance

Phase 5.2 benchmark and parity evidence was used to accept the v1.0 parser direction before
worker integration. The product owner:

- accepted current measured performance on 2026-05-02,
- accepted p95 artifact/raw ratio above 10% as non-blocking because every successful default
  artifact stayed below 100 KB, and
- accepted the four known malformed/non-JSON all-raw failures when old/new failure parity
  matched.

Those benchmark/parity tools were migration acceptance aids, not maintained post-v1 product
features. Phase 5.2 performance and size evidence was accepted for v1.0, and the old-vs-new
benchmark/parity tools are no longer part of the active post-v1 workflow.

## Planning and documentation map

- `.planning/PROJECT.md`: full project context, active requirements, constraints, and
  decisions.
- `.planning/REQUIREMENTS.md`: v1 requirements and phase traceability.
- `.planning/ROADMAP.md`: milestone phase plan.
- `.planning/STATE.md`: current GSD state and completed quick tasks.
- `.planning/research/SUMMARY.md`: technical research and architecture rationale.
- `.planning/phases/04-event-semantics-and-aggregates/04-VERIFICATION.md`: Phase 4
  verification result.
- `.planning/phases/04-event-semantics-and-aggregates/04-SECURITY.md`: Phase 4 threat
  mitigation verification.
- `.planning/phases/04-event-semantics-and-aggregates/04-UAT.md`: Phase 4 acceptance
  evidence.
- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md`: Phase 1
  legacy parser command/runtime baseline.
- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md`: Phase 1 full-history
  corpus profile summary.
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md`: Phase 1
  legacy filters, identity, and output-surface inventory.
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md`:
  Phase 1 mismatch taxonomy and cross-app interface notes.
- `gsd-briefs/`: project briefs for `replays-fetcher`, `replay-parser-2`, `server-2`, and
  `web`.
- `AGENTS.md`: repository-specific instructions for AI agents.
