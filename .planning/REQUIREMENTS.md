# Requirements: replay-parser-2

**Defined:** 2026-04-24
**Core Value:** Parse OCAP JSON replays quickly and deterministically into normalized raw events plus aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public statistics.

## v1 Requirements

Requirements for the initial Rust parser release. Each maps to roadmap phases.

### Project Documentation

- [x] **DOC-01**: Repository has a root `README.md` that is kept current with project purpose, scope, current GSD phase, architecture direction, validation data, user-facing commands, and integration/development workflow whenever those facts change.
- [x] **DOC-02**: `README.md` explicitly states that project development is performed only by AI agents using the GSD workflow; direct non-GSD development is out of process, and project-changing work must be captured in GSD planning, phase, or quick-task artifacts.

### Project Workflow

- [x] **WF-01**: Every completed agent or GSD work session must leave the repository with a clean git working tree; intended results must be committed atomically instead of left as uncommitted changes.
- [x] **WF-02**: Agents must never delete, revert, or discard completed work merely to make the git tree clean; if it is unclear whether changes should be committed, preserved uncommitted, or excluded from the task, the agent must ask the user before acting.
- [x] **WF-03**: Agents must challenge or decline user instructions that conflict with current project logic, architecture, domain boundaries, accepted planning decisions, test/quality standards, maintainability, or require a disproportionately large change for the stated goal.
- [x] **WF-04**: When challenging an instruction, agents must not stop at refusal; they must explain the concrete project risk, cite the relevant architecture/planning concern where possible, and propose 1-3 safer alternatives or a smaller GSD path.
- [x] **WF-05**: If the user still wants the risky direction after the explanation, agents must ask for explicit confirmation before proceeding and record the warning, user decision, and chosen alternative or override in the relevant GSD artifact or summary.

### Product Integration

- [x] **INT-01**: Solid Stats is treated as a multi-project product composed of `replays-fetcher`, `replay-parser-2`, `server-2`, and `web`, with each application owning distinct responsibilities and integration contracts.
- [x] **INT-02**: Before executing any task, agents must check whether the requested change conflicts with or requires compatibility updates for the other Solid Stats applications, especially parser contract fields, replay ingest/staging assumptions, RabbitMQ/S3 job flow, PostgreSQL/API ownership, public UI expectations, authentication, moderation workflow, and canonical identity boundaries.
- [x] **INT-03**: GSD workflow rules for AI-only development, clean git handoff, AI pushback, README maintenance, and cross-application compatibility are product-wide standards that should be applied consistently in `replays-fetcher`, `replay-parser-2`, `server-2`, and `web`.
- [x] **INT-04**: Cross-application compatibility checks are risk-based: local-only changes may rely on current repo planning docs and `gsd-briefs`, while parser contracts, ingest staging/source identity assumptions, queue/storage messages, API/data shape, identity/auth, moderation, and UI-visible behavior changes require inspecting adjacent app docs/repos or asking the user when evidence is unavailable.

### Legacy Baseline

- [x] **LEG-01**: Developer can run and document the old parser baseline from `/home/afgan0r/Projects/SolidGames/replays-parser`.
- [x] **LEG-02**: Developer can identify the exact old parser command, commit, runtime versions, environment inputs, worker count, logs, and output locations used for parity.
- [x] **LEG-03**: System has a corpus manifest for `~/sg_stats/raw_replays`, `~/sg_stats/results`, and `~/sg_stats/lists/replaysList.json`.
- [x] **LEG-04**: System documents old parser game-type filters, skip rules, exclusions, and known config inputs from `config/excludeReplays.json`, `config/includeReplays.json`, and `config/excludePlayers.json`.
- [x] **LEG-05**: System defines a mismatch taxonomy for old-vs-new comparisons, including compatible, intentional change, old bug preserved, old bug fixed, new bug, insufficient data, and human review.

### Output Contract

