# Project Research Summary

**Project:** replays-parser-2
**Domain:** Rust OCAP JSON replay parser, compatibility harness, CLI, and RabbitMQ/S3 worker service
**Researched:** 2026-04-24
**Confidence:** MEDIUM-HIGH

## Executive Summary

`replays-parser-2` is a replacement parser for SolidGames OCAP JSON replays. Experts should build this as a deterministic parsing engine with thin runtime adapters: one pure Rust parser core, one versioned output contract, one local CLI, one RabbitMQ/S3 worker, and one first-class migration harness. The old TypeScript parser at `/home/afgan0r/Projects/SolidGames/replays-parser` is not optional background material. It is the required behavioral reference for v1 statistics semantics, legacy output fields, skip rules, name-compatibility behavior, runtime assumptions, and benchmark baselines.

The recommended approach is to start with corpus and legacy behavior discovery, then lock the output contract, then implement the Rust core and aggregate projection against executable parity tests. The parser should emit normalized observed facts with source references first, then derive legacy-compatible aggregate summaries from those normalized events. CLI and worker modes must call the same core; worker-specific RabbitMQ, S3, checksum, retry, and artifact concerns belong outside the parser core.

The main risks are semantic drift from the old parser, aggregate-only output that cannot be audited, accidental canonical identity matching inside the parser, nondeterministic JSON output, false 10x performance claims, and unsafe worker acknowledgement/checksum behavior. Mitigation is mostly roadmap order: prove old behavior and corpus shapes before contract design, make normalized events primary, keep canonical identity and PostgreSQL in `server-2`, require old-vs-new diff reports, benchmark identical file sets, and acknowledge RabbitMQ jobs only after durable artifact/result publication.

## Key Findings

### Recommended Stack

Use a Rust 2024 Cargo workspace with shared contract, core, CLI, worker, harness, tests, and benchmark code. Start with `serde_json` and typed Serde models for correctness; keep `simd-json` as an optional later optimization only after golden tests and benchmarks prove it is worth the extra complexity. Keep Node/pnpm entirely dev-only for running the old parser baseline.

The stack is mature for this service shape: `tokio` for worker async I/O, `clap` for CLI and env-backed worker config, `lapin` for RabbitMQ AMQP 0-9-1, official `aws-sdk-s3` for S3-compatible object access, `schemars` plus semver for versioned JSON Schema contracts, and `tracing` for structured diagnostics. Output determinism depends on discipline: use structs and `BTreeMap` where dynamic maps are needed, avoid `HashMap` leakage into serialized JSON, sort contract collections by stable keys, and avoid wall-clock timestamps in deterministic artifacts.

**Core technologies:**
- Rust stable 1.95.0, edition 2024: core language and binary toolchain - current enough for the chosen crate set; pin in `rust-toolchain.toml`.
- `serde` and `serde_json`: OCAP JSON decoding and output serialization - correctness-first default with strong ecosystem support.
- `tokio`: async runtime - needed for worker mode, S3, RabbitMQ, shutdown, and bounded async orchestration.
- `clap`: CLI and worker config - subcommands and env-backed deployment options.
- `lapin`: RabbitMQ client - fits AMQP 0-9-1 parse job queues with manual ack and publisher confirms.
- `aws-config` and `aws-sdk-s3`: S3-compatible object input/output - official SDK with custom endpoint and path-style support.
- `schemars`, `semver`, `sha2`, `hex`: contract schemas, version checks, and checksum validation - required for safe server integration.
- `tracing`, `tracing-subscriber`, `thiserror`, `anyhow`: structured logs and typed failures - structured parse failures must become product artifacts.
- `insta`, `similar-asserts`, `assert_cmd`, `proptest`, `criterion`, `hyperfine`, `testcontainers`: targeted snapshots, CLI tests, invariants, benchmarks, and integration tests.

**Critical version notes:**
- Use Rust 1.95.0 initially; latest AWS SDK crates imply an effective MSRV around Rust 1.91.1.
- Use `serde_json` without `preserve_order` in core output.
- Keep `simd-json` optional until the real OCAP corpus shows material speedup without output drift.

### Expected Features

