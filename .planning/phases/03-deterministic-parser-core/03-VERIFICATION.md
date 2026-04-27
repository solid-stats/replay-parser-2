---
phase: 03-deterministic-parser-core
verified: 2026-04-27T06:17:28Z
status: passed
score: 42/42 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 40/42
  gaps_closed:
    - "Developer can inspect normalized unit/player, vehicle, and static weapon entities with source IDs, observed names/classes, side/group/role fields, player flags, and available identity fields."
    - "D-14 / PARS-04: unit/player entities preserve source ID, observed name, side, group, role/description, player flag evidence, and source refs."
  gaps_remaining: []
  regressions: []
---

# Phase 3: Deterministic Parser Core Verification Report

**Phase Goal:** The Rust parser core can read historical OCAP JSON and return deterministic normalized metadata and observed entity facts without transport concerns.
**Verified:** 2026-04-27T06:17:28Z
**Status:** passed
**Re-verification:** Yes - after fix commit `742580c`

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | Developer can parse representative historical OCAP JSON files and receive normalized replay metadata from observed top-level fields. | VERIFIED | `parse_replay` decodes caller bytes with `serde_json::from_slice`, valid object roots build `RawReplay`, and `normalize_metadata` maps `missionName`, `worldName`, `missionAuthor`, `playersCount`, `captureDelay`, and `endFrame` into `ParseArtifact.replay`. Metadata tests pass. |
| 2 | Developer can inspect normalized unit/player, vehicle, and static weapon entities with source IDs, observed names/classes, side/group/role fields, player flags, and available identity fields. | VERIFIED | Previous gap is closed. `ObservedEntity` now has `is_player: FieldPresence<bool>`; `raw.rs` reads source `isPlayer`; `entities.rs` populates present/unknown/not-applicable values with source refs; tests cover true, false, schema drift, and vehicle not-applicable. |
| 3 | Known OCAP schema drift results in structured warnings, explicit unknown states, or structured failures instead of parser panics. | VERIFIED | Invalid JSON/root failures produce `ParseFailure`; metadata and entity drift emit path-based diagnostics and explicit unknown states; diagnostic cap and partial-status tests pass. |
| 4 | Repeated parser-core runs on the same input and contract version produce stable JSON ordering. | VERIFIED | Deterministic output tests assert byte-identical `serde_json::to_string` for repeated parses, ordered entity IDs `[10, 20, 30]`, and `produced_at: None`. |
| 5 | Connected-player backfill and duplicate-slot same-name compatibility behavior are preserved for later aggregate projection while raw observed identifiers remain available. | VERIFIED | Connected-player backfill uses inferred names and rule/source provenance; duplicate-slot same-name hints preserve all entities and sorted related IDs; legacy compatibility tests pass, including last connected nickname behavior from fix `742580c`. |

