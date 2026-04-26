# Phase 2: Versioned Output Contract - Research

**Researched:** 2026-04-26
**Domain:** Rust parser output contract, JSON schema, explicit unknown states, source references, and structured parse failures
**Confidence:** HIGH for local scope and legacy handoff; MEDIUM for final crate versions until execution pins `Cargo.lock`

<user_constraints>
## User Constraints From CONTEXT.md

Phase 2 defines the stable `ParseArtifact` and `ParseFailure` contract that `server-2` and parser tooling consume. It does not implement parsing behavior, event semantics, aggregate calculation, CLI commands, worker behavior, old-vs-new comparison execution, or adjacent app changes.

Locked decisions that must shape the plan:

- D-01 through D-04 require one unified `ParseArtifact` envelope, separate `contract_version` and parser build/provenance metadata, a full v1 skeleton now, `snake_case` fields, and deterministic serialization.
- D-05 through D-08 require explicit presence semantics for optional data, compact tagged-union unknown/null/inferred values, and path-based diagnostics for non-fatal drift.
- D-09 through D-13 require source references for normalized events and aggregate contributions, stable rule IDs for derived values, an aggregate contribution reference shape, and no raw replay snippets in normal artifacts.
- D-14 through D-19 require status values `success`, `partial`, `skipped`, `failed`, legacy aggregate skips separated from hard parse failures, structured retryability, schema generation from Rust types, parser-owned contract boundaries, and stable namespaced error codes.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Requirement | Planning implication |
|----|-------------|----------------------|
| OUT-01 | Stable JSON `ParseArtifact` with parser version, contract version, replay/source IDs, checksum, status metadata | Create a Rust workspace and `parser-contract` crate before parser core exists. Define version/source/status envelope types and example artifacts. |
| OUT-02 | Normalized replay metadata | Define `ReplayMetadata` with mission/world/author/player-count/capture/end-frame/time-boundary fields using explicit presence values where data can be absent. |
| OUT-03 | Observed identity without canonical matching | Define `ObservedIdentity` and entity references that preserve nickname, side/faction, group/squad, role/description, entity ID, and SteamID presence state. |
| OUT-04 | Explicit unknown/null states | Create a reusable tagged union such as `FieldPresence<T>` with `present`, `explicit_null`, `unknown`, `inferred`, and `not_applicable` states. |
| OUT-05 | Source refs for normalized events and aggregate contributions | Define `SourceRef`, stable `RuleId`, `NormalizedEvent`, and `AggregateContributionRef` skeletons with replay/file/frame/event/entity/json-path/rule coordinates. |
| OUT-06 | JSON Schema generation or equivalent validation | Generate schema from Rust contract types with `schemars`; commit `schemas/parse-artifact-v1.schema.json`; validate example success/failure artifacts in tests. |
| OUT-07 | Structured `ParseFailure` output | Define failure stage, namespaced error code, retryability enum, message, source cause, source refs, and job/replay/file identifiers. |
</phase_requirements>

## Local Findings

There is no Rust workspace or production code yet. Current tracked files are documentation, planning artifacts, and product briefs. Phase 2 should therefore create the initial Rust workspace and keep the first implementation surface small: one contract crate with tests, examples, and schema output.

Phase 1 established these constraints for Phase 2:

- Legacy game-type filters, `sgs` exclusion, `sm` cutoff, empty/mace skip behavior, config exclusions, name changes, and same-name compatibility belong to parity harness or compatibility projection, not parser-core contract filtering.
- Observed identity must stay raw enough for `server-2` to own canonical player matching.
- Annual/yearly nomination outputs remain v2-only historical references and must not enter the v1 ordinary parser artifact.
- Current-vs-regenerated legacy result drift remains `human review`; the contract should expose source refs and diagnostics that make later mismatch triage possible.

Legacy TypeScript source confirms the initial contract fields:

