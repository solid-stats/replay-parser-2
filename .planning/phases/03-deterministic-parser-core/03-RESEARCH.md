---
phase: 03-deterministic-parser-core
status: complete
researched: 2026-04-26
mode: inline
---

# Phase 3: Deterministic Parser Core - Research

## Research Summary

Phase 3 should implement the parser as a pure Rust crate that accepts replay bytes and explicit caller-provided source metadata, decodes OCAP JSON with a strict-root and tolerant-field policy, and returns Phase 2 `parser-contract` types. The work should stay transport-free: no CLI file handling, no RabbitMQ/S3, no database writes, no old-vs-new command harness, and no combat aggregate formulas.

The main implementation risk is not JSON decoding itself. It is preserving enough old-parser behavior for entity identity while keeping the new artifact auditable and deterministic. The legacy TypeScript parser treats unit/player entities, vehicle entities, connected events, and same-name duplicate slots as inputs to later aggregate behavior. Phase 3 needs to preserve those facts and compatibility hooks without collapsing observed identity into canonical identity.

## Key Findings

### Crate Boundary

Create a new workspace crate at `crates/parser-core` that depends on `parser-contract`. The public API should be small and adapter-friendly:

- `ParserInput<'a>`: replay bytes, `ReplaySource`, `ParserInfo`, and parser options.
- `ParserOptions`: deterministic options such as diagnostic limit, with a stable default.
- `parse_replay(input: ParserInput<'_>) -> ParseArtifact`: returns a success, partial, skipped, or failed artifact.

The parser core should not read files or compute deployment concerns. CLI and worker adapters later provide bytes, replay/job identifiers, checksum state, and output handling.

### Contract Extension Needed

Phase 2 `ObservedEntity` covers identity, source ID, broad kind, and source refs, but Phase 3 requirement PARS-05 needs vehicle/static weapon names and classes. The safest Phase 3 contract update is typed and explicit:

- Add `observed_name: FieldPresence<String>` to `ObservedEntity`.
- Add `observed_class: FieldPresence<String>` to `ObservedEntity`.
- Change `ObservedEntity.source_refs` from `Vec<SourceRef>` to `SourceRefs` so normalized entities cannot serialize as unauditable empty refs.
- Add a typed `EntityCompatibilityHint` surface for legacy same-name slot hints and compatibility metadata.

This is a parser artifact shape change, but it remains within this repository's parser-owned contract. `server-2` and `web` compatibility impact is limited because no adjacent implementation exists yet; the plan should still update schema examples and tests.

### OCAP Raw Decode

The historical corpus consistently contains the top-level OCAP fields `captureDelay`, `endFrame`, `entities`, `events`, `Markers`, `missionAuthor`, `missionName`, `playersCount`, and `worldName`; `EditorMarkers` is present in most but not all valid files. Phase 1 recorded 23,469 successfully parsed raw files and 4 malformed raw files.

Use `serde_json` as the correctness-first decoder. Invalid JSON, EOF-truncated JSON, and a root value that is not an OCAP object should produce a `ParseArtifact` with `status: failed`, `ParseStage::JsonDecode` or `ParseStage::Schema`, and a non-empty `ParseFailure.source_refs`.

After root decode succeeds, prefer tolerant extraction from `serde_json::Value` or small raw adapter structs. Local field drift should produce diagnostics and explicit unknowns instead of panics. The raw adapter should be the only place that knows OCAP tuple indexes and loose source shapes.

### Metadata Normalization

Phase 3 fills `ReplayMetadata` from observed top-level fields:

- `missionName` -> `mission_name`
- `worldName` -> `world_name`
- `missionAuthor` -> `mission_author`
- `playersCount` -> `players_count`
- `captureDelay` -> `capture_delay`
- `endFrame` -> `end_frame`
- `frame_bounds.start_frame` -> `0`
- `frame_bounds.end_frame` -> `endFrame`
- `time_bounds.start_seconds` -> `0.0`
- `time_bounds.end_seconds` -> `endFrame * captureDelay` only if both values are present and finite

Absent or malformed source fields should use `FieldPresence::Unknown` with `UnknownReason::SourceFieldAbsent` or `UnknownReason::SchemaDrift`, plus source refs where available.

### Entity Normalization

The old TypeScript `getEntities.ts` creates players from entities where:

- `entity.type === "unit"`
- `entity.isPlayer` is truthy
- `entity.description.length` is non-zero
- `entity.name` is present

It creates vehicles from entities where `entity.type === "vehicle"`, preserving `id`, `name`, `class`, and `positions`.

Phase 3 should normalize:

