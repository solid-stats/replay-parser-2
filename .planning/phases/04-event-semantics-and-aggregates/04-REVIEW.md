---
phase: 04-event-semantics-and-aggregates
reviewed: 2026-04-27T12:48:38Z
depth: deep
files_reviewed: 36
files_reviewed_list:
  - README.md
  - crates/parser-contract/examples/parse_artifact_success.v1.json
  - crates/parser-contract/examples/parse_failure.v1.json
  - crates/parser-contract/src/aggregates.rs
  - crates/parser-contract/src/artifact.rs
  - crates/parser-contract/src/events.rs
  - crates/parser-contract/src/lib.rs
  - crates/parser-contract/src/schema.rs
  - crates/parser-contract/src/side_facts.rs
  - crates/parser-contract/tests/aggregate_contract.rs
  - crates/parser-contract/tests/artifact_envelope.rs
  - crates/parser-contract/tests/combat_event_contract.rs
  - crates/parser-contract/tests/failure_contract.rs
  - crates/parser-contract/tests/replay_side_facts_contract.rs
  - crates/parser-contract/tests/schema_contract.rs
  - crates/parser-contract/tests/source_ref_contract.rs
  - crates/parser-core/src/aggregates.rs
  - crates/parser-core/src/artifact.rs
  - crates/parser-core/src/events.rs
  - crates/parser-core/src/lib.rs
  - crates/parser-core/src/raw.rs
  - crates/parser-core/src/side_facts.rs
  - crates/parser-core/src/vehicle_score.rs
  - crates/parser-core/tests/aggregate_projection.rs
  - crates/parser-core/tests/combat_event_semantics.rs
  - crates/parser-core/tests/deterministic_output.rs
  - crates/parser-core/tests/fixtures/aggregate-combat.ocap.json
  - crates/parser-core/tests/fixtures/combat-events.ocap.json
  - crates/parser-core/tests/fixtures/killed-events.ocap.json
  - crates/parser-core/tests/fixtures/side-facts.ocap.json
  - crates/parser-core/tests/fixtures/vehicle-score.ocap.json
  - crates/parser-core/tests/parser_core_api.rs
  - crates/parser-core/tests/raw_event_accessors.rs
  - crates/parser-core/tests/side_facts.rs
  - crates/parser-core/tests/vehicle_score.rs
  - schemas/parse-artifact-v1.schema.json
findings:
  critical: 8
  warning: 3
  info: 0
  total: 11
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-04-27T12:48:38Z
**Depth:** deep
**Files Reviewed:** 36
**Status:** issues_found

## Summary

Deep review traced the Phase 4 contract, parser-core event normalization, aggregate projection, vehicle score, side facts, tests, examples, committed schema, planning decisions, and legacy parser references. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo doc --workspace --no-deps` pass, but the implementation still has correctness and contract defects that would produce wrong legacy aggregates, under-validated parse artifacts, or unauditable vehicle-score evidence.

## Blockers

### BL-01: Non-player unit entities are counted as players in combat and aggregate projections

**Classification:** BLOCKER
**File:** `crates/parser-core/src/events.rs:538`
**Also affected:** `crates/parser-core/src/aggregates.rs:359`, `crates/parser-core/src/side_facts.rs:142`
**Issue:** `is_player()` returns true for every `EntityKind::Unit`, and `player_projection_identities()` includes every unit. The legacy parser only creates player records for unit entities with `isPlayer`, non-empty `description`, and `name`, plus connected-event backfill for non-vehicle entities. Current Phase 4 logic will count AI units, empty-name units, and non-player units as kill/death/teamkill/bounty/vehicle-score players.
**Fix:** Add a shared legacy player eligibility predicate based on observed facts and compatibility backfill, then use it in combat classification and aggregate projections.

```rust
fn has_connected_player_backfill(entity: &ObservedEntity) -> bool {
    entity.compatibility_hints.iter().any(|hint| {
        hint.kind == EntityCompatibilityHintKind::ConnectedPlayerBackfill
            && observed_string(&hint.observed_name).is_some_and(|name| !name.is_empty())
    })
}

fn is_legacy_player_entity(entity: &ObservedEntity) -> bool {
    matches!(entity.kind, EntityKind::Unit)
        && ((present_bool(&entity.is_player) == Some(true)
            && observed_string(&entity.observed_name).is_some_and(|name| !name.is_empty())
            && observed_string(&entity.identity.description).is_some_and(|desc| !desc.is_empty()))
            || has_connected_player_backfill(entity))
}
```

Add regression fixtures where `type = "unit"` but `isPlayer = 0` or `description = ""`; those entities must remain observable but must not feed legacy counters, bounty inputs, or vehicle score player rows.

### BL-02: Players with no combat contributions disappear from per-replay legacy results

**Classification:** BLOCKER
**File:** `crates/parser-core/src/aggregates.rs:493`
**Issue:** `legacy_player_game_results()` creates rows only while iterating `LegacyCounter` contributions. A player who participated in a replay but had zero kills, deaths, teamkills, or vehicle kills gets no `legacy.player_game_results` row, so downstream `totalPlayedGames`, score, weekly, squad, and rotation aggregates undercount ordinary participants. The legacy pipeline starts from all parsed players and increments `totalPlayedGames` for every `PlayerGameResult`.
**Fix:** Initialize one `PlayerResultAccumulator` for every eligible player group before applying contributions.

```rust
let mut rows = groups
    .iter()
    .map(|(key, group)| (key.clone(), PlayerResultAccumulator::new(group.clone())))
    .collect::<BTreeMap<_, _>>();
