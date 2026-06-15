# Graph Report - .  (2026-06-15)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 533 nodes · 1603 edges · 14 communities
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 52 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `72116db6`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]

## God Nodes (most connected - your core abstractions)
1. `ObservedEntity` - 22 edges
2. `normalize_killer_event()` - 21 edges
3. `Option` - 20 edges
4. `RawValue` - 20 edges
5. `SourceContext` - 19 edges
6. `Option` - 19 edges
7. `Result` - 19 edges
8. `ObservedEntity` - 18 edges
9. `normalize_null_killer_event()` - 18 edges
10. `derive_minimal_tables_from_killed_events()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `success_artifact()` --calls--> `derive_minimal_tables_from_killed_events()`  [INFERRED]
  crates/parser-core/src/artifact.rs → crates/parser-core/src/aggregates.rs
- `minimal_players()` --calls--> `is_legacy_player_entity()`  [INFERRED]
  crates/parser-core/src/aggregates.rs → crates/parser-core/src/legacy_player.rs
- `classify_minimal_null_killer_event()` --calls--> `is_legacy_player_entity()`  [INFERRED]
  crates/parser-core/src/aggregates.rs → crates/parser-core/src/legacy_player.rs
- `classify_minimal_killer_event()` --calls--> `is_legacy_player_entity()`  [INFERRED]
  crates/parser-core/src/aggregates.rs → crates/parser-core/src/legacy_player.rs
- `unknown_death_or_no_stats()` --calls--> `is_legacy_player_entity()`  [INFERRED]
  crates/parser-core/src/aggregates.rs → crates/parser-core/src/legacy_player.rs

## Import Cycles
- 1-file cycle: `crates/parser-core/src/diagnostics.rs -> crates/parser-core/src/diagnostics.rs`
- 1-file cycle: `crates/parser-core/src/events.rs -> crates/parser-core/src/events.rs`
- 1-file cycle: `crates/parser-core/src/lib.rs -> crates/parser-core/src/lib.rs`
- 1-file cycle: `crates/parser-core/src/raw.rs -> crates/parser-core/src/raw.rs`
- 1-file cycle: `crates/parser-core/src/raw_compact.rs -> crates/parser-core/src/raw_compact.rs`

## Communities (14 total, 0 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.09
Nodes (79): BTreeSet, ConnectedEventObservation, Diagnostic, DiagnosticAccumulator, EntityKind, EntitySide, FieldPresence, ObservedEntity (+71 more)

### Community 1 - "Community 1"
Cohesion: 0.09
Nodes (67): BTreeMap, DiagnosticAccumulator, EntityKind, EntitySide, FieldPresence, FnOnce, KilledEventObservation, ObservedEntity (+59 more)

### Community 2 - "Community 2"
Cohesion: 0.11
Nodes (58): CommanderOutcomeMessage, CommanderSideFact, DiagnosticAccumulator, EntitySide, EventActorRef, FieldPresence, MissionMessageEventObservation, ObservedEntity (+50 more)

### Community 3 - "Community 3"
Cohesion: 0.15
Nodes (57): BountyEligibility, BountyExclusionReason, CombatSemantic, CombatVictimKind, BTreeMap, DiagnosticAccumulator, EntityKind, EntitySide (+49 more)

### Community 4 - "Community 4"
Cohesion: 0.13
Nodes (42): Box, Option, RawField, RawValue, String, Vec, KilledEventKillInfo, boxed_raw() (+34 more)

### Community 5 - "Community 5"
Cohesion: 0.11
Nodes (38): ConnectedEventObservation, FnOnce, KilledEventObservation, MissionMessageEventObservation, Option, RawEntityObservation, RawField, RawOcapRoot (+30 more)

### Community 6 - "Community 6"
Cohesion: 0.09
Nodes (35): FieldPresence, Option, ParseArtifact, ParserInfo, ParserInput, RawOcapRoot, ReplayMetadata, ReplaySource (+27 more)

### Community 7 - "Community 7"
Cohesion: 0.14
Nodes (16): A, Result, Self, D, Deserialize, E, Error, Formatter (+8 more)

### Community 8 - "Community 8"
Cohesion: 0.16
Nodes (26): Diagnostic, DiagnosticAccumulator, FieldPresence, Option, RawField, RawReplay, ReplayMetadata, RuleId (+18 more)

### Community 9 - "Community 9"
Cohesion: 0.18
Nodes (13): Diagnostic, Option, ParserOptions, Self, SourceContext, Vec, From, ParseStatus (+5 more)

### Community 10 - "Community 10"
Cohesion: 0.15
Nodes (13): ContractVersion, Diagnostic, NormalizedEvent, ObservedEntity, Option, ParserInfo, ParserInput, ReplayMetadata (+5 more)

### Community 11 - "Community 11"
Cohesion: 0.26
Nodes (12): FieldPresence, ObservedEntity, Option, SourceRefs, String, has_connected_player_backfill(), is_legacy_player_entity(), legacy_player_entity_should_ignore_not_applicable_identity_strings() (+4 more)

### Community 12 - "Community 12"
Cohesion: 0.22
Nodes (7): ParserInfo, ParserOptions, ReplaySource, Self, Default, ParserInput, ParserOptions

### Community 13 - "Community 13"
Cohesion: 0.50
Nodes (7): ParseArtifact, ParserInput, DebugParseArtifact, parse_replay(), parse_replay_debug(), public_parse_artifact(), public_parse_replay()

## Knowledge Gaps
- **74 isolated node(s):** `Vec`, `MinimalWeaponRow`, `MinimalDeathCounter`, `KillClassification`, `FnOnce` (+69 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `success_artifact()` connect `Community 6` to `Community 0`, `Community 1`, `Community 2`, `Community 5`, `Community 8`?**
  _High betweenness centrality (0.281) - this node is a cross-community bridge._
- **Why does `parse_replay_debug()` connect `Community 10` to `Community 0`, `Community 2`, `Community 3`, `Community 4`, `Community 8`?**
  _High betweenness centrality (0.205) - this node is a cross-community bridge._
- **Why does `decode_compact_root()` connect `Community 4` to `Community 0`, `Community 5`, `Community 6`, `Community 7`, `Community 10`?**
  _High betweenness centrality (0.167) - this node is a cross-community bridge._
- **What connects `Vec`, `MinimalWeaponRow`, `MinimalDeathCounter` to the rest of the system?**
  _74 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.09018987341772151 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.09462915601023018 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.11242937853107345 - nodes in this community are weakly interconnected._