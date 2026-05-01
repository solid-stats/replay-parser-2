# replay-parser-2

## What This Is

`replay-parser-2` is a Rust replay parsing application for Solid Stats. It parses local OCAP JSON replay files into compact, deterministic server-facing parse artifacts that `server-2` can persist, audit at the contribution level, compare against golden data, and use for public SolidGames statistics.

The default v1 parser output must reduce replay information for `server-2`; it must not simply translate a 10-15 MB OCAP JSON file into another 10-15 MB JSON file. Full normalized event/entity dumps are not part of the default server artifact. If deeper evidence is needed for debugging, parity review, or future analytics, it belongs in optional sidecar/debug tooling or raw replay reprocessing, not in the primary worker artifact.

Solid Stats is a multi-project product made of `replays-fetcher`, `replay-parser-2`, `server-2`, and `web`. This project owns the parsing engine and parsing result contract only. Replay discovery/fetching, public website behavior, Steam OAuth, moderation UI, correction requests, canonical player identity, and PostgreSQL business persistence belong to adjacent applications.

## Core Value

Parse OCAP JSON replays quickly and deterministically into compact server-facing statistics artifacts with enough contribution evidence for `server-2` to persist, audit, compare against golden data, and use for public statistics.

## Current State

Phase 5 execution produced the local `replay-parser-2` CLI, compact golden behavior fixtures, selected old-vs-new comparison reports, strict coverage postprocessing, deterministic fault-report gating, and benchmark report validation, but UAT rejected the current parser direction as a product-fit gap. The current parser mostly reserializes a large normalized artifact instead of reducing replay information for `server-2`; the curated old/new benchmark records only a small speedup, `ten_x_status=fail`, and `parity_status=human_review`; and the generated comparison surface is too large for practical human review.

Phase 5.1 redesigned the default artifact as a compact server-facing result, moved full event/entity detail out of the default contract, and added a selective parser boundary, but its acceptance evidence is still not sufficient for worker integration.

Phase 5.2 has been inserted before worker integration to make the default artifact minimal rather than merely compact, record the product decision to remove GitHub issue #13 vehicle score from v1, put detailed evidence behind an explicit debug sidecar, and prove the new performance gates: x3 on one large representative replay and x10 across all files in `~/sg_stats/raw_replays`.

## Requirements

### Validated

- [x] Phase 1 validated current README/project documentation requirements, including the AI agents using GSD workflow rule.
- [x] Phase 1 validated clean git handoff, AI pushback, and risk-based cross-application compatibility workflow requirements.
- [x] Phase 1 validated Solid Stats ownership boundaries across `replay-parser-2`, `server-2`, and `web`.
- [x] Phase 1 validated the legacy baseline command/runtime dossier, corpus manifest, legacy rule/output inventory, and old-vs-new mismatch taxonomy.
- [x] Phase 2 validated the versioned `ParseArtifact` and `ParseFailure` contract shape, generated JSON Schema, explicit presence states, observed identity boundary, checksum/source-reference invariants, and schema-backed example validation.
- [x] Phase 3 validated deterministic parser-core output, tolerant OCAP root/metadata/entity normalization, schema-drift diagnostics, connected-player backfill, and duplicate-slot same-name compatibility hints without canonical identity matching.
- [x] Phase 4 validated normalized combat/outcome semantics, legacy and relationship aggregate projections, bounty inputs, issue #13 vehicle score inputs, source-reference-backed recalculation evidence, and replay-side commander/outcome facts without parser-owned persistence or canonical identity.

### Active

- [x] Provide a CLI that parses a local OCAP JSON file and writes parser output JSON.
- [ ] Redesign the default parser output as a compact server-facing artifact rather than a full normalized event/entity dump.
- [ ] Replace the current compact artifact with a minimal flat v1 statistics artifact containing `players[]`, `player_stats[]`, `kills[]`, `destroyed_vehicles[]`, and `diagnostics[]`.
- [ ] Move heavy audit/parity/debug detail out of the default worker artifact; optional sidecars must not be required for ordinary `server-2` ingestion.
- [ ] Emit detailed event/entity/source-reference evidence only through an explicit debug sidecar mode, not through ordinary parser output.
- [ ] Remove GitHub issue #13 vehicle score and `vehicle_score` output from the v1 contract, schema, examples, tests, docs, and plans.
- [ ] Implement a selective parser path that avoids full JSON-to-JSON roundtrips and reads only the OCAP metadata, entities, and events required for v1 statistics.
- [ ] Replace unreadable large comparison output with summary-first parity reports plus machine-readable detailed evidence.
- [ ] Provide a worker/container mode that consumes parse jobs from RabbitMQ and reads replay files from S3-compatible storage.
- [x] Use `~/sg_stats` historical data for golden tests and old-vs-new result comparisons.
- [x] Enforce 100% statement, branch, function, and line coverage for reachable production Rust code, with unit and regression tests following the `unit-tests-philosophy` RITE/AAA/TDD standards.
- [ ] Include benchmark evidence that establishes the current parser baseline, proves x3 end-to-end CLI speedup on one large representative replay, proves x10 end-to-end throughput across all raw files in `~/sg_stats/raw_replays`, and reports artifact-size percentiles.

