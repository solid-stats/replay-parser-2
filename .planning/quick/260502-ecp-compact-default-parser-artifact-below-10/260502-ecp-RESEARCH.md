# Quick Task 260502-ecp - Research

**Researched:** 2026-05-02 [VERIFIED: environment context]
**Domain:** Rust Serde contract compaction for the default parser artifact [VERIFIED: .planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-CONTEXT.md]
**Confidence:** HIGH for local surfaces and size drivers; MEDIUM for the exact final byte count until implemented and rerun on the selected replay. [VERIFIED: local source inspection and /tmp selected-replay parse]

## User Constraints

All constraints in this section are copied or condensed from the quick-task context file. [VERIFIED: .planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-CONTEXT.md]

### Task Boundary

Reduce the default parser artifact size so the selected large replay no longer exceeds the 100,000 byte hard limit. The change should target the default server-facing artifact, not raw OCAP files or debug sidecars. [VERIFIED: CONTEXT.md]

The parser must stay within `replay-parser-2` ownership: no canonical player matching, no PostgreSQL persistence, no public API/UI behavior, and no replay discovery/fetching. [VERIFIED: CONTEXT.md]

### Locked Implementation Decisions

- Merge `player_stats[]` counters into `players[]` rows. [VERIFIED: CONTEXT.md]
- Use numeric `source_entity_id`/`eid` as the replay-local player reference. [VERIFIED: CONTEXT.md]
- Omit zero counters instead of serializing them explicitly. [VERIFIED: CONTEXT.md]
- In `kills[]` and `destroyed_vehicles[]`, replace repeated names and side values with player/entity ID references. [VERIFIED: CONTEXT.md]
- Keep verbose identity/event evidence in debug sidecar output only. [VERIFIED: CONTEXT.md]
- Default bounty eligibility is computed downstream from kill classification; omit `bounty_eligible` and `bounty_exclusion_reasons` from the default artifact. [VERIFIED: CONTEXT.md]
- Default event rows keep vehicle class plus source entity ID where relevant. [VERIFIED: CONTEXT.md]
- Vehicle names are debug-only. [VERIFIED: CONTEXT.md]
- Do not remove ordinary `vehicleKills` and `killsFromVehicle` counters. [VERIFIED: CONTEXT.md]
- Add a weapon dictionary and reference weapons by compact IDs in event rows. [VERIFIED: CONTEXT.md]
- Default JSON omits `null`, empty arrays, and zero counters. [VERIFIED: CONTEXT.md]
- Use short schema-backed keys for compact default JSON. [VERIFIED: CONTEXT.md]
- Avoid tuple-array rows unless the object form still fails the selected replay hard limit. [VERIFIED: CONTEXT.md]

### Size Evidence From Context

- Current selected large artifact: `203683` bytes. [VERIFIED: CONTEXT.md and .planning/generated/phase-05/benchmarks/selected-large-artifact.json]
- Omitting only `null` and empty arrays is about `173 KB`, not sufficient. [VERIFIED: CONTEXT.md]
- Merging player stats, omitting zero counters, and using ID-linked events is about `91 KB`. [VERIFIED: CONTEXT.md]
- With weapon dictionary and bounty-field removal, the artifact is about `86 KB` before short-key savings. [VERIFIED: CONTEXT.md]

## Project Constraints

- `replay-parser-2` owns parser artifacts, parser-core, CLI/worker adapters, and parity harnesses only; replay discovery is owned by `replays-fetcher`, persistence/API/canonical identity are owned by `server-2`, and UI is owned by `web`. [VERIFIED: AGENTS.md]
- Parser output must preserve observed replay identity only and must not perform canonical player matching. [VERIFIED: AGENTS.md]
- CLI and worker modes must use the same parser core. [VERIFIED: AGENTS.md]
- Default artifacts must stay compact server-facing outputs; heavy evidence belongs in debug sidecars or raw replay reprocessing. [VERIFIED: .planning/PROJECT.md]
- Phase 6 remains blocked until selected size, selected x3/parity, and all-raw x10/zero-failure/size gates pass or receive explicit user acceptance. [VERIFIED: .planning/STATE.md]
- This quick task should not modify files outside this requested research artifact. [VERIFIED: user request]