```

Add a fixture with a connected/valid player who has no killed events and assert that the projection contains the player with `totalPlayedGames = 1` and zero counters.

### BL-03: Vehicle-score category mapping drops real raw vehicle classes

**Classification:** BLOCKER
**File:** `crates/parser-core/src/vehicle_score.rs:7`
**Also affected:** `crates/parser-core/src/events.rs:667`, `crates/parser-contract/examples/parse_artifact_success.v1.json:295`
**Issue:** Vehicle score category mapping only accepts already-normalized strings such as `"tank"` or `"apc"`. The contract example preserves raw class evidence like `"rhs_t72ba_tv"` while labeling it as `"tank"`, and legacy `getEntities.ts` preserves `entity.class` as raw vehicle class evidence. With real raw classes, `vehicle_score_category_from_class()` returns `Unknown`; `vehicle_score_contribution()` then drops the contribution entirely.
**Fix:** Use one mapper for both parser code paths and classify real OCAP/Arma class evidence into issue #13 categories. Keep raw class in the artifact, but map through a reviewed table or deterministic prefix/rule set.

```rust
pub fn category_from_vehicle_class(raw_class: Option<&str>) -> VehicleScoreCategory {
    let Some(raw) = raw_class.map(str::to_ascii_lowercase) else {
        return VehicleScoreCategory::Unknown;
    };
    if raw.contains("t72") || raw.contains("tank") {
        return VehicleScoreCategory::Tank;
    }
    // Continue with reviewed APC/heli/plane/truck/car/static rules.
}
```

Add tests using raw classes from the corpus/contract examples, not only synthetic category names.

### BL-04: Friendly vehicle/static destruction is awarded as vehicle-score gain instead of penalty

**Classification:** BLOCKER
**File:** `crates/parser-core/src/events.rs:183`
**Also affected:** `crates/parser-core/src/aggregates.rs:273`
**Issue:** When a player kills a vehicle/static weapon, `normalize_killer_event()` immediately builds `VehicleDestroyed` without comparing killer and victim sides. `vehicle_score_contribution()` treats every `VehicleDestroyed` as `Award`. A friendly vehicle/static destruction from a vehicle should be a vehicle-teamkill penalty input, with the teamkill clamp applied.
**Fix:** Compare known sides before `build_vehicle_destroyed_event()`. Emit a penalty semantic or a vehicle-destroyed event with explicit same-side penalty metadata when the killed vehicle/static belongs to the killer side.

```rust
if is_player(killer) && is_vehicle_or_static(victim) {
    return match same_present_side(killer, victim) {
        Some(true) => build_vehicle_teamkill_event(...),
        _ => build_vehicle_destroyed_event(...),
    };
}
```

Add a fixture where a tank destroys a same-side static weapon and assert `sign = "penalty"`, `denominator_eligible = false`, and `applied_weight = max(matrix_weight, 1.0)`.

### BL-05: JSON Schema does not validate typed aggregate contribution payloads

**Classification:** BLOCKER
**File:** `crates/parser-contract/src/aggregates.rs:134`
**Also affected:** `crates/parser-contract/src/schema.rs:23`, `schemas/parse-artifact-v1.schema.json:204`
**Issue:** `AggregateContributionRef.value` is `serde_json::Value`, and the generated schema leaves it unconstrained. `schema.rs` adds helper definitions such as `VehicleScoreInputValue`, but those definitions are not referenced by `AggregateContributionRef.value`. A `vehicle_score_input` contribution with missing `applied_weight`, a string `matrix_weight`, or arbitrary payload still validates against the committed schema.
**Fix:** Replace raw `Value` with a tagged enum, or add schema conditionals keyed by `kind`/`rule_id`.

```json
{
  "if": { "properties": { "kind": { "const": "vehicle_score_input" } } },
  "then": { "properties": { "value": { "$ref": "#/$defs/VehicleScoreInputValue" } } }
}
```

Add schema regression tests that mutate each contribution kind to an invalid payload and expect validation rejection.

### BL-06: Vehicle-score contribution source refs do not include the vehicle/entity evidence used for category mapping

**Classification:** BLOCKER
**File:** `crates/parser-core/src/aggregates.rs:328`
**Also affected:** `crates/parser-core/src/events.rs:584`
**Issue:** Every aggregate contribution copies only `event.source_refs`. Vehicle-score values include `raw_attacker_vehicle_name`, `raw_attacker_vehicle_class`, target class, mapped categories, and weights, but the contribution source refs do not include the attacker vehicle entity or target entity source refs that justify those raw values. `server-2` cannot audit or recalculate the vehicle-score category from the contribution alone.
**Fix:** Build vehicle-score contribution refs from event source refs plus source refs attached to `attacker_vehicle_name`, `attacker_vehicle_class`, `attacker_vehicle_category`, and `target_category`.

```rust
let source_refs = collect_vehicle_score_source_refs(event, &combat.vehicle_context)?;
```

Add tests that a vehicle-score contribution for a vehicle kill contains both the killed event source path and the attacker vehicle entity source path.

### BL-07: Conflicting outcome fields are silently ignored after the first recognized winner

**Classification:** BLOCKER
**File:** `crates/parser-core/src/side_facts.rs:48`
**Issue:** `normalize_outcome()` scans `winner`, `winningSide`, and `outcome`, then returns the first recognized side. If a replay contains `winner = "WEST"` and `outcome = "EAST"`, the parser emits a known west winner with no diagnostic. Phase 4 decisions require conflicting or ambiguous source evidence to produce diagnostics and potentially partial status.
**Fix:** Collect all present outcome candidates first. If recognized candidates disagree, emit an ambiguity diagnostic with `DiagnosticImpact::DataLoss` and return an explicit unknown or ambiguous inferred state.

```rust
let recognized = candidates.iter().filter_map(|c| accepted_winner_side(&c.value).map(|s| (c, s)));
if recognized_sides.len() > 1 {
    push_conflicting_outcome_diagnostic(...);
    return unknown_outcome();
}
```

Add a test with conflicting valid outcome fields and assert a diagnostic plus non-known outcome.

### BL-08: Malformed `killed` events can be dropped before diagnostics are emitted

**Classification:** BLOCKER
**File:** `crates/parser-core/src/raw.rs:365`
**Issue:** `killed_event()` uses `?` while reading the tuple array, event type, and frame. A source event that is intended to be `"killed"` but has a malformed frame or tuple shape is filtered out by `killed_events()` and never reaches combat normalization, so no unknown event or diagnostic is emitted. This violates the Phase 4 threat model for preserving malformed event source coordinates without silent data loss.
**Fix:** Once an event can be identified as a killed tuple, return a `KilledEventObservation` with malformed frame/field state instead of `None`, or emit a diagnostic from the raw accessor layer.

```rust
if event.get(1).and_then(Value::as_str) == Some("killed") {
    // Preserve json_path even when frame/killed_id/kill_info is malformed.
}
```

Add tests for non-numeric frame and non-array killed tuple shapes that assert partial status and an `event.killed_shape_unknown` diagnostic.

## Warnings

### WR-01: Commander keyword matching produces broad false positives

**Classification:** WARNING
**File:** `crates/parser-core/src/side_facts.rs:178`
**Issue:** `contains_commander_keyword()` checks `value.contains("ks")`, so ordinary names/roles containing those two letters, such as `Maksim` or `Marksman`, can become commander candidates. Candidates are labeled, but this still pollutes commander-side facts and downstream moderator workflows.
**Fix:** Use token or boundary matching for `KS`, bracketed tags, and explicit commander words.

```rust
value.split(|c: char| !c.is_alphanumeric())
    .any(|token| token.eq_ignore_ascii_case("ks") || token == "командир")
