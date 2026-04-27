# Phase 4: Event Semantics and Aggregates - Context

**Gathered:** 2026-04-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 4 normalizes combat, commander-side, and outcome semantics on top of the
Phase 3 parser-core metadata/entity foundation, then derives auditable
per-replay aggregate contributions and projections for legacy fields, bounty
inputs, and vehicle score from GitHub issue #13.

This phase does not implement CLI commands, full-corpus parity commands,
benchmarks, RabbitMQ/S3 worker behavior, PostgreSQL persistence, public APIs,
public UI, canonical identity matching, replay discovery, annual/yearly
nomination statistics, or final cross-replay `server-2` calculation storage.

</domain>

<decisions>
## Implementation Decisions

### Combat Event Classification
- **D-01:** Model each source `killed` tuple as one primary source-backed
  normalized event with a dominant semantic classification and typed/structured
  attributes. Aggregate contributions expose the derived counter effects rather
  than duplicating the same source tuple into multiple independent events.
- **D-02:** Preserve legacy counter semantics for edge cases: null-killer player
  deaths mark the victim dead without killer counters; suicide marks death but
  not teamkill; same-side non-suicide kills count as teamkills and not normal
  kills.
- **D-03:** If actor/entity lookup is incomplete or victim type is unclear,
  preserve a source-backed unknown/partial event plus diagnostics. Do not emit
  aggregate contributions unless enough evidence exists to audit the
  classification.
- **D-04:** Valid enemy kills emit bounty-eligible contribution data. Teamkills,
  suicides, null-killer deaths, and unknown actor cases remain auditable events
  with exclusion reasons and must not award bounty inputs.

### Aggregate Artifact Shape
- **D-05:** One parse artifact should emit per-replay aggregate contributions
  and per-replay projections. Full multi-replay global, weekly, squad, and
  rotation outputs are derived later by `server-2` or the Phase 5 parity
  harness.
- **D-06:** Use namespaced aggregate projection keys such as
  `legacy.player_game_results`, `legacy.relationships`, `bounty.inputs`, and
  `vehicle_score.inputs`. Old field names can appear inside the legacy
  compatibility projection, not as unscoped artifact-wide fields.
- **D-07:** No silent counters: every aggregate counter/projection value must be
  derived from traceable contributions with event IDs, source refs, and rule
  IDs. Incomplete traceability should produce diagnostics/partial status rather
  than unauditable counters.
- **D-08:** Use dual identity in aggregate output: contributions carry observed
  entity IDs and observed identity facts, while legacy projections may apply
  Phase 3 same-name compatibility hints with explicit rule/source provenance.
  Do not introduce canonical player IDs in the parser.

### Vehicle Score Evidence
- **D-09:** Represent vehicle score taxonomy as raw observed evidence plus a
  mapped issue #13 category. Preserve raw vehicle/entity class/name evidence
  alongside normalized categories such as static weapon, car, truck, APC, tank,
  heli, plane, and player.
- **D-10:** Emit vehicle score as per-event contributions, not only totals. Each
  eligible vehicle kill or vehicle-teamkill penalty contribution must carry the
  player, target category, raw evidence, weight, sign, rule ID, and source refs.
- **D-11:** Treat the denominator as an aggregate input: parser emits per-player
  per-replay denominator eligibility for games where the player has at least one
  kill from a vehicle. `server-2` and the parity harness compute cross-replay
  final scores from numerator contributions and denominator inputs.
- **D-12:** Record the teamkill penalty clamp at contribution level. Penalty
  contributions should include the raw matrix weight and the applied penalty
  weight after `max(matrix_weight, 1)` so issue #13 scores can be recalculated
  and audited.

### Commander and Outcome Confidence
- **D-13:** Commander-side extraction may emit explicit observed facts and
  inferred candidates. Heuristic commander candidates must be labeled as
  inferred/candidate facts with confidence, rule ID, and source refs; they are
  not canonical truth.
- **D-14:** Winner/outcome should be present only when reliable source evidence
  exists. Older or ambiguous replays should emit explicit unknown outcome; manual
  or final winner correction remains `server-2` responsibility.
- **D-15:** Add or plan a typed parser-contract section for replay-side
  commander/outcome facts. Do not make `extensions` the primary integration path
  for data that `server-2` is expected to consume.
