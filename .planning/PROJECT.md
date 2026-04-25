# replays-parser-2

## What This Is

`replays-parser-2` is a Rust replay parsing application for Solid Stats. It parses local OCAP JSON replay files into deterministic normalized raw events plus aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public SolidGames statistics.

Solid Stats is a multi-project product made of `sg-replay-parser-2`, `server-2`, and `web`. This project owns the parsing engine and parsing result contract only. Public website behavior, Steam OAuth, moderation UI, correction requests, canonical player identity, and PostgreSQL business persistence belong to `server-2` and `web`.

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
- [ ] Support the vehicle score metric from GitHub issue #13, based only on kills from vehicles, weighted by attacker vehicle type and killed entity type, with teamkill penalties clamped to at least 1.
- [ ] Version the parser output contract and include source references that allow aggregates to be traced back to normalized events.
- [ ] Preserve observed identifiers from the replay without attempting canonical player matching.
- [ ] Represent missing winner, SteamID, and other absent identity fields explicitly as unknown/null states.
- [ ] Produce structured parse failures tied to replay file and job identifiers.
- [ ] Use `~/sg_stats` historical data for golden tests and old-vs-new result comparisons.
- [ ] Enforce 100% statement, branch, function, and line coverage for reachable production Rust code, with unit and regression tests following the `unit-tests-philosophy` RITE/AAA/TDD standards.
- [ ] Keep root `README.md` current with project scope, current GSD phase, architecture direction, validation data, user-facing commands, and the required AI + GSD-only development workflow.
- [ ] Leave the git working tree clean after every completed work session by committing intended results; never delete completed work just to make the tree clean, and ask the user when ownership or commit intent is unclear.
- [ ] Require AI agents to challenge instructions that conflict with architecture, current logic, accepted decisions, test/quality standards, maintainability, or proportional scope; agents must explain the risk, propose safer alternatives, and ask for explicit confirmation before any risky override.
- [ ] Treat Solid Stats as a multi-project product composed of `sg-replay-parser-2`, `server-2`, and `web`; before executing tasks, verify the change does not contradict those applications and remains compatible with their contracts and ownership boundaries.
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

The product is split across three applications:

- `sg-replay-parser-2` owns OCAP replay parsing, deterministic parse artifacts, parser contract schema, CLI/worker modes, and old-parser parity.
- `server-2` owns PostgreSQL persistence, APIs, canonical identity, Steam OAuth, roles, moderation, parse job orchestration, aggregate/bounty calculation, and operational visibility.
- `web` owns the browser UI, public stats experience, authenticated request UX, moderator/admin screens, and API consumption from `server-2`.

Every project-changing task must be checked against these application boundaries before execution. Parser changes must stay compatible with `server-2` message/API/storage expectations and `web` user-facing data needs, or explicitly call out the cross-project change required.

The current historical reference data lives at `~/sg_stats`:

- `~/sg_stats/raw_replays` contains around 3,938 OCAP JSON replay files.
- `~/sg_stats/results` contains existing calculated results.
- `~/sg_stats/lists/replaysList.json` contains replay list metadata.
- The archive is for tests and golden validation only, not production import.

The old parser lives at `/home/afgan0r/Projects/SolidGames/replays-parser` and is a required behavioral reference for this project. The Rust parser is a replacement, but it must be based on the old parser's observed parsing behavior, statistics semantics, output fields, runtime assumptions, and known exclusions before deliberately changing anything.

Important old parser facts:

- It is a TypeScript/Node project named `sg-replay-parser`.
- Main parse command: `pnpm run parse`, which runs `tsx src/start.ts`.
- Compiled parse command: `pnpm run parse:dist`, which runs `node dist/start.js`.
- Existing architecture reference: `/home/afgan0r/Projects/SolidGames/replays-parser/docs/architecture.md`.
- Main runtime entrypoints include `src/start.ts`, `src/index.ts`, `src/schedule.ts`, `src/jobs/prepareReplaysList/start.ts`, and `src/!yearStatistics/index.ts`.
- The old pipeline stages are replay discovery/download in `src/jobs/prepareReplaysList/*`, replay selection/worker dispatch in `src/1 - replays/*`, single-replay parsing in `src/2 - parseReplayInfo/*`, aggregation in `src/3 - statistics/*`, and output publication in `src/4 - output/*`.
- The old parser uses worker threads and a file-backed runtime rooted at `~/sg_stats`.
- Existing config exclusions in the old parser, such as `config/excludeReplays.json`, `config/includeReplays.json`, and `config/excludePlayers.json`, are compatibility inputs to understand before defining parity.

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

