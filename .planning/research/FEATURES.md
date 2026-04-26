# Feature Research

**Domain:** Rust OCAP JSON replay parser service for Solid Stats
**Researched:** 2026-04-24
**Confidence:** MEDIUM-HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = v1 is not a credible replacement for the old parser.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Legacy parser parity contract | The old `replays-parser` is the primary behavioral reference for v1. Existing output fields, formulas, game-type filters, skip rules, and aggregate semantics must remain comparable during migration. | HIGH | Build a compatibility comparison harness around old per-player fields: `kills`, `killsFromVehicle`, `vehicleKills`, `teamkills`, `deaths`, `kdRatio`, `killsFromVehicleCoef`, `totalScore`, `killed`, `killers`, `teamkilled`, `teamkillers`, plus global/squad/rotation outputs. |
| OCAP JSON schema ingestion | OCAP recordings contain mission metadata, markers, entities, positions, fired frames, and event arrays. The local corpus confirms 3,938 raw JSON files with stable top-level keys and high-volume event arrays. | HIGH | Parse v1.1-style OCAP JSON keys: `worldName`, `missionName`, `missionAuthor`, `captureDelay`, `endFrame`, `playersCount`, `Markers`, `EditorMarkers`, `entities`, and `events`. Treat schema drift as structured failure or explicit unknown, not panic. |
| Entity normalization | Stats depend on resolving OCAP unit/vehicle entities into stable observed participants and vehicle context. | HIGH | Normalize `unit` entities with `id`, `name`, `group`, `side`, `description`, `isPlayer`; normalize `vehicle` entities with `id`, `name`, `class`, positions. Preserve OCAP entity IDs as source IDs. |
| Connected-player backfill | The old parser adds players from `connected` events when entity data alone is insufficient. | MEDIUM | Keep this behavior for parity. Without it, some replay participants disappear from aggregates. |
| Duplicate-slot player merge | Players can change slots and appear as multiple entities in one replay. The old parser merges same-name entities to avoid overcounting games. | MEDIUM | Merge duplicate observed names within a replay while summing kills/deaths/weapons/vehicles/other-player relationships. Mark this as legacy-compatible, not canonical identity matching. |
| Kill/death/teamkill extraction | This is the core parser value. Server bounty, player stats, and audit all depend on accurate event normalization. | HIGH | Support `killed` events with killer entity, killed entity, weapon, distance, null killer, same-side teamkill, suicide, player killed, and vehicle killed cases. |
| Vehicle kill context | Existing outputs distinguish infantry kills, kills from vehicles, and vehicle kills. | HIGH | Preserve old rule where a kill weapon matching a vehicle name counts as `killsFromVehicle`; killed vehicle events increment `vehicleKills`. Also emit normalized context so this rule can be audited later. |
| Vehicle score metric | User requested GitHub issue #13 during project initialization. The metric uses only kills from vehicles, weighted by attacker vehicle type and killed entity type, then averaged over games where the player had at least one vehicle kill. | MEDIUM | Implement as a derived aggregate from normalized vehicle kill/teamkill events. Teamkill penalties must clamp matrix values below 1 up to 1. Keep source references for each score contribution. |
| Death classification | Old score/KD formulas depend on total deaths and deaths by teamkill. | MEDIUM | Emit `is_dead`, `is_dead_by_teamkill`, `death_kind`, killer/victim source references, and aggregate `deaths.total` / `deaths.byTeamkills`. |
| Observed identity preservation | Project constraints explicitly forbid canonical player matching in the parser. | MEDIUM | Output observed nickname, squad prefix if parsed, SteamID if present in future data, side, group, role/description, and null/unknown states. Do not decide canonical player IDs in Rust v1. |
| Legacy name-normalization compatibility mode | Old parser strips squad prefixes and applies `nameChanges.csv` during aggregation. v1 must compare against old outputs even if canonical identity moves to `server-2`. | MEDIUM | Put this behind a clearly named compatibility aggregation step. New normalized events should retain raw observed names. |
| Current aggregate output fields | Solid Stats needs continuity with existing generated result files. | HIGH | Emit aggregate summaries for all old fields plus trace links back to normalized events: global player stats, per-week stats, squad stats, rotation stats, weapon stats, vehicle stats, killed/killer relationships, teamkill relationships. |
| Game-type selection parity | Old parser processes `sg`, `mace`, and `sm`, excludes `sgs`, applies an `sm` date cutoff, and applies `mace` minimum-player skip behavior. | MEDIUM | v1 should either reproduce these in compatibility mode or emit enough classification metadata for `server-2` to apply them identically. For migration, implement parity in the parser comparison path. |
| Commander-side and winner extraction | New Solid Stats requirements include KS/commander-side data and winner/outcome where present. | MEDIUM | Parse mission messages and `mission_info`/future structured fields into `outcome` with `unknown` when absent. Include confidence/source because older data often only has free-text Russian mission messages. |
| Explicit unknown/null states | Missing winner, missing SteamID, null killer, and absent identity data are normal historical cases. | LOW | Model as explicit enum/null fields, not empty strings. This is required for trustworthy server persistence and audit. |
| Versioned normalized output contract | `server-2` needs a stable contract it can persist, audit, and recalculate from. | MEDIUM | Every artifact should include `parser_contract_version`, parser build/version, source checksum, replay ID/job ID, and generated-at metadata. Pair with JSON Schema for CI validation. |
| Source references for audit | Aggregates must be traceable back to normalized events and source replay fields. | MEDIUM | Every aggregate contribution should carry replay ID, frame, event index, entity IDs, and rule/classification used. This is the main guard against unexplainable stats disputes. |
| CLI local parse mode | Developers and operators need local reproducibility without queue/storage infrastructure. | LOW | `parse <file> --out <path> --contract-version <version>` should produce the same artifact as worker mode after input acquisition. |
| Worker mode with RabbitMQ and S3-compatible storage | The intended integration flow is `server-2` -> RabbitMQ job -> S3 object -> parser -> completed/failed message. | HIGH | Consume jobs containing `job_id`, `replay_id`, `object_key`, `checksum`, and contract version. Download object, verify checksum, parse, publish completed/failed. Use manual ack/nack and bounded prefetch. |
| Structured failures and skips | Parse failures must be machine-readable for retry, DLQ, and operator decisions. | MEDIUM | Include `job_id`, `replay_id`, `object_key`, `filename`, failure stage, error code, message, retryability, and source cause. Preserve legacy skip reasons such as `empty_replay` and `mace_min_players`. |
| Golden corpus regression tests | The project has `~/sg_stats/raw_replays` and old `~/sg_stats/results`; v1 correctness depends on proving old-vs-new compatibility. | HIGH | Select representative fixtures plus full-corpus CI/manual modes. Compare normalized events and legacy aggregate fields separately so intentional contract improvements do not hide parity regressions. |
| Old parser baseline runner | The new Rust parser must be based on the old parser, so migration needs executable comparisons against legacy behavior. | HIGH | Add a harness that runs old `replays-parser` or consumes old saved outputs, records baseline artifacts, and reports exact diffs with tolerances documented per field. |
| Benchmark harness | The stated target is roughly 10x faster than the current parser, and that must be measured. | MEDIUM | Benchmark parse-only, aggregate-only, end-to-end CLI, and worker job throughput. Report files/sec, MB/sec, events/sec, p50/p95/p99 latency, and memory. |
| Structured observability | Parser services need enough telemetry to debug bad files, slow jobs, queue pressure, and data-quality drift. | MEDIUM | Emit JSON logs and metrics for parse duration, replay size, event/entity counts, failure codes, checksum failures, RabbitMQ ack/nack counts, S3 download duration, golden diff counts, and benchmark summaries. |
| Deterministic output ordering | Stable diffs and golden tests require output order to be deterministic. | LOW | Sort events by source index/frame, players by stable key where contract allows, and aggregate lists by documented sort rules. Avoid hash-map iteration order leaking into JSON. |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required for the first parity proof, but valuable because they make Solid Stats more auditable than the legacy batch pipeline.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Dual artifact output: normalized events plus legacy-compatible aggregates | Keeps v1 migration safe while unlocking future recalculation, corrections, and public audit. | HIGH | The old parser jumps from replay JSON to aggregate results. Rust v1 should expose the intermediate normalized event layer as a first-class artifact. |
| Rule-level provenance | Explains why a kill counted as enemy kill, teamkill, vehicle kill, or ignored. | MEDIUM | Include classification rule IDs such as `legacy_same_side_teamkill`, `legacy_vehicle_name_weapon_match`, or `null_killer_death`. |
| Corpus schema profiler | Turns historical OCAP drift into data instead of surprises. | MEDIUM | Scan the 3,938-file corpus and report top-level keys, event variants, entity variants, unknown side values, malformed files, and unsupported event shapes. |
| Contract diff tooling | Makes parser upgrades reviewable by showing old vs new semantic deltas. | MEDIUM | Generate JSON/HTML diff reports by replay, aggregate field, and rule category. This is highly valuable for migration sign-off. |
| Confidence-scored outcome extraction | Winner/commander data is often free-text or absent; confidence protects downstream stats. | MEDIUM | Emit `outcome.status = known/unknown/inferred`, `source = mission_message/mission_info/custom_field`, and raw source text. |
| Bounty input artifact | Lets `server-2` calculate bounty points without reinterpreting replay internals. | LOW | Emit only valid enemy-kill candidates with killer/victim observed identity, frame/time, vehicle context, side context, and exclusion reason for non-awarding kills. |
| Replay quality report | Operators can see which replays are malformed, empty, low-player, missing winner, missing SteamID, or schema-drifted. | MEDIUM | Useful for migration and future correction workflows. Keep it diagnostic, not user-facing moderation. |
| Full-corpus benchmark dashboard artifact | Makes the 10x target evidence-based and repeatable. | MEDIUM | Store benchmark JSON with old baseline version, new parser version, CPU, corpus sample, and throughput metrics. |
| Idempotent worker artifact storage | Worker can safely retry and dedupe by checksum/contract version. | MEDIUM | Store parse artifacts under deterministic keys such as `parse-artifacts/{contract_version}/{replay_id}/{checksum}.json`. |
| Replay subset selectors | Speeds debugging and migration validation. | LOW | CLI flags for one file, replay IDs, date range, game type, changed since baseline, failures only. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Canonical player identity matching in parser | It would make aggregates look complete in one service. | Project constraints put canonical identity in `server-2`; old replay names, SteamIDs, nicknames, and real players are many-to-many. Parser guesses would become hard-to-debug business logic. | Preserve observed identity and optionally emit legacy compatibility IDs only for old-output comparison. |
| Direct PostgreSQL writes | It feels simpler than emitting artifacts and messages. | Violates service boundary; makes parser hard to test, replay, and audit. | Emit versioned artifacts and completed/failed messages; let `server-2` persist business tables. |
| Public stats/business scoring in parser | Parser already calculates some legacy scores, so it is tempting to keep all stats logic here. | Bounty and public business rules will evolve faster than replay parsing. Mixing them causes parser redeploys for product changes. | Parser emits raw normalized events, parity aggregates, and bounty inputs; `server-2` owns final public persistence and points. |
| Support every replay format in v1 | Future-proofing sounds attractive. | v1 value is OCAP JSON parity and migration; broad format support delays correctness. | Target OCAP JSON only; make parser internals modular enough to add formats later. |
| Real-time streaming of every replay event | Looks modern and could power live dashboards. | OCAP files are batch artifacts, and the product needs deterministic stats, not live playback. | Batch parse one replay into a deterministic artifact; publish one completed/failed message per job. |
| Storing full position timelines in primary stats artifacts | Positions exist in OCAP and are useful for playback. | Huge payloads, slower contracts, and mostly irrelevant to current Solid Stats outputs. | Store source references and selected context. Defer full trajectory export unless a specific v2 feature needs it. |
| Free-text winner as authoritative outcome | Older mission messages often contain localized natural language. | Misclassification could corrupt commander/winner stats. | Emit inferred outcome with confidence and raw source; leave unknown when confidence is low. |
| Manual correction/editing in parser | Operators may want to fix bad stats close to parsing. | Parser should not become a moderation workflow or mutable stats editor. | Emit explainable artifacts; corrections belong in `server-2` with parser re-run/recalculate support. |
| Exact legacy output as the only contract | Eases migration. | Freezes old limitations and blocks normalized audit/event needs. | Provide a legacy compatibility projection generated from a richer normalized contract. |
| Auto-ack RabbitMQ jobs before artifact durability | It can simplify worker code. | Worker crashes can silently lose parse jobs. | Ack only after artifact/result publication succeeds; nack/requeue or fail/DLQ based on structured error policy. |