## Current Surfaces

| Surface | Current Finding | Planning Impact |
|---|---|---|
| Contract envelope | `ParseArtifact` currently serializes `players`, `player_stats`, `kills`, `destroyed_vehicles`, `diagnostics`, `replay`, `source`, `failure`, and `extensions`. [VERIFIED: crates/parser-contract/src/artifact.rs] | Remove the default `player_stats` table, add a schema-backed weapon dictionary, and add `skip_serializing_if` defaults for empty/default success fields. |
| Player rows | `MinimalPlayerRow` duplicates both `player_id: "entity:<id>"` and numeric `source_entity_id`, plus name, side, group, role, SteamID, and `compatibility_key`. [VERIFIED: crates/parser-contract/src/minimal.rs] | Use numeric `eid` as the primary ref; include compatibility key only when it differs from the derivable default. |
| Stat rows | `MinimalPlayerStatsRow` repeats `player_id` and `source_entity_id` for every player and serializes every zero counter. [VERIFIED: crates/parser-contract/src/minimal.rs] | Merge non-zero counters into `players[]` with defaults and `skip_serializing_if = "is_zero_u64"`. |
| Kill rows | `MinimalKillRow` repeats killer/victim IDs, names, sides, weapon strings, attacker vehicle names/classes, `bounty_eligible`, and exclusion arrays. [VERIFIED: crates/parser-contract/src/minimal.rs] | Keep compact numeric refs, classification, weapon ID, attacker vehicle entity ID/class, and remove names/sides/bounty fields from default output. |
| Destroyed vehicle rows | `MinimalDestroyedVehicleRow` repeats attacker identity, side, weapon string, attacker vehicle name/class, destroyed name/class/type/side. [VERIFIED: crates/parser-contract/src/minimal.rs] | Keep attacker entity ref, classification, weapon ID, attacker vehicle entity/class, destroyed entity ID/type/class; move names and repeated side strings to debug sidecar. |
| Parser-core builder | `derive_minimal_tables` builds separate `players`, `player_stats`, `kills`, and `destroyed_vehicles` from normalized entities/events. [VERIFIED: crates/parser-core/src/aggregates.rs] | The accumulator should produce one compact player row plus compact event rows and a deterministic weapon dictionary in the same pass. |
| Debug sidecar | `parse_replay_debug` emits full entities, events, side facts, and full diagnostics separately from `ParseArtifact`. [VERIFIED: crates/parser-core/src/debug_artifact.rs] | Do not shrink debug sidecar as part of this task; it is the correct home for verbose evidence. |
| CLI default | `parse` writes minified JSON with `serde_json::to_vec`; `--pretty` uses `to_vec_pretty`; `--debug-artifact` calls `parse_replay_debug` only when explicitly requested. [VERIFIED: crates/parser-cli/src/main.rs] | Keep compaction in typed contract/core serialization, not in an ad hoc CLI JSON post-processor. |
| Schema | Schema generation closes default artifact definitions with `additionalProperties: false`; current tests expect v3 fields and reject debug fields in minimal rows. [VERIFIED: crates/parser-contract/src/schema.rs and crates/parser-contract/tests/schema_contract.rs] | Regenerate schema/examples and update closed definitions for compact row names, missing defaulted fields, and weapon dictionary. |
| Comparison harness | `comparison.rs` deserializes `MinimalPlayerRow`, `MinimalPlayerStatsRow`, `MinimalKillRow`, and `MinimalDestroyedVehicleRow`; bounty inputs currently filter `kill.bounty_eligible`. [VERIFIED: crates/parser-harness/src/comparison.rs] | Update comparison to derive legacy stats from merged player counters and derive bounty inputs from classification/ref fields. |
| Benchmark gate | The selected artifact passes only when `artifact_bytes <= 100000`; all-raw passes only when median ratio <= 0.05, p95 <= 0.10, max <= 100000, and oversized count is 0. [VERIFIED: crates/parser-harness/src/benchmark_report.rs] | The final proof must be benchmark evidence, not a unit-test-only estimate. |

