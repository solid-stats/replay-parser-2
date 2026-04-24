# Pitfalls Research

**Domain:** Rust OCAP JSON replay parser service replacing an old parser with deterministic, versioned output
**Researched:** 2026-04-24
**Confidence:** HIGH for parser/contract/integration pitfalls and old-parser compatibility risks; MEDIUM for exact OCAP edge-case semantics until the full historical corpus is characterized.

## Phase Vocabulary Used Below

No roadmap exists yet, so this research maps pitfalls to recommended roadmap phases:

1. **Corpus Baseline and Legacy Reference** - inventory `~/sg_stats`, preserve representative OCAP samples, identify the old `replays-parser` command baseline, audit legacy assumptions, capture old-parser outputs, and define mismatch taxonomy.
2. **Output Contract and Compatibility Semantics** - design versioned JSON schema, source references, unknown/null states, aggregate traceability, and explicit compatibility mapping for old output fields.
3. **Deterministic Parser Core** - implement typed Rust parser for metadata, entities, event tuples, and stable output ordering.
4. **Event Semantics and Aggregates** - derive kills/deaths/teamkills, vehicle context, commander-side data, winner/outcome, old-compatible aggregate fields, and new Solid Stats aggregate inputs.
5. **Golden Validation, Migration Parity, and Benchmarks** - compare old-vs-new outputs across corpus slices, measure speed/memory against the old parser executable baseline, and publish parity reports.
6. **Worker Integration and Artifacts** - RabbitMQ consumer/publisher, S3-compatible reads/writes, idempotency, checksums, structured failures.
7. **Operational Hardening** - resource limits, observability, malformed replay handling, container health, production readiness.

## Old Parser Compatibility Requirements

The old parser lives at `/home/afgan0r/Projects/SolidGames/replays-parser` and is a required behavioral reference, not just an optional historical artifact. The new Rust parser must be based on it enough to preserve compatible behavior where compatibility is intentional and to document every intentional break.

Key old-parser facts discovered locally:

- Runtime command baseline is exposed through `package.json`: `pnpm run parse` (`tsx src/start.ts`) and `pnpm run parse:dist` (`node dist/start.js`).
- Runtime storage is file-backed under `~/sg_stats`: `raw_replays`, `lists/replaysList.json`, `results`, `temp_results`, `config/nameChanges.csv`, and logs.
- Main flow is `src/index.ts`: prepare folders, load name-change data, select replays, parse with a worker pool, aggregate global/rotation stats, then atomically replace `~/sg_stats/results`.
- Core replay parsing is in `src/2 - parseReplayInfo/*`: `getEntities.ts`, `getKillsAndDeaths.ts`, `combineSamePlayersInfo.ts`.
- Legacy aggregate fields include `kills`, `killsFromVehicle`, `vehicleKills`, `teamkills`, `deaths`, `kdRatio`, `killsFromVehicleCoef`, `score`/`totalScore`, `totalPlayedGames`, weekly stats, weapon stats, and other-player relationship lists.
- Legacy behavior includes risk-bearing assumptions: player entities require `type === 'unit'`, `isPlayer`, non-empty `description`, and name; `connected` events can create players; same-name entities are merged; name-change CSV can map normalized names to stable IDs; `mace` replays with fewer than 10 parsed players are skipped; `sm` replays are filtered after January 2023; KD/score subtract teamkills and teamkill deaths in specific ways.

Roadmap implication: Phase 1 must produce an "old parser behavior dossier" before Phase 2 finalizes the new contract. Phase 5 must run the old parser executable and new Rust parser on the same fixture set and report field-by-field parity.

## Critical Pitfalls

### Pitfall 1: Treating the old `replays-parser` as either absolute truth or disposable legacy

**What goes wrong:**
The new parser either copies historical bugs forever or silently changes public statistics without an audit trail. Because the old parser is now an explicit required behavioral reference, ignoring its executable command, source assumptions, and output fields would also make migration parity impossible to prove.

**Why it happens:**
Legacy replacement projects often start from code opinions instead of a corpus oracle. This project has around 3,938 historical OCAP JSON files, existing result JSON, and a live old parser project. The old parser is a behavioral reference and a risk source: it contains compatibility-critical output semantics plus legacy assumptions such as same-name entity merging, name-change ID lookup, game-type filtering, and skip rules.

**How to avoid:**
Build a corpus and legacy-behavior manifest first. Record the old parser repo commit, command (`pnpm run parse` or `pnpm run parse:dist`), Node/package-manager requirements, required `~/sg_stats` inputs, environment assumptions, and output locations. Classify fixtures by mission date, world, file size, event count, known winner presence/absence, SteamID presence/absence, vehicle-heavy rounds, old-parser skip behavior, and malformed/partial cases. For old-vs-new comparisons, require a mismatch taxonomy: `compatible-match`, `intentional-new-contract-change`, `old-parser-bug-preserved-for-compat`, `old-parser-bug-fixed`, `new-parser-bug`, `insufficient-source-data`, and `needs-human-review`. Store fixture metadata, old command logs, output hashes, and comparison summaries as test artifacts.