v1 is credible only if it proves legacy behavior while exposing a better normalized artifact for `server-2`. The old parser's fields and rules must be mapped before the Rust contract is considered stable: kills, vehicle kills, kills from vehicles, teamkills, deaths, KD/score formulas, played games, weekly stats, squad/rotation outputs, relationship lists, connected-player backfill, same-name slot merging, game-type filtering, and skip behavior.

The product should not become a broader stats platform. Parser-owned output is observed replay facts, normalized events, source references, compatibility aggregates, structured failures, and bounty inputs. Canonical identity, PostgreSQL persistence, corrections, public display, Steam OAuth, rotations, and final bounty point calculation remain in `server-2` and `web`.

**Must have (table stakes):**
- Legacy parser parity contract - old `replays-parser` is the required behavioral reference.
- OCAP JSON ingestion - support the historical `~/sg_stats/raw_replays` corpus and observed top-level keys.
- Entity normalization - units, vehicles, sides, groups, observed names, and source entity IDs.
- Connected-player backfill and duplicate-slot merge compatibility - required for old aggregate parity.
- Kill/death/teamkill extraction - core event semantics for stats and bounty inputs.
- Vehicle kill context - preserve old `killsFromVehicle` and `vehicleKills` semantics while adding audit references.
- Observed identity preservation - no canonical player matching in parser output.
- Versioned normalized output contract - include parser/contract version, source checksum, job/replay metadata, and schema.
- Source references for audit - aggregate contributions must point back to replay/event/entity evidence.
- CLI parse mode - local deterministic reproduction without RabbitMQ/S3.
- Golden corpus and old-vs-new diff harness - parity must be executable, not asserted.
- Benchmark harness - measure 10x goal against the pinned old parser command on identical inputs.
- Worker mode - RabbitMQ parse requests, S3-compatible downloads, checksum verification, structured completed/failed results.
- Structured failures/skips and observability - machine-readable operator and retry decisions.

**Should have (competitive):**
- Dual artifact output - normalized events plus legacy-compatible aggregates.
- Rule-level provenance - named classification rules for enemy kill, teamkill, vehicle kill, ignored event, and compatibility behavior.
- Corpus schema profiler - turn historical OCAP drift into known data before broad implementation.
- Contract diff tooling - old vs new reports by replay, field, and rule category.
- Confidence-scored commander/winner extraction - distinguish known, unknown, inferred, and unparseable outcomes.
- Bounty input artifact - enemy-kill candidates with vehicle, side, replay, and exclusion context.
- Replay quality report and failure-only reruns - useful for migration triage.
- Idempotent artifact keys - keyed by contract version, replay ID, and checksum.

**Defer (v2+):**
- Full trajectory/position export - large payload, not needed for current stats.
- Additional replay formats - OCAP JSON parity is the v1 value.
- Streaming event API - current workload is deterministic batch parsing.
- Advanced anomaly detection - needs stable normalized events first.
- Parser-owned correction workflow - should likely remain in `server-2`.

### Architecture Approach

The architecture should be "pure core, imperative shell" plus a dedicated legacy harness. `parser-core` should accept bytes/readers and explicit metadata, decode tolerant raw OCAP shapes, normalize observed facts and events, derive aggregates from normalized events only, validate invariants, and return contract types. CLI, worker, golden tests, benchmarks, and comparison tools should all use that same core.

**Major components:**
1. Old parser reference - executable source of legacy behavior, command baselines, and stage mapping.
2. Legacy parity harness - old command runner/captured outputs, comparable-field diff reports, benchmark baselines, and mismatch taxonomy.
3. `parser-contract` - `ParseArtifact`, `ParseFailure`, request/result messages, source refs, unknown states, aggregate summaries, semver, JSON Schema.
4. `parser-core` - raw OCAP decode, entity index, normalized event pipeline, aggregate projection, invariant validation.
5. Raw OCAP adapter - tolerant tuple/shape layer that isolates source quirks from domain logic.
6. Normalization modules - metadata, entities, connected players, events, vehicle context, commander, outcome, and observed identity.
7. Source-reference builder - stable event/entity/frame pointers for audit and aggregate traceability.
8. CLI adapter - deterministic local parse/schema/compare commands and structured failure exit behavior.
9. Worker adapter - bounded RabbitMQ/S3 lifecycle, checksum validation, durable artifact write, completed/failed publish, safe ack/nack.
10. Golden and benchmark harness - corpus manifests, fixtures, parity reports, repeatability checks, and throughput/RSS reports.

