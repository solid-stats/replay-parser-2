---
phase: 04
slug: event-semantics-and-aggregates
status: complete
researched: 2026-04-27
confidence: medium-high
---

# Phase 04 Research: Event Semantics and Aggregates

## Research Question

What needs to be known to plan Phase 4 well?

Phase 4 must turn the Phase 3 metadata/entity foundation into auditable combat,
commander/outcome, bounty, legacy aggregate, and vehicle-score artifact data
without taking ownership of `server-2` persistence, canonical identity, public
API, UI, replay discovery, worker transport, or full-corpus parity commands.

## Scope And Boundary Findings

- Phase 4 is parser-core and parser-contract work only. CLI commands, worker
  mode, RabbitMQ/S3 artifact keys, full old-vs-new comparison commands,
  benchmarks, coverage enforcement, and production persistence remain later
  phases. [VERIFIED: `.planning/ROADMAP.md`]
- Parser output may change artifact shape and therefore has `server-2`
  persistence/recalculation impact. Available adjacent app evidence is brief
  level for `server-2` and `web`; those repositories currently contain only
  `gsd-briefs/`. `replays-fetcher` has full planning docs confirming it owns
  only raw replay staging and not parser artifacts. [VERIFIED: local adjacent
  repo file listing + `gsd-briefs/*.md` + `replays-fetcher/.planning/PROJECT.md`]
- Artifact changes must keep observed identity raw. Legacy same-name behavior
  can be used in compatibility projections, but parser must not introduce
  canonical player IDs. [VERIFIED: `.planning/PROJECT.md`,
  `.planning/phases/04-event-semantics-and-aggregates/04-CONTEXT.md`]

## Current Code Facts

- `crates/parser-contract/src/events.rs` already has `NormalizedEvent`,
  `NormalizedEventKind`, `EventActorRef`, non-empty `SourceRefs`, `RuleId`, and
  a deterministic `BTreeMap<String, Value>` attributes object.
- `crates/parser-contract/src/aggregates.rs` already has
  `AggregateContributionRef`, `AggregateContributionKind`, source refs, rule ID,
  and namespaced projection map support.
- `ParseArtifact` currently contains `events: Vec<NormalizedEvent>` and
  `aggregates: AggregateSection`, but no typed replay-side commander/outcome
  section. [VERIFIED: `crates/parser-contract/src/artifact.rs`]
- `parse_replay` currently normalizes metadata and entities, then emits empty
  `events` and a default `AggregateSection`. [VERIFIED:
  `crates/parser-core/src/artifact.rs`]
- Raw OCAP shape handling lives in `crates/parser-core/src/raw.rs`, using
  borrowed `RawReplay`, `RawField<T>`, stable JSON paths, and focused helpers
  such as `connected_events`.
- Diagnostics and status escalation already use `DiagnosticAccumulator` with
  `DiagnosticImpact::DataLoss` to mark artifacts partial when source evidence
  is dropped or cannot be audited. [VERIFIED:
  `crates/parser-core/src/diagnostics.rs`]

## Legacy Event Semantics

Legacy `getKillsAndDeaths.ts` is the authoritative v1 behavior reference for
combat counters:

- Source kill tuple shape is `[frame, "killed", killed_id, kill_info, distance]`,
  where `kill_info` is `[killer_id, weapon]` or `["null"]`. [VERIFIED:
  legacy `src/0 - types/replay.d.ts`]
- Null killer and player victim: victim becomes dead; no killer counters are
  incremented. Null killer and vehicle victim is ignored by old counters.
- Enemy player kill: killer increments `kills`; victim increments death;
  relationship arrays record killer/killed; weapon stats update if weapon exists.
- Same-side non-suicide player kill: killer increments `teamkills`; victim
  becomes dead by teamkill; teamkill relationship arrays update; normal kill
  counter does not increment.
- Suicide player kill: victim becomes dead, `isDeadByTeamkill` is false, and
  neither normal kill nor teamkill counters increment.
- Killed vehicle with player killer: killer increments `vehicleKills`.
- Kills from vehicle are detected by matching the kill tuple weapon string
  against a vehicle entity name, not by temporal vehicle occupancy. [VERIFIED:
  legacy `getKillsAndDeaths.ts`]
- Legacy tests confirm null killers, same-side teamkills, same-name slot
  compatibility, vehicle kills, kills-from-vehicle, and suicides. [VERIFIED:
  legacy `src/!tests/unit-tests/1 - replays, 2 - parseReplayInfo/data/parseReplays.ts`]