- **D-16:** Expected missing commander/winner data does not make an artifact
  partial by itself. Conflicting, ambiguous, or lossy source evidence should
  produce diagnostics and may make the artifact partial when auditability is
  materially degraded.

### the agent's Discretion
- Exact Rust module/file names, helper boundaries, diagnostic code strings, and
  test fixture layout are planner discretion if they follow existing workspace
  style and strict quality gates.
- Exact JSON payload details inside typed combat/aggregate/vehicle score fields
  are planner discretion, provided the decisions above remain machine-checkable,
  deterministic, and auditable.
- Exact confidence scale thresholds for commander candidates are planner
  discretion, but every inferred candidate needs source refs and stable rule IDs.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project and Phase Scope
- `.planning/PROJECT.md` - Current project scope, parser ownership, issue #13
  matrix, old-parser reference, and cross-app boundary rules.
- `.planning/REQUIREMENTS.md` - Phase 4 requirements `PARS-08` through
  `PARS-11` and `AGG-01` through `AGG-11`.
- `.planning/ROADMAP.md` - Phase 4 goal, dependencies, and success criteria.
- `.planning/STATE.md` - Current focus and accumulated decisions from completed
  phases.
- `.planning/research/SUMMARY.md` - Research rationale for normalized events as
  primary artifact and aggregate projection as a derived layer.
- `README.md` - Current repository status, architecture direction, validation
  data, and AI/GSD workflow expectations.

### Prior Phase Decisions
- `.planning/phases/03-deterministic-parser-core/03-CONTEXT.md` - Parser-core
  boundary, raw adapter policy, deterministic output, observed entity facts,
  connected-player backfill, and same-name compatibility hints.
- `.planning/phases/03-deterministic-parser-core/03-05-SUMMARY.md` - Implemented
  Phase 3 compatibility hooks and handoff to event/aggregate work.
- `.planning/phases/02-versioned-output-contract/02-CONTEXT.md` - Contract
  envelope, presence semantics, source refs, rule IDs, diagnostics, and
  aggregate contribution decisions.
- `.planning/phases/02-versioned-output-contract/02-VERIFICATION.md` - Verified
  contract invariants and schema/source-reference behavior.
- `.planning/phases/01-legacy-baseline-and-corpus/01-CONTEXT.md` - Legacy
  baseline decisions, observed identity boundary, and yearly nomination v2
  deferral.
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md`
  - Legacy game-type filters, skip rules, identity compatibility, ordinary
  output surfaces, and comparable fields.
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md`
  - Required mismatch categories and parser/server/UI impact dimensions.
- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` - Corpus
  shape, event/entity samples, malformed files, and fixture-selection evidence.
- `.planning/phases/01-legacy-baseline-and-corpus/fixture-index.json` - Seed
  fixture list and cross-app relevance notes for Phase 5 expansion.

### Contract and Parser-Core Code
- `crates/parser-contract/src/events.rs` - Current normalized event kind,
  actor ref, source refs, rule ID, and attributes skeleton.
- `crates/parser-contract/src/aggregates.rs` - Current aggregate contribution
  kinds, contribution refs, and projection map skeleton.
- `crates/parser-contract/src/source_ref.rs` - Source reference, checksum, and
  rule ID invariants.
- `crates/parser-contract/src/identity.rs` - Observed entity/identity fields
  and Phase 3 compatibility hint contract.
- `crates/parser-contract/src/artifact.rs` - Parse artifact envelope, status,
  events, aggregates, diagnostics, and failure placement.
- `crates/parser-core/src/artifact.rs` - Current artifact construction and empty
  events/aggregates handoff point.
- `crates/parser-core/src/raw.rs` - Raw OCAP adapter pattern and existing
  connected-event tuple handling.
- `crates/parser-core/src/entities.rs` - Observed entity normalization,
  connected-player backfill, same-name hints, and data-loss diagnostic policy.

### Legacy Parser Semantics
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/2 - parseReplayInfo/getKillsAndDeaths.ts`
  - Legacy killed tuple handling, null killer deaths, same-side teamkills,
  suicides, kills from vehicle, killed vehicle counter, and relationships.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/2 - parseReplayInfo/getEntities.ts`
  - Legacy player/vehicle extraction and connected-player backfill.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/2 - parseReplayInfo/combineSamePlayersInfo.ts`
  - Legacy duplicate same-name aggregate merge behavior.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/0 - types/replay.d.ts`
  - Legacy replay event/entity/player type shapes.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/0 - utils/calculateScore.ts`
  - Legacy score formula.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/0 - utils/calculateKDRatio.ts`
  - Legacy KD formula.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/0 - utils/calculateVehicleKillsCoef.ts`
  - Legacy vehicle kill coefficient formula.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/3 - statistics/global/add.ts`
  - Legacy global per-player aggregation and `excludePlayers` compatibility
  boundary.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/3 - statistics/global/addToResultsByWeek.ts`
  - Legacy weekly aggregation formula.