- Raw OCAP top-level metadata includes `missionName`, `worldName`, `missionAuthor`, `playersCount`, `captureDelay`, `endFrame`, `entities`, `events`, `Markers`, and usually `EditorMarkers`.
- Player entities carry `id`, `name`, `side`, `group`, `description`, `isPlayer`, `positions`, and frame fields.
- Vehicle entities carry `id`, `name`, `class`, and `positions`.
- Kill events have tuple shape `[frame, "killed", killed_id, kill_info, distance]`, where `kill_info` can be `["null"]` for null killer.
- Connected events can backfill observed players by `name` and `entity_id`, but the compatibility merge by same name must not erase raw observed identity in the contract.

## Recommended Contract Shape

Use a `parser-contract` crate with modules:

- `version`: `ContractVersion`, `ParserInfo`, and semantic version/build metadata.
- `source_ref`: `SourceRef`, `RuleId`, and coordinate types.
- `presence`: `FieldPresence<T>`, `UnknownReason`, `NullReason`, and inferred-value metadata.
- `metadata`: `ReplayMetadata` and time/frame boundary types.
- `identity`: `ObservedEntity`, `ObservedIdentity`, `EntityKind`, `EntitySide`, and entity references.
- `events`: `NormalizedEvent`, `NormalizedEventKind`, participant refs, and event source refs.
- `aggregates`: `AggregateSection`, `AggregateContributionRef`, and placeholder contribution kinds for later Phase 4 semantics.
- `diagnostic`: path-based warnings with expected shape, observed shape, parser action, and source refs.
- `failure`: `ParseFailure`, `ParseStage`, `Retryability`, and namespaced `ErrorCode`.
- `artifact`: top-level `ParseArtifact` and `ParseStatus`.
- `schema`: JSON Schema generation helpers.

Use structs, enums, vectors, and `BTreeMap` for extension maps. Do not expose serialized `HashMap` fields because deterministic ordering is a project-level constraint.

## Validation Architecture

Phase 2 has real Rust code, so validation should be automated from the start.

Recommended commands:

- Quick command: `cargo test -p parser-contract`
- Full command: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- Schema/example command: `cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v1.schema.json && cargo test -p parser-contract schema_contract`

Unit tests should follow the RITE/AAA standard:

- Test public contract serialization and deserialization through crate exports, not private helpers.
- Use explicit fixture builders for success artifacts, missing SteamID, null killer, skipped artifact, and parse failure.
- Assert exact JSON field names and enum states such as `contract_version`, `parser_version`, `status`, `explicit_null`, `unknown`, `source_refs`, `rule_id`, `retryability`, and `error_code`.
- Include negative tests for invalid namespaced error code strings and schema/example mismatches.

## Plan Split Recommendation

Create five executable plans:

1. `02-00`: Rust workspace and contract crate foundation.
2. `02-01`: Artifact envelope, version/source/status metadata, diagnostics, and deterministic JSON example.
3. `02-02`: Replay metadata, observed identity, and explicit presence semantics.
4. `02-03`: Source references, normalized event skeleton, aggregate contribution references, and rule IDs.
5. `02-04`: Structured failures, schema generation, committed examples/schema, README handoff, and final validation.

This order keeps Wave 1 small, lets identity and envelope work run after the crate exists, and waits to generate the final schema until all public contract modules exist.

## Open Risks

- Exact dependency versions should be pinned by execution through `Cargo.lock`; this research intentionally does not claim current registry latest versions.
- `server-2` has only brief-level integration evidence in this repo. Phase 2 can define parser-owned artifact shape, but Phase 6 should inspect adjacent backend code or docs before queue/artifact transport details are frozen.
- JSON Schema validation details depend on the crate API available when execution pins dependencies. The plan should require generated schema and example validation, not a handwritten schema.

## Research Complete

Phase 2 can be planned now. The phase should create the first Rust workspace and contract crate, but should not parse OCAP files or implement aggregate formulas.