Planning implication: Phase 4 should create one normalized event per source
`killed` tuple, then derive counter effects as aggregate contributions. The
event carries the semantic classification; contributions carry specific legacy,
bounty, relationship, and vehicle-score effects.

## Legacy Aggregate Formulas

Legacy global/weekly aggregate formulas are straightforward and should be
represented as per-replay projection inputs in Phase 4:

- `deaths.total` increments when `isDead` is true.
- `deaths.byTeamkills` increments when `isDeadByTeamkill` is true.
- `kdRatio = round((kills - teamkills) / abs(deaths.total - deaths.byTeamkills), 2)`;
  when denominator is zero, return `kills - teamkills`.
- `score` / `totalScore = round((kills - teamkills) / (totalPlayedGames - deaths.byTeamkills), 2)`;
  when denominator is `<= 0`, return `kills - teamkills`.
- `killsFromVehicleCoef = round(killsFromVehicle / kills, 2)`, or `0` when
  either value is zero.
- Relationship lists are `killed`, `killers`, `teamkilled`, and `teamkillers`.
  [VERIFIED: legacy `calculateKDRatio.ts`, `calculateScore.ts`,
  `calculateVehicleKillsCoef.ts`, `calculateDeaths.ts`, `global/add.ts`,
  `global/addToResultsByWeek.ts`]

Planning implication: parser-core should emit per-replay projections, not
multi-replay all-time/weekly/rotation final results. The parity harness and
`server-2` can combine per-replay projections later.

## Vehicle Score Requirement

GitHub issue #13 is captured in project planning and requires:

- Use only kills from vehicles.
- Weight each contribution by attacker vehicle category and killed entity
  category.
- Subtract weighted vehicle-teamkill penalties.
- Divide final cross-replay score by the number of games where the player had at
  least one kill from a vehicle.
- Clamp teamkill penalty multipliers below 1 up to 1.
- Expose source references for each contribution. [VERIFIED:
  `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`]

Issue #13 category mapping should start from raw evidence:

| Raw class/evidence | Issue #13 category |
|--------------------|--------------------|
| `static-weapon` | `static_weapon` |
| `car` | `car` |
| `truck` | `truck` |
| `apc` | `apc` |
| `tank` | `tank` |
| `heli` | `heli` |
| `plane` | `plane` |
| player victim | `player` |

Raw classes such as `parachute`, `sea`, missing class, or unmatched weapon name
should be preserved as raw evidence and classified as unknown/not eligible for
vehicle-score contribution until a later human-reviewed mapping exists.

Planning implication: vehicle score should be a dedicated plan after combat
classification exists. It needs tests for APC/tank weights below 1 and the
teamkill clamp (`raw_matrix_weight < 1`, `applied_penalty_weight == 1`).

## Commander And Outcome Data

- The old ordinary parser does not have a first-class commander/outcome aggregate
  path. The annual nomination code has a commander-slot heuristic that groups
  player entities by side and group, then treats the first grouped entity as a
  side commander candidate. This is v2 historical context, not canonical truth.
  [VERIFIED: legacy `src/!yearStatistics/nominations/mostKillsFromCommanderSlot.ts`]
- `server-2` owns final commander-side stats persistence, manual winner
  correction, canonical identity, and public APIs. [VERIFIED:
  `gsd-briefs/server-2.md`]
- Phase 4 context requires a typed contract section for commander/outcome facts;
  `extensions` must not be the primary path for data `server-2` is expected to
  consume. [VERIFIED: D-15 in `04-CONTEXT.md`]
- Missing commander/winner data is expected in legacy replays and should not by
  itself mark the artifact partial. Conflicting or lossy evidence should produce
  diagnostics and may make the artifact partial. [VERIFIED: D-16]

Planning implication: add a typed replay-side facts section and implement
conservative extraction. Known/explicit fields can be `present`; heuristics must
be `inferred`/candidate with confidence, rule ID, and source refs; absent facts
must be explicit unknown states.

## Recommended Architecture

1. Extend `parser-contract` before filling parser-core output:
   - Add typed combat payload structures or stable attribute payload shape for
     `NormalizedEvent`.
   - Add typed replay-side facts for commander and outcome.
   - Add schema/example tests for event payloads, aggregate projections, vehicle
     score contributions, and replay-side facts.
