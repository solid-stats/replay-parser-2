# Requirements: replays-parser-2

**Defined:** 2026-04-24
**Core Value:** Parse OCAP JSON replays quickly and deterministically into normalized raw events plus aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public statistics.

## v1 Requirements

Requirements for the initial Rust parser release. Each maps to roadmap phases.

### Project Documentation

- [ ] **DOC-01**: Repository has a root `README.md` that is kept current with project purpose, scope, current GSD phase, architecture direction, validation data, user-facing commands, and integration/development workflow whenever those facts change.
- [ ] **DOC-02**: `README.md` explicitly states that project development is performed only by AI agents using the GSD workflow; direct non-GSD development is out of process, and project-changing work must be captured in GSD planning, phase, or quick-task artifacts.

### Project Workflow

- [ ] **WF-01**: Every completed agent or GSD work session must leave the repository with a clean git working tree; intended results must be committed atomically instead of left as uncommitted changes.
- [ ] **WF-02**: Agents must never delete, revert, or discard completed work merely to make the git tree clean; if it is unclear whether changes should be committed, preserved uncommitted, or excluded from the task, the agent must ask the user before acting.

### Legacy Baseline

- [ ] **LEG-01**: Developer can run and document the old parser baseline from `/home/afgan0r/Projects/SolidGames/replays-parser`.
- [ ] **LEG-02**: Developer can identify the exact old parser command, commit, runtime versions, environment inputs, worker count, logs, and output locations used for parity.
- [ ] **LEG-03**: System has a corpus manifest for `~/sg_stats/raw_replays`, `~/sg_stats/results`, and `~/sg_stats/lists/replaysList.json`.
- [ ] **LEG-04**: System documents old parser game-type filters, skip rules, exclusions, and known config inputs from `config/excludeReplays.json`, `config/includeReplays.json`, and `config/excludePlayers.json`.
- [ ] **LEG-05**: System defines a mismatch taxonomy for old-vs-new comparisons, including compatible, intentional change, old bug preserved, old bug fixed, new bug, insufficient data, and human review.

### Output Contract

- [ ] **OUT-01**: Parser writes a stable JSON `ParseArtifact` containing parser version, contract version, replay ID or source file, source checksum, and parse status metadata.
- [ ] **OUT-02**: Parser contract includes normalized replay metadata: mission name, world name, mission author, player count, capture delay, end frame, and time/frame boundaries where available.
- [ ] **OUT-03**: Parser contract includes observed identity fields without canonical matching: nickname, side/faction, group/squad fields, role/description, source entity ID, and SteamID when present.
- [ ] **OUT-04**: Parser contract represents missing winner, missing SteamID, null killer, absent commander, and unknown source fields as explicit unknown/null states.
- [ ] **OUT-05**: Parser contract includes source references that link normalized events and aggregate contributions back to replay, frame, event index, entity ID, and rule ID where available.
- [ ] **OUT-06**: Parser contract includes JSON Schema generation or equivalent machine-readable schema validation for `server-2` integration.
- [ ] **OUT-07**: Parser contract includes structured `ParseFailure` output with job/replay/file identifiers, stage, error code, message, retryability, and source cause.
- [ ] **OUT-08**: Parser output ordering is deterministic across repeated runs on the same input and contract version.

### Parser Core