**Score:** 42/42 detailed must-haves verified. Roadmap success criteria are 5/5 verified.

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `crates/parser-contract/src/identity.rs` | Typed observed entity fields, source refs, player flag, compatibility hints | VERIFIED | `ObservedEntity` contains `observed_name`, `observed_class`, `is_player`, `identity`, `compatibility_hints`, and non-empty `SourceRefs`; no canonical identity fields are present. |
| `schemas/parse-artifact-v1.schema.json` | Generated schema with entity source refs, compatibility hint shape, and `is_player` | VERIFIED | Fresh schema export is byte-identical to the committed schema; schema tests assert `is_player` and compatibility hint shape. |
| `crates/parser-core/src/lib.rs` | Pure public `parse_replay` API | VERIFIED | Public API delegates to artifact assembly without transport adapters. |
| `crates/parser-core/src/input.rs` | Caller-provided bytes/source/parser/options | VERIFIED | Parser input owns no filesystem, queue, S3, or database concerns; default diagnostic limit is 100. |
| `crates/parser-core/src/artifact.rs` | Artifact assembly and structured failure path | VERIFIED | Valid roots normalize metadata and entities; invalid JSON/non-object roots return failed artifacts; `produced_at` remains `None`. |
| `crates/parser-core/src/raw.rs` | Tolerant OCAP field helpers | VERIFIED | Raw helpers isolate top-level, entity, player flag, positions, and connected-event shape handling. |
| `crates/parser-core/src/metadata.rs` | Replay metadata normalization | VERIFIED | Metadata fields and derived frame/time bounds are populated with source refs or explicit unknowns. |
| `crates/parser-core/src/entities.rs` | Observed entity normalization and compatibility hooks | VERIFIED | Normalizes unit/player, vehicle, static weapon, player flags, source refs, connected backfill, duplicate-slot hints, and drift diagnostics. |
| `crates/parser-core/src/diagnostics.rs` | Diagnostic cap and status policy | VERIFIED | Capped accumulator, omitted summary diagnostic, and data-loss partial status are implemented and tested. |
| `crates/parser-core/tests/*` | Behavior tests | VERIFIED | Parser-core now has 30 passing integration tests covering metadata, entities, player flags, drift/status, determinism, failures, and legacy compatibility. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `parser_core::parse_replay` | `artifact::parse_replay` | public delegation | VERIFIED | Pure API returns a `ParseArtifact` and does not access files, S3, RabbitMQ, or databases. |
| `artifact.rs` | `serde_json` root decode | `serde_json::from_slice::<Value>` | VERIFIED | Invalid JSON and non-object roots become structured failed artifacts. |
| `artifact.rs` | `metadata.rs` | `normalize_metadata(&raw, ...)` | VERIFIED | Valid root artifacts set `replay: Some(replay)`. |
| `artifact.rs` | `entities.rs` | `normalize_entities(&raw, ...)` | VERIFIED | Entity vector is populated from `$.entities` after metadata normalization. |
| `raw.rs` | `entities.rs` | `entity_is_player(entity, index)` | VERIFIED | Source `isPlayer` accepts booleans and numeric `0`/`1` and feeds `player_flag_presence`. |
| `entities.rs` | `parser-contract identity` | `ObservedEntity { is_player, ... }` | VERIFIED | Unit flags become `FieldPresence::Present` or `Unknown(SchemaDrift)` with `entity.is_player.observed` source refs; non-unit entities are `NotApplicable`. |
| `entities.rs` | connected-player compatibility | `connected_events` and inferred `FieldPresence` | VERIFIED | Valid connected events update missing observed names and player nicknames with rule `entity.connected_player_backfill`. |
| `entities.rs` | duplicate-slot compatibility | `EntityCompatibilityHintKind::DuplicateSlotSameName` | VERIFIED | Hints are attached without merging entities and retain sorted related IDs/source refs. |
| `parser-contract schema` | committed schema | schema export and `cmp` | VERIFIED | `cargo run -p parser-contract --example export_schema` output matches `schemas/parse-artifact-v1.schema.json`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `artifact.rs` | `ParseArtifact.replay` | `ParserInput.bytes` -> `serde_json::Value` -> `RawReplay` -> `normalize_metadata` | Yes | VERIFIED |
| `artifact.rs` | `ParseArtifact.entities` | `ParserInput.bytes` -> `$.entities` -> raw entity helpers -> `normalize_entities` | Yes | VERIFIED |
| `identity.rs` / `entities.rs` | `ObservedEntity.is_player` | `$.entities[N].isPlayer` -> `entity_is_player` -> `player_flag_presence` -> serialized artifact/schema | Yes | VERIFIED |
| `entities.rs` | `observed_name` / `identity.nickname` backfill | `$.events[N]` connected tuples -> `connected_events` -> `FieldPresence::Inferred` | Yes, for valid connected tuples | VERIFIED |
| `entities.rs` | `compatibility_hints` | normalized entity groups by same present/inferred name -> typed hints | Yes | VERIFIED |
| `artifact.rs` | `ParseArtifact.diagnostics/status` | metadata/entity drift helpers -> `DiagnosticAccumulator::finish` | Yes | VERIFIED |
| `artifact.rs` | `ParseArtifact.events` / `aggregates` | Phase 4-owned event semantics and aggregate projection | N/A | NOT A PHASE 3 GAP |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Contract entity/player-flag serialization | `cargo test -p parser-contract --test metadata_identity_contract` | 11 tests passed | PASS |
| Contract schema/example freshness | `cargo test -p parser-contract --test schema_contract` | 15 tests passed | PASS |
| Player flag output and entity normalization | `cargo test -p parser-core --test entity_normalization` | 9 tests passed, including true/false/drift/not-applicable player flag cases | PASS |
| Connected backfill and duplicate-slot behavior | `cargo test -p parser-core --test legacy_entity_compatibility` | 6 tests passed, including last connected nickname behavior | PASS |
| Full parser-core suite | `cargo test -p parser-core` | 30 parser-core integration tests passed | PASS |
| Full workspace suite | `cargo test --workspace` | 81 workspace tests passed across parser-contract and parser-core | PASS |
| Formatting gate | `cargo fmt --all -- --check` | exit 0 | PASS |
| Strict lint gate | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 | PASS |
| Documentation gate | `cargo doc --workspace --no-deps` | exit 0 | PASS |
| Schema freshness | `cargo run -p parser-contract --example export_schema > /tmp/parse-artifact-v1.schema.json` then `cmp /tmp/parse-artifact-v1.schema.json schemas/parse-artifact-v1.schema.json` | both exit 0 | PASS |
| Whitespace diff gate | `git diff --check` | exit 0 | PASS |
| GSD SDK query helper | `gsd-sdk query roadmap.get-phase 3 --raw` | exit 1, installed CLI has no `query` subcommand | SKIP - roadmap parsed directly from files |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| OUT-08 | Roadmap; 03-00..03-05 | Deterministic output ordering across repeated runs | SATISFIED | Deterministic serialization tests pass; entities sort by stable keys; `produced_at` remains `None`; dynamic maps use ordered structures. |
| PARS-01 | Roadmap; 03-01..03-05 | Parser reads OCAP JSON matching historical raw replay files | SATISFIED | Parser accepts caller bytes, decodes with `serde_json`, requires object root, and uses tolerant raw helpers for observed top-level/entity shapes. Full corpus parity remains Phase 5 by roadmap. |
| PARS-02 | Roadmap; 03-01..03-05 | Schema drift produces warnings, unknowns, or failures without panics | SATISFIED | JSON/root failures are structured; metadata/entity drift emits diagnostics and unknowns; diagnostic cap and partial-status tests pass. |
| PARS-03 | Roadmap; 03-02, 03-04, 03-05 | Extract replay metadata from observed top-level fields | SATISFIED | `normalize_metadata` maps mission/world/author/player count/capture delay/end frame and derived bounds with source refs. |
| PARS-04 | Roadmap; 03-00, 03-03, 03-05 | Normalize unit/player entities with source IDs, names, side, group, role/description, player flags, identity fields | SATISFIED | Previous blocker closed. Unit/player output includes source IDs, name, side/group/role/description, unknown SteamID, `is_player` field, and source refs; tests cover present true/false and drift. |
| PARS-05 | Roadmap; 03-00, 03-03, 03-05 | Normalize vehicle/static weapon entities with source IDs, names, classes, side/context, positions evidence | SATISFIED | Vehicle/static kind, observed name/class, side, not-applicable player fields, entity/positions source refs, and stable ordering tests pass. |
| PARS-06 | Roadmap; 03-00, 03-05 | Preserve connected-player backfill behavior | SATISFIED | Backfilled names use `FieldPresence::Inferred`, rule `entity.connected_player_backfill`, event/entity source refs, and last connected nickname behavior; tests pass. |
| PARS-07 | Roadmap; 03-00, 03-05 | Preserve duplicate-slot same-name compatibility while retaining raw identifiers | SATISFIED | Hints use `EntityCompatibilityHintKind::DuplicateSlotSameName`, sorted related IDs, no entity merge, and success status without side conflict; tests pass. |