**Warning signs:**
Only one or two hand-picked replays are tested; old parser source is not read during planning; old parser command cannot be run repeatably; golden tests assert aggregate totals only; mismatches are resolved by changing code without recording why; benchmark samples are not the same files used for correctness comparisons.

**Phase to address:**
Phase 1 prevents this by creating the corpus manifest, old-parser behavior dossier, command baseline, and comparison taxonomy. Phase 5 verifies migration parity across broader corpus slices before worker integration.

---

### Pitfall 2: Designing the contract around aggregates instead of normalized source events

**What goes wrong:**
The parser can output `kills`, `deaths`, or `score`, but `server-2` cannot audit, recalculate, explain corrections, or trace bounty inputs back to raw replay evidence.

**Why it happens:**
Aggregate parity is tempting because old Solid Stats results are aggregate-heavy. The project requirement is broader: deterministic normalized raw events plus aggregate summaries and source references.

**How to avoid:**
Make normalized events the primary artifact and keep old-compatible aggregates as derived projections. Every aggregate row must cite event IDs/source references: replay ID, source top-level section, event index or entity ID, frame, observed actor IDs, and derivation rule version. Aggregates should be reproducible from the normalized artifact without reparsing the original OCAP JSON. For each legacy output field, document whether it is preserved exactly, preserved with a named compatibility rule, replaced by a new field, or intentionally dropped.

**Warning signs:**
Aggregate structs are implemented before normalized event structs; source file positions or event indices are absent; bounty inputs require re-reading `events` or `entities`; tests compare only final totals; old output fields such as `killsFromVehicleCoef`, `kdRatio`, or other-player relationship lists are not mapped.

**Phase to address:**
Phase 2 prevents this by locking the artifact model before parser implementation. Phase 4 verifies aggregate traceability from normalized events.

---

### Pitfall 3: Collapsing observed identity into canonical player identity

**What goes wrong:**
Stats become wrongly merged or split when players change nicknames, share names, lack SteamIDs, use multiple accounts, or appear as AI/vehicle entities. The parser would leak `server-2` identity policy into an irreversible raw artifact.

**Why it happens:**
Replay events expose names, entity IDs, side, group, `isPlayer`, and sometimes UID-like fields in newer OCAP models, but canonical player matching is many-to-many and explicitly out of parser scope. The old parser also performs compatibility-oriented name normalization and ID lookup through `nameChanges.csv`; that behavior is useful for parity tests but dangerous if copied into the new normalized raw-event contract as truth.

**How to avoid:**
Emit observed identities only in normalized parser output. Use explicit fields such as `observed_name`, `observed_steam_id`, `entity_id`, `entity_type`, `side`, `group`, `is_player`, and `identity_confidence`. Never emit `canonical_player_id`. Preserve unknown SteamID and missing identity as typed null/unknown states, not empty strings. If an old-compatible aggregate projection needs legacy name-change behavior, isolate it behind a compatibility derivation rule and source it from the old-parser behavior dossier.

**Warning signs:**
Parser structs contain `player_id` with server semantics; nickname is used as a map key for normalized events; missing SteamID becomes `""`, `0`, or `"unknown"` without a typed state; legacy `combineSamePlayersInfo` behavior is copied into raw parsing instead of a compatibility projection.

**Phase to address:**
Phase 2 prevents this in the contract. Phase 3 enforces it in parser data structures. Phase 4 verifies aggregate grouping uses observed identities only.

---

### Pitfall 4: Assuming Rust output is deterministic by default

**What goes wrong:**
The same input replay produces different JSON byte output or event ordering between runs, platforms, thread counts, or build versions. Golden tests flake and `server-2` cannot trust artifact checksums.

**Why it happens:**
Rust `HashMap` iteration order is arbitrary and randomly seeded. JSON objects are unordered by spec. Rayon preserves data-race freedom, but side effects and unordered iterators can still produce different output order. Parallel reductions can also reorder floating point and collection merges.

**How to avoid:**
Define deterministic order in the contract. Sort events by `(frame, source_index, normalized_event_id)`. Sort maps before serialization or use ordered structs/BTreeMap only at serialization boundaries. Keep parser concurrency isolated to per-file or indexed chunks whose results are reassembled in source order. Use stable numeric formatting and avoid floating point accumulation for public aggregate counters.

**Warning signs:**
Golden snapshots change between runs; tests sort expected data after parsing but production output is unsorted; code serializes `HashMap` directly; parallel code sends parsed events through an unordered channel into the output vector.

**Phase to address:**
Phase 2 defines canonical ordering. Phase 3 implements deterministic collection and serialization rules. Phase 5 runs repeatability tests across multiple runs/thread counts.

---

### Pitfall 5: Hard-coding OCAP tuple offsets without a version/shape layer

**What goes wrong:**
Events parse correctly for one sample but break on other mission dates, older recorder behavior, vehicle kills, disconnect deaths, mission messages, or future OCAP recorder changes.