## Feature Dependencies

```text
Old parser behavioral inventory
    --requires-> Legacy parity contract
                       --requires-> Old-vs-new comparison harness
                                          --requires-> Golden corpus regression tests

OCAP JSON schema ingestion
    --requires-> Entity normalization
                       --requires-> Connected-player backfill
                       --requires-> Duplicate-slot player merge
                       --requires-> Kill/death/teamkill extraction
                                          --requires-> Vehicle kill context
                                          --requires-> Death classification
                                          --requires-> Bounty input artifact

Normalized event artifact
    --requires-> Versioned output contract
                       --requires-> Source references for audit
                                          --enhances-> Rule-level provenance
                                          --enhances-> Contract diff tooling

CLI local parse mode
    --requires-> OCAP JSON schema ingestion
    --enhances-> Golden corpus regression tests
    --enhances-> Benchmark harness

Worker mode with RabbitMQ and S3-compatible storage
    --requires-> Versioned output contract
    --requires-> Structured failures and skips
    --requires-> Structured observability
    --requires-> Idempotent worker artifact storage

Current aggregate output fields
    --requires-> Legacy name-normalization compatibility mode
    --requires-> Game-type selection parity
    --requires-> Kill/death/teamkill extraction

Commander-side and winner extraction
    --requires-> Explicit unknown/null states
    --enhances-> Confidence-scored outcome extraction

Canonical player identity matching
    --conflicts-> Observed identity preservation
```