No orphaned Phase 3 requirements were found in `REQUIREMENTS.md`: Phase 3 maps to OUT-08 and PARS-01 through PARS-07.

### Plan Must-Haves Coverage

| Plan | Must-have | Status | Evidence |
|---|---|---|---|
| 03-00 | D-13/PARS-05 typed observed name/class fields | VERIFIED | `ObservedEntity` has `observed_name` and `observed_class`; schema/tests include them. |
| 03-00 | D-16/OUT-08 entity source refs use `SourceRefs` | VERIFIED | Contract uses `SourceRefs`; schema test rejects empty entity source refs. |
| 03-00 | D-20/PARS-07 duplicate-slot compatibility typed hints | VERIFIED | `EntityCompatibilityHintKind::DuplicateSlotSameName` and runtime hint population exist. |
| 03-00 | D-17 no canonical identity fields | VERIFIED | `rg` finds no forbidden canonical/account/user fields in parser-contract or parser-core source. |
| 03-00 | D-18 connected-player typed hint surface | VERIFIED | `ConnectedPlayerBackfill` hint kind exists and is populated by parser-core. |
| 03-01 | D-01 pure `parse_replay(ParserInput)` API | VERIFIED | Public API returns `ParseArtifact`. |
| 03-01 | D-02 no transport/storage/DB side effects | VERIFIED | parser-core source/deps contain no file, S3, RabbitMQ, network, or DB access; API takes bytes. |
| 03-01 | D-05/PARS-02 invalid JSON/root failure artifacts | VERIFIED | Failure tests pass for JSON decode and root-object schema failures. |
| 03-01 | D-10/OUT-08 deterministic shell and `produced_at: None` | VERIFIED | Code sets `produced_at: None`; tests assert it. |
| 03-01 | D-03 failures use contract types | VERIFIED | `ParseFailure`, `ParseStatus::Failed`, `ParseStage`, `ErrorCode`, and `SourceRefs` are used. |
| 03-02 | D-04/PARS-01 uses `serde_json` decoder | VERIFIED | `artifact.rs` calls `serde_json::from_slice`. |
| 03-02 | D-05/PARS-02 metadata drift diagnostic/unknown | VERIFIED | `metadata.rs` emits `schema.metadata_field` and `UnknownReason::SchemaDrift`. |
| 03-02 | D-06/PARS-03 valid root produces metadata | VERIFIED | `normalize_metadata` is wired into successful artifacts; tests pass. |
| 03-02 | D-07 diagnostics include path/shape/action/source refs | VERIFIED | Diagnostic structs populate `json_path`, expected/observed shape, parser action, and `SourceRefs`. |
| 03-02 | D-09 raw quirks stay in `raw.rs` | VERIFIED | OCAP field shape helpers are isolated in `raw.rs`; normalizers consume `RawField`. |
| 03-02 | D-10/OUT-08 metadata deterministic | VERIFIED | Deterministic output and metadata tests pass. |
| 03-03 | D-11/OUT-08 entity order stable | VERIFIED | `compare_entities` sorts by ID/kind/name/class/source path; tests assert `[10, 20, 30]`. |
| 03-03 | D-14/PARS-04 unit/player facts include player flag evidence | VERIFIED | `ObservedEntity.is_player` is populated from `isPlayer`; tests assert true, false, drift unknown, and non-unit not-applicable states. |
| 03-03 | D-14/PARS-05 vehicle/static facts | VERIFIED | Vehicle/static tests pass for name, class, kind, source refs, and not-applicable `is_player`. |
| 03-03 | D-15 broad kind only; raw class/name preserved | VERIFIED | Vehicle score taxonomy absent; raw class/name preserved. |
| 03-03 | D-16 original source JSON paths after sorting | VERIFIED | Tests assert original `$.entities[0]` and positions path remain after sorting. |
| 03-03 | D-17 no canonical/cross-replay identity | VERIFIED | Serialization test and `rg` check show no forbidden fields. |
| 03-04 | D-07/PARS-02 diagnostics capped | VERIFIED | Diagnostic cap summary test passes. |
| 03-04 | D-08 data-loss diagnostics set partial | VERIFIED | Metadata and entity drift status tests pass; data-loss impacts map to `ParseStatus::Partial`. |
| 03-04 | D-08 non-loss diagnostics can remain success | VERIFIED | No-conflict duplicate hint test keeps success status. |
| 03-04 | D-10/OUT-08 repeated serialization byte-identical | VERIFIED | Deterministic serialization test passes. |
| 03-04 | D-12 produced_at remains `None` | VERIFIED | Code and tests verify no parser-core timestamp. |
| 03-05 | D-18/PARS-06 connected-player backfill inferred facts | VERIFIED | Backfill fixture test verifies inferred `BackfilledName`. |
| 03-05 | D-19 backfill rule/source provenance | VERIFIED | Tests assert `entity.connected_player_backfill` and event/entity source refs. |
| 03-05 | D-20/PARS-07 duplicate-slot hints without merge | VERIFIED | Tests assert 3 entities remain and hints include `[21, 22]`. |
| 03-05 | D-21 compatibility hooks alone do not force partial | VERIFIED | Duplicate-slot no-conflict fixture status remains success; conflict code emits data-loss diagnostic. |
| 03-05 | D-22 game-type/yearly behavior outside parser-core | VERIFIED | No parser-core implementation for game-type filters or yearly nomination behavior; roadmap defers those surfaces. |
| 03-05 | D-23 focused fixtures; corpus/golden parity Phase 5 | VERIFIED | Tests use focused fixtures; ROADMAP Phase 5 owns full golden parity. |
| 03-05 | D-24 behavior tests cover Phase 3 surfaces | VERIFIED | Tests cover metadata/entity extraction, player flags, schema drift, malformed input, explicit unknowns, deterministic ordering, source refs, connected-player backfill, and duplicate-slot hints. |
| 03-05 | D-25 legacy hook tests reference legacy behavior without full corpus parity | VERIFIED | Compatibility tests cover connected and same-name behavior through public artifacts. |
| 03-05 | D-26 tests assert public `ParseArtifact` behavior | VERIFIED | Integration tests call `parse_replay` and inspect returned artifacts. |
| 03-05 | D-27 strict Rust gates pass | VERIFIED | fmt, clippy, test, doc, schema freshness, and diff checks passed locally. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---:|---|---|---|
| `crates/parser-core/src/artifact.rs` | 71-72 | Empty events/default aggregates | INFO | Explicitly Phase 4-owned by ROADMAP; not a Phase 3 gap. |
| `crates/parser-core/src/entities.rs` | 451 | Initial `compatibility_hints: Vec::new()` | INFO | Hints are populated after entity creation by connected-player and duplicate-slot passes; not a stub. |
| `crates/parser-core/src/entities.rs` | 45 | Return empty entities when `entities` is absent/non-array | INFO | This path emits diagnostics before returning; not a silent stub. |
| `crates/parser-core/src/raw.rs` | 180-189 | Connected-event helper ignores absent/malformed event tuples | INFO | Phase 3 only uses valid connected tuples for compatibility backfill; full event semantics and event drift policy are Phase 4-owned. |

No TODO/FIXME/placeholder/unimplemented markers were found in the scanned Phase 3 code and tests. No blocker anti-patterns were found.

### Human Verification Required

None. Phase 3 is deterministic parser-core Rust behavior with no visual, realtime, external-service, or manual workflow surface.

### Gaps Summary

No blocking gaps remain. The previous player-flag blocker is closed by the fix commit: player flags are now part of the contract, parser output, schema/example surface, and parser-core behavior tests.

Existing empty `events` and aggregate sections are not Phase 3 gaps because Phase 4 owns combat/outcome semantics and aggregate projection.

---

_Verified: 2026-04-27T06:17:28Z_
_Verifier: the agent (gsd-verifier)_