- [x] **OUT-01**: Parser writes a stable JSON `ParseArtifact` containing parser version, contract version, replay ID or source file, source checksum, and parse status metadata.
- [x] **OUT-02**: Parser contract includes normalized replay metadata: mission name, world name, mission author, player count, capture delay, end frame, and time/frame boundaries where available.
- [x] **OUT-03**: Parser contract includes observed identity fields without canonical matching: nickname, side/faction, group/squad fields, role/description, source entity ID, and SteamID when present.
- [x] **OUT-04**: Parser contract represents missing winner, missing SteamID, null killer, absent commander, and unknown source fields as explicit unknown/null states.
- [x] **OUT-05**: Parser contract includes source references that link normalized events and aggregate contributions back to replay, frame, event index, entity ID, and rule ID where available.
- [x] **OUT-06**: Parser contract includes JSON Schema generation or equivalent machine-readable schema validation for `server-2` integration.
- [x] **OUT-07**: Parser contract includes structured `ParseFailure` output with job/replay/file identifiers, stage, error code, message, retryability, and source cause.
- [x] **OUT-08**: Parser output ordering is deterministic across repeated runs on the same input and contract version.

### Parser Core

- [x] **PARS-01**: Parser reads OCAP JSON files matching historical files in `~/sg_stats/raw_replays`.
- [x] **PARS-02**: Parser tolerates known OCAP schema drift without panics by producing structured warnings, explicit unknowns, or structured failures.
- [x] **PARS-03**: Parser extracts replay metadata from observed top-level fields such as `EditorMarkers`, `Markers`, `captureDelay`, `endFrame`, `entities`, `events`, `missionAuthor`, `missionName`, `playersCount`, and `worldName`.
- [x] **PARS-04**: Parser normalizes unit/player entities with source IDs, observed names, side, group, role/description, player flags, and available identity fields.
- [x] **PARS-05**: Parser normalizes vehicle/static weapon entities with source IDs, names, classes, side/context where available, and source positions where needed for audit.
- [x] **PARS-06**: Parser preserves old connected-player backfill behavior needed for parity when entity data alone omits participants.
- [x] **PARS-07**: Parser preserves old duplicate-slot same-name merge compatibility behavior for aggregate projection while retaining raw observed identifiers in normalized events.
- [x] **PARS-08**: Parser extracts normalized kill, death, teamkill, suicide, null-killer, player-killed, and vehicle-killed event semantics.
- [x] **PARS-09**: Parser extracts vehicle kill context sufficient to distinguish `killsFromVehicle`, `vehicleKills`, infantry kills, and vehicle/entity type contributions.
- [x] **PARS-10**: Parser extracts commander-side data when present, including replay identifier, side identifier/name, commander observed identity fields, source, and confidence metadata.
- [x] **PARS-11**: Parser extracts winner/outcome data when present and emits unknown/inferred states when older replay data does not contain reliable winner data.

### Aggregates

- [x] **AGG-01**: Parser emits legacy-compatible aggregate summaries for existing comparable fields including kills, kills from vehicle, vehicle kills, teamkills, deaths, KD ratio, vehicle kill coefficient, score, played games, weekly stats, squad stats, and rotation stats.
- [x] **AGG-02**: Parser derives aggregate summaries from normalized events and source references rather than directly mutating aggregate counters without audit trail.
- [x] **AGG-03**: Parser emits killed/killer and teamkilled/teamkiller relationship summaries compatible with old output comparison needs.
- [x] **AGG-04**: Parser preserves old game-type selection behavior for `sg`, `mace`, and `sm`, or emits enough metadata for identical compatibility filtering during comparison.
- [x] **AGG-05**: Parser preserves legacy name-normalization compatibility in aggregate projections while keeping normalized observed identity raw.
- [x] **AGG-06**: Parser emits bounty calculation inputs for valid enemy kills, including killer/victim observed identity, frame/time, side context, replay context, and vehicle/infantry context.
- [x] **AGG-07**: Parser excludes teamkills from bounty-awarding inputs while still exposing them as auditable normalized events.
- [x] **AGG-08**: Parser emits vehicle score contributions from GitHub issue #13 using only kills from vehicles and the defined attacker-vehicle by killed-entity weight matrix.
- [x] **AGG-09**: Parser computes vehicle score by subtracting weighted teamkill penalties from weighted vehicle-kill score and dividing by the count of games where the player had at least one kill from a vehicle.
- [x] **AGG-10**: Parser clamps vehicle score teamkill penalty multipliers below 1 up to 1, even when the normal matrix value for that attacker/killed type is lower.
- [x] **AGG-11**: Parser exposes source references for every vehicle score contribution so the score can be audited and recalculated by `server-2` if needed.

