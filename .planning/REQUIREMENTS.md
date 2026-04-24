# Requirements: replays-parser-2

**Defined:** 2026-04-24
**Core Value:** Parse OCAP JSON replays quickly and deterministically into normalized raw events plus aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public statistics.

## v1 Requirements

Requirements for the initial Rust parser release. Each maps to roadmap phases.

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
| LEG-01 | TBD | Pending |
| LEG-02 | TBD | Pending |
| LEG-03 | TBD | Pending |
| LEG-04 | TBD | Pending |
| LEG-05 | TBD | Pending |
| OUT-01 | TBD | Pending |
| OUT-02 | TBD | Pending |
| OUT-03 | TBD | Pending |
| OUT-04 | TBD | Pending |
| OUT-05 | TBD | Pending |
| OUT-06 | TBD | Pending |
| OUT-07 | TBD | Pending |
| OUT-08 | TBD | Pending |
| PARS-01 | TBD | Pending |
| PARS-02 | TBD | Pending |
| PARS-03 | TBD | Pending |
| PARS-04 | TBD | Pending |
| PARS-05 | TBD | Pending |
| PARS-06 | TBD | Pending |
| PARS-07 | TBD | Pending |
| PARS-08 | TBD | Pending |
| PARS-09 | TBD | Pending |
| PARS-10 | TBD | Pending |
| PARS-11 | TBD | Pending |
| AGG-01 | TBD | Pending |
| AGG-02 | TBD | Pending |
| AGG-03 | TBD | Pending |
| AGG-04 | TBD | Pending |
| AGG-05 | TBD | Pending |
| AGG-06 | TBD | Pending |
| AGG-07 | TBD | Pending |
| AGG-08 | TBD | Pending |
| AGG-09 | TBD | Pending |
| AGG-10 | TBD | Pending |
| AGG-11 | TBD | Pending |
| CLI-01 | TBD | Pending |
| CLI-02 | TBD | Pending |
| CLI-03 | TBD | Pending |
| CLI-04 | TBD | Pending |
| TEST-01 | TBD | Pending |
| TEST-02 | TBD | Pending |
| TEST-03 | TBD | Pending |
| TEST-04 | TBD | Pending |
| TEST-05 | TBD | Pending |
| TEST-06 | TBD | Pending |
| WORK-01 | TBD | Pending |
| WORK-02 | TBD | Pending |
| WORK-03 | TBD | Pending |
| WORK-04 | TBD | Pending |
| WORK-05 | TBD | Pending |
| WORK-06 | TBD | Pending |
| WORK-07 | TBD | Pending |
| WORK-08 | TBD | Pending |
| WORK-09 | TBD | Pending |

**Coverage:**
- v1 requirements: 54 total
- Mapped to phases: 0
- Unmapped: 54

---
*Requirements defined: 2026-04-24*
*Last updated: 2026-04-24 after initial definition*
