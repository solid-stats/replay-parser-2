# replay-parser-2

`replay-parser-2` is the Rust replacement for the legacy SolidGames OCAP replay parser.

The parser will turn OCAP JSON replay files into deterministic, versioned artifacts: normalized replay events, source references, structured parse failures, and aggregate outputs that the Solid Stats backend can persist, audit, compare against golden data, and use for public statistics.

## Current Status

Phase 5 execution is complete, but verification found a benchmark/parity gap. The repository contains the Rust workspace with `crates/parser-contract`, generated JSON Schema, committed success/failure examples, contract tests, the pure parser core at `crates/parser-core`, the parser harness at `crates/parser-harness`, and the CLI adapter binary `replay-parser-2`. Parser-core decodes OCAP JSON bytes through an adapter-safe API, normalizes replay metadata and observed entity facts, emits schema-drift diagnostics, preserves deterministic output ordering, records connected-player backfill plus duplicate-slot same-name compatibility as auditable observed facts/hints, and emits normalized combat events, aggregate contributions/projections, bounty inputs, vehicle score inputs, and typed side facts.

The CLI can parse a local OCAP JSON file into a deterministic `ParseArtifact`, export the current parser contract schema, and compare selected old/new artifacts or a selected replay against a saved old artifact. Compact golden fixtures and behavior regressions are in place. `scripts/coverage-gate.sh` runs `cargo llvm-cov` and a parser-harness JSON postprocessor that fails on unallowlisted production coverage gaps. `scripts/fault-report-gate.sh` prefers `cargo mutants` when installed and otherwise validates a deterministic fault-injection report. `scripts/benchmark-phase5.sh --ci` runs parser-stage benchmarks and, when `/home/afgan0r/Projects/SolidGames/replays-parser` plus `~/sg_stats` are available, records curated selected old/new comparison evidence under `.planning/generated/phase-05/comparison/`. The latest generated benchmark report is valid but records `ten_x_status=fail`, `parity_status=human_review`, so the 10x target remains open. RabbitMQ/S3 worker mode, full-corpus comparison automation, PostgreSQL persistence, public APIs, canonical identity handling, public UI, and annual/yearly nomination product support are not implemented in this parser yet.

- Current phase: Phase 5, `CLI, Golden Parity, Benchmarks, and Coverage Gates` (execution complete; benchmark/parity verification gap).
- Roadmap: 7 phases.
- v1 requirements: 71 mapped requirements.
- Contract crate: `crates/parser-contract`.
- Parser-core crate: `crates/parser-core`.
- CLI crate: `crates/parser-cli`.
- Harness crate: `crates/parser-harness`.
- Contract schema: `schemas/parse-artifact-v1.schema.json`.
- Example artifacts: `crates/parser-contract/examples/parse_artifact_success.v1.json` and `crates/parser-contract/examples/parse_failure.v1.json`.
- Phase 3 plans: `.planning/phases/03-deterministic-parser-core/03-00-PLAN.md` through `03-05-PLAN.md`.
- Phase 4 plans: `.planning/phases/04-event-semantics-and-aggregates/04-00-PLAN.md` through `04-06-PLAN.md`.
- Phase 5 plans: `.planning/phases/05-cli-golden-parity-benchmarks-and-coverage-gates/05-00-PLAN.md` through `05-05-PLAN.md`.

The implemented developer validation commands are:

```bash
cargo test -p parser-contract
cargo test -p parser-core
cargo check -p parser-cli --all-targets
cargo test -p parser-cli
cargo test -p parser-harness
cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v1.schema.json
```