### Out of Scope

- Replay discovery and production fetching from the external replay source - owned by `replays-fetcher`.
- Public website and UI - owned by `web`.
- Steam OAuth - owned by `server-2`/`web`.
- User roles, moderation, and correction request workflow - owned by `server-2` and `web`.
- PostgreSQL persistence as parser-owned source of truth - `server-2` owns business tables and persistence.
- Direct editing or correction of stats - parser emits artifacts, server applies corrections/recalculations.
- Replay formats other than OCAP JSON - v1 targets OCAP JSON only.
- Production Kubernetes deployment - container/worker readiness is needed, production orchestration is not.
- Financial rewards or payout logic - parser only emits bounty calculation inputs.
- Annual/yearly nomination statistics - legacy `src/!yearStatistics` and `~/sg_stats/year_results` are separate from normal stats and are deferred to v2.

## Context

Solid Stats is a public SolidGames statistics platform that replaces the current replay-parser/statistics workflow. It needs fast, trustworthy replay parsing, public player/squad/rotation/commander-side statistics, player-submitted correction requests, and bounty points based on player and squad effectiveness.

The product is split across four applications:

- `replays-fetcher` owns replay discovery from the external source, raw S3 object writes, source metadata, and ingestion staging/outbox records.
- `replay-parser-2` owns OCAP replay parsing, deterministic parse artifacts, parser contract schema, CLI/worker modes, and old-parser parity.
- `server-2` owns PostgreSQL persistence, APIs, canonical identity, Steam OAuth, roles, moderation, parse job orchestration, aggregate/bounty calculation, and operational visibility.
- `web` owns the browser UI, public stats experience, authenticated request UX, moderator/admin screens, and API consumption from `server-2`.

Every project-changing task must be checked against these application boundaries before execution. Parser changes must stay compatible with `replays-fetcher` raw object/checksum assumptions, `server-2` message/API/storage expectations, and `web` user-facing data needs, or explicitly call out the cross-project change required.

GSD workflow rules are product-wide standards for all four applications. Compatibility checks are risk-based:

- For local-only parser documentation or implementation changes, checking this repo's planning docs, README, AGENTS rules, and `gsd-briefs` is enough.
- For parser contract, RabbitMQ/S3 job message, raw replay object key/checksum assumptions, artifact shape, API/data model, canonical identity, auth, moderation, or UI-visible behavior changes, agents must inspect the adjacent application docs/repos when available or ask the user before proceeding.
- If compatibility evidence is missing or contradictory, agents must pause, explain the uncertainty, and propose a smaller GSD path or a cross-project plan.

The current historical reference data lives at `~/sg_stats`:

- `~/sg_stats/raw_replays` contains 23,473 raw replay JSON files in the current full-history corpus.
- `~/sg_stats/lists/replaysList.json` contains 23,456 replay-list rows prepared at `2026-04-25T04:42:54.889Z`.
- `~/sg_stats/results` contains 88,485 existing calculated result files.
- `~/sg_stats/year_results` contains 14 legacy annual nomination output files and is a v2 reference, not ordinary v1 stats.
- The archive is for tests and golden validation only, not production import.

The old parser lives at `/home/afgan0r/Projects/SolidGames/replays-parser` and is a required behavioral reference for this project. The Rust parser is a replacement, but it must be based on the old parser's observed parsing behavior, statistics semantics, output fields, runtime assumptions, and known exclusions before deliberately changing anything.

Important old parser facts:

- It is a TypeScript/Node project named `sg-replay-parser`.
- Main parse command: `pnpm run parse`, which runs `tsx src/start.ts`.
- Compiled parse command: `pnpm run parse:dist`, which runs `node dist/start.js`.
- Existing architecture reference: `/home/afgan0r/Projects/SolidGames/replays-parser/docs/architecture.md`.
- Main runtime entrypoints include `src/start.ts`, `src/index.ts`, `src/schedule.ts`, `src/jobs/prepareReplaysList/start.ts`, and `src/!yearStatistics/index.ts`.
- The old pipeline stages are replay discovery/download in `src/jobs/prepareReplaysList/*`, replay selection/worker dispatch in `src/1 - replays/*`, single-replay parsing in `src/2 - parseReplayInfo/*`, aggregation in `src/3 - statistics/*`, and output publication in `src/4 - output/*`.
- The old yearly statistics pipeline under `src/!yearStatistics` is a separate legacy surface that produces annual nomination outputs in `~/sg_stats/year_results`; v1 should treat it as historical reference material only, with product support deferred to v2.
- The old parser uses worker threads and a file-backed runtime rooted at `~/sg_stats`.
- Existing config exclusions in the old parser, such as `config/excludeReplays.json`, `config/includeReplays.json`, and `config/excludePlayers.json`, are compatibility inputs to understand before defining parity.

Existing result fields include `kills`, `killsFromVehicle`, `vehicleKills`, `teamkills`, `deaths`, `kdRatio`, `killsFromVehicleCoef`, `score`, `totalPlayedGames`, `week`, `startDate`, and `endDate`.

Observed OCAP top-level keys include `EditorMarkers`, `Markers`, `captureDelay`, `endFrame`, `entities`, `events`, `missionAuthor`, `missionName`, `playersCount`, and `worldName`.

The intended integration flow is:

1. `replays-fetcher` discovers a replay from the external source, stores the raw replay object under S3 `raw/`, computes checksum/source metadata, and writes an ingestion staging/outbox record.
2. `server-2` polls/promotes staging rows, handles duplicate conflicts, creates canonical `replays` and `parse_jobs`, and publishes a RabbitMQ parse request containing `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`.
3. `replay-parser-2` downloads the replay file from storage and verifies the checksum before parsing.
4. `replay-parser-2` writes a deterministic compact parse artifact under S3 `artifacts/` and emits `parse.completed` with an artifact reference, or emits `parse.failed` with structured error data. Optional debug/parity sidecars, if implemented, are separate from the default server-facing artifact.
5. `server-2` persists results into PostgreSQL and publishes aggregate statistics.

In this domain, "KS" means commander of a side. The parser should detect commander-side data when present, including replay identifier, side identifier/name, commander observed identity fields, winner/outcome if present, and source/confidence metadata if available.

Bounty points are calculated by `server-2`, but parser output must support the required inputs. For each valid kill event, the parser should output killer and victim observed identity, enemy-kill/teamkill classification, kill timestamp/frame, relevant vehicle/infantry context, replay context, and side context. Only valid enemy kills award bounty points in v1; teamkills do not.

GitHub issue #13 vehicle score was implemented as Phase 4 evidence, but it is no longer a v1 product target. Phase 5.2 must remove `vehicle_score` from the v1 contract, schema, examples, tests, docs, and default artifact. V1 still keeps ordinary `killsFromVehicle`, `vehicleKills`, weapon/vehicle context, and destroyed-vehicle facts so future work can reprocess raw OCAP files if vehicle score is revisited.

Open implementation details for later phases:

- Exact README sections and command examples once the Rust workspace, CLI, worker, validation, and benchmark commands exist.
- Exact old parser command used for baseline benchmark.
- Exact old/new comparison tolerances.
- Final minimal flat artifact field names, JSON schema names, field types, and artifact-size target.
- Exact debug sidecar CLI flag/output policy for detailed event/entity/source-reference evidence.
- Selective parser implementation strategy and acceptance thresholds for parse-only, aggregate-only, and end-to-end throughput.
- Exact deterministic artifact key format under S3 `artifacts/`.
- Exact RabbitMQ exchange and routing key naming.
- Exact `replays-fetcher`/`server-2` staging schema and raw S3 object key format are adjacent-app contracts; parser worker only relies on the `object_key` and `checksum` in parse jobs.
- How v2 should model annual/yearly nomination statistics across parser evidence, `server-2` calculation, and `web` presentation.

## Constraints