GitHub issue #13 adds a required vehicle score statistic: https://github.com/solid-stats/sg-replay-parser/issues/13. The score counts kills from vehicles only, divides by the count of games where the player had at least one kill from a vehicle, and uses a weight matrix where the attacker vehicle type is the row and killed entity type is the column.

Vehicle score weight matrix:

| Attacker vehicle | Static weapon | Car | Truck | APC | Tank | Heli | Plane | Player |
|------------------|---------------|-----|-------|-----|------|------|-------|--------|
| Static weapon | 1 | 1 | 1 | 1 | 1.5 | 2 | 2 | 2 |
| Car | 1 | 1 | 1 | 1 | 1.5 | 2 | 2 | 2 |
| Truck | 1 | 1 | 1 | 1 | 1.5 | 2 | 2 | 2 |
| APC | 0.5 | 1 | 1 | 1 | 1 | 2 | 2 | 2 |
| Tank | 0.25 | 0.5 | 0.5 | 0.5 | 1 | 1.5 | 2 | 2 |
| Heli | 0.5 | 0.5 | 1 | 1 | 1.5 | 1.5 | 2 | 2 |
| Plane | 0.25 | 0.5 | 0.5 | 0.5 | 1 | 1.5 | 2 | 2 |

For teamkills, the penalty multiplier must not be below 1 even if the normal matrix value is lower. The parser output must expose enough source references to audit each contribution to this score.

Open implementation details for later phases:

- Exact README sections and command examples once the Rust workspace, CLI, worker, validation, and benchmark commands exist.
- Exact old parser command used for baseline benchmark.
- Exact old/new comparison tolerances.
- Final normalized JSON schema names and field types.
- Final contract field name for vehicle score and whether `server-2` stores the derived score or recalculates it from parser-provided vehicle score inputs.
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
- **Brownfield reference**: `/home/afgan0r/Projects/SolidGames/replays-parser` - new behavior must be grounded in old parser semantics and comparison tests.
- **Test coverage**: 100% reachable-code statement, branch, function, and line coverage is a release gate; exclusions must be explicit, justified, and allowlisted.
- **Development workflow**: Project development is performed only by AI agents using GSD; README and planning artifacts must make that workflow visible and current.
- **Git hygiene**: Completed work must end with a clean git working tree by committing intended results, not by deleting or reverting them; ambiguous changes require asking the user.
- **AI pushback**: Agents must not blindly execute requests that violate current architecture, logic, quality, maintainability, or proportional scope; they must explain the issue, propose better options, and ask for explicit confirmation before a risky override.
- **Cross-application compatibility**: Changes must be checked against `server-2` and `web` ownership and contracts before execution; incompatible changes need an explicit cross-project plan or user confirmation.

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
| Base v1 behavior on old `replays-parser` | The legacy TypeScript parser is the only authoritative implementation of current SolidGames parsing/statistics behavior. | - Pending |
| Include vehicle score from issue #13 | Explicit user-requested statistic that depends on correct vehicle kill context and teamkill penalty semantics. | - Pending |
| Require 100% reachable-code test coverage | Parser trust depends on behavior tests that catch regressions, not only golden parity; coverage gates must be paired with RITE/AAA tests and mutation/fault checks. | - Pending |
| Maintain README as current public project context | README is the first repository-facing contract for scope, status, commands, and workflow; it must clearly state that development happens only through AI + GSD. | - Pending |
| Require clean git tree after completed work | Clean status makes handoff and review reliable; intended results should be committed, while ambiguous or user-owned changes require explicit user direction. | - Pending |
| Require AI pushback on bad instructions | Blind compliance can damage architecture and project velocity; agents should explain why a request is risky and offer safer GSD-compatible alternatives. | - Pending |
| Treat Solid Stats as a multi-project product | Parser work must remain compatible with `server-2` and `web`; checking adjacent application contracts prevents local parser changes from breaking product flows. | - Pending |

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
*Last updated: 2026-04-25 after adding multi-project product compatibility requirements*
