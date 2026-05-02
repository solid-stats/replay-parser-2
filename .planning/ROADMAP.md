# Roadmap: replay-parser-2

## Overview

This roadmap replaces the legacy TypeScript replay parser with a deterministic Rust parser that is grounded in the old parser's behavior, emits a versioned contract for `server-2`, proves parity and speed against `~/sg_stats`, and then exposes the same parser core through local CLI and RabbitMQ/S3 worker modes.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Legacy Baseline and Corpus** - Pin the old parser baseline and historical corpus facts that define v1 parity. (completed 2026-04-25)
- [x] **Phase 2: Versioned Output Contract** - Define the stable parse artifact, failure, schema, unknown-state, and source-reference contract. (completed 2026-04-26)
- [x] **Phase 3: Deterministic Parser Core** - Parse OCAP JSON into deterministic normalized metadata and observed entity facts. (completed 2026-04-27)
- [x] **Phase 4: Event Semantics and Aggregates** - Normalize combat/outcome semantics and derive auditable legacy, bounty, and vehicle score aggregates. (completed 2026-04-28)
- [ ] **Phase 5: CLI, Golden Parity, Benchmarks, and Coverage Gates** - Make local parsing, schema export, old-vs-new comparison, fixtures, determinism checks, 100% coverage gates, and speed reports executable. (execution complete 2026-04-28; verification gap escalated into Phase 5.1 redesign)
- [ ] **Phase 5.1: Compact Artifact and Selective Parser Redesign** (INSERTED) - Redesign the default artifact as compact server-facing output, remove full normalized event/entity dumps from ordinary ingestion, make comparison reports human-reviewable, and implement/select a selective parsing path that can meet the 10x target. (execution complete 2026-04-29; benchmark/parity acceptance gap blocks Phase 6)
- [x] **Phase 5.2: Minimal Artifact and Performance Acceptance** (INSERTED) - Replace the compact artifact with minimal v1 statistics output, retire issue #13 vehicle score from v1, add debug sidecar output, and record accepted selected/all-raw benchmark evidence before worker integration. Execution is complete; on 2026-05-02 the product owner accepted current performance, p95 > 10% as non-blocking, and the 4 known malformed/non-JSON all-raw failures when old/new error parity matches.
- [ ] **Phase 6: RabbitMQ/S3 Worker Integration** - Consume parse jobs, fetch objects, verify checksums, publish results, and use safe queue acknowledgement.
- [ ] **Phase 7: Parallel and Container Hardening** - Prove multi-worker safety and container-ready observability.

## Phase Details

### Phase 1: Legacy Baseline and Corpus
**Goal**: Developers can reproduce and inspect the legacy parser and historical data baseline that define v1 parity.
**Depends on**: Nothing (first phase)
**Requirements**: DOC-01, DOC-02, WF-01, WF-02, WF-03, WF-04, WF-05, INT-01, INT-02, INT-03, INT-04, LEG-01, LEG-02, LEG-03, LEG-04, LEG-05
**Success Criteria** (what must be TRUE):
  1. Repository has a current `README.md` that documents project purpose, scope, current GSD phase, architecture direction, validation data, and the AI + GSD-only development workflow.
  2. Completed GSD/agent work leaves a clean git working tree by committing intended results, never by deleting or discarding completed work; unclear cases are escalated to the user.
  3. Agents challenge instructions that conflict with architecture, current logic, quality standards, maintainability, or proportional scope; they explain the risk and propose safer alternatives instead of blindly complying.
  4. Agents can identify Solid Stats as a multi-project product made of `replays-fetcher`, `replay-parser-2`, `server-2`, and `web`, and apply product-wide GSD workflow rules across those projects.
  5. Agents use risk-based cross-application compatibility checks: local docs/briefs for local-only changes, adjacent app docs/repos or user confirmation for contract, ingest staging/source identity assumptions, queue/storage, API/data, identity/auth, moderation, or UI-visible changes.
  6. Developer can run the pinned old parser baseline and see the command, commit, runtime versions, environment inputs, worker count, logs, and output locations used for parity.
  7. Developer can inspect a corpus manifest for `~/sg_stats/raw_replays`, `~/sg_stats/results`, and `~/sg_stats/lists/replaysList.json`.
  8. Developer can inspect documented old parser game-type filters, skip rules, exclusions, and config inputs.
  9. Developer can classify any old-vs-new difference using the agreed mismatch taxonomy.