- [ ] **PARS-01**: Parser reads OCAP JSON files matching historical files in `~/sg_stats/raw_replays`.
- [ ] **PARS-02**: Parser tolerates known OCAP schema drift without panics by producing structured warnings, explicit unknowns, or structured failures.
- [ ] **PARS-03**: Parser extracts replay metadata from observed top-level fields such as `EditorMarkers`, `Markers`, `captureDelay`, `endFrame`, `entities`, `events`, `missionAuthor`, `missionName`, `playersCount`, and `worldName`.
- [ ] **PARS-04**: Parser normalizes unit/player entities with source IDs, observed names, side, group, role/description, player flags, and available identity fields.
- [ ] **PARS-05**: Parser normalizes vehicle/static weapon entities with source IDs, names, classes, side/context where available, and source positions where needed for audit.
- [ ] **PARS-06**: Parser preserves old connected-player backfill behavior needed for parity when entity data alone omits participants.
- [ ] **PARS-07**: Parser preserves old duplicate-slot same-name merge compatibility behavior for aggregate projection while retaining raw observed identifiers in normalized events.
- [ ] **PARS-08**: Parser extracts normalized kill, death, teamkill, suicide, null-killer, player-killed, and vehicle-killed event semantics.
- [ ] **PARS-09**: Parser extracts vehicle kill context sufficient to distinguish `killsFromVehicle`, `vehicleKills`, infantry kills, and vehicle/entity type contributions.
- [ ] **PARS-10**: Parser extracts commander-side data when present, including replay identifier, side identifier/name, commander observed identity fields, source, and confidence metadata.
- [ ] **PARS-11**: Parser extracts winner/outcome data when present and emits unknown/inferred states when older replay data does not contain reliable winner data.

### Aggregates

- [ ] **AGG-01**: Parser emits legacy-compatible aggregate summaries for existing comparable fields including kills, kills from vehicle, vehicle kills, teamkills, deaths, KD ratio, vehicle kill coefficient, score, played games, weekly stats, squad stats, and rotation stats.
- [ ] **AGG-02**: Parser derives aggregate summaries from normalized events and source references rather than directly mutating aggregate counters without audit trail.
- [ ] **AGG-03**: Parser emits killed/killer and teamkilled/teamkiller relationship summaries compatible with old output comparison needs.
- [ ] **AGG-04**: Parser preserves old game-type selection behavior for `sg`, `mace`, and `sm`, or emits enough metadata for identical compatibility filtering during comparison.
- [ ] **AGG-05**: Parser preserves legacy name-normalization compatibility in aggregate projections while keeping normalized observed identity raw.
- [ ] **AGG-06**: Parser emits bounty calculation inputs for valid enemy kills, including killer/victim observed identity, frame/time, side context, replay context, and vehicle/infantry context.
- [ ] **AGG-07**: Parser excludes teamkills from bounty-awarding inputs while still exposing them as auditable normalized events.
- [ ] **AGG-08**: Parser emits vehicle score contributions from GitHub issue #13 using only kills from vehicles and the defined attacker-vehicle by killed-entity weight matrix.
- [ ] **AGG-09**: Parser computes vehicle score by subtracting weighted teamkill penalties from weighted vehicle-kill score and dividing by the count of games where the player had at least one kill from a vehicle.
- [ ] **AGG-10**: Parser clamps vehicle score teamkill penalty multipliers below 1 up to 1, even when the normal matrix value for that attacker/killed type is lower.
- [ ] **AGG-11**: Parser exposes source references for every vehicle score contribution so the score can be audited and recalculated by `server-2` if needed.

### CLI and Validation