### Dependency Notes

- **Old parser behavioral inventory requires legacy parity contract:** v1 must first define what the TypeScript parser currently does, including odd rules, before deciding what Rust output must preserve or intentionally improve.
- **Legacy parity requires old-vs-new comparison harness:** parity is not a claim; it must be a repeatable diff over old outputs and selected raw replay fixtures.
- **Entity normalization gates kill extraction:** kill events reference entity IDs, so player/vehicle maps must exist before classification.
- **Normalized events should precede aggregate outputs:** aggregate fields are projections; traceability is much harder if aggregates are calculated without source event IDs.
- **Worker mode depends on the same parse core as CLI:** the only difference should be input/output transport, not parsing behavior.
- **Canonical identity conflicts with observed identity preservation:** parser may emit observed name/prefix and compatibility IDs, but final identity resolution belongs to `server-2`.

## MVP Definition

### Launch With (v1)

Minimum viable product for migration and service integration.

- [ ] Legacy parser behavioral inventory and parity matrix - required because old `replays-parser` is the primary v1 reference.
- [ ] OCAP JSON parser for the historical corpus - required to parse existing `~/sg_stats/raw_replays`.
- [ ] Normalized replay metadata, entity observations, kill/death/teamkill events, vehicle context, and explicit unknowns - required for audit and server persistence.
- [ ] Legacy-compatible aggregate projection - required for old output fields and migration confidence.
- [ ] Vehicle score metric from GitHub issue #13 - required by the current project brief update and depends on audited vehicle kill context.
- [ ] Versioned JSON output contract with source references - required for `server-2` integration and recalculation.
- [ ] CLI parse mode - required for reproducible local debugging and golden tests.
- [ ] Golden corpus tests and old-vs-new diff harness - required to prove parity and detect regressions.
- [ ] Benchmark harness against old parser baseline - required to validate the 10x performance goal.
- [ ] Structured parse failures/skips - required for both CLI and worker reliability.
- [ ] RabbitMQ/S3 worker mode with checksum verification and ack-after-success - required for the planned production integration flow.
- [ ] Structured logs and baseline metrics - required to operate worker mode and diagnose parser drift.

