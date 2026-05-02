---
status: planned
mode: quick-full
phase: quick-260502-ecp
plan: 01
type: execute
wave: 1
depends_on: []
autonomous: true
requirements:
  - OUT-09
  - OUT-10
  - OUT-11
  - OUT-12
  - PARS-12
  - AGG-12
  - TEST-06
  - TEST-13
  - TEST-14
  - TEST-15
files_modified:
  - crates/parser-contract/src/minimal.rs
  - crates/parser-contract/src/artifact.rs
  - crates/parser-contract/src/schema.rs
  - crates/parser-contract/tests/schema_contract.rs
  - crates/parser-contract/tests/artifact_envelope.rs
  - crates/parser-contract/tests/failure_contract.rs
  - crates/parser-contract/examples/parse_artifact_success.v3.json
  - crates/parser-contract/examples/parse_failure.v3.json
  - schemas/parse-artifact-v3.schema.json
  - crates/parser-core/src/aggregates.rs
  - crates/parser-core/src/artifact.rs
  - crates/parser-core/tests/aggregate_projection.rs
  - crates/parser-core/tests/deterministic_output.rs
  - crates/parser-core/tests/debug_artifact.rs
  - crates/parser-core/tests/combat_event_semantics.rs
  - crates/parser-core/tests/fault_injection_regressions.rs
  - crates/parser-core/tests/golden_fixture_behavior.rs
  - crates/parser-core/tests/parser_core_api.rs
  - crates/parser-core/tests/fixtures/golden/manifest.json
  - crates/parser-harness/src/comparison.rs
  - crates/parser-harness/tests/comparison_report.rs
  - crates/parser-harness/benches/parser_pipeline.rs
  - crates/parser-cli/tests/parse_command.rs
  - crates/parser-cli/tests/schema_command.rs
  - scripts/benchmark-phase5.sh
  - .planning/generated/phase-05/benchmarks/selected-large-artifact.json
  - .planning/generated/phase-05/benchmarks/benchmark-report.json
  - README.md
  - .planning/PROJECT.md
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/STATE.md
  - .planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-SUMMARY.md
must_haves:
  truths:
    - "Default parser output for the selected large replay serializes to no more than 100000 bytes."
    - "Default success artifacts merge player counters into players rows and do not emit player_stats."
    - "Default kill and destroyed-vehicle rows use numeric replay-local references, weapon IDs, and compact schema-backed keys."
    - "Verbose identity, source refs, event indexes, rule IDs, vehicle names, and normalized evidence remain debug-sidecar-only."
    - "Old-vs-new comparison still derives legacy player rows, relationships, and bounty inputs from the compact default artifact."
    - "No canonical identity, PostgreSQL persistence, public API, UI, replay discovery, or adjacent-app behavior is added."
  artifacts:
    - path: "crates/parser-contract/src/minimal.rs"
      provides: "Compact row structs, merged player counters, and weapon dictionary row"
      contains: "serde rename and skip_serializing_if rules"
    - path: "crates/parser-contract/src/artifact.rs"
      provides: "Default ParseArtifact without player_stats and with optional weapons"
      contains: "skip_serializing_if for null, empty, and default fields"
    - path: "crates/parser-core/src/aggregates.rs"
      provides: "Compact table derivation from normalized combat events"
      contains: "BTreeMap-backed deterministic weapon dictionary"
    - path: "crates/parser-harness/src/comparison.rs"
      provides: "Legacy comparison view derived from compact merged player rows"
      contains: "bounty derivation from kill classification and refs"
    - path: ".planning/generated/phase-05/benchmarks/selected-large-artifact.json"
      provides: "Regenerated selected-large default artifact evidence"
      max_bytes: 100000
  key_links:
    - from: "crates/parser-core/src/artifact.rs"
      to: "crates/parser-core/src/aggregates.rs"
      via: "derive_minimal_tables result assignment"
      pattern: "derive_minimal_tables"
    - from: "crates/parser-core/src/aggregates.rs"
      to: "crates/parser-contract/src/minimal.rs"
      via: "MinimalPlayerRow, MinimalKillRow, MinimalDestroyedVehicleRow, MinimalWeaponRow"
      pattern: "MinimalWeaponRow"
    - from: "crates/parser-cli/src/main.rs"
      to: "crates/parser-contract/src/artifact.rs"
      via: "serde_json::to_vec default minified artifact serialization"
      pattern: "serde_json::to_vec"
    - from: "scripts/benchmark-phase5.sh"
      to: ".planning/generated/phase-05/benchmarks/selected-large-artifact.json"
      via: "release CLI parse and wc -c artifact byte measurement"
      pattern: "selected_artifact_bytes"