### Critical Pitfalls

1. **Treating the old parser as either absolute truth or disposable legacy** - create an old-parser behavior dossier, pin command/commit/environment, and classify every mismatch as compatible, intentional change, old bug preserved, old bug fixed, new bug, insufficient data, or human review.
2. **Designing around aggregates instead of normalized source events** - make normalized events the primary artifact and derive aggregates from event IDs/source refs only.
3. **Collapsing observed identity into canonical identity** - emit observed names, SteamIDs, sides, groups, entity IDs, and typed unknowns; isolate old `nameChanges.csv` behavior to compatibility projections.
4. **Hard-coding OCAP tuple offsets and misclassifying event semantics** - use a raw adapter layer, fixture every known event/entity shape, and test enemy kills, teamkills, suicides, unknown actors, vehicle kills, and disconnected/deleted entities.
5. **Trusting runtime defaults for determinism, benchmarks, and worker reliability** - define stable output order, benchmark identical release-mode old/new workloads, validate S3 checksums, and ack RabbitMQ jobs only after durable artifact/result publication.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Legacy Reference and Corpus Baseline

**Rationale:** This must come first because the old parser is the required behavioral reference. A clean Rust parser that does not match current SolidGames semantics is a failed migration.

**Delivers:** old-parser behavior dossier, old repo commit/command/env baseline, `~/sg_stats` corpus manifest, fixture taxonomy, OCAP shape inventory, old output hash/run logs, old stage to new responsibility mapping, mismatch taxonomy.

**Addresses:** legacy parser parity contract, old parser baseline runner, golden corpus foundation, game-type/skip behavior discovery.

**Avoids:** ignoring old behavior, copying legacy assumptions without labels, hard-coded OCAP shape assumptions, false benchmark targets.

### Phase 2: Output Contract and Compatibility Semantics

**Rationale:** `server-2` integrates with contract artifacts, not parser internals. The contract must be grounded in Phase 1 before core implementation hardens around the wrong shape.

**Delivers:** Cargo workspace skeleton, `parser-contract`, `ParseArtifact`, `ParseFailure`, request/result message types, source refs, explicit unknown/null states, semver policy, JSON Schema generation, old-field compatibility map, observed-identity boundary, aggregate traceability requirements.

**Uses:** `serde`, `schemars`, `semver`, `sha2`, deterministic structs/ordered maps.

**Implements:** public API boundary between parser, worker, harness, and `server-2`.

**Avoids:** aggregate-only output, canonical identity leakage, false winner/SteamID facts, version-as-string-only contracts, parser scope creep into `server-2`.

### Phase 3: Deterministic Parser Core Foundation

**Rationale:** The parser core should be implemented and tested without RabbitMQ/S3 so correctness does not get mixed with transport failures.

**Delivers:** `parser-core`, typed/tolerant raw OCAP decode, replay metadata, entity and vehicle indexes, observed player/entity normalization, connected-player backfill, duplicate-slot compatibility behavior, deterministic serialization rules, structured parse warnings/failures for unknown shapes.

**Uses:** `serde_json`, `serde_path_to_error`, `thiserror`, deterministic ordering rules, typed raw DTOs.

**Implements:** pure core API used by CLI, worker, golden tests, benchmarks, and harness.

**Avoids:** parser panics, nondeterministic output, memory-heavy cloning, raw OCAP quirks leaking into contract types.

### Phase 4: Event Semantics and Aggregate Projection

**Rationale:** Kill/death/teamkill semantics and aggregate formulas are where most migration risk lives. These must be derived from normalized events, not raw tuples.

**Delivers:** normalized kill/death/teamkill events, vehicle context, death classification, commander-side candidates, winner/outcome unknown/inferred handling, bounty input artifact, legacy-compatible aggregate projection for player/squad/rotation/weekly/weapon/vehicle/relationship fields, rule-level provenance.

**Uses:** Phase 1 old parser behavior mapping and Phase 2 contract source refs.

**Implements:** aggregate-from-normalized-events architecture pattern.

**Avoids:** teamkill/enemy-kill mistakes, vehicle attribution drift, aggregate totals without audit trail, missing outcome treated as negative fact.

### Phase 5: CLI, Golden Parity, and Benchmarks

**Rationale:** Local reproducibility and measurable parity should gate service integration. The 10x goal is only meaningful after old and new run on identical file sets with comparable output.

