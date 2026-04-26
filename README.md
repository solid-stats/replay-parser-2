# replay-parser-2

`replay-parser-2` is the planned Rust replacement for the legacy SolidGames OCAP replay parser.

The parser will turn OCAP JSON replay files into deterministic, versioned artifacts: normalized replay events, source references, structured parse failures, and aggregate outputs that the Solid Stats backend can persist, audit, compare against golden data, and use for public statistics.

## Current Status

Phase 2 contract work is implemented. The repository now contains the initial Rust workspace with `crates/parser-contract`, generated JSON Schema, committed success/failure examples, and contract tests. It does not yet contain the parser core, CLI binary, RabbitMQ/S3 worker, golden parity harness, or benchmark suite.

- Current phase: Phase 2, `Versioned Output Contract`.
- Roadmap: 7 phases.
- v1 requirements: 71 mapped requirements.
- Contract crate: `crates/parser-contract`.
- Contract schema: `schemas/parse-artifact-v1.schema.json`.
- Example artifacts: `crates/parser-contract/examples/parse_artifact_success.v1.json` and `crates/parser-contract/examples/parse_failure.v1.json`.

The implemented developer validation commands are:

```bash
cargo test -p parser-contract
cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v1.schema.json
```

The broader workspace gate is:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Parse, worker, comparison, and benchmark commands are still planned for later phases.

## Product Context

Solid Stats is a multi-project SolidGames statistics product:

- `replay-parser-2` owns replay parsing, deterministic parse artifacts, parser contract schema, CLI/worker modes, and parity with the old parser.
- `server-2` owns PostgreSQL persistence, public and private APIs, canonical player identity, Steam OAuth, roles, moderation, parse job orchestration, aggregate and bounty calculation, and operations visibility.
- `web` owns the browser UI, public stats pages, authenticated request flows, moderator/admin screens, and typed API consumption from `server-2`.

This project only owns parser behavior and parser output contracts. Website behavior, authentication, canonical identity matching, moderation workflows, and PostgreSQL business-table writes belong to the adjacent applications.

## What v1 Should Deliver

The first release should provide:

- A Rust parser for historical OCAP JSON replay files.
- A local CLI for parsing a replay file and writing normalized JSON output.
- A RabbitMQ/S3 worker mode for `server-2` integration.
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
- Pure parser core shared by CLI, worker, tests, benchmarks, and comparison tools.
- Thin runtime adapters for CLI and RabbitMQ/S3 worker mode.
- `serde` / `serde_json` for correctness-first OCAP JSON parsing.
- Deterministic contract serialization with stable ordering.
- `schemars` and semantic versioning for machine-readable parser contracts.
- `tracing` and structured `ParseFailure` output for diagnostics.

Parser output must preserve observed replay identity fields only, such as nickname, side, squad/group fields, entity IDs, and SteamID when available. Canonical player matching belongs to `server-2`.

`replay-parser-2` owns the parser artifact contract and schema. `server-2` remains responsible for validating/storing parser artifacts, mapping them into PostgreSQL and OpenAPI-owned API shapes, and coordinating any API-visible changes with `web`.

## Planned User Commands

These commands are not implemented yet. They describe the intended shape of the developer and operator interface:

```bash
# Parse one replay file to a normalized artifact
sg-replay-parser parse path/to/replay.json --output path/to/artifact.json

# Emit the current parser contract schema
sg-replay-parser schema --output path/to/schema.json

# Compare new parser output against legacy or golden data
sg-replay-parser compare --replay path/to/replay.json --golden path/to/expected.json

# Run worker mode for server integration
sg-replay-parser worker
```

Exact command names and flags will be finalized during the CLI phase.

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
- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md`: Phase 1 legacy parser command/runtime baseline.
- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md`: Phase 1 full-history corpus profile summary.
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md`: Phase 1 legacy filters, identity, and output-surface inventory.
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md`: Phase 1 mismatch taxonomy and cross-app interface notes.
- `gsd-briefs/`: project briefs for `replay-parser-2`, `server-2`, and `web`.
- `AGENTS.md`: repository-specific instructions for AI agents.

## README Maintenance

This README is the human-facing entry point for the repository. Keep it useful for SolidGames maintainers, product reviewers, and developers who are not already familiar with the GSD planning history.

Last updated: 2026-04-26.