---

# Quick Task 260502-ecp: Compact Default Parser Artifact Below 100 KB - Plan

<objective>
Reduce the default server-facing parser artifact below the hard 100000-byte selected-large limit by implementing the locked compact object-row shape: merged player counters, numeric entity refs, weapon dictionary IDs, omitted null/empty/zero fields, and debug-only verbose evidence.

Purpose: Unblock Phase 6 readiness by fixing the Phase 5.2 artifact-size blocker without moving canonical identity, persistence, API, UI, replay discovery, or worker behavior into this parser quick task.

Output: A typed compact default artifact contract, parser-core builder, comparison/CLI/benchmark updates, regenerated selected-large size evidence, and a quick-task summary.
</objective>

<execution_context>
@/home/afgan0r/.codex/get-shit-done/workflows/execute-plan.md
@/home/afgan0r/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/research/SUMMARY.md
@.planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-CONTEXT.md
@.planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-RESEARCH.md
@AGENTS.md
@crates/parser-contract/src/minimal.rs
@crates/parser-contract/src/artifact.rs
@crates/parser-core/src/aggregates.rs
@crates/parser-core/src/artifact.rs
@crates/parser-harness/src/comparison.rs
@scripts/benchmark-phase5.sh

<decision_trace>
D-01: Merge player_stats counters into players rows; use numeric source_entity_id/eid as the replay-local player reference; omit zero counters.
D-02: In kills and destroyed_vehicles, replace repeated names and side values with player/entity ID refs; verbose identity/event evidence remains debug-sidecar-only.
D-03: Omit bounty_eligible and bounty_exclusion_reasons from default output; downstream computes bounty eligibility from kill classification.
D-04: Keep vehicle class plus relevant source entity ID in default event rows; keep vehicle names debug-only; preserve ordinary vehicleKills and killsFromVehicle counters.
D-05: Add a deterministic weapon dictionary and reference weapons by compact IDs in event rows.
D-06: Omit nulls, empty arrays, and zero counters; use short schema-backed object keys; only move to tuple arrays if object rows still fail the selected hard byte gate.
</decision_trace>

<interfaces>
Target default success artifact shape:
```json
{
  "players": [{"eid": 1, "n": "Player", "s": "west", "k": 2, "kfv": 1, "vk": 1}],
  "weapons": [{"id": 1, "n": "arifle_MX_F"}],
  "kills": [{"k": 1, "v": 2, "c": "enemy_kill", "w": 1, "av": 10, "avc": "B_MRAP_01_hmg_F"}],
  "destroyed_vehicles": [{"a": 1, "c": "enemy", "w": 1, "av": 10, "avc": "B_MRAP_01_hmg_F", "de": 20, "dt": "vehicle", "dc": "O_MRAP_02_F"}],
  "diagnostics": []
}
```

Counters on `players[]`: `k` kills, `d` deaths, `tk` teamkills, `su` suicides, `nkd` null-killer deaths, `ud` unknown deaths, `vk` vehicleKills, `kfv` killsFromVehicle. Fields with zero values are absent from serialized default JSON and default to zero on deserialize.