**Delivers:** CLI parse/schema/compare commands, structured CLI failures, curated fixtures, full-corpus/manual comparison mode, old-vs-new diff reports, deterministic repeatability tests, Criterion parser benchmarks, hyperfine old/new command benchmarks, memory/RSS measurements, initial 10x report.

**Uses:** `clap`, `assert_cmd`, `insta`, `similar-asserts`, `criterion`, `hyperfine`.

**Implements:** migration acceptance gate for parser correctness and performance.

**Avoids:** stale `~/sg_stats/results` as sole oracle, pass/fail-only golden reports, debug-build speed claims, parser-only benchmark comparisons against old full pipeline.

### Phase 6: RabbitMQ/S3 Worker and Durable Artifacts

**Rationale:** Worker mode should come after core parity is credible. Queue/storage bugs should not hide parser semantic bugs.

**Delivers:** worker config, RabbitMQ consumer with manual ack and bounded prefetch, parse request validation, S3-compatible download, checksum/size verification, artifact write under deterministic key, `parse.completed`/`parse.failed` publish with confirms, nack/retry/DLQ behavior, structured worker logs.

**Uses:** `tokio`, `lapin`, `aws-sdk-s3`, `tempfile`, `tracing`, `tokio-util`, `testcontainers`.

**Implements:** server integration path while preserving parser ownership boundaries.

**Avoids:** auto-ack job loss, S3 ETag/checksum mistakes, oversized RabbitMQ result payloads, parser direct PostgreSQL writes, unbounded worker memory.

### Phase 7: Operational Hardening and Container Readiness

**Rationale:** Production readiness depends on measured resource limits, failure behavior, and observable worker lifecycle after integration exists.

**Delivers:** Docker image, env/config validation, health/readiness, graceful shutdown, in-flight job handling, metrics/log fields, malformed/hostile replay tests, corrupt/missing/oversized object tests, duplicate/redelivery tests, resource limits, production runbook notes.

**Uses:** `tracing`, Tokio shutdown primitives, container integration tests, measured RSS/throughput from Phase 5.

**Implements:** service hardening for `server-2` integration and repeatable replay reprocessing.

**Avoids:** crashes on malformed replay content, secret leakage in failure payloads, poison-message loops, health checks that only mean "process is alive".

### Phase Ordering Rationale

- Legacy behavior and corpus profiling precede contract design because the old parser defines current SolidGames statistics semantics.
- Contract precedes core because `server-2`, tests, and harnesses need a stable artifact boundary with source refs and unknown states.
- Core precedes worker because parsing correctness must be transport-independent.
- Event semantics and aggregate projection follow raw/entity normalization because kill events depend on entity and side context.
- CLI, golden parity, and benchmarks are the migration gate before RabbitMQ/S3 infrastructure.
- Worker integration is late because it is an adapter around a proven core.
- Operational hardening is last because resource limits and health behavior depend on measured parser and worker behavior.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1:** local old-parser behavior and corpus profiling are project-specific; run `/gsd-research-phase` or an equivalent spike if old command execution, output paths, skip rules, or corpus shape inventory are unclear.
- **Phase 4:** commander-side/winner extraction, temporal vehicle context, and exact legacy formulas need deeper corpus-backed research before locking acceptance criteria.
- **Phase 6:** exact `server-2` RabbitMQ exchange/routing keys, retry/DLX policy, checksum algorithm, object key conventions, and artifact payload/reference choice need integration research with `server-2`.
- **Phase 7:** resource limits, container health semantics, and S3-compatible behavior should be validated against the actual deployment target, not only AWS docs.