**Why it happens:**
Historical OCAP JSON uses compact arrays for events. In a sampled large replay, `events` contained tuple lengths of 3, 4, and 5 for different event kinds, while entity records had dense `positions` arrays with variable auxiliary fields. OCAP changelogs also show event and tracking semantics changed over time.

**How to avoid:**
Create an input adapter layer that converts raw OCAP shapes into typed internal variants. Pattern-match by event type and tuple length, validate each field, and emit structured parse warnings/failures for unknown shapes. Fixture every known event tuple shape before implementing aggregates.

**Warning signs:**
Code uses `event[4].as_i64().unwrap()` outside a raw adapter; adding one event kind touches aggregate code; parser panics on unknown tuple length; fixtures only cover `killed` events.

**Phase to address:**
Phase 1 inventories shapes. Phase 3 owns raw-to-typed parsing. Phase 7 hardens unknown-shape handling.

---

### Pitfall 6: Misclassifying kills, teamkills, deaths, and vehicle context

**What goes wrong:**
Bounty inputs and public stats are wrong: teamkills counted as enemy kills, vehicle kills attributed to the wrong player/entity, suicide/disconnect/death events counted as valid kills, or victim/killer side resolved from the wrong frame.

**Why it happens:**
Kill tuples can reference entity IDs, weapon/vehicle names, and frames. Entity side, player status, vehicle occupancy, and alive/dead state are temporal. OCAP also distinguishes units, vehicles, projectiles, hits, and mission events in newer models. The old parser has specific compatibility behavior here: it detects kills from vehicles by matching the weapon string against vehicle names, counts killed vehicles separately, subtracts teamkills in KD/score formulas, and uses `isDeadByTeamkill` in death aggregation.

**How to avoid:**
Resolve kill semantics with a temporal entity index keyed by frame. For each kill event, output both raw references and derived classification: `enemy_kill`, `teamkill`, `suicide_or_self`, `unknown_actor`, `vehicle_context`, `classification_confidence`, and `classification_rule_version`. Require tests for infantry kill, vehicle kill, teamkill, disconnected/deleted entity, unknown side, and same-frame side/vehicle changes.

**Warning signs:**
Teamkill is implemented as `killer_name == victim_name`; side is read only from final entity state; vehicle context is a string attached to the weapon only; old formula compatibility is not tested; bounty tests do not include vehicle kills and teamkills.

**Phase to address:**
Phase 4 prevents this with semantic derivation and aggregate tests. Phase 5 validates against old results and hand-reviewed fixtures.

---

### Pitfall 7: Treating absent winner, SteamID, or commander data as negative facts

**What goes wrong:**
Older replays appear to have "no winner", "no commander", or "non-Steam player" when the truth is only "not recorded". This corrupts commander-side stats and downstream filtering.

**Why it happens:**
OCAP export can include a simple ending message, a winning side, and a tag, but not all missions call export with winner data. The project explicitly requires unknown/null states for missing winner and SteamID.

**How to avoid:**
Model optional facts as tri-state or richer enums: `known(value)`, `unknown_not_recorded`, `unknown_unparseable`, and where useful `not_applicable`. Include source/confidence metadata for commander-side and winner extraction. Do not infer a winner from kill totals unless the contract labels it as a heuristic outside v1 truth.

**Warning signs:**
Winner defaults to `false`, `NONE`, `CIV`, or an empty string; tests only cover newer replays with winner metadata; UI/server requirements force parser fields to be non-null before evidence exists.

**Phase to address:**
Phase 2 prevents this in schema design. Phase 4 validates commander/winner extraction and unknown handling.

---

### Pitfall 8: Versioning the contract as a string instead of a compatibility system

**What goes wrong:**
`parser_contract_version` exists but clients cannot know what changed, which fields are stable, or whether stored artifacts can be safely recalculated. Breaking changes slip in as "bug fixes".

**Why it happens:**
Adding a `version` field feels sufficient. It is not: semantic versioning requires a declared public API, and JSON Schema validation exists to assert what an output document must look like.

**How to avoid:**
Publish a JSON Schema per contract version plus a human changelog. Define compatibility rules: major for breaking field/semantic changes, minor for additive backward-compatible fields, patch for compatible bug fixes. Store parser binary version, contract version, source checksum, old-parser reference commit when producing compatibility reports, and derivation rule versions in every artifact. Add contract fixture tests consumed by both parser and `server-2`. Maintain a legacy field map for every old result artifact path and field.

**Warning signs:**
Only `contract_version: "1"` exists; no schema files are committed; old artifacts are overwritten without preserving the version that produced them; `server-2` tests deserialize parser output using permissive maps; old fields disappear from the roadmap as "implementation detail".

**Phase to address:**
Phase 2 prevents this. Phase 6 verifies version negotiation in job messages. Phase 7 adds artifact migration/reprocessing policy if needed.

---

### Pitfall 9: Benchmarking parser speed without measuring the replacement goal