## Size Findings

I regenerated the selected large replay to `/tmp/replay-parser-2-selected-current.json` with the current release CLI to separate current-code evidence from stale generated files. [VERIFIED: `cargo run -q -p parser-cli --bin replay-parser-2 --release -- parse /home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json --output /tmp/replay-parser-2-selected-current.json`]

| Measurement | Value | Meaning |
|---|---:|---|
| Current regenerated selected artifact | 198,943 bytes | Still fails the hard 100,000-byte gate. [VERIFIED: stat /tmp/replay-parser-2-selected-current.json] |
| Saved generated selected artifact | 203,683 bytes | Matches the blocker, but it contains verbose diagnostic evidence that current source/schema no longer allow. [VERIFIED: stat .planning/generated/phase-05/benchmarks/selected-large-artifact.json and crates/parser-contract/src/diagnostic.rs] |
| Current row counts | 245 players, 245 stat rows, 224 kills, 22 destroyed vehicles, 10 diagnostics | The size problem is row duplication, not row count explosion. [VERIFIED: jq /tmp/replay-parser-2-selected-current.json] |
| Current subtree bytes | players 46,045; player_stats 43,397; kills 94,981; destroyed_vehicles 9,058; diagnostics 1,900 | `kills[]`, `players[]`, and `player_stats[]` are the main byte drivers. [VERIFIED: local JSON size script] |
| Omit null/empty/zero only | 141,540 bytes | This is not enough, matching the discussion. [VERIFIED: local JSON transform] |
| Decision-shaped object rows | about 42,160 bytes | Merging stats, numeric refs, dictionary weapons, bounty removal, and omitted defaults should clear the selected hard limit without tuple rows. [VERIFIED: local scratch transform against /tmp selected artifact] |

## Recommended Shape

Keep top-level table names readable and schema-backed: `players`, `kills`, `destroyed_vehicles`, `weapons`, and `diagnostics`; remove top-level `player_stats` from default success artifacts. [VERIFIED: CONTEXT.md; recommendation based on existing ParseArtifact shape]

Use short keys inside high-cardinality rows. Recommended map: [ASSUMED: exact short key names are planner choice]

| Row | Keys |
|---|---|
| `players[]` | `eid`, `n`, `s`, `g`, `r`, `sid`, `ck`, `k`, `d`, `tk`, `su`, `nkd`, `ud`, `vk`, `kfv` |
| `kills[]` | `k`, `v`, `c`, `w`, `av`, `avc` |
| `destroyed_vehicles[]` | `a`, `c`, `w`, `av`, `avc`, `de`, `dt`, `dc` |
| `weapons[]` | `id`, `n` |

Rules for those keys: `eid` is numeric source entity ID; `ck` is only needed when compatibility key is not derivable from `eid`; `w` references `weapons[].id`; `av` is attacker vehicle entity ID; `avc` is attacker vehicle class; vehicle names stay debug-only. [VERIFIED: CONTEXT.md and crates/parser-core/src/aggregates.rs]

The weapon dictionary should be built deterministically with ordered data structures already standard in the repo, such as `BTreeMap`, then serialized in stable ID order. [VERIFIED: std::collections::BTreeMap usage in crates/parser-core/src/aggregates.rs and crates/parser-contract/src/artifact.rs]

## Serde And Schema Pattern

Use typed Serde attributes on contract structs rather than serializing a verbose artifact and mutating a generic `serde_json::Value`. [VERIFIED: crates/parser-cli/src/main.rs uses typed `serde_json::to_vec`; ASSUMED: planner should preserve this architecture]