**Plans**: 5 plans
**Execution waves**: Wave 1 runs `01-00-PLAN.md`; Wave 2 runs `01-01-PLAN.md`, `01-02-PLAN.md`, and `01-03-PLAN.md`; Wave 3 runs `01-04-PLAN.md`.
Plans:
- [x] 01-00-PLAN.md — Generated-artifact hygiene and canonical source-command gate.
- [x] 01-01-PLAN.md — Non-destructive isolated old-parser baseline command/runtime evidence.
- [x] 01-02-PLAN.md — Full-history corpus manifest, profile evidence, and fixture index.
- [x] 01-03-PLAN.md — Legacy filters, skip rules, config inputs, identity compatibility, and output surfaces.
- [x] 01-04-PLAN.md — Mismatch taxonomy, cross-app interface notes, README update, and final coverage checks.

### Phase 2: Versioned Output Contract
**Goal**: `server-2` and parser tooling can rely on a stable, machine-checkable parse artifact and failure contract.
**Depends on**: Phase 1
**Requirements**: OUT-01, OUT-02, OUT-03, OUT-04, OUT-05, OUT-06, OUT-07
**Success Criteria** (what must be TRUE):
  1. Developer can validate a current `ParseArtifact` JSON document that includes parser version, contract version, replay/source identifiers, checksum, and parse status metadata.
  2. Server integrator can consume normalized replay metadata, observed identity fields, and explicit unknown/null states without canonical player matching.
  3. Developer can trace normalized events and aggregate contributions back to replay, frame, event index, entity ID, and rule ID where available.
  4. Developer can validate structured `ParseFailure` output with job/replay/file identifiers, stage, error code, message, retryability, and source cause.
**Plans**: 6 plans
**Execution waves**: Wave 1 runs `02-00-PLAN.md`; Wave 2 runs `02-01-PLAN.md`; Wave 3 runs `02-02-PLAN.md`; Wave 4 runs `02-03-PLAN.md`; Wave 5 runs `02-04-PLAN.md`; Wave 6 runs `02-05-PLAN.md`.
Plans:
- [x] 02-00-PLAN.md — Rust workspace and contract crate foundation.
- [x] 02-01-PLAN.md — ParseArtifact envelope, status metadata, source identity, diagnostics, and success example.
- [x] 02-02-PLAN.md — Replay metadata, observed identity, and explicit presence semantics.
- [x] 02-03-PLAN.md — Source references, normalized event skeleton, aggregate contribution references, and rule IDs.
- [x] 02-04-PLAN.md — Structured failures, generated schema, validated examples, README handoff, and final checks.
- [x] 02-05-PLAN.md — Gap closure for checksum, failure, source-reference, error-code, and confidence invariants.

### Phase 3: Deterministic Parser Core
**Goal**: The Rust parser core can read historical OCAP JSON and return deterministic normalized metadata and observed entity facts without transport concerns.
**Depends on**: Phase 2
**Requirements**: OUT-08, PARS-01, PARS-02, PARS-03, PARS-04, PARS-05, PARS-06, PARS-07
**Success Criteria** (what must be TRUE):
  1. Developer can parse representative historical OCAP JSON files and receive normalized replay metadata from observed top-level fields.
  2. Developer can inspect normalized unit/player, vehicle, and static weapon entities with source IDs, observed names/classes, side/group/role fields, player flags, and available identity fields.
  3. Known OCAP schema drift results in structured warnings, explicit unknown states, or structured failures instead of parser panics.
  4. Repeated parser-core runs on the same input and contract version produce stable JSON ordering.
  5. Connected-player backfill and duplicate-slot same-name compatibility behavior are preserved for later aggregate projection while raw observed identifiers remain available.