Debug sidecar remains the home for verbose evidence from `parse_replay_debug`; default artifacts must not recursively contain source refs, rule IDs, frame/time, event indexes, entity snapshots, attacker vehicle names, destroyed vehicle names, killer/victim names in event rows, or bounty eligibility/exclusion fields.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Compact The Contract, Schema, And Examples</name>
  <files>crates/parser-contract/src/minimal.rs, crates/parser-contract/src/artifact.rs, crates/parser-contract/src/schema.rs, crates/parser-contract/tests/schema_contract.rs, crates/parser-contract/tests/artifact_envelope.rs, crates/parser-contract/tests/failure_contract.rs, crates/parser-contract/examples/parse_artifact_success.v3.json, crates/parser-contract/examples/parse_failure.v3.json, schemas/parse-artifact-v3.schema.json</files>
  <behavior>
    - Contract success examples deserialize when `player_stats`, `failure`, empty arrays, nulls, and zero counters are omitted per D-01 and D-06.
    - Fresh schema contains `players`, `weapons`, `kills`, and `destroyed_vehicles`, and does not contain `player_stats`, `bounty_eligible`, `bounty_exclusion_reasons`, verbose event identity names, source refs, rule IDs, frame, or event index properties per D-02, D-03, and D-05.
    - Failed artifacts still require `failure`; non-failed artifacts reject `failure` when present.
  </behavior>
  <action>Update the typed contract rather than adding a JSON post-processor. In `minimal.rs`, fold non-zero counter fields into `MinimalPlayerRow` using short serde keys (`eid`, `n`, `s`, `g`, `r`, `sid`, `ck`, `k`, `d`, `tk`, `su`, `nkd`, `ud`, `vk`, `kfv`) with `#[serde(default)]` and `skip_serializing_if` predicates for zero/none values per D-01 and D-06. Replace verbose kill/destruction row fields with compact refs (`k`, `v`, `a`, `c`, `w`, `av`, `avc`, `de`, `dt`, `dc`) per D-02, D-03, and D-04. Add `MinimalWeaponRow { id, n }` per D-05. In `artifact.rs`, remove default `player_stats`, add `weapons`, and omit null/empty/default success fields while preserving status/failure invariants. Regenerate `schemas/parse-artifact-v3.schema.json` from `parse_artifact_schema()` and update examples/tests to lock the short-key object schema. Do not add canonical player fields, database IDs, API DTOs, UI labels, or adjacent-app contracts.</action>
  <verify>
    <automated>cargo test -p parser-contract --tests</automated>
  </verify>
  <done>Parser-contract tests pass; committed schema matches fresh generation; examples validate; schema and examples prove merged players, weapon dictionary, omitted defaults, and absence of verbose/default-removed fields.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Build Compact Rows From Parser Core</name>
  <files>crates/parser-core/src/aggregates.rs, crates/parser-core/src/artifact.rs, crates/parser-core/tests/aggregate_projection.rs, crates/parser-core/tests/deterministic_output.rs, crates/parser-core/tests/debug_artifact.rs, crates/parser-core/tests/combat_event_semantics.rs, crates/parser-core/tests/fault_injection_regressions.rs, crates/parser-core/tests/golden_fixture_behavior.rs, crates/parser-core/tests/parser_core_api.rs, crates/parser-core/tests/fixtures/golden/manifest.json</files>
  <behavior>
    - `parse_replay` emits merged player rows with non-zero counters and no `player_stats` table per D-01.
    - `kills[]` and `destroyed_vehicles[]` contain numeric entity refs, classifications, weapon IDs, attacker vehicle entity/class, and destroyed entity type/class without event-row names or side repetition per D-02, D-03, and D-04.
    - `weapons[]` is deterministic across repeated parses and is omitted when no compact row references a weapon per D-05 and D-06.
    - `parse_replay_debug` still exposes verbose evidence for debugging, including source refs and normalized detail, while default artifacts do not.
  </behavior>
  <action>Refactor `MinimalTables` and `derive_minimal_tables` so the accumulator mutates per-player counters directly on compact player rows and returns `players`, `weapons`, `kills`, and `destroyed_vehicles`. Use `BTreeMap` for entity indexes, player rows, and weapon-name-to-ID assignment; assign stable weapon IDs from sorted non-empty weapon names before building event rows. Preserve ordinary `vehicleKills` and `killsFromVehicle` counters on the player row per D-04. Map bounty-compatible rows from `KillClassification::EnemyKill` with known killer and victim refs instead of serializing bounty fields per D-03. Keep all verbose names, sides, source refs, rule IDs, frame/time, event indexes, and entity snapshots on debug-only paths per D-02. Avoid redundant cloning in loops; borrow from normalized combat/entity structs until owned compact strings are required by the contract.</action>
  <verify>
    <automated>cargo test -p parser-core --tests</automated>
  </verify>
  <done>Parser-core tests prove compact default rows, deterministic weapon dictionary ordering, merged counters, debug-only verbose evidence, retained ordinary vehicle counters, and no default artifact contamination by detailed evidence.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Update Consumers And Prove Selected Size</name>
  <files>crates/parser-harness/src/comparison.rs, crates/parser-harness/tests/comparison_report.rs, crates/parser-harness/benches/parser_pipeline.rs, crates/parser-cli/tests/parse_command.rs, crates/parser-cli/tests/schema_command.rs, scripts/benchmark-phase5.sh, .planning/generated/phase-05/benchmarks/selected-large-artifact.json, .planning/generated/phase-05/benchmarks/benchmark-report.json, README.md, .planning/PROJECT.md, .planning/REQUIREMENTS.md, .planning/ROADMAP.md, .planning/STATE.md, .planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-SUMMARY.md</files>
  <behavior>
    - CLI default minified output omits `null`, empty arrays, zero counters, `player_stats`, bounty fields, source refs, event indexes, rule IDs, and verbose event identity/vehicle names per D-01 through D-06.
    - CLI `--debug-artifact` remains the explicit verbose evidence path and is not needed for ordinary default output per D-02.
    - Comparison reports derive legacy player game results, relationships, and bounty inputs from compact players/kills/destroyed_vehicles without `MinimalPlayerStatsRow` or `bounty_eligible` per D-01 and D-03.
    - The selected large replay artifact regenerated by the release CLI is `<= 100000` bytes.
  </behavior>
  <action>Update CLI/schema tests for the compact root and recursive absence checks. Update `comparison.rs` to deserialize compact rows, derive legacy player stats from merged `players[]` counters, derive bounty inputs from `KillClassification::EnemyKill` plus known killer/victim refs, and count vehicle kills from the retained player counter or destroyed-vehicle rows. Update benchmark generation only as needed so selected-large evidence is regenerated from the current release CLI output and reports the hard size gate accurately. Update README and local planning docs so they no longer describe the default artifact as containing `player_stats` or default bounty/verbose event fields. If the schema-backed object row form still produces a selected artifact above 100000 bytes, convert only the high-cardinality default rows to schema-backed tuple arrays in the same contract/core/harness surfaces, using the tuple order from the target compact keys, then rerun the same proof before declaring done per D-06.</action>
  <verify>
    <automated>cargo test -p parser-cli --tests</automated>
    <automated>cargo test -p parser-harness --tests</automated>
    <automated>cargo run -q -p parser-cli --bin replay-parser-2 --release -- parse /home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json --output /tmp/replay-parser-2-selected-compact.json</automated>
    <automated>python3 -c 'from pathlib import Path; p=Path("/tmp/replay-parser-2-selected-compact.json"); n=p.stat().st_size; print(f"selected_large_artifact_bytes={n}"); assert n <= 100000'</automated>
    <automated>scripts/benchmark-phase5.sh --ci</automated>
    <automated>cargo run -q -p parser-harness --bin benchmark-report-check -- --report .planning/generated/phase-05/benchmarks/benchmark-report.json --mode structural</automated>
  </verify>
  <done>CLI and harness tests pass, benchmark report remains structurally valid, regenerated selected-large artifact is no more than 100000 bytes, and docs/planning state describe the compact merged default artifact without adjacent-app scope drift.</done>