- **Language**: Rust - chosen for deterministic parsing, performance, and deployable CLI/worker binaries.
- **Replay format**: OCAP JSON only - supporting other formats is outside v1 scope.
- **Validation data**: `~/sg_stats` - historical data is the golden/test baseline, not a production import source.
- **Performance**: At least x3 faster than the current parser for end-to-end CLI parsing on one large representative replay and at least x10 faster across all raw replay files in `~/sg_stats/raw_replays`, with raw-input size, output-artifact size, artifact-size percentiles, wall time, throughput, skip/failure counts, and parity/triage status reported before any performance claim.
- **Runtime modes**: CLI plus worker/container mode - local reproducibility and server integration are both required.
- **Queue integration**: RabbitMQ - worker mode consumes parse requests and publishes parse results/failures.
- **File input**: S3-compatible object storage - parser worker reads replay content by object key/checksum.
- **Replay discovery ownership**: `replays-fetcher` discovers/fetches production replay files and stages raw S3 objects; parser never crawls the external replay source.
- **Result artifact delivery**: Successful worker parses write artifacts to S3 and publish artifact references, not full artifacts, over RabbitMQ.
- **Database ownership**: `server-2` owns PostgreSQL persistence - parser does not mutate business tables in v1.
- **Identity**: Parser preserves observed identifiers only - canonical player matching belongs to `server-2`.
- **History reprocessing**: Server may overwrite derived results in v1 - parser must make output repeatable and versioned.
- **Brownfield reference**: `/home/afgan0r/Projects/SolidGames/replays-parser` - new behavior must be grounded in old parser semantics and comparison tests.
- **Test coverage**: 100% reachable-code statement, branch, function, and line coverage is a release gate; exclusions must be explicit, justified, and allowlisted.
- **Development workflow**: Project development is performed only by AI agents using GSD; README and planning artifacts must make that workflow visible and current.
- **Git hygiene**: Completed work must end with a clean git working tree by committing intended results, not by deleting or reverting them; ambiguous changes require asking the user.
- **AI pushback**: Agents must not blindly execute requests that violate current architecture, logic, quality, maintainability, or proportional scope; they must explain the issue, propose better options, and ask for explicit confirmation before a risky override.
- **Cross-application compatibility**: Changes must be checked against `server-2` and `web` ownership and contracts before execution; incompatible changes need an explicit cross-project plan or user confirmation.
- **Risk-based compatibility depth**: Local-only changes can rely on local planning docs and `gsd-briefs`; contract, API/data, queue/storage, identity/auth, moderation, or UI-visible changes require adjacent app evidence or a user question.

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
| Make the default server artifact compact | UAT showed that reserializing a 10-15 MB replay into another 10-15 MB JSON artifact does not deliver parser value. `server-2` needs a reduced statistics/evidence artifact, not a full normalized dump. | - Pending |
| Keep full event/entity dumps out of the default v1 artifact | Detailed evidence can be useful for parity and debugging, but it should not slow or bloat ordinary worker ingestion. | - Pending |
| Reprocess raw files for annual/yearly statistics instead of adding a large v1 side artifact | Yearly nominations are deferred to v2, and raw OCAP files plus legacy yearly outputs remain the reference when that product surface is revisited. | - Pending |
| Use selective parsing before worker integration | Worker mode should not be built on a parser hot path that performs an expensive full JSON-to-JSON translation. | - Pending |
| Base v1 behavior on old `replays-parser` | The legacy TypeScript parser is the only authoritative implementation of current SolidGames parsing/statistics behavior. | - Pending |
| Retire vehicle score from issue #13 from v1 | Product decision on 2026-05-01: v1 should stay focused on minimal current statistics and can reprocess raw replays later if this statistic returns. | - Pending |
| Require 100% reachable-code test coverage | Parser trust depends on behavior tests that catch regressions, not only golden parity; coverage gates must be paired with RITE/AAA tests and mutation/fault checks. | - Pending |
| Maintain README as current public project context | README is the first repository-facing contract for scope, status, commands, and workflow; it must clearly state that development happens only through AI + GSD. | - Pending |
| Require clean git tree after completed work | Clean status makes handoff and review reliable; intended results should be committed, while ambiguous or user-owned changes require explicit user direction. | - Pending |
| Require AI pushback on bad instructions | Blind compliance can damage architecture and project velocity; agents should explain why a request is risky and offer safer GSD-compatible alternatives. | - Pending |
| Treat Solid Stats as a multi-project product | Parser work must remain compatible with `server-2` and `web`; checking adjacent application contracts prevents local parser changes from breaking product flows. | - Pending |
| Apply GSD rules product-wide with risk-based checks | The same AI/GSD standards should apply across parser, backend, and web; compatibility checks should be deep only when the requested change can affect another app. | - Pending |
| Defer annual nomination statistics to v2 | Legacy yearly statistics are a separate nomination surface, not ordinary player/squad/rotation stats; v1 should preserve the old pipeline and outputs as references without implementing product support. | - Pending |
| Split replay discovery into `replays-fetcher` | Production source crawling and raw replay staging are ingestion concerns, not parser or backend parser-worker concerns. | - Pending |
| Return worker parse results by S3 artifact reference | Keeps RabbitMQ messages small and makes parser artifacts durable/auditable for `server-2`. | - Pending |

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
*Last updated: 2026-05-01 after Phase 5.2 insertion for minimal artifact and performance acceptance*