**Plans**: 6 plans
**Execution waves**: Wave 1 runs `03-00-PLAN.md`; Wave 2 runs `03-01-PLAN.md`; Wave 3 runs `03-02-PLAN.md`; Wave 4 runs `03-03-PLAN.md`; Wave 5 runs `03-04-PLAN.md`; Wave 6 runs `03-05-PLAN.md`.
Plans:
- [x] 03-00-PLAN.md — Contract extension for typed observed entity facts and compatibility hints.
- [x] 03-01-PLAN.md — Parser-core crate foundation, pure API, and structured failure shell.
- [x] 03-02-PLAN.md — Tolerant OCAP root decode and replay metadata normalization.
- [x] 03-03-PLAN.md — Observed unit/player, vehicle, and static weapon entity normalization.
- [x] 03-04-PLAN.md — Schema-drift diagnostics, partial status policy, and deterministic output tests.
- [x] 03-05-PLAN.md — Connected-player backfill, duplicate-slot hints, README handoff, and final quality gates.

### Phase 4: Event Semantics and Aggregates
**Goal**: Users of the parse artifact can audit normalized combat/outcome events and derived aggregate summaries, including vehicle score from GitHub issue #13.
**Depends on**: Phase 3
**Requirements**: PARS-08, PARS-09, PARS-10, PARS-11, AGG-01, AGG-02, AGG-03, AGG-04, AGG-05, AGG-06, AGG-07, AGG-08, AGG-09, AGG-10, AGG-11
**Success Criteria** (what must be TRUE):
  1. Developer can inspect normalized kill, death, teamkill, suicide, null-killer, player-killed, vehicle-killed, vehicle-context, commander-side, and winner/outcome semantics.
  2. Developer can inspect legacy-compatible aggregate summaries for player, squad, rotation, weekly, score, vehicle, and relationship fields derived from normalized events and source references.
  3. Bounty calculation inputs include valid enemy-kill facts with killer/victim identity, frame/time, side, replay, and vehicle/infantry context, while teamkills remain auditable but do not award bounty inputs.
  4. Vehicle score contributions use only kills from vehicles, apply the issue #13 attacker-vehicle by killed-entity weight matrix, subtract weighted teamkill penalties, divide by games with at least one vehicle kill, and clamp teamkill penalty multipliers below 1 up to 1.
  5. Every vehicle score contribution exposes source references that let `server-2` audit or recalculate the score.
**Plans**: 7 plans
**Execution waves**: Wave 1 runs `04-00-PLAN.md`; Wave 2 runs `04-01-PLAN.md`; Wave 3 runs `04-02-PLAN.md`; Wave 4 runs `04-03-PLAN.md`; Wave 5 runs `04-04-PLAN.md` and `04-05-PLAN.md`; Wave 6 runs `04-06-PLAN.md`.
Cross-cutting constraints:
- Normalized events and source references are the primary artifact; aggregate projections must derive from auditable contribution refs.
- Parser output preserves observed identity and legacy compatibility keys only; canonical player identity, PostgreSQL persistence, public APIs, and UI presentation stay in adjacent apps.
- Missing commander/winner data is represented as explicit unknown and does not by itself make an artifact partial.
- Vehicle score contributions must preserve raw evidence, issue #13 categories, raw/applied weights, denominator inputs, and source refs.
Plans:
- [x] 04-00-PLAN.md — Contract extensions for combat payloads, aggregate contribution values, vehicle score evidence, and replay-side facts.
- [x] 04-01-PLAN.md — Raw killed-event tuple accessors and event source references.
- [x] 04-02-PLAN.md — Combat event normalization for kills, deaths, teamkills, suicides, null killers, vehicle victims, and unknown actors.
- [x] 04-03-PLAN.md — Legacy per-replay projections, relationships, game-type compatibility metadata, squad/rotation inputs, and bounty inputs.
- [x] 04-04-PLAN.md — Issue #13 vehicle score taxonomy, weights, contributions, denominator inputs, and teamkill clamp tests.
- [x] 04-05-PLAN.md — Typed commander-side and winner/outcome facts with conservative known/unknown/candidate semantics.
- [x] 04-06-PLAN.md — Schema/example refresh, deterministic populated artifact tests, README handoff, and final quality gates.