**What goes wrong:**
The project claims 10x speedup but measured a different workload: debug builds, cached tiny files, parser-only microbenchmarks, no old-parser baseline, or no artifact serialization/checksum cost.

**Why it happens:**
Rust makes microbenchmarks easy, but this project's value is end-to-end replay parse throughput against the historical corpus and old parser behavior.

**How to avoid:**
Define benchmark tiers: parser core on fixed files, CLI end-to-end on local files, old parser baseline on the same files, and worker end-to-end with S3/RabbitMQ mocked or localstack-style services. For the old parser, pin whether the baseline is `pnpm run parse` or `pnpm run parse:dist`, the old repo commit, Node version, package manager version, worker count, and `~/sg_stats` inputs. Record file bytes/sec, events/sec, max RSS, output bytes, and comparison result. Use release builds and stable hardware notes. Save named baselines.

**Warning signs:**
Benchmarks use generated JSON instead of OCAP files; results omit memory; only median latency is reported; old parser command is unknown; old parser includes aggregation/output time while Rust measures parser-only time; speed target is declared before comparable fields are correct.

**Phase to address:**
Phase 1 captures old-parser command and corpus sample sets. Phase 5 prevents false speed claims with comparable benchmarks.

---

### Pitfall 10: Loading and cloning dense replay state until memory dominates performance

**What goes wrong:**
A parser that is fast on small samples becomes slow or OOM-prone on the largest corpus files or concurrent worker runs. The 10x goal is missed even though Rust code looks "efficient".

**Why it happens:**
OCAP entity `positions` arrays are dense. In a sampled 18.9 MB replay, 340 entities averaged about 1,316 position samples each, with some matching `endFrame + 1`. Naive `serde_json::Value` traversal, repeated clones, string copies, and all-at-once corpus processing magnify memory pressure.

**How to avoid:**
Use typed deserialization at the raw adapter boundary and borrow where practical. Keep only indexes needed for event semantics and source references. Avoid cloning position arrays into every derived event. Benchmark max RSS and parse several large files concurrently to match worker reality. Consider streaming only after typed all-file parsing proves insufficient; do not prematurely build a custom JSON parser.

**Warning signs:**
Parser passes tests but memory grows with corpus size; flamegraphs show `serde_json::Value` cloning and string allocation; worker concurrency is set before per-file max RSS is known; benchmarks parse one file per process and hide allocator churn.

**Phase to address:**
Phase 3 prevents structural memory waste. Phase 5 quantifies memory and throughput. Phase 7 sets production resource limits.

---

### Pitfall 11: Acknowledging RabbitMQ jobs before the parse artifact is durable

**What goes wrong:**
Jobs are lost, duplicated, or reprocessed indefinitely. `server-2` sees completed messages whose artifact is missing, failed messages that were actually retriable, or duplicate completed artifacts for one replay.

**Why it happens:**
RabbitMQ manual acknowledgements and publisher confirms solve different sides of reliability. Auto-ack is unsafe for work that must complete durably. Immediate requeue on transient failures can create hot redelivery loops. Unbounded prefetch can overload parser memory.

**How to avoid:**
Use manual consumer acknowledgements. Ack only after the parser has written the artifact or failure record and the publish/result path is confirmed. Make jobs idempotent by `(job_id, replay_id, checksum, parser_contract_version)`. Configure bounded prefetch from measured per-file memory. Route permanent parse failures to structured `parse.failed`; retry transient S3/network errors with bounded attempts/backoff/dead-lettering.

**Warning signs:**
`auto_ack=true`; worker publishes result then acks without publisher confirms; failures always `nack(requeue=true)`; duplicate jobs create duplicate artifacts; prefetch is unlimited or copied from a tutorial.

**Phase to address:**
Phase 6 prevents this in worker design. Phase 7 verifies redelivery, duplicate job, and crash-after-artifact scenarios.

---

### Pitfall 12: Trusting S3 object identity without validating checksum and size

**What goes wrong:**
The worker parses the wrong file, a partial/corrupt object, or an unexpectedly huge object. Golden comparisons become meaningless and parser failures are misattributed.

**Why it happens:**
The parse request includes `object_key` and `checksum`, but S3-compatible storage behavior and SDK checksum support vary. ETag is not a universal full-object checksum, especially with multipart uploads.

**How to avoid:**
Treat `checksum` from `server-2` as part of the job identity. Validate downloaded bytes against the expected checksum before parsing. Record object size, checksum algorithm, storage endpoint/bucket, and object key in structured logs and output metadata. Enforce max object size and timeout limits. Do not use object keys as local file paths.

**Warning signs:**
Worker ignores request checksum; checksum failures become generic parse failures; object size is unknown until after parse; artifact metadata lacks source checksum; tests use only local files and never simulate corrupt S3 content.

**Phase to address:**
Phase 6 prevents this. Phase 7 adds corrupt download, missing object, and oversized object tests.

---

### Pitfall 13: Letting parser scope creep into `server-2`