### Add After Validation (v1.x)

Features to add once parity and service integration are stable.

- [ ] Contract diff reports with per-field tolerances - add after initial old-vs-new harness produces stable machine-readable diffs.
- [ ] Corpus schema profiler - add when unsupported shapes or unknown fields start affecting implementation confidence.
- [ ] Confidence-scored commander/winner inference rules - add after representative winner/KS examples are identified in old and new replays.
- [ ] Idempotent artifact cache keyed by replay checksum and contract version - add when worker retry volume or full-corpus reprocessing cost justifies it.
- [ ] Replay subset selectors and failure-only reruns - add as soon as migration debugging becomes repetitive.
- [ ] Replay quality report - add after first full-corpus parse to prioritize bad-data cleanup.

### Future Consideration (v2+)

Features to defer until v1 is trusted.

- [ ] Full trajectory/position export - defer until a concrete analytics or audit feature needs it.
- [ ] Additional replay formats - defer until OCAP JSON parity and Solid Stats migration are complete.
- [ ] Streaming event API - defer because current workload is deterministic batch parsing.
- [ ] Advanced anomaly detection - defer until normalized events and baseline aggregates are stable.
- [ ] Parser-owned correction workflow - likely should stay out of parser; revisit only if `server-2` needs parser-specific recalculation hooks.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Legacy parser parity contract | HIGH | HIGH | P1 |
| OCAP JSON schema ingestion | HIGH | HIGH | P1 |
| Entity normalization | HIGH | HIGH | P1 |
| Kill/death/teamkill extraction | HIGH | HIGH | P1 |
| Vehicle kill context | HIGH | HIGH | P1 |
| Vehicle score metric | MEDIUM | MEDIUM | P1 |
| Legacy-compatible aggregate projection | HIGH | HIGH | P1 |
| Versioned output contract | HIGH | MEDIUM | P1 |
| Source references for audit | HIGH | MEDIUM | P1 |
| CLI local parse mode | HIGH | LOW | P1 |
| Golden corpus regression tests | HIGH | HIGH | P1 |
| Old-vs-new comparison harness | HIGH | HIGH | P1 |
| Benchmark harness | HIGH | MEDIUM | P1 |
| Structured failures and skips | HIGH | MEDIUM | P1 |
| RabbitMQ/S3 worker mode | HIGH | HIGH | P1 |
| Structured observability | MEDIUM | MEDIUM | P1 |
| Commander-side and winner extraction | MEDIUM | MEDIUM | P2 |
| Bounty input artifact | MEDIUM | LOW | P2 |
| Contract diff tooling | MEDIUM | MEDIUM | P2 |
| Corpus schema profiler | MEDIUM | MEDIUM | P2 |
| Idempotent worker artifact storage | MEDIUM | MEDIUM | P2 |
| Replay quality report | MEDIUM | MEDIUM | P2 |
| Full trajectory export | LOW | HIGH | P3 |
| Additional replay formats | LOW | HIGH | P3 |
| Streaming event API | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor / Reference Feature Analysis

