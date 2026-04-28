---
phase: 05
artifact: research
status: complete
researched_at: 2026-04-28
---

# Phase 05 Research - CLI, Golden Parity, Benchmarks, and Coverage Gates

## Research Question

What does Phase 5 need in order to plan local CLI tooling, old-vs-new parity,
coverage enforcement, mutation or fault reporting, and benchmark evidence without
violating parser-core purity or adjacent application ownership?

## Executive Summary

Phase 5 should add thin runtime and validation adapters around the verified
`parser-core::parse_replay(ParserInput)` API. The CLI should be a new workspace
binary named `replay-parser-2` with `parse`, `schema`, and `compare` subcommands.
It should compute SHA-256 locally, write a `ParseArtifact` to `--output` for both
successful parses and parser failures, emit only concise human summaries on
stderr, and return non-zero for malformed, unreadable, or unsupported inputs.

Golden parity should be implemented as a harness layer, not as parser-core
behavior. The harness owns legacy replay selection, skip classification, config
inputs, old/current result loading, and the Phase 1 mismatch taxonomy. Full
corpus runs and bulky reports belong under ignored generated paths; committed
fixtures should stay compact and traceable to `fixture-index.json`, Phase 4 edge
cases, and explicit requirement coverage.

Coverage and mutation gates should be treated as release gates around behavior,
not substitutes for strong tests. Use `cargo llvm-cov` as the canonical coverage
tool and require 100% reachable production Rust coverage with a reviewable
allowlist for generated, impossible, or defensive code. Use mutation testing
where available, or an equivalent deterministic fault-injection report that
classifies caught, missed, timeout, and unviable cases.

Benchmarks should be two-tier: Rust-stage benchmarks for parser-core phases and
command-level benchmarks for equivalent old/new workloads. Every speed report
must include workload identity, parity status, old baseline profile, throughput,
memory/RSS where practical, and 10x status as pass, fail, or unknown.

## Existing Implementation Hooks

| Area | Current hook | Planning implication |
|------|--------------|----------------------|
| Parser API | `parser_core::parse_replay(ParserInput)` | CLI, harness, tests, and benchmarks should call this API rather than duplicating parsing. |
| Source metadata | `ParserInput` carries `ReplaySource`, `ParserInfo`, and `ParserOptions` | CLI parse can compute SHA-256 and populate local source metadata without requiring user-supplied checksums. |
| Failure artifacts | `parse_replay` returns failed `ParseArtifact` for invalid JSON/root shape | CLI can write structured failure artifacts to `--output` and use exit status/stderr for command semantics. |
| Schema export | `parser_contract::schema::parse_artifact_schema()` and `export_schema` example | CLI schema command can promote existing source-of-truth generation. |
| Determinism tests | `crates/parser-core/tests/deterministic_output.rs` | Phase 5 should add CLI and golden determinism checks rather than changing parser-core determinism rules. |
| Fixtures | `crates/parser-core/tests/fixtures/` | Add compact focused fixtures and a committed fixture manifest; keep full corpus generated outputs ignored. |

## Old Parser and Corpus Inputs

The canonical old baseline remains `/home/afgan0r/Projects/SolidGames/replays-parser`
at the repaired source-command baseline. The semantic profile is
`HOME=<fake-home> WORKER_COUNT=1 pnpm run parse`, with Node `v18.14.0` and pnpm
`10.33.0`. The default-worker profile is diagnostic only because Phase 1 found
regenerated output drift between worker profiles.

Important Phase 1 evidence for Phase 5:

- `~/sg_stats/raw_replays` has 23,473 raw replay JSON files; 23,469 parsed in the
  corpus profiler and 4 were malformed.
- `~/sg_stats/results` has 88,485 current historical result files.
- Regenerated deterministic old-parser results have 86,895 result files and an
  aggregate digest that differs from current historical results.
- Current-vs-regenerated and deterministic-vs-default-worker differences remain
  `human review` until reports explain the cause and an approved preserve/fix
  decision exists.
- Yearly nomination outputs in `~/sg_stats/year_results` are v2-only reference
  material and must not enter ordinary v1 parity reports.

## Recommended Implementation Shape

### CLI

Create a new workspace member `crates/parser-cli` with a binary target named
`replay-parser-2`. The crate should depend on `parser-core`, `parser-contract`,
`clap`, `serde_json`, `sha2`, and `hex`.

Recommended subcommands:

- `replay-parser-2 parse <input> --output <path> [--replay-id <id>]`
- `replay-parser-2 schema [--output <path>]`
- `replay-parser-2 compare --replay <path> --old-artifact <path> --output <path>`
- `replay-parser-2 compare --new-artifact <path> --old-artifact <path> --output <path>`

The public command should keep output stable enough for `assert_cmd` integration
tests. Human stderr should summarize command failure only; the JSON artifact or
comparison report remains the primary machine-readable output.

### Comparison Harness

Use a separate reusable harness module or crate, such as `crates/parser-harness`,
so comparison logic can be used by the CLI, tests, and later CI scripts. The
harness should own:

- legacy selection and skip categories from Phase 1;
- current `~/sg_stats/results` and regenerated old-parser evidence loading;
- per-field or per-surface comparable values;
- mismatch categories: `compatible`, `intentional change`, `old bug preserved`,
  `old bug fixed`, `new bug`, `insufficient data`, and `human review`;
- impact dimensions: parser artifact, `server-2` persistence, `server-2`
  recalculation, and UI-visible public-stats impact.

Parser-core should not learn legacy replay-list filtering, config inputs, or
old output folder layout.