**What goes wrong:**
The parser starts mutating PostgreSQL business tables, applying corrections, matching canonical players, calculating payouts, or encoding web-specific display rules. The service becomes hard to test independently and impossible to replace safely.

**Why it happens:**
Parser output is immediately useful for player stats, bounty points, and public pages, so downstream concerns can leak back into parsing.

**How to avoid:**
Keep parser responsibility limited to normalized observed facts, aggregate summaries, derivation metadata, and structured parse failures. `server-2` owns persistence, canonical identity, corrections, bounty calculation, and publication. Integration tests should assert the boundary: parser output is an artifact, not database mutation.

**Warning signs:**
Parser has PostgreSQL business migrations; parser code imports `server-2` identity models; correction workflows change parser logic; bounty award rules live in parser instead of derived inputs.

**Phase to address:**
Phase 2 prevents this in contract boundaries. Phase 6 preserves it in integration.

---

### Pitfall 14: Treating malformed or hostile replay content as impossible

**What goes wrong:**
One bad replay panics the worker, leaks secrets in logs, exhausts memory, or causes repeated job redelivery. Player/admin text from replays can also pollute logs if emitted raw without structure.

**Why it happens:**
Historical files are trusted test data, but worker mode reads from object storage through a queue. JSON parsers and schemas permit large numbers/strings unless the application sets limits.

**How to avoid:**
Use fallible parsing everywhere. Convert parse errors into structured `parse.failed` with replay/job identifiers and sanitized context. Enforce file size, parse time, nesting/array shape expectations, and output size limits. Escape or structure replay-provided strings in logs. Run fuzz/property tests for malformed tuples and truncated JSON after the core adapter exists.

**Warning signs:**
`unwrap()` or `expect()` appears in parse paths; malformed fixture tests are absent; worker crash is the only failure signal; replay-provided strings are concatenated into log lines.

**Phase to address:**
Phase 3 prevents panics in parser core. Phase 7 hardens resource limits and malformed-input testing.

---

### Pitfall 15: Copying legacy assumptions into the Rust parser without labeling compatibility behavior

**What goes wrong:**
The Rust parser becomes a faster clone of legacy behavior but cannot distinguish observed replay facts from historical Solid Stats policy. Future features such as SteamID-aware identity, commander-side stats, bounty inputs, and correction/recalculation workflows inherit hidden old assumptions.

**Why it happens:**
The old parser mixes several responsibilities in one pipeline: raw replay parsing, entity merging, name-change lookup, game-type filtering, skip rules, aggregate formulas, output folder replacement, and derived relationship lists. Those behaviors are all important for migration parity, but not all belong in the parser core contract.

**How to avoid:**
Split old-parser behavior into three categories during Phase 1:

- **Input semantics to preserve:** OCAP tuple interpretation, player/entity discovery, kill/death/teamkill behavior needed to reproduce current stats.
- **Compatibility projections:** legacy aggregate fields, `kdRatio`, `score`/`totalScore`, `killsFromVehicleCoef`, same-name merging, and old relationship lists.
- **Out-of-contract legacy policy:** canonical/name-change identity, public presentation filtering, old output folder layout, scraping concerns, and one-off schedule/job behavior.

Then implement the Rust parser core around observed facts and add explicitly named compatibility derivation layers for old output parity. Every compatibility rule should cite the old source file and have a fixture.

**Warning signs:**
Rust modules are named after old pipeline stages instead of contract responsibilities; `nameChanges.csv` is required to emit normalized events; `sm`/`mace` skip rules live in raw parse code; old formulas are copied without tests that show the legacy input and expected output; a parity mismatch is fixed by changing normalized facts instead of the compatibility projection.

