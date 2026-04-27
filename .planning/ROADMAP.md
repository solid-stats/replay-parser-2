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
- [ ] **Phase 4: Event Semantics and Aggregates** - Normalize combat/outcome semantics and derive auditable legacy, bounty, and vehicle score aggregates. (in progress)
- [ ] **Phase 5: CLI, Golden Parity, Benchmarks, and Coverage Gates** - Make local parsing, schema export, old-vs-new comparison, fixtures, determinism checks, 100% coverage gates, and speed reports executable.
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
- [ ] 04-03-PLAN.md — Legacy per-replay projections, relationships, game-type compatibility metadata, squad/rotation inputs, and bounty inputs.
- [ ] 04-04-PLAN.md — Issue #13 vehicle score taxonomy, weights, contributions, denominator inputs, and teamkill clamp tests.
- [ ] 04-05-PLAN.md — Typed commander-side and winner/outcome facts with conservative known/unknown/candidate semantics.
- [ ] 04-06-PLAN.md — Schema/example refresh, deterministic populated artifact tests, README handoff, and final quality gates.

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
**Plans**: TBD

### Phase 6: RabbitMQ/S3 Worker Integration
**Goal**: `server-2` can hand parse jobs to a worker that fetches replay objects, verifies them, writes durable S3 artifacts, and publishes success or failure results.
**Depends on**: Phase 5
**Requirements**: WORK-01, WORK-02, WORK-03, WORK-04, WORK-05, WORK-06, WORK-07
**Success Criteria** (what must be TRUE):
  1. Worker consumes RabbitMQ parse request jobs containing `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`.
  2. Worker downloads replay files from S3-compatible storage with configurable endpoint, bucket, credentials, and path-style settings, then fails structurally on checksum mismatch.
  3. Successful jobs write deterministic parse artifacts to S3 and emit `parse.completed` messages with job/replay identifiers, contract version, checksum, and artifact reference.
  4. Failed jobs emit `parse.failed` messages with structured error data and retryability.
  5. RabbitMQ jobs are acknowledged only after result or artifact publication succeeds, with manual ack/nack behavior for failure paths.
**Plans**: TBD

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
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Legacy Baseline and Corpus | 5/5 | Complete    | 2026-04-25 |
| 2. Versioned Output Contract | 6/6 | Complete | 2026-04-26 |
| 3. Deterministic Parser Core | 6/6 | Complete | 2026-04-27 |
| 4. Event Semantics and Aggregates | 3/7 | In progress | - |
| 5. CLI, Golden Parity, Benchmarks, and Coverage Gates | 0/TBD | Not started | - |
| 6. RabbitMQ/S3 Worker Integration | 0/TBD | Not started | - |
| 7. Parallel and Container Hardening | 0/TBD | Not started | - |