</task>

</tasks>

<source_coverage_audit>
GOAL: Compact default parser artifact below 100000 bytes for the selected large replay. Covered by Tasks 1-3, with Task 3 carrying the blocking byte proof.
REQ: OUT-09, OUT-10, OUT-11, OUT-12, PARS-12, AGG-12, TEST-06, TEST-13, TEST-14, TEST-15 are covered through contract/core/default-output/comparison/benchmark changes. Worker requirements remain out of scope.
RESEARCH: Contract envelope, player/stat duplication, verbose kill/destruction rows, parser-core builder, debug sidecar, CLI serialization, schema closure, comparison harness, and benchmark gate findings are each mapped to at least one task.
CONTEXT: D-01 through D-06 are referenced in task actions and verification. Deferred/adjacent app behavior is not planned.
</source_coverage_audit>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| raw OCAP JSON -> parser-core | Untrusted replay bytes are decoded and normalized into typed compact rows. |
| parser artifact -> server-2 ingestion | Compact JSON contract is consumed by downstream services later, so omitted/default fields must remain schema-backed and deterministic. |
| CLI path -> local filesystem | Local parse command reads raw replay files and writes artifacts for validation evidence. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-260502-ecp-01 | Tampering | `crates/parser-contract/src/artifact.rs` | mitigate | Preserve schema validation and status/failure invariants while omitting success defaults; contract tests reject verbose or removed default fields. |
| T-260502-ecp-02 | Information Disclosure | `crates/parser-core/src/aggregates.rs` | mitigate | Keep verbose source refs, names in event rows, vehicle names, frame/time, rule IDs, and normalized evidence out of default serialization; verify recursive absence in CLI/core tests. |
| T-260502-ecp-03 | Repudiation | `crates/parser-harness/src/comparison.rs` | mitigate | Derive comparison and bounty evidence deterministically from compact refs/classification so parity review remains reproducible after removing serialized bounty fields. |
| T-260502-ecp-04 | Denial of Service | `scripts/benchmark-phase5.sh` | mitigate | Gate selected-large output at exactly 100000 bytes and regenerate evidence from release CLI output; do not accept stale generated artifacts as proof. |
| T-260502-ecp-05 | Elevation of Privilege | canonical identity boundary | accept | This quick task emits observed replay-local refs only and does not introduce auth, persistence, canonical IDs, or adjacent app mutation paths. |
</threat_model>