The broader workspace gate is:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
scripts/coverage-gate.sh --check
scripts/coverage-gate.sh
scripts/fault-report-gate.sh
scripts/benchmark-phase5.sh --ci
```

Short cargo aliases are also available:

```bash
cargo fmt-check
cargo lint
cargo quality-check
cargo quality-test
cargo quality-doc
```

Worker mode and full-corpus parity automation are still planned for later phases. Phase 5 cannot be closed until the selected comparison surfaces are reviewed and the benchmark report either passes the 10x target or records an explicitly accepted gap.

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
- A local CLI for parsing a replay file and writing normalized JSON output.
- A RabbitMQ/S3 worker mode for `server-2` integration.
- S3 artifact-reference result delivery for successful worker parses.
- Deterministic normalized event output with source references.
- Explicit unknown/null states for missing winner, SteamID, killer, commander, or source fields.
- Legacy-compatible aggregate projections for current SolidGames statistics.
- Vehicle score support from GitHub issue #13.
- Golden corpus comparisons against `~/sg_stats`.
- Benchmarks against the pinned legacy parser baseline, targeting roughly 10x faster parsing on comparable workloads.
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
- Annual/yearly nomination statistics and nomination pages; these are a separate v2 product surface.

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
Annual/yearly nomination statistics are a separate legacy surface and are deferred to v2.

Phase 1 dossiers:

- `baseline-command-runtime.md`: canonical old-parser command, runtime versions, fake-HOME baseline runs, logs, and output hashes.
- `corpus-manifest.md`: current full-history corpus counts, schema/profile evidence, malformed files, game-type distribution, and fixture seed rationale.
- `legacy-rules-output-surfaces.md`: old parser game-type filters, skip rules, config inputs, identity compatibility rules, ordinary output surfaces, and v2-deferred yearly references.
- `mismatch-taxonomy-interface-notes.md`: old-vs-new mismatch categories plus parser artifact, `server-2`, and `web` impact dimensions.

## Architecture Direction

The expected implementation shape is:

- Rust 2024 Cargo workspace.
- Current contract crate at `crates/parser-contract`.
- Pure parser core at `crates/parser-core`, shared by the CLI, future worker, tests, benchmarks, and comparison tools.
- Thin runtime adapters for CLI and RabbitMQ/S3 worker mode.
- `serde` / `serde_json` for correctness-first OCAP JSON parsing.
- Deterministic contract serialization with stable ordering.
- `schemars` and semantic versioning for machine-readable parser contracts.
- `tracing` and structured `ParseFailure` output for diagnostics.

Parser output must preserve observed replay identity fields only, such as nickname, side, squad/group fields, entity IDs, and SteamID when available. Canonical player matching belongs to `server-2`.

Production raw replay discovery is owned by `replays-fetcher`: it writes raw replay objects under S3 `raw/` and ingestion staging records. `server-2` promotes staged records into canonical `replays` and `parse_jobs`, then passes `object_key` and `checksum` to this parser through RabbitMQ.

`replay-parser-2` owns the parser artifact contract and schema. Successful worker parses write deterministic parser artifacts under S3 `artifacts/` and publish `parse.completed` with an artifact reference. `server-2` remains responsible for validating/storing parser artifacts, mapping them into PostgreSQL and OpenAPI-owned API shapes, and coordinating any API-visible changes with `web`.

## User Commands

Implemented local CLI commands:

```bash
# Parse one replay file to a normalized artifact
replay-parser-2 parse path/to/replay.json --output path/to/artifact.json

# Emit the current parser contract schema
replay-parser-2 schema --output path/to/schema.json

# Compare new parser output against legacy or golden data
replay-parser-2 compare --replay path/to/replay.json --old-artifact path/to/old.json --output path/to/report.json
replay-parser-2 compare --new-artifact path/to/new.json --old-artifact path/to/old.json --output path/to/report.json
```

Coverage and fault gates:

```bash
scripts/coverage-gate.sh --check
scripts/coverage-gate.sh
scripts/fault-report-gate.sh
scripts/benchmark-phase5.sh --ci
```

Reserved command slots for later phases:

```bash
# Run worker mode for server integration
replay-parser-2 worker
```

Worker mode is Phase 6 scope and is not exposed by the current CLI.

## Development Workflow

Project development is performed only by AI agents using the GSD workflow. Direct non-GSD development is out of process for this repository.

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

Last updated: 2026-04-28 after Phase 5 curated benchmark gap evidence.