### Fixtures and Tests

Committed golden material should be compact:

- focused hand-written fixtures for bad JSON, old shape, partial/schema drift,
  winner-present, winner-missing, vehicle-kill, teamkill, commander-side, null
  killer, duplicate-slot same-name, and connected-player backfill;
- a committed manifest mapping each fixture to requirements and decisions;
- optional scripts that copy selected real corpus seeds into ignored generated
  paths for local/full-corpus runs.

Tests should follow RITE/AAA from `unit-tests-philosophy`: readable names,
isolated fixture state, thorough success/error/boundary coverage, explicit
observable assertions, and no test-only production exports.

### Coverage

Use `cargo llvm-cov` as the canonical gate. Plan for command wrappers that:

- run all workspace tests with coverage instrumentation;
- exclude test code from the denominator;
- require 100% lines, functions, regions, and branches where the installed
  `cargo-llvm-cov` supports those metrics;
- fail when production code is uncovered and not listed in the allowlist;
- print allowlist entries with file path, line/pattern, reason, and reviewer.

Coverage exclusions should be committed in a reviewable file such as
`coverage/allowlist.toml`, and inline code rationale should sit near excluded
branches.

### Mutation or Fault Reporting

Prefer `cargo-mutants` for parser-core and aggregate logic if available. If the
tool is unavailable or impractical for a specific target, use an equivalent
fault-injection harness that mutates or perturbs decision tables, killed-event
tuples, vehicle score weights, failure paths, and aggregate contribution rules.

The required report must classify each case as:

- `caught` - tests failed as expected;
- `missed` - tests passed despite the fault;
- `timeout` - execution exceeded limit;
- `unviable` - mutation could not compile or does not represent reachable
  behavior.

Any high-risk `missed` case blocks Phase 5 until tests are strengthened or the
case is documented as accepted non-applicable.

### Benchmarks

Use Rust benchmark tooling for parser-core stages and command-level benchmarking
for old/new workload comparison.

Required report fields:

- workload tier: small CI sample, curated representative sample, or manual full
  corpus;
- exact fixture list or corpus selector;
- old baseline profile, normally deterministic `WORKER_COUNT=1`;
- parity status for the measured sample;
- parse-only, aggregate-only, and end-to-end timing where measurable;
- files/sec, MB/sec or events/sec, wall time, and memory/RSS where practical;
- 10x status: `pass`, `fail`, or `unknown`.

If the benchmark report is below roughly 10x, the phase should not mark success
until a triage report identifies bottlenecks, proves parity was not sacrificed,
and either fixes the issue or records an accepted gap.

## Cross-Application Compatibility

Phase 5 is mostly local tooling and validation, but comparison reports can
classify differences that would affect `server-2` persistence, `server-2`
recalculation, or `web` public stats. The plan should preserve this boundary:

- no PostgreSQL writes or schema ownership in this repository;
- no RabbitMQ/S3 worker implementation before Phase 6;
- no replay discovery/fetching before or during Phase 5;
- no canonical player identity matching in CLI or harness;
- report downstream impact as metadata, not by changing adjacent apps.

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| CLI hides structured failure details in stderr only | Always write success or failure `ParseArtifact` to `--output`; stderr remains concise human context. |
| Harness imports legacy filters into parser-core | Keep comparison code in `parser-harness`/CLI adapters and reference Phase 1 boundaries in tests. |
| Full corpus artifacts bloat git | Commit only compact fixtures/manifests; write bulky outputs under `.planning/generated/phase-05/`. |
| Coverage target encourages brittle private tests | Use behavior-level RITE tests and allowlist only impossible/generated/defensive unreachable code. |
| Mutation tooling is unavailable | Provide deterministic fault-injection fallback and keep report schema identical. |
| 10x claim is measured on unmatched workloads | Require parity status, workload list, old profile, and debug/release mode in every benchmark report. |

## Validation Architecture

| Dimension | Required gate |
|-----------|---------------|
| CLI behavior | `cargo test -p parser-cli` with `assert_cmd` tests for parse/schema/compare and non-zero failure exits. |
| Core and contract regressions | `cargo test -p parser-contract` and `cargo test -p parser-core`. |
| Golden fixture coverage | Fixture manifest tests assert each Phase 5 requirement and edge category has at least one focused fixture or explicit unavailable note. |
| Comparison reports | Harness tests assert mismatch category validation and all four impact dimensions are required. |
| Determinism | CLI and core tests serialize repeated artifacts byte-identically for the same input. |
| Coverage | `cargo llvm-cov` wrapper fails below 100% reachable production coverage unless allowlisted. |
| Mutation/fault report | Report validator fails on high-risk `missed` cases without an accepted non-applicable note. |
| Benchmarks | Report validator requires workload identity, parity status, old profile, throughput, memory/RSS note, and 10x status. |
| Final workspace quality | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps`, and `git diff --check`. |

## Planning Implications

The phase should be split into six plans:

1. CLI crate with parse/schema commands and structured failure behavior.
2. Golden fixture curation and manifest-driven behavior coverage.
3. Harness and compare command with mismatch taxonomy and impact dimensions.
4. Coverage gates and behavior-test strengthening.
5. Mutation/fault reporting gate.
6. Benchmarks, README updates, final report validation, and full quality gates.

## Research Complete

This research is sufficient for Phase 5 planning. No external web lookup is
required because Phase 5 uses established Rust tooling patterns and local project
evidence; the uncertainty is project-specific fixture and baseline selection,
which is covered by Phase 1 artifacts.