| Feature | Legacy `replays-parser` | OCAP ecosystem | Our Approach |
|---------|--------------------------|----------------|--------------|
| Replay ingestion | Reads local raw OCAP JSON from `~/sg_stats/raw_replays`. | OCAP records units, vehicles, events, markers, projectiles, mines/explosives, mission messages. | Parse local file and S3 object inputs through one Rust core; preserve OCAP source IDs and unsupported source fields where useful. |
| Event normalization | Directly calculates per-player results from `killed` events and connected players. | OCAP event list includes connect/disconnect, kills/deaths, hits/injuries, custom events, and ending descriptions. | Emit first-class normalized event artifacts, then derive legacy-compatible aggregates from them. |
| Vehicle context | Counts kills from vehicles via legacy weapon-name-to-vehicle-name matching and counts killed vehicles. | OCAP tracks vehicle entities and crewing over time. | Preserve legacy count behavior for parity, but include source references so better vehicle attribution can be researched later. |
| Output contract | Publishes generated JSON folders for global, squad, rotation, weekly, weapons, vehicles, and other-player stats. | OCAP is primarily playback/AAR, not Solid Stats aggregate output. | Provide versioned parser artifact plus compatibility aggregate projection for migration. |
| Worker integration | Local batch worker-thread pool, not RabbitMQ/S3 service. | OCAP web/server tooling saves and serves recordings. | Add RabbitMQ/S3 worker mode as service integration while keeping CLI parity path. |
| Validation | Jest unit tests and existing generated results. | OCAP docs define JSON format examples but not SolidGames stats parity. | Use golden corpus plus old parser baseline as authoritative behavioral tests. |
| Observability | Pino logs and progress logging. | OCAP has debug logging options for recording. | Structured logs, metrics, and failure codes suitable for container workers and comparison runs. |

