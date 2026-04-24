# replays-parser-2

## What This Is

`replays-parser-2` is a Rust replay parsing application for Solid Stats. It parses local OCAP JSON replay files into deterministic normalized raw events plus aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public SolidGames statistics.

This project owns the parsing engine and parsing result contract only. Public website behavior, Steam OAuth, moderation UI, correction requests, canonical player identity, and PostgreSQL business persistence belong to `server-2` and `web`.

## Core Value

Parse OCAP JSON replays quickly and deterministically into normalized raw events plus aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public statistics.

## Requirements

### Validated

(None yet - ship to validate)

### Active

- [ ] Build a Rust parser for OCAP JSON replay files matching the historical `~/sg_stats/raw_replays` corpus.
- [ ] Provide a CLI that parses a local OCAP JSON file and writes normalized result JSON.
- [ ] Provide a worker/container mode that consumes parse jobs from RabbitMQ and reads replay files from S3-compatible storage.
- [ ] Emit deterministic normalized event output for replay metadata, observed player/entity identity, kill/death/teamkill events, vehicle context, commander-side data, and winner/outcome where present.
- [ ] Emit aggregate summaries for current Solid Stats fields and new stats needed by player, squad, rotation, commander-side, and bounty point calculations.
- [ ] Version the parser output contract and include source references that allow aggregates to be traced back to normalized events.
- [ ] Preserve observed identifiers from the replay without attempting canonical player matching.
- [ ] Represent missing winner, SteamID, and other absent identity fields explicitly as unknown/null states.
- [ ] Produce structured parse failures tied to replay file and job identifiers.
- [ ] Use `~/sg_stats` historical data for golden tests and old-vs-new result comparisons.
- [ ] Include a benchmark harness that establishes the current parser baseline and targets roughly 10x faster parsing.

### Out of Scope

- Public website and UI - owned by `web`.
- Steam OAuth - owned by `server-2`/`web`.
- User roles, moderation, and correction request workflow - owned by `server-2` and `web`.
- PostgreSQL persistence as parser-owned source of truth - `server-2` owns business tables and persistence.
- Direct editing or correction of stats - parser emits artifacts, server applies corrections/recalculations.
- Replay formats other than OCAP JSON - v1 targets OCAP JSON only.
- Production Kubernetes deployment - container/worker readiness is needed, production orchestration is not.
- Financial rewards or payout logic - parser only emits bounty calculation inputs.

## Context

Solid Stats is a public SolidGames statistics platform that replaces the current replay-parser/statistics workflow. It needs fast, trustworthy replay parsing, public player/squad/rotation/commander-side statistics, player-submitted correction requests, and bounty points based on player and squad effectiveness.

The current historical reference data lives at `~/sg_stats`:

- `~/sg_stats/raw_replays` contains around 3,938 OCAP JSON replay files.
- `~/sg_stats/results` contains existing calculated results.
- `~/sg_stats/lists/replaysList.json` contains replay list metadata.
- The archive is for tests and golden validation only, not production import.

Existing result fields include `kills`, `killsFromVehicle`, `vehicleKills`, `teamkills`, `deaths`, `kdRatio`, `killsFromVehicleCoef`, `score`, `totalPlayedGames`, `week`, `startDate`, and `endDate`.

Observed OCAP top-level keys include `EditorMarkers`, `Markers`, `captureDelay`, `endFrame`, `entities`, `events`, `missionAuthor`, `missionName`, `playersCount`, and `worldName`.

The intended integration flow is:

1. `server-2` stores a replay file in S3-compatible storage.
2. `server-2` publishes a RabbitMQ parse request containing `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`.
3. `replays-parser-2` downloads the replay file from storage.
4. `replays-parser-2` emits either `parse.completed` with a normalized parse artifact reference or payload, or `parse.failed` with structured error data.
5. `server-2` persists results into PostgreSQL and publishes aggregate statistics.

In this domain, "KS" means commander of a side. The parser should detect commander-side data when present, including replay identifier, side identifier/name, commander observed identity fields, winner/outcome if present, and source/confidence metadata if available.

Bounty points are calculated by `server-2`, but parser output must support the required inputs. For each valid kill event, the parser should output killer and victim observed identity, enemy-kill/teamkill classification, kill timestamp/frame, relevant vehicle/infantry context, replay context, and side context. Only valid enemy kills award bounty points in v1; teamkills do not.

Open implementation details for later phases:

- Exact old parser command used for baseline benchmark.
- Exact old/new comparison tolerances.
- Final normalized JSON schema names and field types.
- Whether parse result payload is sent directly over RabbitMQ or stored as an artifact in S3.
- Exact RabbitMQ exchange and routing key naming.

## Constraints

- **Language**: Rust - chosen for deterministic parsing, performance, and deployable CLI/worker binaries.
- **Replay format**: OCAP JSON only - supporting other formats is outside v1 scope.
- **Validation data**: `~/sg_stats` - historical data is the golden/test baseline, not a production import source.
- **Performance**: Roughly 10x faster than the current parser - must be measured against a baseline benchmark.
- **Runtime modes**: CLI plus worker/container mode - local reproducibility and server integration are both required.
- **Queue integration**: RabbitMQ - worker mode consumes parse requests and publishes parse results/failures.
- **File input**: S3-compatible object storage - parser worker reads replay content by object key/checksum.
- **Database ownership**: `server-2` owns PostgreSQL persistence - parser does not mutate business tables in v1.
- **Identity**: Parser preserves observed identifiers only - canonical player matching belongs to `server-2`.
- **History reprocessing**: Server may overwrite derived results in v1 - parser must make output repeatable and versioned.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use Rust | Parsing performance and deterministic binaries are central to project value. | - Pending |
| Support OCAP JSON only in v1 | Historical data and immediate Solid Stats needs are OCAP JSON. | - Pending |
| Treat `~/sg_stats` as golden/test baseline | Existing raw replays and result data enable regression tests and benchmarks. | - Pending |
| Provide both CLI and worker mode | CLI enables local reproducibility; worker mode enables `server-2` integration. | - Pending |
| Integrate worker through RabbitMQ and S3-compatible storage | Keeps parsing service independent and fits the proposed `server-2` flow. | - Pending |
| Keep canonical identity outside parser | Nicknames, SteamIDs, and real players are many-to-many; `server-2` owns matching. | - Pending |
| Keep PostgreSQL persistence outside parser | Parser output should be an explicit contract, not direct table mutation. | - Pending |
| Version the parser output contract | `server-2` must be able to audit, compare, and recalculate safely. | - Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check - still the right priority?
3. Audit Out of Scope - reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-24 after initialization*