### CLI and Validation

- [x] **CLI-01**: CLI can parse a local OCAP JSON file and write normalized result JSON to a requested output path.
- [x] **CLI-02**: CLI can emit contract schema information for the current parser contract version.
- [x] **CLI-03**: CLI can run old-vs-new comparison on selected replay files or saved old output artifacts.
- [x] **CLI-04**: CLI exits with structured error output and non-zero status on malformed, unreadable, or unsupported replay files.
- [x] **TEST-01**: Golden fixtures are derived from `~/sg_stats` and include representative normal, malformed, partial, old-format, winner-present, winner-missing, vehicle-kill, teamkill, and commander-side cases where available.
- [x] **TEST-02**: Existing result comparisons cover comparable old fields and report per-field mismatch categories.
- [x] **TEST-03**: Determinism tests prove repeated parser runs on the same input produce stable JSON output.
- [ ] **TEST-04**: Benchmark harness reports parse-only, aggregate-only, and end-to-end throughput against the pinned old parser baseline.
- [ ] **TEST-05**: Benchmark reporting includes files/sec, MB/sec or events/sec, memory/RSS where practical, and whether output parity passed for the measured sample.
- [ ] **TEST-06**: Benchmark target is approximately 10x faster than the current parser on an equivalent workload.
- [ ] **TEST-07**: CI enforces 100% statement, branch, function, and line coverage for all reachable production Rust code in parser core, contract, CLI, worker, harness, and aggregate modules; exclusions are allowed only for impossible-to-execute platform glue, generated code, or defensive unreachable branches with an inline rationale and reviewable allowlist entry.
- [x] **TEST-08**: Every parser behavior requirement has at least one behavior-level test with a strong oracle, including success, boundary, error, malformed input, unknown/null state, deterministic ordering, parity, and source-reference scenarios where applicable.
- [x] **TEST-09**: Unit tests follow the `unit-tests-philosophy` RITE standard: readable names, explicit Arrange/Act/Assert structure, isolated fixtures/state, deterministic time/randomness/environment, and assertions against observable behavior rather than private implementation details.
- [x] **TEST-10**: Test data uses typed builders, minimal focused fixtures, or curated golden corpus samples instead of unsafe casts, ad-hoc duplicated object graphs, or tests that require production-only API changes.
- [x] **TEST-11**: The test suite includes negative and regression tests for known legacy compatibility traps, including schema drift, malformed events/entities, null killers, duplicate-slot same-name behavior, connected-player backfill, teamkill classification, vehicle score penalties, and missing identity/outcome fields.
- [ ] **TEST-12**: Release gating includes a mutation-testing or equivalent fault-injection report for parser-core and aggregate logic; any surviving high-risk mutant or fault class must be fixed with stronger tests or documented as an accepted non-applicable case.

### Worker Integration

