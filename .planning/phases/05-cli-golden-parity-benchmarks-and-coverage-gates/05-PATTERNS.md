---
phase: 05
artifact: patterns
status: complete
created: 2026-04-28
---

# Phase 05 Pattern Map

## Purpose

Map Phase 5 planned files to the closest existing code and planning artifacts so
execution follows established local patterns rather than inventing new ones.

## Planned File Families

| Planned file family | Role | Closest existing analog | Pattern to preserve |
|---------------------|------|-------------------------|---------------------|
| `crates/parser-cli/src/main.rs` | Public CLI binary | `crates/parser-contract/examples/export_schema.rs` | Thin adapter around contract/core APIs; I/O outside parser-core. |
| `crates/parser-cli/tests/*.rs` | CLI command tests | `crates/parser-core/tests/*.rs`, `crates/parser-contract/tests/*.rs` | Behavior-oriented test names, explicit fixture setup, strong observable assertions. |
| `crates/parser-harness/src/*.rs` | Comparison, fixture, benchmark report support | `crates/parser-core/src/aggregates.rs`, Phase 1 dossiers | Deterministic report structs, stable ordering, no parser-core ownership drift. |
| `crates/parser-core/tests/fixtures/*` | Compact focused fixtures | Existing parser-core fixtures | Small OCAP JSON examples checked into git; bulky corpus outputs ignored. |
| `coverage/allowlist.toml` | Coverage exclusion review data | `.planning/phases/01-legacy-baseline-and-corpus/*.md` | Explicit path/reason records; no blanket hidden exclusions. |
| `scripts/coverage-gate.sh` | Coverage command wrapper | `.cargo/config.toml` cargo aliases | Reproducible local command that fails closed with clear missing-tool guidance. |
| `scripts/fault-report-gate.sh` | Mutation/fault command wrapper | Phase 5 validation strategy | Deterministic report location and blocked high-risk missed cases. |
| `benches/*.rs` or `crates/parser-harness/benches/*.rs` | Parser-stage benchmark | Existing parser-core public API | Bench through `parse_replay`, not duplicated parsing internals. |
| `README.md` | User-facing command/status docs | Current README | Keep implemented vs planned commands accurate and AI/GSD workflow visible. |

## Existing APIs To Reuse

- `parser_core::parse_replay(ParserInput)` is the only parser entrypoint CLI,
  harnesses, fixtures, and benchmarks should use.
- `ParserInput` owns replay bytes, source metadata, parser info, and options.
- `ReplaySource` already carries `replay_id`, `source_file`, and checksum
  presence state.
- `parser_contract::schema::parse_artifact_schema()` is the schema source of
  truth and should power `replay-parser-2 schema`.
- `ParseArtifact` already represents both success/partial and failed parser
  outcomes.
- Phase 1 mismatch categories and impact dimensions are the report vocabulary
  for comparison code.

## Existing Test Style

Current tests use:

- integration-style tests through public crate APIs;
- focused OCAP JSON fixtures under `crates/parser-core/tests/fixtures`;
- `expect` only in test code with explicit allow rationale;
- deterministic JSON serialization checks;
- grep-verifiable schema/example freshness checks.

Phase 5 tests should continue this style and apply the RITE/AAA standard from
`unit-tests-philosophy`.

## Boundary Constraints

Do not add these concerns during Phase 5:

- RabbitMQ, S3 worker mode, ack/nack, or artifact upload behavior;
- PostgreSQL, API, OpenAPI, or public UI changes;
- replay discovery or production raw fetches;
- canonical player identity matching;
- annual/yearly nomination product support.

Reports may describe downstream impact, but they must not implement adjacent app
behavior.

## Pattern Mapping Complete

The plan can proceed with a CLI adapter crate, a reusable harness crate, compact
fixtures, script/report gates, and benchmark docs without changing parser-core
ownership.