**Phase to address:**
Phase 1 prevents this by creating the old-parser behavior dossier. Phase 2 classifies contract vs compatibility fields. Phase 4 implements compatibility projections. Phase 5 verifies old/new parity and intentional breaks.

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Parse directly from `serde_json::Value` everywhere | Fast prototyping | Untyped shape bugs, clones, late panics, poor errors | Only in a spike before Phase 3 |
| Serialize `HashMap` fields directly | Less code | Nondeterministic artifacts and flaky golden tests | Never for contract output |
| Match old aggregate totals without normalized event trace | Early parity | No audit/recalculation path; bounty bugs hidden | Never |
| Use `""`, `0`, or `false` for unknown winner/SteamID | Simpler schema | Loses distinction between missing and negative facts | Never |
| Implement worker with auto-ack | Quick demo | Lost jobs on crash or parse failure | Never |
| Store full parse result only in RabbitMQ message | Avoid artifact storage | Message size pressure, retry duplication, no durable artifact | Only for tiny local dev payloads, not production |
| Let parser write business tables | Fewer services in first demo | Scope coupling to `server-2`, hard rewrites | Never in v1 |
| Benchmark only one warm local file | Quick speed number | False 10x claim | Only as local smoke test |
| Use existing `~/sg_stats/results` without rerunning old parser | Faster parity setup | Cannot tell current behavior from stale/corrupt artifacts | Only for initial discovery, never as Phase 5 gate |
| Copy old TypeScript functions line-for-line into Rust | Fast apparent parity | Preserves hidden policy in parser core and blocks better contract design | Only inside labeled compatibility projection tests |
| Drop old output fields because new contract is cleaner | Simplifies v1 schema | Breaks downstream consumers and migration parity | Only with documented replacement and `server-2` signoff |
| Reuse old `nameChanges.csv` identity logic in normalized output | Matches old player IDs | Violates observed-identity boundary | Only in old-compatible aggregate projection |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| RabbitMQ consumer | Auto-ack or ack before artifact/result durability | Manual ack after durable artifact/failure and confirmed publish |
| RabbitMQ retry | `nack(requeue=true)` forever | Bounded attempts, backoff, dead-letter or structured permanent failure |
| RabbitMQ prefetch | Unlimited/high prefetch copied from examples | Size from measured max RSS per file and target concurrency |
| S3-compatible storage | Trust `object_key`/ETag as enough identity | Validate request checksum, size, and expected contract metadata |
| S3-compatible storage | Treat object key as local path | Use SDK object APIs only; sanitize metadata/logging |
| `server-2` contract | Permissive map deserialization | Shared contract fixtures and JSON Schema validation per version |
| Old parser CLI baseline | Rely on remembered command or stale `results` | Pin old repo commit, command, environment, fixture inputs, output hashes, and run logs |
| CLI | Mix result JSON with logs on stdout | Write artifact to requested path/stdout only; logs/errors to stderr |
| Container | Health means process is running | Health/readiness must cover config, queue connection, storage reachability if enabled |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full `Value` tree plus repeated clones | High RSS, allocator-heavy flamegraphs | Typed raw adapter, borrow/copy only needed references | Largest observed files around 18.9 MB; concurrent workers amplify |
| Global mutable aggregate locks in parallel parsing | CPU usage high but throughput flat | Per-file/per-chunk local results, deterministic merge | More than one worker thread |
| Sorting huge intermediate maps repeatedly | Time spent in sort/compare, not parse | Keep source order vectors; sort only final contract boundaries | Large entity/event counts |
| Pretty-printing large artifacts in hot path | Output dominates runtime | Compact deterministic JSON for artifacts; pretty only debug tooling | Corpus benchmark and worker mode |
| Benchmarking debug builds or warm tiny samples | Unrealistic 10x claim | Release builds, fixed corpus sample tiers, old-parser baseline, bytes/sec and RSS | Phase 5 acceptance |
| Comparing Rust parser core to old full pipeline | Misleading speedup or slowdown | Benchmark equivalent tiers: old full pipeline vs new full pipeline, old per-replay worker vs new per-replay parser | Any 10x claim |
| Sending full artifacts through RabbitMQ | Publish latency, broker memory pressure | Store artifact in S3-compatible storage and publish reference for large outputs | Large replays or concurrent jobs |
| Parsing whole corpus in one process without isolation | Memory grows across files, flaky benchmark | Per-file benchmark isolation or explicit allocator/reset measurement | Corpus-wide runs |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Unbounded input size/time | Worker OOM or CPU exhaustion | Max object size, parse timeout, bounded worker concurrency |
| Panic on malformed tuples | Job crash and redelivery loop | Fallible adapters and structured parse failures |
| Logging raw replay text as plain log lines | Log injection, unreadable multilingual admin/player messages | Structured logging with escaped fields and truncation |
| Trusting queue object key as filesystem path | Path traversal if local fallback is added | Treat object keys as opaque storage IDs |
| Emitting credentials/endpoints in failure payloads | Secret leakage to `server-2` or logs | Sanitize error chains; separate internal logs from public failure payload |
| Accepting checksum mismatch as parse failure | Corruption hidden as data issue | Dedicated `source_integrity_failed` failure category |

## UX Pitfalls

User/operator experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Output is not diff-friendly | Reviewers cannot explain stat changes | Stable ordering, source refs, compact but deterministic artifact |
| Parse failures are generic | Operators cannot distinguish corrupt file, unknown OCAP shape, or integration outage | Structured error codes with job/replay/object/checksum context |
| CLI has hidden environment assumptions | Local reproduction fails | Explicit input/output args, contract version, checksum/report flags |
| Golden comparison produces only pass/fail | Roadmap cannot prioritize semantic gaps | Mismatch taxonomy and per-field diff summaries |
| Benchmark report is a single number | Speed claims are not actionable | Time, throughput, RSS, file count, event count, old/new command details |

## "Looks Done But Isn't" Checklist