2. Add raw accessors in `parser-core/src/raw.rs`:
   - `killed_events(raw) -> Vec<KilledEventObservation>`.
   - Optional top-level outcome field helpers for `winner`, `winningSide`,
     `outcome`, and marker/source-text candidates.
3. Add focused parser-core modules:
   - `events.rs` for combat normalization.
   - `aggregates.rs` for contribution/projection derivation.
   - `vehicle_score.rs` for taxonomy and issue #13 weights.
   - `side_facts.rs` for commander/outcome extraction.
4. Update `artifact.rs` to assemble metadata, entities, combat events,
   replay-side facts, aggregates, diagnostics, and final status in one
   deterministic path.

## Validation Architecture

Phase 4 should use existing Cargo infrastructure and focused behavior tests:

- Contract/schema tests:
  - `cargo test -p parser-contract combat_event_contract`
  - `cargo test -p parser-contract aggregate_contract`
  - `cargo test -p parser-contract replay_side_facts_contract`
  - `cargo test -p parser-contract schema_contract`
- Parser-core tests:
  - `cargo test -p parser-core raw_event_accessors`
  - `cargo test -p parser-core combat_event_semantics`
  - `cargo test -p parser-core aggregate_projection`
  - `cargo test -p parser-core vehicle_score`
  - `cargo test -p parser-core side_facts`
  - `cargo test -p parser-core deterministic_output`
- Full gate:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo doc --workspace --no-deps`

Test fixtures should stay small and behavior-focused in Phase 4. Full corpus and
golden parity remain Phase 5.

## Threat And Risk Notes

| Risk | Mitigation |
|------|------------|
| Event semantics drift from legacy behavior | Use `getKillsAndDeaths.ts` and legacy tests as read-first oracles in combat plans. |
| Aggregate counters become unauditable | Every projection value must be derived from `AggregateContributionRef` with event ID, source refs, and rule ID. |
| Vehicle score category guessing corrupts stats | Preserve raw evidence, emit unknown diagnostics for unmapped classes, and require source refs for every contribution. |
| Parser leaks canonical identity ownership | Use observed entity IDs/names plus compatibility keys only; no canonical player IDs. |
| Missing commander/winner becomes false data loss | Absence emits explicit unknown, not partial status. |
| Artifact shape affects `server-2`/`web` expectations | Keep parser artifact changes documented in plans and route public API shape through `server-2`, not direct `web` consumption. |

## Suggested Plan Split

1. Contract extensions for combat payloads, aggregate projection payloads,
   vehicle score evidence, and replay-side facts.
2. Raw killed tuple accessors and source references.
3. Combat event normalization and diagnostics.
4. Legacy per-replay aggregate projections, relationships, game-type
   compatibility metadata, and bounty inputs.
5. Vehicle score taxonomy, weights, contributions, denominator inputs, and
   clamp tests.
6. Commander/outcome facts with unknown/candidate policy.
7. Artifact integration, README handoff, schema regeneration, and full quality
   gates.

## Sources

- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/research/SUMMARY.md`
- `.planning/phases/04-event-semantics-and-aggregates/04-CONTEXT.md`
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md`
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md`
- `crates/parser-contract/src/events.rs`
- `crates/parser-contract/src/aggregates.rs`
- `crates/parser-contract/src/artifact.rs`
- `crates/parser-core/src/raw.rs`
- `crates/parser-core/src/entities.rs`
- `crates/parser-core/src/artifact.rs`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/2 - parseReplayInfo/getKillsAndDeaths.ts`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/2 - parseReplayInfo/getEntities.ts`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/2 - parseReplayInfo/combineSamePlayersInfo.ts`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/0 - types/replay.d.ts`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/0 - utils/calculateKDRatio.ts`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/0 - utils/calculateScore.ts`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/0 - utils/calculateVehicleKillsCoef.ts`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/3 - statistics/global/add.ts`
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/3 - statistics/global/addToResultsByWeek.ts`
- `gsd-briefs/replay-parser-2.md`, `gsd-briefs/server-2.md`, `gsd-briefs/web.md`
- `/home/alexandr/Projects/SolidGames/replays-fetcher/.planning/PROJECT.md`

## Research Complete

This research is sufficient to plan Phase 4. Remaining uncertainty is execution
data uncertainty, not planning uncertainty: exact representative real-corpus
winner/commander examples and full old-vs-new aggregate tolerances belong to
Phase 5 parity work or later human review.