### Phase 5: CLI, Golden Parity, Benchmarks, and Coverage Gates
**Goal**: Developers can reproduce parser results locally, compare against the old parser, enforce 100% reachable-code coverage, and measure the 10x target on equivalent workloads.
**Depends on**: Phase 4
**Requirements**: CLI-01, CLI-02, CLI-03, CLI-04, TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, TEST-06, TEST-07, TEST-08, TEST-09, TEST-10, TEST-11, TEST-12
**Success Criteria** (what must be TRUE):
  1. User can parse a local OCAP JSON file to a requested output path and request the current contract schema from the CLI.
  2. User receives structured error JSON and a non-zero exit status for bad, unreadable, or unsupported replay files.
  3. Developer can run old-vs-new comparison on selected replay files or saved old output artifacts and receive per-field mismatch categories.
  4. Golden fixtures cover representative normal, bad, partial, old-shape, winner-present, winner-missing, vehicle-kill, teamkill, and commander-side cases where available.
  5. CI blocks release unless all reachable production Rust code reports 100% statement, branch, function, and line coverage, with every exclusion explicitly justified and allowlisted.
  6. Unit and regression tests follow the `unit-tests-philosophy` RITE/AAA standard, cover behavior-level success, edge, error, malformed, compatibility, determinism, and source-reference scenarios, and use strong observable oracles.
  7. Mutation-testing or equivalent fault-injection reporting covers parser-core and aggregate logic, and high-risk survivors are either fixed by stronger tests or documented as non-applicable.
  8. Benchmark reports include parse-only, aggregate-only, and end-to-end throughput plus memory/RSS where practical, parity status for the measured sample, and whether the roughly 10x target is met.
**Plans**: 6 plans
**Execution waves**: Wave 1 runs `05-00-PLAN.md`; Wave 2 runs `05-01-PLAN.md`; Wave 3 runs `05-02-PLAN.md`; Wave 4 runs `05-03-PLAN.md`; Wave 5 runs `05-04-PLAN.md`; Wave 6 runs `05-05-PLAN.md`.
Cross-cutting constraints:
- Public local command names are `replay-parser-2 parse`, `replay-parser-2 schema`, and `replay-parser-2 compare`; the old `sg-replay-parser` name remains legacy baseline context only.
- Golden parity and old-parser compatibility logic live in CLI/harness adapters, not parser-core, and unexplained current-vs-regenerated drift remains `human review`.
- Coverage and mutation/fault gates must prove behavior through public APIs with reviewable allowlists only for generated, impossible, or defensive unreachable code.
- Benchmark reports must include workload identity, parity status, deterministic old baseline profile, throughput/memory evidence, and 10x pass/fail/unknown status before any performance claim.
Plans:
- [x] 05-00-PLAN.md — Public CLI binary, parse/schema commands, structured failure artifacts, and CLI tests.
- [x] 05-01-PLAN.md — Compact golden fixture manifest, curated fixtures, and behavior regression tests.
- [x] 05-02-PLAN.md — Selected-input comparison harness and `replay-parser-2 compare` reports.
- [x] 05-03-PLAN.md — `cargo llvm-cov` coverage gate, allowlist validation, and behavior-test strengthening.
- [x] 05-04-PLAN.md — Mutation or equivalent fault-injection report gate.
- [x] 05-05-PLAN.md — Benchmark reports, README command handoff, and final quality gates.