- [ ] **Parser emits JSON:** Verify schema validation, contract version, deterministic field/event order, and source references.
- [ ] **Old parser baseline exists:** Verify old repo commit, command, Node/package manager versions, `~/sg_stats` inputs, run logs, and output hashes are recorded.
- [ ] **Legacy output compatibility exists:** Verify every old field is mapped to exact preserve, compatibility projection, replacement, or intentional drop.
- [ ] **Kills parsed:** Verify enemy kill, teamkill, suicide/self, vehicle context, unknown actor, and disconnected/deleted entity fixtures.
- [ ] **Aggregates match one replay:** Verify comparable fields across corpus slices and classify mismatches.
- [ ] **Winner parsed:** Verify older replay with missing winner emits unknown, not false.
- [ ] **Identity parsed:** Verify nickname and SteamID are preserved as observed facts only; no canonical matching.
- [ ] **Benchmarks exist:** Verify old-parser baseline command, release build, bytes/sec, events/sec, RSS, and same file set.
- [ ] **Migration parity exists:** Verify field-by-field old/new diff reports for global, weekly, rotation, weapon, vehicle, and other-player outputs.
- [ ] **Worker consumes jobs:** Verify crash-after-download, crash-after-artifact, duplicate job, checksum mismatch, and poison replay behavior.
- [ ] **Container runs:** Verify health/readiness, config validation, logs, and graceful shutdown with in-flight message handling.
- [ ] **Error handling exists:** Verify malformed JSON, unknown tuple shape, oversized file, missing object, and corrupt object fixtures.

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Old/new results diverge without classification | MEDIUM | Freeze rollout, run corpus diff, classify mismatches, add fixtures, publish semantic changelog |
| Old parser baseline was never pinned | MEDIUM | Pin old repo commit, rebuild old parser, rerun fixed fixtures, store command/logs/output hashes, restart parity work |
| Old field dropped accidentally | HIGH | Add compatibility projection or documented replacement, update schema/changelog, rerun downstream `server-2` contract tests |
| Contract shipped without source refs | HIGH | Add v2 contract, backfill artifacts by reparsing raw replays, update `server-2` consumers |
| Parser merged canonical identities | HIGH | Remove canonical fields, re-emit observed artifacts, invalidate derived stats in `server-2` |
| Nondeterministic output discovered late | MEDIUM | Add repeatability tests, canonicalize ordering, regenerate golden artifacts with version bump if bytes changed |
| Worker loses jobs due auto-ack | HIGH | Switch to manual ack, rebuild idempotency table/artifact naming, reconcile queue/source artifacts |
| S3 checksum ignored | MEDIUM | Add checksum validation, mark old artifacts with `source_integrity_unverified`, reprocess critical stats |
| Memory too high for concurrency | MEDIUM | Reduce prefetch/concurrency immediately, profile, remove clones, reconsider typed/streaming strategy |
| Unknown OCAP shapes panic parser | LOW-MEDIUM | Convert panics to structured unknown-shape failures, add fixture, implement adapter support |
| Legacy policy copied into parser core | HIGH | Move policy into compatibility projection, restore observed-fact artifact, add tests separating raw facts from old-compatible aggregate output |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Old parser treated as perfect or irrelevant | Phase 1: Corpus Baseline and Legacy Reference | Corpus manifest, old-parser command, old repo commit, run logs, output hashes, and mismatch taxonomy committed |
| Legacy assumptions copied without labels | Phase 1, Phase 2, and Phase 4 | Old behavior dossier classifies input semantics, compatibility projections, and out-of-contract policy |
| Aggregates without normalized source events | Phase 2: Output Contract and Compatibility Semantics | Schema requires normalized events and aggregate source refs |
| Old output field compatibility lost | Phase 2 and Phase 5 | Field mapping table plus old/new parity report for every legacy result artifact |
| Observed identity collapsed into canonical identity | Phase 2: Output Contract and Compatibility Semantics | Contract has observed identity fields only; no `canonical_player_id`; legacy name-change behavior is isolated to compatibility projections |
| Nondeterministic Rust/JSON output | Phase 2 and Phase 3 | Same input produces byte-identical output across repeated runs/thread counts |
| Hard-coded OCAP tuple offsets | Phase 1 and Phase 3 | Shape inventory plus adapter tests for every known event/entity shape |
| Kill/teamkill/vehicle misclassification | Phase 4: Event Semantics and Aggregates | Fixture matrix covers enemy, teamkill, vehicle, suicide/unknown, disconnect cases |
| Missing winner/SteamID treated as false | Phase 2 and Phase 4 | Fixtures assert typed unknown/null states |
| Version string without compatibility system | Phase 2 | JSON Schema, contract changelog, shared fixtures, semver policy, old-field compatibility map |
| False 10x benchmark | Phase 5: Golden Validation, Migration Parity, and Benchmarks | Release-mode old-vs-new benchmark on identical files with RSS and throughput |
| Memory dominated by dense replay state | Phase 3 and Phase 5 | Large-file benchmark includes max RSS and worker concurrency estimate |
| RabbitMQ ack/idempotency errors | Phase 6: Worker Integration and Artifacts | Crash/redelivery/duplicate-job tests pass |
| S3 checksum/object identity ignored | Phase 6 | Corrupt object and checksum mismatch tests emit integrity failures |
| Parser scope creeps into `server-2` | Phase 2 and Phase 6 | Parser owns artifacts only; no business DB/canonical matching dependencies |
| Malformed/hostile replay crashes worker | Phase 3 and Phase 7 | Malformed fixtures produce structured failures; resource limits enforced |

