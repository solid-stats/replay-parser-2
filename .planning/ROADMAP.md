# Roadmap: replays-parser-2

## Overview

This roadmap replaces the legacy TypeScript replay parser with a deterministic Rust parser that is grounded in the old parser's behavior, emits a versioned contract for `server-2`, proves parity and speed against `~/sg_stats`, and then exposes the same parser core through local CLI and RabbitMQ/S3 worker modes.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Legacy Baseline and Corpus** - Pin the old parser baseline and historical corpus facts that define v1 parity.
- [ ] **Phase 2: Versioned Output Contract** - Define the stable parse artifact, failure, schema, unknown-state, and source-reference contract.
- [ ] **Phase 3: Deterministic Parser Core** - Parse OCAP JSON into deterministic normalized metadata and observed entity facts.
- [ ] **Phase 4: Event Semantics and Aggregates** - Normalize combat/outcome semantics and derive auditable legacy, bounty, and vehicle score aggregates.
- [ ] **Phase 5: CLI, Golden Parity, and Benchmarks** - Make local parsing, schema export, old-vs-new comparison, fixtures, determinism checks, and speed reports executable.
- [ ] **Phase 6: RabbitMQ/S3 Worker Integration** - Consume parse jobs, fetch objects, verify checksums, publish results, and use safe queue acknowledgement.
- [ ] **Phase 7: Parallel and Container Hardening** - Prove multi-worker safety and container-ready observability.

## Phase Details

### Phase 1: Legacy Baseline and Corpus
**Goal**: Developers can reproduce and inspect the legacy parser and historical data baseline that define v1 parity.
**Depends on**: Nothing (first phase)
**Requirements**: LEG-01, LEG-02, LEG-03, LEG-04, LEG-05
**Success Criteria** (what must be TRUE):
  1. Developer can run the pinned old parser baseline and see the command, commit, runtime versions, environment inputs, worker count, logs, and output locations used for parity.
  2. Developer can inspect a corpus manifest for `~/sg_stats/raw_replays`, `~/sg_stats/results`, and `~/sg_stats/lists/replaysList.json`.
  3. Developer can inspect documented old parser game-type filters, skip rules, exclusions, and config inputs.
  4. Developer can classify any old-vs-new difference using the agreed mismatch taxonomy.
**Plans**: TBD

### Phase 2: Versioned Output Contract
**Goal**: `server-2` and parser tooling can rely on a stable, machine-checkable parse artifact and failure contract.
**Depends on**: Phase 1
**Requirements**: OUT-01, OUT-02, OUT-03, OUT-04, OUT-05, OUT-06, OUT-07
**Success Criteria** (what must be TRUE):
  1. Developer can validate a current `ParseArtifact` JSON document that includes parser version, contract version, replay/source identifiers, checksum, and parse status metadata.
  2. Server integrator can consume normalized replay metadata, observed identity fields, and explicit unknown/null states without canonical player matching.
  3. Developer can trace normalized events and aggregate contributions back to replay, frame, event index, entity ID, and rule ID where available.
  4. Developer can validate structured `ParseFailure` output with job/replay/file identifiers, stage, error code, message, retryability, and source cause.
**Plans**: TBD

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
**Plans**: TBD

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
**Plans**: TBD

### Phase 5: CLI, Golden Parity, and Benchmarks
**Goal**: Developers can reproduce parser results locally, compare against the old parser, and measure the 10x target on equivalent workloads.
**Depends on**: Phase 4
**Requirements**: CLI-01, CLI-02, CLI-03, CLI-04, TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, TEST-06
**Success Criteria** (what must be TRUE):
  1. User can parse a local OCAP JSON file to a requested output path and request the current contract schema from the CLI.
  2. User receives structured error JSON and a non-zero exit status for bad, unreadable, or unsupported replay files.
  3. Developer can run old-vs-new comparison on selected replay files or saved old output artifacts and receive per-field mismatch categories.
  4. Golden fixtures cover representative normal, bad, partial, old-shape, winner-present, winner-missing, vehicle-kill, teamkill, and commander-side cases where available.
  5. Benchmark reports include parse-only, aggregate-only, and end-to-end throughput plus memory/RSS where practical, parity status for the measured sample, and whether the roughly 10x target is met.
**Plans**: TBD

### Phase 6: RabbitMQ/S3 Worker Integration
**Goal**: `server-2` can hand parse jobs to a worker that fetches replay objects, verifies them, and publishes durable success or failure results.
**Depends on**: Phase 5
**Requirements**: WORK-01, WORK-02, WORK-03, WORK-04, WORK-05, WORK-06, WORK-07
**Success Criteria** (what must be TRUE):
  1. Worker consumes RabbitMQ parse request jobs containing `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`.
  2. Worker downloads replay files from S3-compatible storage with configurable endpoint, bucket, credentials, and path-style settings, then fails structurally on checksum mismatch.
  3. Successful jobs write or publish deterministic parse artifacts and emit `parse.completed` messages with job/replay identifiers, contract version, checksum, and artifact reference or payload.
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
| 1. Legacy Baseline and Corpus | 0/TBD | Not started | - |
| 2. Versioned Output Contract | 0/TBD | Not started | - |
| 3. Deterministic Parser Core | 0/TBD | Not started | - |
| 4. Event Semantics and Aggregates | 0/TBD | Not started | - |
| 5. CLI, Golden Parity, and Benchmarks | 0/TBD | Not started | - |
| 6. RabbitMQ/S3 Worker Integration | 0/TBD | Not started | - |
| 7. Parallel and Container Hardening | 0/TBD | Not started | - |