Current verification gap:
- `scripts/benchmark-phase5.sh --ci` now runs curated selected old/new evidence when `/home/afgan0r/Projects/SolidGames/replays-parser` and `~/sg_stats` are available. The latest generated report is structurally valid but records `ten_x_status=fail`, `parity_status=human_review`, and only a small speedup. UAT also found that this benchmark compared parsing a single replay, while the product-relevant performance claim must be based on parsing the whole replay list/corpus. The current artifact mostly reserializes large replay data and comparison output is too large for practical review. Phase 5.1 is inserted to fix the artifact, comparison, and parser-performance direction before worker integration.

### Phase 5.1: Compact Artifact and Selective Parser Redesign
**Goal**: `server-2` receives a compact, deterministic parser result that contains the statistics and minimal contribution evidence it needs, while parser performance is rebuilt around selective OCAP extraction instead of full JSON-to-JSON translation.
**Depends on**: Phase 5
**Requirements**: OUT-09, OUT-10, PARS-12, TEST-06, TEST-13, TEST-14
**Status**: Execution complete with benchmark/parity acceptance gap after Phase 5 UAT rejection.
**Success Criteria** (what must be TRUE):
  1. The default server-facing artifact excludes full normalized event/entity dumps and contains only replay/source metadata, observed participant references, aggregate/stat contribution inputs, bounty/vehicle-score inputs, diagnostics, and schema/version data needed by `server-2`.
  2. Any heavy event/entity/audit detail is optional debug/parity output or raw-replay reprocessing material, not required for ordinary worker ingestion.
  3. Annual/yearly nomination statistics do not force a large second v1 artifact; when that v2 product surface is revisited, it can reprocess raw OCAP files and compare against `~/sg_stats/year_results`.
  4. The parser has a selective extraction path or an accepted implementation plan that avoids unnecessary full-DOM parse/clone/serialize work for v1 statistics.
  5. Benchmark reports include raw input size, default artifact size, parse-only throughput, aggregate-only throughput, end-to-end throughput, memory/RSS where practical, parity status, and explicit 10x pass/fail evidence for both selected single-replay checks and whole-list/corpus parsing.
  6. Comparison reports are summary-first and reviewable by a human, with top mismatches, counts by category/impact, and detailed machine-readable evidence separated from the default review surface.
**Plans**: 8 plans
Plans: 8 plans
**Execution waves**: Wave 1 runs `05.1-00-PLAN.md`; Wave 2 runs `05.1-01-PLAN.md`; Wave 3 runs `05.1-02-PLAN.md`; Wave 4 runs `05.1-03-PLAN.md`; Wave 5 runs `05.1-04-PLAN.md` and `05.1-05-PLAN.md`; Wave 6 runs `05.1-06-PLAN.md`; Wave 7 runs `05.1-07-PLAN.md`.
Cross-cutting constraints:
- This phase may revise the Phase 2-5 contract and harness decisions, but must preserve observed identity boundaries and keep canonical player matching, PostgreSQL persistence, public APIs, UI, and yearly nomination product behavior outside this parser.
- Worker Phase 6 must not proceed until the compact artifact and selective parsing direction are planned and accepted.
- Parser contract changes in this phase require `server-2` compatibility review or an explicit user decision because they alter the artifact shape that worker integration will deliver.
- The default artifact must use compact `participants`, `facts`, `summaries`, `side_facts`, diagnostics, status/failure, source, parser, and contract metadata; full top-level `entities` and `events` dumps are not supported default output.
- The normal parser path must avoid full `serde_json::Value` DOM decode and preserve source refs, rule IDs, event index, frame, entity ID, and JSON path evidence.
- Parser emits compact bounty and vehicle-score facts only; final bounty points and cross-replay vehicle score calculation remain `server-2`/parity responsibilities.
Plans:
- [x] 05.1-00-PLAN.md — Server compatibility review and compact contract implementation gate.
- [x] 05.1-01-PLAN.md — Compact contract envelope, schema, examples, and status/failure invariants.
- [x] 05.1-02-PLAN.md — Selective OCAP root/entity/event extraction without full DOM normal path.
- [x] 05.1-03-PLAN.md — Compact participant refs, combat facts, contribution facts, summaries, side facts, and determinism.
- [x] 05.1-04-PLAN.md — CLI parse/schema/golden docs for compact default output.
- [x] 05.1-05-PLAN.md — Summary-first comparison reports with optional structured detail evidence.
- [x] 05.1-06-PLAN.md — Compact artifact-size and selected plus whole-list/corpus benchmark evidence.
- [x] 05.1-07-PLAN.md — Final quality gates, README/ROADMAP/STATE handoff, and Phase 6 blocker status.