Phases with standard patterns where generic research can usually be skipped:
- **Phase 2:** Rust contract crates, JSON Schema generation, semver policy, and Serde modeling are well-documented; focus planning on domain schema decisions.
- **Phase 3:** Serde-based raw adapters and pure-core architecture are standard; use local corpus findings rather than more external research.
- **Phase 5:** CLI tests, snapshots, Criterion, and hyperfine have established patterns; uncertainty is in fixture selection and old parser baseline, not tooling.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core Rust, Serde, Tokio, Clap, RabbitMQ, S3, schema, testing, and benchmark choices are backed by official docs and current crate metadata. RabbitMQ/S3 operational details remain dependent on `server-2` contracts. |
| Features | MEDIUM-HIGH | Strong local project, old parser, and corpus evidence supports table stakes. Commander/winner inference and some future aggregate inputs still need representative examples. |
| Architecture | HIGH | Service boundaries, pure core/adapters, contract-first output, old-parser harness, and `server-2` ownership boundaries are consistent across research files. Exact crate split can remain pragmatic during implementation. |
| Pitfalls | HIGH | Risks are concrete and project-specific, especially old-parser compatibility, source refs, observed identity, deterministic output, worker ack/checksum, and benchmark validity. Exact OCAP edge cases need corpus profiling. |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- Old parser executable baseline: pin exact repo commit, Node version, pnpm version, command (`pnpm run parse` vs `pnpm run parse:dist`), environment, worker count, inputs, logs, and output hashes.
- Old/new comparison tolerances: define per legacy field which differences are exact failures, tolerated formatting drift, intentional new-contract changes, or old bugs preserved for compatibility.
- Final contract names and field types: choose schema names for observed identity, source refs, unknown states, events, aggregates, failures, and result messages.
- Payload vs artifact result path: research recommends S3 artifact reference as default, but `server-2` must confirm message body limits and artifact ownership.
- RabbitMQ policy: exchange names, routing keys, queue names, prefetch, retry attempts, backoff, DLQ, and publisher-confirm requirements are not finalized.
- S3-compatible storage behavior: endpoint, bucket, path-style mode, checksum algorithm, size limits, and corrupt-object handling must be validated against the chosen service.
- Commander-side and winner examples: need representative replays with present, absent, free-text, and ambiguous outcome/KS data.
- Performance target baseline: the 10x goal must be measured against equivalent old/new workloads in release mode with bytes/sec, events/sec, RSS, and output parity status.

## Sources

### Primary (HIGH confidence)

- `.planning/PROJECT.md` - project scope, constraints, old parser requirement, `server-2` boundary, corpus context.
- `.planning/research/STACK.md` - stack versions, crate choices, testing/benchmark tools, worker integration recommendations.
- `.planning/research/FEATURES.md` - table-stakes features, differentiators, anti-features, MVP definition, dependency map.
- `.planning/research/ARCHITECTURE.md` - service boundaries, component responsibilities, Cargo workspace shape, old parser migration map, build order.
- `.planning/research/PITFALLS.md` - critical pitfalls, phase mapping, integration gotchas, recovery strategies.
- `/home/afgan0r/Projects/SolidGames/replays-parser` - required old parser behavioral reference. Key areas include `package.json`, `docs/architecture.md`, `src/start.ts`, `src/index.ts`, `src/2 - parseReplayInfo/*`, `src/3 - statistics/*`, and `src/4 - output/*`.
- `~/sg_stats/raw_replays`, `~/sg_stats/results`, `~/sg_stats/lists/replaysList.json` - historical corpus and old result baseline; observed around 3,938 raw replay JSON files.
- Official Rust/Cargo/Serde/Tokio/Clap docs - workspace, serialization, async runtime, and CLI patterns.
- RabbitMQ docs - consumer acknowledgements, publisher confirms, prefetch, reliability, negative ack, and DLX behavior.
- AWS SDK for Rust and S3 object integrity docs - endpoint customization, S3-compatible access, and checksum guidance.
- JSON Schema 2020-12, RFC 8259 JSON, and Semantic Versioning 2.0.0 - contract validation, JSON ordering/semantics, and version compatibility rules.
- Criterion, hyperfine, insta, schemars, testcontainers docs - benchmark, snapshot/golden, schema, and integration-test tooling.

### Secondary (MEDIUM confidence)

- OCAP2 README, wiki, changelog, and extension docs - domain context for OCAP event/model behavior; local SolidGames corpus remains authoritative for v1 shape coverage.
- Lapin docs.rs and Context7 lookup - RabbitMQ Rust client API and patterns; exact version and runtime behavior should be pinned during implementation.
- OpenTelemetry RabbitMQ semantic conventions - useful for metrics naming if observability expands beyond `tracing`.

### Tertiary (LOW confidence)

- Generic approval/golden-master testing references - useful pattern vocabulary, but implementation should use Rust-native fixtures, snapshots, and diff reports.

---
*Research completed: 2026-04-24*
*Ready for roadmap: yes*