- [ ] **CLI-01**: CLI can parse a local OCAP JSON file and write normalized result JSON to a requested output path.
- [ ] **CLI-02**: CLI can emit contract schema information for the current parser contract version.
- [ ] **CLI-03**: CLI can run old-vs-new comparison on selected replay files or saved old output artifacts.
- [ ] **CLI-04**: CLI exits with structured error output and non-zero status on malformed, unreadable, or unsupported replay files.
- [ ] **TEST-01**: Golden fixtures are derived from `~/sg_stats` and include representative normal, malformed, partial, old-format, winner-present, winner-missing, vehicle-kill, teamkill, and commander-side cases where available.
- [ ] **TEST-02**: Existing result comparisons cover comparable old fields and report per-field mismatch categories.
- [ ] **TEST-03**: Determinism tests prove repeated parser runs on the same input produce stable JSON output.
- [ ] **TEST-04**: Benchmark harness reports parse-only, aggregate-only, and end-to-end throughput against the pinned old parser baseline.
- [ ] **TEST-05**: Benchmark reporting includes files/sec, MB/sec or events/sec, memory/RSS where practical, and whether output parity passed for the measured sample.
- [ ] **TEST-06**: Benchmark target is approximately 10x faster than the current parser on an equivalent workload.
- [ ] **TEST-07**: CI enforces 100% statement, branch, function, and line coverage for all reachable production Rust code in parser core, contract, CLI, worker, harness, and aggregate modules; exclusions are allowed only for impossible-to-execute platform glue, generated code, or defensive unreachable branches with an inline rationale and reviewable allowlist entry.
- [ ] **TEST-08**: Every parser behavior requirement has at least one behavior-level test with a strong oracle, including success, boundary, error, malformed input, unknown/null state, deterministic ordering, parity, and source-reference scenarios where applicable.
- [ ] **TEST-09**: Unit tests follow the `unit-tests-philosophy` RITE standard: readable names, explicit Arrange/Act/Assert structure, isolated fixtures/state, deterministic time/randomness/environment, and assertions against observable behavior rather than private implementation details.
- [ ] **TEST-10**: Test data uses typed builders, minimal focused fixtures, or curated golden corpus samples instead of unsafe casts, ad-hoc duplicated object graphs, or tests that require production-only API changes.
- [ ] **TEST-11**: The test suite includes negative and regression tests for known legacy compatibility traps, including schema drift, malformed events/entities, null killers, duplicate-slot same-name behavior, connected-player backfill, teamkill classification, vehicle score penalties, and missing identity/outcome fields.
- [ ] **TEST-12**: Release gating includes a mutation-testing or equivalent fault-injection report for parser-core and aggregate logic; any surviving high-risk mutant or fault class must be fixed with stronger tests or documented as an accepted non-applicable case.

### Worker Integration

- [ ] **WORK-01**: Worker consumes RabbitMQ parse request jobs containing `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`.
- [ ] **WORK-02**: Worker downloads replay files from S3-compatible object storage using configurable endpoint, bucket, credentials, and path-style settings.
- [ ] **WORK-03**: Worker verifies downloaded object checksum before parsing and emits structured failure on mismatch.
- [ ] **WORK-04**: Worker writes or publishes successful parse artifacts using a deterministic artifact key or payload format agreed with `server-2`.
- [ ] **WORK-05**: Worker publishes `parse.completed` result messages with job/replay identifiers, parser contract version, checksum, and artifact reference or payload.
- [ ] **WORK-06**: Worker publishes `parse.failed` result messages with structured error data and retryability.
- [ ] **WORK-07**: Worker uses manual ack/nack behavior and acknowledges RabbitMQ jobs only after result/artifact publication succeeds.
- [ ] **WORK-08**: Worker can run multiple instances in parallel without duplicate artifact corruption or nondeterministic parser output.
- [ ] **WORK-09**: Worker has structured logs and health/readiness endpoints or probes suitable for container operation.

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Future Analytics