- [ ] **WORK-01**: Worker consumes RabbitMQ parse request jobs containing `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`.
- [ ] **WORK-02**: Worker downloads replay files from S3-compatible object storage using configurable endpoint, bucket, credentials, and path-style settings.
- [ ] **WORK-03**: Worker verifies downloaded object checksum before parsing and emits structured failure on mismatch.
- [ ] **WORK-04**: Worker writes successful parse artifacts to S3-compatible storage using a deterministic artifact key agreed with `server-2`.
- [ ] **WORK-05**: Worker publishes `parse.completed` result messages with job/replay identifiers, parser contract version, checksum, and artifact reference.
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
- **FUT-06**: Solid Stats can support annual/yearly nomination statistics as a separate v2 product surface. The legacy `src/!yearStatistics` pipeline and `~/sg_stats/year_results` outputs are historical references only in v1 and must not be folded into ordinary player, squad, rotation, weekly, or bounty statistics.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Public website and UI | Owned by `web`, not parser. |
| Replay discovery and production fetching from the external replay source | Owned by `replays-fetcher`; parser consumes local files or `server-2` parse jobs only. |
| Steam OAuth | Owned by `server-2`/`web`; parser only preserves observed identifiers. |
| Canonical player matching | `server-2` owns real player identity across nicknames and SteamIDs. |
| User roles, moderation, and correction workflow | Product workflow belongs to `server-2` and `web`. |
| Parser-owned PostgreSQL business persistence | Parser emits artifacts/messages; `server-2` owns database writes. |
| Direct stat editing/corrections | Corrections should be represented in `server-2` and applied through recalculation. |
| Replay formats other than OCAP JSON in v1 | v1 migration value is OCAP JSON parity. |
| Production Kubernetes deployment | Container/worker readiness is in scope, cluster deployment is not. |
| Financial rewards or payout logic | Parser emits bounty inputs; reward rules belong to `server-2`. |
| Annual/yearly nomination statistics in v1 | These are a separate legacy surface with unique nominations; defer product support to v2 while preserving `src/!yearStatistics` and `~/sg_stats/year_results` as historical references. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| DOC-01 | Phase 1 | Complete |
| DOC-02 | Phase 1 | Complete |
| WF-01 | Phase 1 | Complete |
| WF-02 | Phase 1 | Complete |
| WF-03 | Phase 1 | Complete |
| WF-04 | Phase 1 | Complete |
| WF-05 | Phase 1 | Complete |
| INT-01 | Phase 1 | Complete |
| INT-02 | Phase 1 | Complete |
| INT-03 | Phase 1 | Complete |
| INT-04 | Phase 1 | Complete |
| LEG-01 | Phase 1 | Complete |
| LEG-02 | Phase 1 | Complete |
| LEG-03 | Phase 1 | Complete |
| LEG-04 | Phase 1 | Complete |
| LEG-05 | Phase 1 | Complete |
| OUT-01 | Phase 2 | Complete |
| OUT-02 | Phase 2 | Complete |
| OUT-03 | Phase 2 | Complete |
| OUT-04 | Phase 2 | Complete |
| OUT-05 | Phase 2 | Complete |
| OUT-06 | Phase 2 | Complete |
| OUT-07 | Phase 2 | Complete |
| OUT-08 | Phase 3 | Complete |
| PARS-01 | Phase 3 | Complete |
| PARS-02 | Phase 3 | Complete |
| PARS-03 | Phase 3 | Complete |
| PARS-04 | Phase 3 | Complete |
| PARS-05 | Phase 3 | Complete |
| PARS-06 | Phase 3 | Complete |
| PARS-07 | Phase 3 | Complete |
| PARS-08 | Phase 4 | Complete |
| PARS-09 | Phase 4 | Complete |
| PARS-10 | Phase 4 | Complete |
| PARS-11 | Phase 4 | Complete |
| AGG-01 | Phase 4 | Complete |
| AGG-02 | Phase 4 | Complete |
| AGG-03 | Phase 4 | Complete |
| AGG-04 | Phase 4 | Complete |
| AGG-05 | Phase 4 | Complete |
| AGG-06 | Phase 4 | Complete |
| AGG-07 | Phase 4 | Complete |
| AGG-08 | Phase 4 | Complete |
| AGG-09 | Phase 4 | Complete |
| AGG-10 | Phase 4 | Complete |
| AGG-11 | Phase 4 | Complete |
| CLI-01 | Phase 5 | Complete |
| CLI-02 | Phase 5 | Complete |
| CLI-03 | Phase 5 | Complete |
| CLI-04 | Phase 5 | Complete |
| TEST-01 | Phase 5 | Complete |
| TEST-02 | Phase 5 | Complete |
| TEST-03 | Phase 5 | Complete |
| TEST-04 | Phase 5 | Pending |
| TEST-05 | Phase 5 | Pending |
| TEST-06 | Phase 5 | Pending |
| TEST-07 | Phase 5 | Pending |
| TEST-08 | Phase 5 | Complete |
| TEST-09 | Phase 5 | Complete |
| TEST-10 | Phase 5 | Complete |
| TEST-11 | Phase 5 | Complete |
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
- v1 requirements: 71 total
- Mapped to phases: 71
- Unmapped: 0

---
*Requirements defined: 2026-04-24*
*Last updated: 2026-04-28 after verifying Phase 4 event semantics and aggregates*