<verification>
Overall verification:
1. `cargo test -p parser-contract --tests`
2. `cargo test -p parser-core --tests`
3. `cargo test -p parser-cli --tests`
4. `cargo test -p parser-harness --tests`
5. `cargo run -q -p parser-cli --bin replay-parser-2 --release -- parse /home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json --output /tmp/replay-parser-2-selected-compact.json`
6. `python3 -c 'from pathlib import Path; p=Path("/tmp/replay-parser-2-selected-compact.json"); n=p.stat().st_size; print(f"selected_large_artifact_bytes={n}"); assert n <= 100000'`
7. `scripts/benchmark-phase5.sh --ci`
8. `cargo run -q -p parser-harness --bin benchmark-report-check -- --report .planning/generated/phase-05/benchmarks/benchmark-report.json --mode structural`
9. `cargo clippy --workspace --all-targets -- -D warnings`
</verification>

<success_criteria>
- Default selected-large artifact byte count is `<= 100000` from current release CLI output.
- Default artifact has merged `players[]` counters and no serialized `player_stats`.
- `weapons[]` is deterministic and event rows reference weapons by ID.
- Zero counters, nulls, and empty arrays are omitted from default JSON while deserialization defaults remain valid.
- Default rows retain ordinary `vehicleKills`, `killsFromVehicle`, vehicle class, and relevant entity IDs.
- Default rows omit verbose identity/evidence and bounty fields; debug sidecar still carries detailed evidence when requested.
- Comparison, schema, CLI, parser-core, harness, and benchmark tests are updated and passing.
- Local docs and planning state reflect the new compact default artifact without planning adjacent app changes.
</success_criteria>

<output>
After completion, create `.planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-SUMMARY.md` with commands run, selected-large artifact byte count, benchmark report status, and any remaining Phase 6 blockers.
</output>