Execution outcome:
- Final code gates passed: format, clippy, workspace tests, docs, coverage smoke, fault report, benchmark report validation, compact boundary grep, and whitespace checks.
- The compact artifact shape and selective parser boundary are implemented and server-2 compatibility was accepted by the user through `05.1-SERVER-COMPATIBILITY.md`.
- The generated benchmark report is valid but not a 10x/parity acceptance pass: selected `ten_x_status=unknown`, selected `parity_status=not_run`, whole-list/corpus evidence is unavailable because `RUN_PHASE5_FULL_CORPUS` was not enabled, and selected artifact/raw ratio is `59.97366881` on the tiny CI fixture.
- Phase 5.1 benchmark/parity gap was superseded by Phase 5.2 minimal-artifact work and the 2026-05-02 benchmark acceptance update.

### Phase 5.2: Minimal Artifact and Performance Acceptance (INSERTED)
**Goal**: The default parser output is reduced to a minimal v1 statistics artifact, issue #13 vehicle score is removed from v1, and accepted benchmark evidence is recorded before worker integration.
**Depends on**: Phase 5.1
**Requirements**: OUT-09, OUT-10, OUT-11, OUT-12, PARS-12, AGG-12, TEST-06, TEST-13, TEST-14, TEST-15
**Status**: Execution complete; code quality gates pass, benchmark gaps accepted by product owner on 2026-05-02.
**Success Criteria** (what must be TRUE):
  1. The default artifact uses compact tables: `players[]`, `weapons[]`, `destroyed_vehicles[]`, and `diagnostics[]`; player-authored enemy/team kill rows are nested under the killer `players[].kills`, and the artifact does not include full normalized event/entity dumps, source-ref dumps, or vehicle-score sections.
  2. `players[].kills` and `destroyed_vehicles[]` include identity and context needed for current stats and bounty inputs: killer/victim observed player references, enemy/teamkill classification, weapon, attacker vehicle, and destroyed vehicle/entity type. Suicide, null-killer, and unknown deaths remain player counters instead of bounty rows.
  3. Frame/time, event indexes, entity snapshots, source references, rule IDs, and normalized event/entity evidence are emitted only through an explicit debug sidecar mode such as `--debug-artifact <path>`, not through ordinary ingestion output.
  4. GitHub issue #13 vehicle score and `vehicle_score` output are removed from the v1 contract, schema, examples, tests, docs, and planning requirements; v1 still preserves kills-from-vehicle, vehicle-kill, weapon/vehicle context, and destroyed-vehicle facts needed by current stats and future raw replay reprocessing.
  5. Benchmark reports capture current old/new baseline evidence on selected and all-raw workloads; historical x3/x10 statuses remain reported, but the current measured performance is accepted by the product owner.
  6. The all-raw corpus gate attempts every file in `~/sg_stats/raw_replays`, reports wall time, files/sec, failure/skip counts, and passes failure compatibility when there are zero failed/skipped artifacts or when failures match a user-approved malformed/non-JSON allowlist and the cached old baseline reports the same failure count.
  7. Successful all-raw artifacts satisfy the accepted default artifact-size gate: every successful default artifact is <= 100 KB (100,000 bytes) and `oversized_artifact_count == 0`; median and p95 ratios remain report-only trend evidence.
  8. Product-owner compatibility acceptance is recorded: `server-2` will adapt later to the minimal artifact, while parser still does not own canonical identity, PostgreSQL persistence, public APIs, UI behavior, or bounty payout calculation.