```

Add negative tests for names/roles that contain `ks` as a substring but are not commander markers.

### WR-02: Winner-side parsing is case/whitespace brittle

**Classification:** WARNING
**File:** `crates/parser-core/src/side_facts.rs:104`
**Issue:** `accepted_winner_side()` accepts only a few exact strings. Values such as `"West"`, `" BLUFOR "`, `"opfor"`, or localized/common labels are treated as unrecognized despite carrying reliable winner evidence.
**Fix:** Normalize with `trim()` and ASCII case folding before matching, then extend accepted aliases from corpus evidence.

```rust
match value.trim().to_ascii_lowercase().as_str() {
    "west" | "blufor" => Some(EntitySide::West),
    "east" | "opfor" => Some(EntitySide::East),
    ...
}
```

Add tests for mixed-case and whitespace-padded winner labels.

### WR-03: README claims Phase 4 is complete and verified while validation artifacts still show pending state

**Classification:** WARNING
**File:** `README.md:9`
**Issue:** The README states Phase 4 work is "complete and verified", but `.planning/STATE.md` still says Phase 04 Plan 06 is ready to execute and `04-VALIDATION.md` is still `status: draft` with pending task rows. This makes repository status unreliable during handoff.
**Fix:** Update README only after phase verification artifacts are complete, or phrase the status as implementation-submitted/pending review until verification and review close.

---

_Reviewed: 2026-04-27T12:48:38Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