## Sources

- Project context: `/home/afgan0r/Projects/SolidGames/replay-parser-2/.planning/PROJECT.md` (HIGH)
- New project brief: `/home/afgan0r/Projects/SolidGames/replay-parser-2/gsd-briefs/replay-parser-2.md` (HIGH)
- Legacy parser project context: `/home/afgan0r/Projects/SolidGames/replays-parser/.planning/PROJECT.md` (HIGH)
- Legacy architecture reference: `/home/afgan0r/Projects/SolidGames/replays-parser/docs/architecture.md` (HIGH)
- Legacy parser type contracts: `/home/afgan0r/Projects/SolidGames/replays-parser/src/0 - types/*.d.ts` (HIGH)
- Legacy parser behavior modules: `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/*`, `/home/afgan0r/Projects/SolidGames/replays-parser/src/3 - statistics/*`, `/home/afgan0r/Projects/SolidGames/replays-parser/src/4 - output/*` (HIGH)
- Local corpus sampling: `~/sg_stats/raw_replays`, `~/sg_stats/results`, `~/sg_stats/lists/replaysList.json` (HIGH). Observed 3,938 raw replay files; sampled/full-corpus event types included `killed`, `disconnected`, `connected`, `mine_exp`, `admin-menu`, `mission_message`, and `mission_info`.
- OCAP README and feature overview: https://github.com/OCAP2/OCAP (MEDIUM-HIGH)
- OCAP JSON Recording Format wiki: https://github.com/OCAP2/OCAP/wiki/JSON-Recording-Format (MEDIUM-HIGH)
- OCAP Custom Game Events wiki: https://github.com/OCAP2/OCAP/wiki/Custom-Game-Events (MEDIUM)
- RabbitMQ Consumers and acknowledgement/prefetch docs: https://www.rabbitmq.com/docs/consumers and https://www.rabbitmq.com/docs/consumer-prefetch (HIGH)
- RabbitMQ negative acknowledgements and DLX docs: https://www.rabbitmq.com/docs/nack and https://www.rabbitmq.com/docs/dlx (HIGH)
- Amazon S3 object integrity docs: https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html (HIGH)
- JSON Schema docs/specification: https://json-schema.org/docs and https://json-schema.org/specification (HIGH)
- Criterion.rs docs for Rust benchmarking: https://docs.rs/criterion and https://criterion-rs.github.io/book/user_guide/command_line_output.html (HIGH)
- Insta snapshot testing docs for golden/snapshot workflows: https://insta.rs/ and https://insta.rs/docs/redactions/ (HIGH)
- OpenTelemetry RabbitMQ semantic conventions and Rust docs: https://opentelemetry.io/docs/specs/semconv/messaging/rabbitmq/ and https://opentelemetry.io/docs/languages/rust/ (MEDIUM)

---
*Feature research for: Rust OCAP JSON replay parser service*
*Researched: 2026-04-24*