**Plans**: 7 plans
**Execution waves**: Wave 1 runs `05.2-00-PLAN.md`; Wave 2 runs `05.2-01-PLAN.md`; Wave 3 runs `05.2-02-PLAN.md`; Wave 4 runs `05.2-03-PLAN.md`; Wave 5 runs `05.2-04-PLAN.md`; Wave 6 runs `05.2-05-PLAN.md`; Wave 7 runs `05.2-06-PLAN.md`.
Cross-cutting constraints:
- Parser contract changes require `05.2-SERVER-COMPATIBILITY.md` acceptance before implementation changes the default artifact shape.
- The default artifact must use `players[]`, `weapons[]`, `destroyed_vehicles[]`, and `diagnostics[]`, with player-authored enemy/team kill rows nested under the killer `players[].kills`; debug-only source refs, rule IDs, frame/time, event indexes, entity snapshots, and normalized event snapshots are not default output.
- GitHub issue #13 vehicle score is removed from active v1 contract, parser-core, schema, examples, tests, docs, and benchmark/comparison surfaces while ordinary `vehicleKills`, `killsFromVehicle`, weapon, attacker vehicle, and destroyed-vehicle facts remain.
- Debug sidecar output is explicit internal tooling through `--debug-artifact <path>` and must not contaminate default parser performance or server-facing contract guarantees.
- Phase 6 can proceed after Phase 5.2 compatibility acceptance plus accepted benchmark evidence: current performance accepted, hard 100 KB max artifact acceptance, and accepted malformed-file parity for the 4 known bad raw files.

Plans:
- [x] 05.2-00-PLAN.md - Minimal artifact server compatibility review and approval gate.
- [x] 05.2-01-PLAN.md - Contract v3 minimal tables, schema/examples, and vehicle-score contract removal.
- [x] 05.2-02-PLAN.md - Parser-core minimal row construction, issue #13 implementation removal, and debug sidecar builder.
- [x] 05.2-03-PLAN.md - CLI minified default output, explicit pretty/debug flags, schema command, and README command updates.
- [x] 05.2-04-PLAN.md - Derived legacy comparison from minimal tables and vehicle-score parity removal.
- [x] 05.2-05-PLAN.md - Selected large replay and all-raw benchmark evidence, failure compatibility, and artifact-size benchmark gates.
- [x] 05.2-06-PLAN.md - Fault target retune, final quality gates, README/ROADMAP/STATE handoff, and Phase 6 blocker status.

Current final-gate evidence:
- `05.2-06` fault gate no longer targets removed issue #13 scoring code and now targets minimal artifact behavior.
- Quick task `260502-jeh` optimized the default parser hot path and recorded full all-raw benchmark evidence using the cached old all-raw baseline. The report records cached old wall `501274.528655ms`, new wall `235598.648803ms`, all-raw speedup `2.1277x`, old/new attempted count `23473`, new successes/failures/skips `23469/4/0`, p95 artifact/raw ratio `0.12417910447761193`, max artifact bytes `48313`, and zero oversized artifacts.
- Product-owner acceptance on 2026-05-02 records that current performance is sufficient, p95 > 10% is acceptable because max artifact size is the primary size criterion, and the 4 malformed/non-JSON all-raw failures are acceptable when the old parser reports the same failure count and new failure paths match `.planning/benchmarks/phase-05-all-raw-accepted-failures.json`.