- unit/player entities as `EntityKind::Unit`
- vehicle entities as `EntityKind::Vehicle`
- class `"static-weapon"` as `EntityKind::StaticWeapon`
- other recognizable but unsupported shapes as `EntityKind::Unknown`

Vehicle score taxonomy such as car/truck/APC/tank/heli/plane remains Phase 4. Phase 3 should preserve the raw observed class and name only.

### Legacy Compatibility Hooks

The old parser backfills a player from a `connected` event when:

- event type is `"connected"`
- event tuple includes a non-empty name and entity ID
- an entity with that ID exists
- the entity is not a vehicle

The new parser should preserve that as inferred observed identity, not as canonical identity. Use inferred `FieldPresence` where applicable, a stable rule ID such as `entity.connected_player_backfill`, and source refs to both the connected event and entity evidence.

The old parser also later merges same-name player slots during aggregate projection. Phase 3 should not merge normalized entities. It should emit typed compatibility hints or diagnostics that cite the related source entity IDs, observed shared name, rule ID `entity.duplicate_slot_same_name`, and source refs. Phase 4/5 can decide how to project that into aggregate compatibility.

### Determinism

Phase 3 should guarantee stable artifact ordering for the pieces it produces:

- sort normalized entities by `source_entity_id` ascending
- tie-break by `kind`, `observed_name`, `observed_class`, and source JSON path if duplicate IDs appear
- keep diagnostics sorted by first source coordinate and code where possible
- use `BTreeMap` for any dynamic maps exposed in serialized output
- keep `produced_at: None` in parser-core output
- keep `events`, `aggregates.contributions`, and `aggregates.projections` empty unless a plan explicitly implements a Phase 3 compatibility hint surface outside those sections

### Status and Diagnostic Policy

Use the context decisions:

- `success`: no semantic data loss; info diagnostics and expected compatibility hints are allowed.
- `partial`: localized schema drift caused unknown values, dropped entity facts, missing source refs, or conflicting source evidence.
- `failed`: invalid JSON, truncated JSON, or root shape that cannot be treated as OCAP.

Diagnostics should be path-based and capped. Each diagnostic should include code, severity, message, `json_path`, expected shape, observed shape, parser action, and non-empty source refs. Repeated problems should collapse into a summary diagnostic after the configured limit.

## Validation Architecture

The validation strategy is Rust behavior tests backed by focused OCAP fixtures. Full corpus parity and benchmark sweeps belong to Phase 5, but Phase 3 needs strong oracles for the parser-core behaviors it owns.

Recommended automated commands:

- Quick: `cargo test -p parser-core`
- Contract after schema changes: `cargo test -p parser-contract metadata_identity_contract schema_contract`
- Full: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps`
- Schema refresh after contract changes: `cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v1.schema.json`

Focused fixtures should cover:

- valid top-level metadata and mixed unit/vehicle/static entities
- malformed JSON and non-object root
- localized metadata drift
- malformed entity entries with usable metadata
- entity ordering determinism from unsorted input
- connected-player backfill from raw events
- duplicate same-name player slot compatibility hints

Tests should follow the RITE/AAA standard: readable names, isolated focused fixtures, thorough success/error/boundary coverage, and explicit assertions on observable `ParseArtifact` output.

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Contract extension breaks schema examples | Extend `parser-contract` first, update examples, regenerate schema, and run schema tests before adding parser-core. |
| Raw OCAP quirks leak across the crate | Keep raw tuple/object handling in `raw.rs` and normalization modules consume typed observations. |
| Partial artifacts silently lose data | Add status policy tests proving drift that drops entity facts yields `ParseStatus::Partial`. |
| Entity source refs are empty or point to sorted indexes | Build source refs from original array indexes before sorting, then assert `$.entities[<source_index>]`. |
| Connected-player backfill becomes canonical matching | Use `FieldPresence::Inferred`, rule IDs, and source refs; do not create cross-replay identity fields. |
| Same-name compatibility collapses raw observations | Emit hints only; do not merge `ObservedEntity` records in Phase 3. |

## Planning Recommendation

Use six sequential plans:

1. Extend the contract for typed entity facts and compatibility hints.
2. Add parser-core crate foundation, API, and failure artifact shell.
3. Implement raw OCAP root decode and metadata normalization.
4. Implement observed entity normalization and deterministic entity ordering.
5. Implement schema-drift diagnostics, partial status, and deterministic artifact tests.
6. Implement connected-player backfill, duplicate-slot hints, README handoff, and final quality gates.

This ordering keeps contract changes first, then builds parser-core from an adapter-safe API, then layers behavior from metadata to entities to compatibility hooks.