- `/home/alexandr/Projects/SolidGames/sg-replay-parser/src/3 - statistics/consts/index.ts`
  - Legacy default aggregate fields.

### Cross-Application Boundaries
- `gsd-briefs/replay-parser-2.md` - Parser-owned commander, bounty, identity,
  and artifact responsibility.
- `gsd-briefs/server-2.md` - Backend ownership of canonical identity,
  persistence, recalculation, commander-side stats, manual winner correction,
  and public API shapes.
- `gsd-briefs/web.md` - Public stats UI expectations for player, squad,
  rotation, commander, and bounty data through `server-2` APIs.
- `gsd-briefs/replays-fetcher.md` - Replay discovery/raw object ownership
  boundary; parser must not absorb ingestion concerns.

### External Requirement
- `https://github.com/solid-stats/sg-replay-parser/issues/13` - Vehicle score
  formula, weight matrix, denominator rule, and teamkill penalty clamp
  requirement.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `NormalizedEvent`, `NormalizedEventKind`, `EventActorRef`, and event
  `attributes` already provide the initial event skeleton for Phase 4.
- `AggregateContributionRef`, `AggregateContributionKind`, and
  `AggregateSection` already provide the source-ref-backed contribution model.
- `SourceRef`, `SourceRefs`, and `RuleId` enforce non-empty evidence and stable
  rule provenance for events and aggregate contributions.
- `ObservedEntity`, `ObservedIdentity`, and `EntityCompatibilityHint` already
  preserve observed identity and Phase 3 compatibility hints without canonical
  player matching.
- `RawReplay` helper functions and `SourceContext` establish the pattern for
  adding tolerant `killed` event tuple accessors and source references.

### Established Patterns
- Parser-core is pure and transport-free; adapters own file access, queues,
  object storage, timestamps, and databases.
- Optional and inferred facts use `FieldPresence<T>` with explicit unknown,
  null, inferred, and source metadata states.
- Deterministic output uses stable vectors/maps and avoids wall-clock values in
  parser-core output.
- Localized schema drift should produce diagnostics and partial status only when
  source evidence is lost, conflicting, or not auditable.
- Legacy compatibility can be represented as named hints/projections, but raw
  observed facts must remain available.

### Integration Points
- `crates/parser-core/src/artifact.rs` currently emits empty `events` and a
  default `AggregateSection`; Phase 4 should fill these from normalized combat
  and outcome semantics.
- `crates/parser-core/src/raw.rs` should gain tolerant accessors for source
  `killed` tuples, preserving event index, frame, killer/killed IDs, weapon, and
  distance where present.
- `crates/parser-contract` likely needs typed additions for combat attributes,
  vehicle score taxonomy/contributions, and commander/outcome facts before
  Phase 4 can satisfy `server-2` integration needs cleanly.
- Phase 5 will build old-vs-new comparison and broader fixture coverage from
  the event/contribution/projection shapes implemented here.
- Any parser artifact shape change that affects `server-2` storage or `web`
  public stats should be checked against adjacent app docs/repos during
  planning, not treated as parser-local only.

</code_context>

<specifics>
## Specific Ideas

- Treat the old parser's `getKillsAndDeaths.ts` as the first concrete oracle for
  legacy counter behavior, but label unsupported or ambiguous source cases with
  diagnostics rather than silently dropping audit evidence.
- Keep old field names such as `killsFromVehicle`, `vehicleKills`,
  `teamkills`, `kdRatio`, `score`, and relationship lists inside namespaced
  legacy projections.
- Vehicle score should be reconstructable from artifact data without trusting a
  final opaque score field.
- Commander candidates can be useful, but final winner correction and canonical
  commander/player identity remain outside the parser.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within Phase 4 scope.

</deferred>

---

*Phase: 04-event-semantics-and-aggregates*
*Context gathered: 2026-04-27*