- **FUT-01**: Parser can export full trajectory or position timelines for analytics features.
- **FUT-02**: Parser can support replay formats other than OCAP JSON.
- **FUT-03**: Parser can expose a streaming event API for live or incremental consumers.
- **FUT-04**: Parser can emit advanced anomaly detection reports after normalized event semantics are stable.
- **FUT-05**: Parser can provide richer replay quality reports for user-facing correction workflows if `server-2` needs parser-specific evidence.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Public website and UI | Owned by `web`, not parser. |
| Steam OAuth | Owned by `server-2`/`web`; parser only preserves observed identifiers. |
| Canonical player matching | `server-2` owns real player identity across nicknames and SteamIDs. |
| User roles, moderation, and correction workflow | Product workflow belongs to `server-2` and `web`. |
| Parser-owned PostgreSQL business persistence | Parser emits artifacts/messages; `server-2` owns database writes. |
| Direct stat editing/corrections | Corrections should be represented in `server-2` and applied through recalculation. |
| Replay formats other than OCAP JSON in v1 | v1 migration value is OCAP JSON parity. |
| Production Kubernetes deployment | Container/worker readiness is in scope, cluster deployment is not. |
| Financial rewards or payout logic | Parser emits bounty inputs; reward rules belong to `server-2`. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| DOC-01 | Phase 1 | Pending |
| DOC-02 | Phase 1 | Pending |
| WF-01 | Phase 1 | Pending |
| WF-02 | Phase 1 | Pending |
| LEG-01 | Phase 1 | Pending |
| LEG-02 | Phase 1 | Pending |
| LEG-03 | Phase 1 | Pending |
| LEG-04 | Phase 1 | Pending |
| LEG-05 | Phase 1 | Pending |
| OUT-01 | Phase 2 | Pending |
| OUT-02 | Phase 2 | Pending |
| OUT-03 | Phase 2 | Pending |
| OUT-04 | Phase 2 | Pending |
| OUT-05 | Phase 2 | Pending |
| OUT-06 | Phase 2 | Pending |
| OUT-07 | Phase 2 | Pending |
| OUT-08 | Phase 3 | Pending |
| PARS-01 | Phase 3 | Pending |
| PARS-02 | Phase 3 | Pending |
| PARS-03 | Phase 3 | Pending |
| PARS-04 | Phase 3 | Pending |
| PARS-05 | Phase 3 | Pending |
| PARS-06 | Phase 3 | Pending |
| PARS-07 | Phase 3 | Pending |
| PARS-08 | Phase 4 | Pending |
| PARS-09 | Phase 4 | Pending |
| PARS-10 | Phase 4 | Pending |
| PARS-11 | Phase 4 | Pending |
| AGG-01 | Phase 4 | Pending |
| AGG-02 | Phase 4 | Pending |
| AGG-03 | Phase 4 | Pending |
| AGG-04 | Phase 4 | Pending |
| AGG-05 | Phase 4 | Pending |
| AGG-06 | Phase 4 | Pending |
| AGG-07 | Phase 4 | Pending |
| AGG-08 | Phase 4 | Pending |
| AGG-09 | Phase 4 | Pending |
| AGG-10 | Phase 4 | Pending |
| AGG-11 | Phase 4 | Pending |
| CLI-01 | Phase 5 | Pending |
| CLI-02 | Phase 5 | Pending |
| CLI-03 | Phase 5 | Pending |
| CLI-04 | Phase 5 | Pending |
| TEST-01 | Phase 5 | Pending |
| TEST-02 | Phase 5 | Pending |
| TEST-03 | Phase 5 | Pending |
| TEST-04 | Phase 5 | Pending |
| TEST-05 | Phase 5 | Pending |
| TEST-06 | Phase 5 | Pending |
| TEST-07 | Phase 5 | Pending |
| TEST-08 | Phase 5 | Pending |
| TEST-09 | Phase 5 | Pending |
| TEST-10 | Phase 5 | Pending |
| TEST-11 | Phase 5 | Pending |
| TEST-12 | Phase 5 | Pending |
| WORK-01 | Phase 6 | Pending |
| WORK-02 | Phase 6 | Pending |
| WORK-03 | Phase 6 | Pending |
| WORK-04 | Phase 6 | Pending |
| WORK-05 | Phase 6 | Pending |
| WORK-06 | Phase 6 | Pending |
| WORK-07 | Phase 6 | Pending |
| WORK-08 | Phase 7 | Pending |
| WORK-09 | Phase 7 | Pending |

**Coverage:**
- v1 requirements: 64 total
- Mapped to phases: 64
- Unmapped: 0

---
*Requirements defined: 2026-04-24*
*Last updated: 2026-04-25 after adding clean git tree workflow requirements*