Recommended pattern:

```rust
fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompactPlayerRow {
    #[serde(rename = "eid")]
    pub source_entity_id: i64,

    #[serde(default, rename = "n", skip_serializing_if = "Option::is_none")]
    pub observed_name: Option<String>,

    #[serde(default, rename = "k", skip_serializing_if = "is_zero_u64")]
    pub kills: u64,

    #[serde(default, rename = "kfv", skip_serializing_if = "is_zero_u64")]
    #[schemars(rename = "kfv")]
    pub kills_from_vehicle: u64,
}
```

- `#[serde(rename = "...")]` is the standard way to serialize a field with a different key. [CITED: https://serde.rs/field-attrs.html]
- `#[serde(skip_serializing_if = "path")]` calls a predicate such as `Option::is_none` or a local `is_zero_u64` function to decide whether to omit a field. [CITED: https://serde.rs/field-attrs.html]
- `#[serde(default)]` gives missing deserialized fields their default value and should be paired with skipped non-Option counters and vectors. [CITED: https://serde.rs/field-attrs.html]
- Schemars derives schemas for the serde JSON representation and generally respects serde attributes; generated schema must still be tested because this repo manually closes schema definitions. [CITED: https://docs.rs/schemars/latest/schemars/derive.JsonSchema.html; VERIFIED: crates/parser-contract/src/schema.rs]
- Avoid adding `serde_with` for this task because the workspace already has enough Serde primitives and no crate currently depends on `serde_with`. [VERIFIED: Cargo.toml files]

## Compatibility Risks

| Risk | Why It Matters | Required Planner Action |
|---|---|---|
| `player_stats[]` removal | Tests, examples, schema text checks, parser-core deterministic tests, CLI tests, and harness comparison currently reference `player_stats`. [VERIFIED: rg player_stats crates] | Update contract examples/schema/tests, parser-core tests, CLI tests, and comparison table loading in one slice. |
| `bounty_eligible` removal | Harness `bounty_inputs` currently filters on `kill.bounty_eligible`; parser-core tests assert this field. [VERIFIED: crates/parser-harness/src/comparison.rs and rg bounty_eligible crates] | Replace bounty filtering with classification/ref logic agreed in context, then update tests to assert field absence. |
| Short-key rows | Current schema closes minimal row definitions and rejects unknown properties. [VERIFIED: crates/parser-contract/src/schema.rs] | Regenerate schema and add tests that short keys validate while verbose row keys are rejected. |
| Omitted fields | Current tests assert top-level `failure`, arrays, and many row fields are present even when empty/null/zero. [VERIFIED: crates/parser-cli/tests/parse_command.rs and crates/parser-contract/tests/schema_contract.rs] | Update tests to assert omitted success defaults deserialize and schema-validate. Failed artifacts must still require `failure`. |
| Weapon dictionary | Adding top-level `weapons` will be rejected by current root schema because `unevaluatedProperties` is false. [VERIFIED: crates/parser-contract/src/schema.rs] | Add the dictionary to `ParseArtifact`, schema examples, and comparison deserialization. |
| Stale generated benchmark artifact | The saved selected artifact has verbose diagnostic `source_refs`, while current `MinimalDiagnosticRow` has only code, severity, message, and parser_action. [VERIFIED: .planning/generated/phase-05/benchmarks/selected-large-artifact.json and crates/parser-contract/src/diagnostic.rs] | Regenerate benchmark artifacts before final size claims; use current CLI output as the baseline. |
| Debug sidecar leakage | Phase 5.2 explicitly moved full source refs, rule IDs, frames, and event indexes to `parse_replay_debug`. [VERIFIED: crates/parser-core/src/debug_artifact.rs and .planning/STATE.md] | Keep all verbose evidence tests on the debug sidecar path and add default-artifact recursive absence checks for removed keys. |

## Test And Verification Plan

1. Contract/schema: update `crates/parser-contract/src/minimal.rs`, `artifact.rs`, and schema examples; then run `cargo test -p parser-contract schema_contract`. [VERIFIED: existing schema tests in crates/parser-contract/tests/schema_contract.rs]
2. Parser-core behavior: update aggregate projection tests for merged counters, numeric refs, weapon dictionary, omitted zero/null fields, and debug-only names/provenance; run `cargo test -p parser-core --test aggregate_projection --test deterministic_output --test debug_artifact --test combat_event_semantics --test fault_injection_regressions`. [VERIFIED: existing test files]
3. CLI behavior: prove default minified output omits `null`, empty arrays, zero counters, verbose names in event rows, bounty fields, source refs, and debug keys; run `cargo test -p parser-cli --test parse_command --test schema_command`. [VERIFIED: crates/parser-cli/tests/parse_command.rs]
4. Harness behavior: update comparison derivation to read merged player counters and derive bounty inputs from compact kills; run `cargo test -p parser-harness comparison_report benchmark_report`. [VERIFIED: crates/parser-harness/src/comparison.rs and crates/parser-harness/src/benchmark_report.rs]
5. Selected size proof: run `cargo run -q -p parser-cli --bin replay-parser-2 --release -- parse /home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json --output /tmp/replay-parser-2-selected-current.json` and require `wc -c < /tmp/replay-parser-2-selected-current.json` to be `<= 100000`. [VERIFIED: selected replay path in .planning/STATE.md]
6. Benchmark report proof: run `scripts/benchmark-phase5.sh --ci` and `cargo run -p parser-harness --bin benchmark-report-check -- --report .planning/generated/phase-05/benchmarks/benchmark-report.json --mode structural`; for acceptance, rerun with full corpus and old baseline prerequisites, then use `--mode acceptance`. [VERIFIED: scripts/benchmark-phase5.sh and crates/parser-harness/src/bin/benchmark-report-check.rs]

## Open Questions

1. Exact short key spellings are not locked beyond `source_entity_id`/`eid`; the recommended map above should be treated as the planner default unless the user wants different public field names. [ASSUMED]
2. If implementation somehow remains above 100,000 bytes with object rows, tuple-array rows are the locked fallback, but local scratch sizing suggests they are not needed for the selected replay. [VERIFIED: CONTEXT.md and local scratch transform]

## Sources

### Primary

- `.planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-CONTEXT.md` - locked quick-task decisions and size targets.
- `AGENTS.md` - parser ownership, cross-app boundaries, workflow constraints.
- `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md` - Phase 5.2 blocker, hard artifact gates, and current project state.
- `crates/parser-contract/src/minimal.rs`, `artifact.rs`, `schema.rs`, `diagnostic.rs` - current contract/schema surfaces.
- `crates/parser-core/src/aggregates.rs`, `artifact.rs`, `debug_artifact.rs` - current default/debug artifact builders.
- `crates/parser-cli/src/main.rs` - minified default output and debug sidecar behavior.
- `crates/parser-harness/src/comparison.rs`, `benchmark_report.rs`, `scripts/benchmark-phase5.sh` - comparison and benchmark gate behavior.
- `.planning/generated/phase-05/benchmarks/selected-large-artifact.json` and `/tmp/replay-parser-2-selected-current.json` - saved and regenerated size evidence.

### Official Docs

- Serde field attributes: https://serde.rs/field-attrs.html
- Serde skip serializing example: https://serde.rs/attr-skip-serializing.html
- Schemars `JsonSchema` derive docs: https://docs.rs/schemars/latest/schemars/derive.JsonSchema.html

## Assumptions Log

| ID | Claim | Risk If Wrong |
|---|---|---|
| A1 | The exact short-key spellings can be chosen by the planner as long as schema/examples/tests lock them. | If downstream has already seen a preferred compact key map, the plan may need a field-name-only adjustment. |
