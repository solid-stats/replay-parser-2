# Architecture Research

**Domain:** Rust OCAP JSON replay parser service
**Researched:** 2026-04-24
**Confidence:** HIGH for service boundaries, migration order, and old-parser reference strategy; MEDIUM for exact OCAP field semantics until a corpus profiling phase and old-vs-new parity harness lock the schema.

## Standard Architecture

### System Overview

Build `replays-parser-2` as a Rust migration of the existing parser at `/home/afgan0r/Projects/SolidGames/replays-parser`, not as an independent parser invented from the raw OCAP corpus alone. The old TypeScript/Node parser is the required behavioral reference for replay interpretation, legacy aggregate fields, `~/sg_stats` runtime assumptions, and benchmark baselines.

The target architecture is still a deterministic parsing engine with thin runtime adapters. The parser core should know nothing about RabbitMQ, S3, filesystems, clocks, environment variables, PostgreSQL, or canonical player identity. CLI, worker, tests, and benchmarks should all call the same pure core API. The old parser informs what that core must do; it should not force the new service to keep old production responsibilities that now belong to `server-2`.

```
+--------------------------------------------------------------------------------+
| Runtime Adapters                                                               |
|                                                                                |
|  +----------------------+        +------------------------------------------+  |
|  | CLI adapter          |        | RabbitMQ/S3 worker adapter               |  |
|  | local file -> JSON   |        | job -> object -> artifact/result message |  |
|  +----------+-----------+        +-------------------+----------------------+  |
|             |                                        |                         |
+-------------+----------------------------------------+-------------------------+
              | ParseInput                            | ParseInput
              v                                       v
+--------------------------------------------------------------------------------+
| Deterministic Parser Core                                                       |
|                                                                                |
|  Raw OCAP decode -> entity index -> normalized observations -> event pipeline   |
|  -> aggregate projection -> schema/version validation -> ParseArtifact          |
+--------------------------------------------------------------------------------+
              |
              v
+--------------------------------------------------------------------------------+
| Output Contract                                                                 |
|                                                                                |
|  Versioned JSON schema, Rust contract types, source refs, structured failures,  |
|  golden fixtures, old-vs-new comparison outputs, benchmark reports              |
+--------------------------------------------------------------------------------+
              |
              v
+--------------------------------------------------------------------------------+
| Legacy Reference Harness                                                        |
|                                                                                |
|  old parser commands, old stage outputs, ~/sg_stats corpus/results, parity      |
|  reports, benchmark baseline, migration decisions                               |
+--------------------------------------------------------------------------------+
              |
              v
+--------------------------------------------------------------------------------+
| External Ownership                                                              |
|                                                                                |
|  server-2: S3 upload, job records, PostgreSQL persistence, canonical identity, |
|  rotations, corrections, bounty formula, public statistics APIs                 |
+--------------------------------------------------------------------------------+
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| Old parser reference | Required behavioral source for how current SolidGames replays are discovered, parsed, aggregated, and published today. | Read and test against `/home/afgan0r/Projects/SolidGames/replays-parser`; run `pnpm run parse` / `pnpm run parse:dist` for baseline behavior; use `docs/architecture.md` and stage modules as migration map. |
| Legacy parity harness | Compare Rust outputs against old parser outputs and historical `~/sg_stats/results` for fields that must remain compatible. | Harness commands that run or consume old parser outputs, normalize comparable fields, and produce old-vs-new reports with documented tolerances. |
| `parser-contract` | Versioned Rust types and generated JSON Schema for parse requests, parse artifacts, failures, source references, unknown states, and aggregate summaries. | `serde` derives, `schemars` schema generation, `jsonschema` validation in tests. Keep this stable and reviewed before worker integration. |
| `parser-core` | Pure OCAP JSON parsing and normalization. Accept bytes/readers plus explicit parse metadata; return `ParseArtifact` or `ParseFailure`. | Synchronous Rust library using strongly typed raw DTOs, domain newtypes, deterministic ordering, no network or filesystem side effects. |
| Raw OCAP model | Represent observed top-level OCAP structure and event tuple variants without prematurely imposing business meaning. | `RawOcapReplay`, `RawEntity`, `RawEvent` with tolerant enum/unknown variants. Preserve unknown event shapes for warnings. |
| Normalization pipeline | Convert raw entities/events into replay metadata, observed identity, side/group context, kill/death/teamkill events, vehicle context, commander-side candidates, and outcome fields. | Small pipeline modules: `metadata`, `entities`, `events`, `outcome`, `commander`, `aggregate`. Aggregates derive only from normalized events. |
| Source-reference builder | Attach audit pointers from outputs back to raw OCAP inputs. | Stable references such as `events[184]`, `entities[id=37]`, `positions[index]`, frame, JSON pointer where practical, and confidence/source kind. |
| CLI adapter | Local reproducibility: parse one local OCAP JSON file and write one normalized result JSON file. | `clap` binary crate; reads input, calls core, writes deterministic pretty or compact JSON; maps structured failures to exit codes. |
| Worker adapter | Production integration: consume parse requests, download replay object, verify checksum, call core, store artifact, publish completed/failed result, acknowledge safely. | Async Tokio service with bounded concurrency, RabbitMQ manual acknowledgements/publisher confirms, S3-compatible client, `tracing` spans. |
| Golden harness | Corpus-backed correctness checks against `~/sg_stats/raw_replays` and historical results. | Integration tests plus snapshot/golden files, corpus manifest, old-vs-new comparison reports. |
| Benchmark harness | Measure baseline old parser and new parser throughput, then drive performance work. | Criterion benchmark suite plus corpus-level benchmark command/report. |

## Old Parser as Required Reference

### Observed Old Parser Architecture

The old parser is a TypeScript/Node project at `/home/afgan0r/Projects/SolidGames/replays-parser`.

| Old Parser Area | Observed Source | Migration Use |
|-----------------|-----------------|---------------|
| Runtime commands | `package.json`: `pnpm run parse` -> `tsx src/start.ts`; `pnpm run parse:dist` -> `node dist/start.js`; `pnpm run build` -> `tsup`. | Baseline command discovery for benchmarks and parity runs. |
| Architecture doc | `docs/architecture.md`. | Treat as source for brownfield runtime semantics and stage boundaries. |
| Entrypoints | `src/start.ts`, `src/index.ts`, `src/schedule.ts`, `src/jobs/prepareReplaysList/start.ts`, `src/!yearStatistics/index.ts`. | Identify behavior that is parser-owned today versus behavior moving to `server-2`. |
| Runtime directories | `src/0 - utils/paths.ts` rooted at `~/sg_stats`: `raw_replays`, `lists`, `results`, `temp_results`, `year_results`, `logs`, `config/nameChanges.csv`. | Preserve local test/golden assumptions; do not silently change fixture locations. |
| Runtime config | `src/0 - utils/runtimeConfig.ts` uses `WORKER_COUNT`, clamps to 1..64, defaults to CPU count minus one. | Reference for benchmark parity and concurrency comparisons, not necessarily worker production config. |
| Replay discovery/download | `src/jobs/prepareReplaysList/*`. | Migration source for replay list metadata and raw corpus preparation; production v1 discovery belongs to `server-2`. |
| Replay selection and worker dispatch | `src/1 - replays/*`, especially `parseReplays.ts`, `fetchReplayInfo.ts`, `workers/workerPool.ts`, `workers/parseReplayWorker.ts`. | Reference for old concurrency behavior, error handling, ordering by date, and per-replay task shape. |
| Single-replay parsing | `src/2 - parseReplayInfo/*`: `getEntities.ts`, `getKillsAndDeaths.ts`, `combineSamePlayersInfo.ts`. | Primary behavioral source for legacy player/entity extraction, kill/teamkill/death classification, vehicle kill context, and duplicate slot merging. |
| Aggregation | `src/3 - statistics/*`. | Reference for legacy compatible aggregate fields and old output comparison. |
| Output publication | `src/4 - output/*`, especially `index.ts`. | Reference for historical published JSON shape and atomic result replacement; production publishing is replaced by `server-2` persistence. |

### Migration Rules

- **Port behavior, not accidental structure.** The Rust core should reproduce old parser semantics where those semantics define current stats, but it should not copy TypeScript folder names, global ambient types, or file-backed production orchestration into the new worker.
- **Classify old responsibilities before implementation.** Each old stage must be marked as one of: keep in parser core, keep in parser harness/CLI only, move to `server-2`, or retire with an explicit decision.
- **Use old code as the initial executable specification.** Before changing a legacy statistic, first prove what the old parser does on representative raw replays and historical outputs.
- **Preserve legacy comparable fields.** Fields such as `kills`, `killsFromVehicle`, `vehicleKills`, `teamkills`, `deaths`, `kdRatio`, `killsFromVehicleCoef`, `score`/`totalScore`, `totalPlayedGames`, weekly stats, squad stats, and rotation stats need old-vs-new comparison coverage before the new contract expands beyond them.
- **Do not preserve legacy canonical identity behavior as final truth.** Old parser name helpers and `nameChanges.csv` explain historical outputs, but the new parser must still emit observed identifiers only; canonical identity moves to `server-2`.

## Recommended Project Structure

Use a Cargo workspace. Cargo workspaces share a lockfile and target directory, and common commands can run across all workspace members. This fits a multi-crate service where contract, core, CLI, worker, tests, and benches must evolve together.

```
Cargo.toml                         # workspace, shared dependency versions, lints
crates/
  parser-contract/
    src/
      lib.rs
      artifact.rs                  # ParseArtifact, normalized events, aggregates
      failure.rs                   # structured failures and warnings
      request.rs                   # worker request/result messages
      source_ref.rs                # raw OCAP source reference model
      version.rs                   # contract versions and compatibility helpers
  parser-core/
    src/
      lib.rs                       # parse_replay(ParseInput) -> ParseArtifact
      raw/                         # tolerant OCAP input DTOs
      normalize/                   # metadata, entities, events, commander, outcome
      aggregate/                   # aggregate projection from normalized events
      validate.rs                  # invariants before serialization
  parser-cli/
    src/main.rs                    # parse-file command only; thin adapter
  parser-worker/
    src/
      main.rs
      config.rs
      queue.rs                     # RabbitMQ consume/publish abstraction
      storage.rs                   # S3 get/put abstraction
      job.rs                       # job lifecycle and ack/nack policy
      health.rs                    # readiness/liveness, if exposed
  parser-harness/
    src/
      corpus.rs                    # ~/sg_stats discovery and fixture manifests
      legacy.rs                    # old parser command/output adapter
      compare.rs                   # old-vs-new comparable fields
      report.rs                    # machine-readable reports for roadmap/UAT
      migrate.rs                   # old stage -> new component classification checks
schemas/
  parser-output.v1.schema.json      # generated from parser-contract
  parser-messages.v1.schema.json
tests/
  fixtures/                         # small hand-curated OCAP fragments
  golden/                           # accepted artifacts or snapshots
  legacy-parity/                    # curated old-vs-new expected reports
benches/
  parse_corpus.rs                   # Criterion benchmarks
  legacy_baseline.rs                # invokes/reads old parser baseline where practical
docs/
  contract.md                       # human contract notes for server-2
  migration-map.md                  # old parser stage mapping and decisions
```

### Structure Rationale

- **Old parser reference before `parser-contract`:** the contract must be shaped by old behavior and new `server-2` needs together. Do not design a contract from one-off OCAP inspection while ignoring the existing parser.
- **`parser-contract` before `parser-core`:** `server-2` integrates with the contract, not with parser internals. Versioned contract types prevent worker details from leaking into business persistence.
- **`parser-core` as a library crate:** all correctness, benchmarks, and adapters share one code path. This is the main protection against "CLI works but worker differs" regressions.
- **Separate CLI and worker crates:** the CLI stays small and easy to run locally; worker-specific async, RabbitMQ, S3, health, and container concerns do not pollute the pure parser.
- **Dedicated harness crate:** old-vs-new comparison, legacy stage mapping, and benchmarks are product-critical, not throwaway scripts. Keep them first-class so roadmap phases can gate on them.
- **Generated schemas in `schemas/`:** `server-2` can validate parser artifacts independently and pin `parser_contract_version`.

## Architectural Patterns

### Pattern 1: Legacy Parity Before Rewrite

**What:** Inventory old parser behavior and create executable parity checks before replacing each stage in Rust.

**When to use:** Every parser-core and aggregate phase. Especially for `src/2 - parseReplayInfo/*` and `src/3 - statistics/*`, where subtle legacy semantics define current public stats.

**Trade-offs:** Slower first milestone, but it prevents a fast Rust parser that is semantically incompatible with historical SolidGames results.

**Example migration table entry:**

```text
Old: src/2 - parseReplayInfo/getKillsAndDeaths.ts
New: parser-core::normalize::events + parser-core::aggregate
Parity: per replay enemy kills, teamkills, deaths, vehicle kills, killed/killer lists
Decision: preserve current comparable semantics, add source refs and explicit unknowns
```

### Pattern 2: Pure Core, Imperative Shell

**What:** Put all deterministic parsing and normalization in a synchronous core function. Adapters do I/O, configuration, logging, retries, and process exit behavior.

**When to use:** Always for this project. It is the only way to make CLI, worker, golden tests, and benchmarks agree.

**Trade-offs:** The worker has to bridge async I/O to a synchronous parser. That is acceptable because parsing is bounded CPU work and should be explicitly concurrency-limited.

**Example:**

```rust
pub struct ParseInput<'a> {
    pub bytes: &'a [u8],
    pub source: SourceContext,
    pub requested_contract: ContractVersion,
}

pub fn parse_replay(input: ParseInput<'_>) -> Result<ParseArtifact, ParseFailure> {
    let raw = raw::decode(input.bytes, &input.source)?;
    let indexed = normalize::entities::index(&raw)?;
    let events = normalize::events::normalize(&raw, &indexed)?;
    let aggregates = aggregate::from_events(&events, &indexed)?;
    validate::artifact(&raw, &events, &aggregates)?;

    Ok(ParseArtifact::new(input.requested_contract, raw, indexed, events, aggregates))
}
```

### Pattern 3: Contract Types Are the Public API

**What:** Treat `ParseArtifact`, `ParseFailure`, and queue message types as public API. Generate JSON Schema from Rust types and validate representative artifacts against the schema.

**When to use:** Before worker work starts and before `server-2` persists parse results.

**Trade-offs:** Contract design slows the first parser phase, but it prevents later rewrites when `server-2` needs auditability, reprocessing, and unknown states.

**Example contract shape:**

```rust
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ParseArtifact {
    pub contract_version: ContractVersion,
    pub parser_version: String,
    pub source: SourceContext,
    pub replay: ReplayMetadata,
    pub observations: Observations,
    pub events: Vec<NormalizedEvent>,
    pub aggregates: AggregateSummary,
    pub warnings: Vec<ParseWarning>,
}
```

### Pattern 4: Aggregate Projection From Normalized Events Only

**What:** Derive player, squad, commander-side, and bounty-input aggregate summaries from normalized event records, not directly from raw OCAP tuples. Old aggregate behavior from `src/3 - statistics/*` must be represented as tests around this projection layer.

**When to use:** All stats fields, including legacy comparable fields such as kills, teamkills, deaths, vehicle kills, and KD.

**Trade-offs:** It requires richer normalized events, but every aggregate can be audited back to source events. This is essential for correction workflows in `server-2`.

### Pattern 5: Bounded Worker Concurrency

**What:** Use RabbitMQ prefetch plus an internal concurrency limit. Do not let the broker push unlimited unacknowledged replay jobs into the worker.

**When to use:** Worker mode from the first integration phase.

**Trade-offs:** Lower peak throughput than unbounded consumption, but predictable memory and CPU. Current local corpus has 3,938 replay files and the largest observed file is about 18.9 MB, so bounded in-memory parsing is reasonable before deeper streaming work. Use the old parser's worker-thread model and `WORKER_COUNT` behavior as benchmark context, not as a production API requirement.

### Pattern 6: Artifact Reference as Primary Worker Output

**What:** Worker should store the full parse artifact in S3-compatible storage and publish `parse.completed` with artifact key, checksum, contract version, parser version, replay/job IDs, and summary metrics. Avoid making large result JSON payloads the default RabbitMQ message body.

**When to use:** Worker integration with `server-2`.

**Trade-offs:** Adds one storage write but gives `server-2` a durable audit artifact, avoids queue payload-size pressure, and makes reprocessing/comparison reproducible. CLI still writes the full artifact locally.

## Data Flow

### Legacy Reference Flow

```
Old parser at /home/afgan0r/Projects/SolidGames/replays-parser
    |
    +--> pnpm run parse      -> tsx src/start.ts
    +--> pnpm run parse:dist -> node dist/start.js
    |
    v
src/index.ts
    |
    +--> prepare folders and name-change data from ~/sg_stats/config
    +--> read replay lists and raw replay JSON from ~/sg_stats
    +--> dispatch per-replay parse work through worker_threads
    +--> parse single replay via src/2 - parseReplayInfo/*
    +--> aggregate via src/3 - statistics/*
    +--> publish JSON/archive outputs via src/4 - output/*
    |
    v
~/sg_stats/results and historical comparison artifacts
```

The Rust roadmap should turn this flow into parity fixtures, stage mapping, and benchmark baselines. It should not blindly preserve old discovery/download/publishing responsibilities in the production worker because the new Solid Stats architecture moves upload, job orchestration, persistence, and public publication to `server-2`.

### CLI Parse Flow

```
User runs parser-cli parse --input replay.json --output artifact.json
    |
    v
CLI validates args and builds SourceContext
    |
    v
Read local file into bounded bytes or BufReader-backed bytes
    |
    v
parser-core::parse_replay(ParseInput)
    |
    +--> raw OCAP decode
    +--> entity/player/side index
    +--> normalized events with source refs
    +--> aggregate projection
    +--> contract invariant validation
    |
    v
Write stable JSON artifact or structured failure JSON
```

### Worker Parse Flow

```
server-2
  stores replay object in S3-compatible storage
  creates parse_jobs row
  publishes parse.request { job_id, replay_id, object_key, checksum, parser_contract_version }
    |
    v
RabbitMQ queue
    |
    v
parser-worker consumes with manual ack and bounded prefetch
    |
    v
Validate request contract and supported parser_contract_version
    |
    v
Download object from S3-compatible storage
    |
    v
Verify checksum from request / object metadata
    |
    v
Call parser-core
    |
    +--> success: write artifact to S3, publish parse.completed, wait for publish confirm, ack request
    |
    +--> parse failure: publish parse.failed with structured failure, wait for publish confirm, ack request
    |
    +--> transient infra failure: nack/requeue or let retry policy handle; never ack before result is durable/published
```

### Golden and Benchmark Flow

```
Corpus manifest from ~/sg_stats/raw_replays, selected old result files, and old parser stage outputs
    |
    v
Fixture selector chooses small, large, recent, legacy, vehicle-heavy, and malformed cases
    |
    v
Old parser baseline and parser-core each parse comparable replay sets
    |
    +--> schema validation
    +--> deterministic snapshot/golden comparison
    +--> old-vs-new comparable field report
    +--> benchmark throughput report
```

### Key Data Flows

1. **Raw replay to normalized audit artifact:** `RawOcapReplay` is decoded, indexed, normalized, and serialized as a versioned artifact. Every normalized event should carry source references.
2. **Normalized events to aggregate summaries:** aggregate stats are a projection layer over normalized events, which keeps future recalculation and moderation patches possible in `server-2`.
3. **Worker job to durable result:** RabbitMQ message triggers S3 download and parser execution; result is durable before the request message is acknowledged.
4. **Old parser to migration gates:** old TypeScript stage behavior feeds Rust acceptance criteria. Each migrated area should have parity reports before it is considered complete.
5. **Historical corpus to roadmap gates:** `~/sg_stats` feeds schema discovery, golden tests, old-vs-new comparisons, and benchmarks. It should not become a production import path.

## Server-2 Boundary

### Parser Owns

- OCAP JSON input compatibility for v1.
- Migration parity with the old parser for legacy comparable parse and aggregate behavior.
- Versioned parse request/result/failure message contracts used by the worker.
- Versioned parse artifact JSON schema.
- Observed replay metadata, observed player/entity identity, side/group/squad observations, commander-side candidates, winner/outcome if present, normalized kill/death/teamkill events, vehicle context, and aggregate inputs.
- Explicit unknown/null states for absent winner, SteamID, commander, side, vehicle, or identity data.
- Structured parse failures tied to `job_id`, `replay_id`, object key/checksum, parser version, and source location when available.
- Deterministic local parse behavior for CLI, tests, and benchmarks.

### Server-2 Owns

- Replay upload flow and object lifecycle.
- Production replacement for old parser replay discovery/download and output publication responsibilities.
- PostgreSQL persistence, parse job records, retries, operational UI, and failed job visibility.
- Canonical player identity, nickname history decisions, Steam OAuth linking, squad history decisions, merge/split/moderation workflow, and manual corrections.
- Rotation assignment, public stats publication, bounty point formula, and recalculation after corrections.
- Deciding when to overwrite current derived parse results in v1.

### Boundary Rule

The parser may output "observed facts" and "calculated inputs"; it must not output "canonical truth". For example, `observed_player { nickname, steam_id: null, side, squad_prefix }` belongs in the artifact; `canonical_player_id` does not. `valid_enemy_kill` input belongs in the artifact; final bounty points belong in `server-2`.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Local development and 3,938-file historical corpus | Single-machine CLI and harness are enough. Parse whole replay bytes with bounded memory; largest observed local file is about 18.9 MB. Prioritize determinism and schema coverage before streaming optimization. |
| Single VPS production v1 | Run 1-N worker containers with bounded RabbitMQ prefetch and per-worker concurrency. Store full artifacts in S3-compatible storage and let `server-2` persist derived rows. |
| High replay backlog or reprocessing waves | Horizontally scale workers, tune queue prefetch/concurrency, and add corpus batch tooling. Keep parser core stateless so multiple workers can parse independently. |
| Much larger future OCAP files | Revisit streaming/partial decoding after benchmarks. Start with typed `serde` decoding because cross-references between entities, positions, and events require indexes anyway. |

### Scaling Priorities

1. **First bottleneck: CPU parse time.** Measure with Criterion/corpus benchmarks before optimizing. Add profiling and reduce allocations in the core only after correctness gates pass.
2. **Second bottleneck: worker memory under concurrency.** Bound queue prefetch and parser concurrency. Do not allow unlimited in-flight replay bytes.
3. **Third bottleneck: result artifact size.** Use S3 artifacts by default and keep RabbitMQ messages small.

## Anti-Patterns

### Anti-Pattern 1: Ignoring the Old Parser

**What people do:** Build from OCAP JSON samples and new requirements only, treating `/home/afgan0r/Projects/SolidGames/replays-parser` as historical trivia.

**Why it is wrong:** Current public stats semantics live in the old parser's parsing and aggregation code. A Rust rewrite can be faster and cleaner while still being behaviorally wrong.

**Do this instead:** Create old-stage inventories, parity tests, old-vs-new reports, and benchmark baselines before replacing behavior.

### Anti-Pattern 2: Blindly Cloning Old Production Ownership

**What people do:** Rebuild old `prepareReplaysList` discovery, `~/sg_stats/results` publishing, name-change identity behavior, and schedule orchestration as production responsibilities in Rust.

**Why it is wrong:** The new product boundary assigns upload, jobs, PostgreSQL persistence, canonical identity, rotations, corrections, and public publishing to `server-2`.

**Do this instead:** Port parse semantics into the core, keep legacy discovery/output behavior in harness or compatibility tools, and emit artifacts for `server-2`.

### Anti-Pattern 3: Parser Core Depends on RabbitMQ or S3

**What people do:** Put queue consumption, object download, and parsing in one worker function.

**Why it is wrong:** Local CLI and golden tests stop exercising the same logic as production, and parse bugs become entangled with transport failures.

**Do this instead:** Keep `parser-core` pure and call it from both adapters.

### Anti-Pattern 4: Direct PostgreSQL Writes From Parser

**What people do:** Parser writes player stats, events, or job state directly into `server-2` tables.

**Why it is wrong:** It breaks ownership. `server-2` owns canonical identity, rotations, moderation, corrections, and persistence.

**Do this instead:** Emit artifacts and result/failure messages. Let `server-2` persist and recalculate.

### Anti-Pattern 5: Aggregate-Only Output

**What people do:** Emit only kills/deaths/teamkills/player totals because those are the current stats fields.

**Why it is wrong:** Corrections, old-vs-new comparison, commander-side stats, bounty inputs, and audit all need normalized events with source references.

**Do this instead:** Make normalized events primary. Aggregates are projections.

### Anti-Pattern 6: Inferring Canonical Identity in the Parser

**What people do:** Treat nickname, SteamID, or squad prefix as a real player identity.

**Why it is wrong:** The product requirements explicitly allow many nicknames, multiple SteamIDs, no SteamID in legacy data, and multi-account cases.

**Do this instead:** Preserve observed identifiers and explicit unknowns. Let `server-2` resolve canonical identity.

### Anti-Pattern 7: Non-Deterministic Serialization

**What people do:** Iterate unsorted `HashMap`s, include wall-clock timestamps, generate random IDs, or depend on worker scheduling in output JSON.

**Why it is wrong:** Golden tests, old-vs-new comparisons, and reprocessing audits become noisy or impossible.

**Do this instead:** Sort output collections by stable keys/source order, use explicit parser version/source checksum, and keep generated IDs deterministic.

### Anti-Pattern 8: Acknowledging Parse Jobs Too Early

**What people do:** Ack the RabbitMQ request immediately after download or parser success, before `parse.completed`/`parse.failed` is durably published.

**Why it is wrong:** A worker crash can lose the only state transition for a job.

**Do this instead:** Use manual acknowledgements and publisher confirms. Ack only after artifact storage and result/failure publish have succeeded.

### Anti-Pattern 9: Over-Streaming Before Profiling

**What people do:** Build a custom streaming parser or hand-written JSON tokenizer before measuring the real corpus.

**Why it is wrong:** OCAP events and entities cross-reference each other, and current files are small enough for bounded whole-file parsing. A custom tokenizer increases correctness risk.

**Do this instead:** Start with strongly typed `serde` decoding and profile. Introduce selective streaming or `RawValue` only after benchmark data proves the need.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| RabbitMQ | Worker consumes `parse.request` with manual ack, bounded prefetch, and publishes `parse.completed`/`parse.failed` with publisher confirms. | Consumer acknowledgements and publisher confirms are separate reliability mechanisms. Use both. Track redeliveries and avoid requeue loops. |
| S3-compatible storage | Worker downloads replay by object key and checksum; writes full parse artifact by deterministic artifact key. | Prefer artifact reference in RabbitMQ result. Verify checksum from request/object metadata before parsing. |
| server-2 | Message contract plus S3 artifact contract. | No direct DB access from parser. Server owns persistence, identity, retries, rotations, bounty, and public APIs. |
| Old parser repo | Behavioral reference and executable baseline. | Located at `/home/afgan0r/Projects/SolidGames/replays-parser`; use `pnpm run parse`, `pnpm run parse:dist`, `docs/architecture.md`, and stage modules for parity and migration mapping. |
| Historical `~/sg_stats` | Test and benchmark input only. | Use as fixture/golden source, not production import. Local inspection confirmed 3,938 raw replay JSON files and observed OCAP keys matching the project brief. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI -> parser-core | Direct library call with `ParseInput`. | CLI owns local paths and exit codes; core owns parse result. |
| Worker -> parser-core | Direct library call inside bounded CPU work. | In async worker, use a bounded blocking strategy or dedicated worker pool so CPU parsing does not stall async I/O. |
| parser-core -> parser-contract | Contract types returned directly. | Core should not serialize ad hoc JSON maps. |
| parser-core raw -> normalized | Typed pipeline stages. | Keep raw DTOs separate from normalized domain types so input quirks do not leak into output contract. |
| normalized events -> aggregates | Pure projection. | Aggregates must be traceable to event IDs/source refs. |
| harness -> old parser | Command invocation or captured old outputs. | Required for baseline benchmark and behavioral comparison. Keep this outside parser-core. |
| harness -> parser-core | Direct library call. | Harness bypasses CLI/worker to measure and validate the engine directly; add CLI smoke tests separately. |

## Build Order and Roadmap Implications

Recommended roadmap order:

1. **Legacy behavior inventory and migration map**
   - Read old parser architecture and stage code; document old stage responsibilities, input/output files, runtime directories, worker behavior, and which responsibilities move to `server-2`.
   - Dependency: none.
   - Why first: `replays-parser-2` must be based on the old parser. Without this map, roadmap phases risk building a clean Rust parser that does not match current SolidGames behavior.

2. **Legacy baseline and parity harness skeleton**
   - Add harness support for old parser commands (`pnpm run parse`, `pnpm run parse:dist`), captured old outputs, benchmark timing, and old-vs-new report format.
   - Dependency: legacy behavior inventory.
   - Why before Rust parsing: every subsequent parser milestone needs a way to prove parity or document intentional divergence.

3. **Corpus profiler and fixture selection**
   - Scan `~/sg_stats/raw_replays`, old `~/sg_stats/results`, and old parser fixtures/tests. Record observed top-level keys, event type shapes, entity field shapes, file sizes, missing fields, and representative fixtures.
   - Dependency: legacy harness skeleton.
   - Why before contract/core: prevents designing from one replay sample and ties fixtures to old parser behavior.

4. **Workspace and contract foundation**
   - Create Cargo workspace, `parser-contract`, base `ParseArtifact`, `ParseFailure`, source refs, contract versioning, generated JSON Schema, and legacy-compatible aggregate fields.
   - Dependency: legacy inventory and corpus profile.
   - Why here: `server-2` boundary and all tests need a stable artifact shape, but that shape must be grounded in old parser semantics.

5. **Pure parser core MVP: old `parseReplayInfo` parity**
   - Port behavior from `src/2 - parseReplayInfo/*`: metadata, player/entity extraction, connected-event fallback players, kill/death/teamkill classification, vehicle kill context, duplicate player merge, and source references.
   - Dependency: contract, fixtures, parity harness.
   - Why before adapters: correctness has to be testable without RabbitMQ/S3 and must match old per-replay semantics first.

6. **Deterministic output and schema validation**
   - Stable JSON serialization, schema generation, schema validation tests, deterministic ordering tests, explicit unknown/null semantics, and legacy comparable field checks.
   - Dependency: parser core MVP.
   - Why here: locks the public artifact before adding aggregate and worker complexity.

7. **Aggregate projection: old statistics parity plus new inputs**
   - Port/validate comparable behavior from `src/3 - statistics/*`; add new inputs for player, squad, rotation, commander-side, and bounty calculations. Aggregates derive from normalized events only.
   - Dependency: normalized event model and old aggregate reports.
   - Why after event model: aggregate definitions should not bypass audit events, and old aggregate compatibility must be measurable.

8. **CLI adapter and legacy-compatible local mode**
   - `parse-file` command for local files, output JSON, structured failure JSON, exit codes, and optional harness commands that read `~/sg_stats` like the old parser for comparison runs.
   - Dependency: parser core and contract.
   - Why before worker: gives developers and `server-2` authors a reproducible manual contract tool and makes old-vs-new debugging practical.

9. **Golden comparison and benchmark hardening**
   - Snapshot/golden tests, old-vs-new comparable field reports, malformed replay tests, fixture update workflow, old parser baseline timing, new parser throughput, and initial 10x target report.
   - Dependency: CLI optional, parser core required.
   - Why before worker integration: protects behavior while tuning and prevents transport work from hiding parser regressions.

10. **RabbitMQ/S3 worker adapter**
   - Consume requests, download objects, verify checksums, call core, write artifacts, publish result/failure, ack/nack safely, structured logs.
   - Dependency: contract, core, CLI/golden confidence.
   - Why late: worker bugs should not be mixed with parser-correctness discovery.

11. **Container and operational readiness**
    - Docker image, config/env validation, health/readiness, metrics/log fields, graceful shutdown, local compose integration with RabbitMQ/S3-compatible storage.
    - Dependency: worker adapter.
    - Why last in v1 architecture: operational hardening depends on the final worker lifecycle.

## Sources

- Local project context: `.planning/PROJECT.md` and `gsd-briefs/replays-parser-2.md` (HIGH).
- Server boundary context: `gsd-briefs/server-2.md` (HIGH).
- Old parser repo: `/home/afgan0r/Projects/SolidGames/replays-parser` (HIGH).
- Old parser architecture reference: `/home/afgan0r/Projects/SolidGames/replays-parser/docs/architecture.md` (HIGH).
- Old parser package scripts: `/home/afgan0r/Projects/SolidGames/replays-parser/package.json` confirms `pnpm run parse` -> `tsx src/start.ts` and `pnpm run parse:dist` -> `node dist/start.js` (HIGH).
- Old parser orchestration/code inspected: `src/start.ts`, `src/index.ts`, `src/0 - utils/paths.ts`, `src/0 - utils/runtimeConfig.ts`, `src/1 - replays/*`, `src/2 - parseReplayInfo/*`, `src/3 - statistics/*`, and `src/4 - output/*` (HIGH for observed current repo structure).
- Local corpus inspection: `~/sg_stats/raw_replays` contains 3,938 JSON files; largest observed file about 18.9 MB; sample top-level keys were `EditorMarkers`, `Markers`, `captureDelay`, `endFrame`, `entities`, `events`, `missionAuthor`, `missionName`, `playersCount`, `worldName` (MEDIUM until corpus profiler phase formalizes this).
- Rust Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html and https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html (HIGH).
- Serde overview and serde_json docs: https://serde.rs/ and https://docs.rs/serde_json/latest/serde_json/struct.StreamDeserializer.html (HIGH).
- Context7 lookup for `serde_json`: `/serde-rs/json`, "Deserializer from_reader strongly typed JSON" (HIGH).
- RabbitMQ acknowledgements, publisher confirms, prefetch, and reliability: https://www.rabbitmq.com/docs/confirms and https://www.rabbitmq.com/docs/reliability (HIGH).
- Lapin AMQP client docs: https://docs.rs/lapin/latest/lapin/ and https://docs.rs/lapin/latest/lapin/struct.Channel.html (MEDIUM-HIGH; docs.rs current crate docs, but exact version should be pinned during stack planning).
- Context7 lookup for `lapin`: `/websites/rs_lapin`, "basic_consume basic_ack basic_qos basic_publish publisher confirms" (MEDIUM-HIGH).
- AWS SDK for Rust S3 examples and S3 checksum guidance: https://docs.aws.amazon.com/code-library/latest/ug/rust_1_s3_code_examples.html and https://docs.aws.amazon.com/sdkref/latest/guide/feature-dataintegrity.html (HIGH).
- Tokio `spawn_blocking` and graceful shutdown docs: https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html and https://tokio.rs/tokio/topics/shutdown (HIGH).
- Context7 lookup for Tokio: `/websites/rs_tokio_1_49_0`, "spawn_blocking graceful shutdown cancellation token" (HIGH).
- `tracing` structured diagnostics: https://docs.rs/tracing/latest/tracing/ (HIGH).
- `schemars`, `jsonschema`, `criterion`, and `insta` crate docs: https://docs.rs/schemars, https://docs.rs/jsonschema, https://docs.rs/criterion, https://docs.rs/insta (MEDIUM-HIGH; use exact versions during stack planning).
- JSON Schema 2020-12 core: https://json-schema.org/draft/2020-12/json-schema-core (HIGH).
- OCAP project overview: https://github.com/OCAP2/OCAP (MEDIUM; useful for domain context, local corpus remains authoritative for actual SolidGames replay shapes).

---
*Architecture research for: Rust OCAP JSON replay parser service*
*Researched: 2026-04-24*