### Phase 6: RabbitMQ/S3 Worker Integration
**Goal**: `server-2` can hand parse jobs to a worker that fetches replay objects, verifies them, writes durable S3 artifacts, and publishes success or failure results.
**Depends on**: Phase 5.2
**Requirements**: WORK-01, WORK-02, WORK-03, WORK-04, WORK-05, WORK-06, WORK-07
**Success Criteria** (what must be TRUE):
  1. Worker consumes RabbitMQ parse request jobs containing `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`.
  2. Worker downloads replay files from S3-compatible storage with configurable endpoint, bucket, credentials, and path-style settings, then fails structurally on checksum mismatch.
  3. Successful jobs write deterministic parse artifacts to S3 and emit `parse.completed` messages with job/replay identifiers, contract version, checksum, and artifact reference.
  4. Failed jobs emit `parse.failed` messages with structured error data and retryability.
  5. RabbitMQ jobs are acknowledged only after result or artifact publication succeeds, with manual ack/nack behavior for failure paths.
**Plans**: 6 plans
**Execution waves**: Wave 1 runs `06-00-PLAN.md`; Wave 2 runs `06-01-PLAN.md`; Wave 3 runs `06-02-PLAN.md` and `06-03-PLAN.md`; Wave 4 runs `06-04-PLAN.md`; Wave 5 runs `06-05-PLAN.md`.
Cross-cutting constraints:
- Worker request/result contracts are typed in `parser-contract` and backed by generated JSON Schema before runtime code consumes or publishes them.
- `parser-core` remains transport-free; RabbitMQ, S3, Tokio runtime, non-deterministic logs, and shutdown behavior live in `parser-worker` and the CLI only delegates.
- Worker mode uses the same minimal public v3 parser artifact path as the CLI default, never the debug sidecar path.
- Manual ack is allowed only after confirmed `parse.completed` or confirmed `parse.failed`; RabbitMQ requeue is reserved for inability to publish a durable outcome.
- Phase 6 defaults to one in-flight job with prefetch `1`; Phase 7 owns multi-worker safety, health/readiness, and container hardening.
Plans:
- [x] 06-00-PLAN.md - Worker request/result contract, schemas, and examples.
- [ ] 06-01-PLAN.md - `parser-worker` crate foundation, worker config, and CLI worker subcommand.
- [ ] 06-02-PLAN.md - S3-compatible raw download, checksum verification, deterministic artifact keys, and artifact write/reuse policy.
- [ ] 06-03-PLAN.md - RabbitMQ consumer, result publisher confirms, and manual ack/nack policy.
- [ ] 06-04-PLAN.md - End-to-end job processor, minimal artifact delivery, handled failures, and graceful shutdown drain.
- [ ] 06-05-PLAN.md - Final worker gates, schema freshness, README/ROADMAP/STATE handoff, and Phase 7 boundary checks.

### Phase 7: Parallel and Container Hardening
**Goal**: Operators can run the worker safely in parallel container mode with observable readiness.
**Depends on**: Phase 6
**Requirements**: WORK-08, WORK-09
**Success Criteria** (what must be TRUE):
  1. Operator can run multiple worker instances in parallel without duplicate artifact corruption or nondeterministic parser output.
  2. Operator can inspect structured logs that identify job, replay, parser stage, and worker lifecycle state.
  3. Container orchestration can use health/readiness endpoints or probes to decide whether a worker can accept jobs.
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 5.1 -> 5.2 -> 6 -> 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Legacy Baseline and Corpus | 5/5 | Complete    | 2026-04-25 |
| 2. Versioned Output Contract | 6/6 | Complete | 2026-04-26 |
| 3. Deterministic Parser Core | 6/6 | Complete | 2026-04-27 |
| 4. Event Semantics and Aggregates | 7/7 | Complete | 2026-04-28 |
| 5. CLI, Golden Parity, Benchmarks, and Coverage Gates | 6/6 | Verification gap escalated | - |
| 5.1. Compact Artifact and Selective Parser Redesign | 8/8 | Execution complete; acceptance gap blocks Phase 6 | - |
| 5.2. Minimal Artifact and Performance Acceptance | 7/7 | Execution complete; accepted benchmark policy unblocks Phase 6 | - |
| 6. RabbitMQ/S3 Worker Integration | 0/6 | Planned; ready to execute | - |
| 7. Parallel and Container Hardening | 0/TBD | Not started | - |