## Sources

- Local project context: `.planning/PROJECT.md` and `gsd-briefs/replays-parser-2.md` (HIGH confidence for scope, constraints, identity boundary, required outputs).
- Old parser project: `/home/afgan0r/Projects/SolidGames/replays-parser` (HIGH confidence for required behavioral reference and migration risk source). Key files inspected: `package.json` (`parse`, `parse:dist`, worker/scripts), `README.md` (deprecated production parser context), `CLAUDE.md` (brownfield compatibility and architecture notes), `src/start.ts`, `src/index.ts`, `src/0 - utils/paths.ts`, `src/1 - replays/getReplays.ts`, `src/1 - replays/parseReplays.ts`, `src/1 - replays/workers/parseReplayWorker.ts`, `src/2 - parseReplayInfo/getEntities.ts`, `src/2 - parseReplayInfo/getKillsAndDeaths.ts`, `src/2 - parseReplayInfo/combineSamePlayersInfo.ts`, `src/0 - utils/getPlayerName.ts`, `src/0 - utils/namesHelper/getId.ts`, `src/0 - utils/calculateKDRatio.ts`, `src/0 - utils/calculateScore.ts`, `src/0 - utils/calculateVehicleKillsCoef.ts`, and `src/4 - output/*`.
- Local corpus sample: `~/sg_stats/raw_replays` contains 3,938 files; largest observed sample was 18,902,626 bytes; sampled OCAP top-level keys matched `EditorMarkers`, `Markers`, `captureDelay`, `endFrame`, `entities`, `events`, `missionAuthor`, `missionName`, `playersCount`, `worldName` (HIGH confidence for local corpus observations).
- OCAP2 README: server-side mission recording, event capture, export with optional winner side/message/tag, capture delay/time tracking, events and markers: https://github.com/OCAP2/OCAP (MEDIUM confidence for historical OCAP behavior; project corpus still needs direct characterization).
- OCAP2 changelog: event/tracking behavior changed over releases and backwards compatibility was called out: https://github.com/OCAP2/OCAP/blob/master/CHANGELOG.md (MEDIUM confidence).
- OCAP2 extension v5 docs: current OCAP model includes typed kill/hit/projectile/end-mission/time/entity concepts, nullable IDs, buffered handlers, and JSON/streaming protocol types: https://pkg.go.dev/github.com/OCAP2/extension/v5 and https://pkg.go.dev/github.com/OCAP2/extension/v5/pkg/core (MEDIUM confidence for current ecosystem direction, not historical corpus schema).
- Rust `HashMap` docs: random seeding and arbitrary iteration order: https://doc.rust-lang.org/beta/std/collections/hash_map/struct.HashMap.html (HIGH confidence).
- RFC 8259 JSON: objects are unordered; duplicate member behavior is unpredictable; number precision has interoperability limits: https://www.rfc-editor.org/rfc/rfc8259 (HIGH confidence).
- Context7/Serde JSON docs and docs.rs: `from_reader` with `BufReader`, typed deserialization, and `Number` arbitrary precision limits: https://docs.rs/serde_json/latest/serde_json/ and https://docs.rs/serde_json/latest/serde_json/struct.Number.html (HIGH confidence).
- Context7/Rayon docs and Rayon README: parallel iterators are data-race free but side effects can occur in a different order: https://github.com/rayon-rs/rayon (HIGH confidence).
- JSON Schema 2020-12 validation spec: schema validation asserts constraints on JSON instance structure: https://json-schema.org/draft/2020-12/json-schema-validation (HIGH confidence).
- Semantic Versioning 2.0.0: public API declaration and major/minor/patch compatibility rules: https://semver.org/ (HIGH confidence).
- RabbitMQ acknowledgements/publisher confirms/requeue/prefetch docs: https://www.rabbitmq.com/docs/confirms and reliability guide https://www.rabbitmq.com/docs/reliability (HIGH confidence).
- AWS S3 checksum and SDK data integrity docs, including Rust SDK support and response checksum validation: https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html and https://docs.aws.amazon.com/sdkref/latest/guide/feature-dataintegrity.html (HIGH confidence for AWS S3; validate MinIO/S3-compatible behavior separately).
- Criterion.rs docs: statistical benchmarking, throughput reporting, and named baselines: https://bheisler.github.io/criterion.rs/book/index.html (HIGH confidence).
- Approval/golden master testing references: ApprovalTests overview https://approvaltests.com/ and PyPI docs https://pypi.org/project/approvaltests/15.3.2/ (MEDIUM confidence for testing pattern; implementation should use Rust-native test tools).

---
*Pitfalls research for: Rust OCAP JSON replay parser service*
*Researched: 2026-04-24*
