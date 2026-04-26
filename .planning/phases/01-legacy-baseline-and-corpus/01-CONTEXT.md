# Phase 1: Legacy Baseline and Corpus - Context

**Gathered:** 2026-04-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 1 establishes the reproducible legacy baseline and historical corpus facts that define v1 parity. It documents the old parser command/runtime, corpus manifests, legacy filters/skip rules/config inputs, output surfaces, and mismatch taxonomy. It does not implement the Rust parser, define the final parse artifact contract, calculate production website statistics, or add annual/yearly nomination statistics to v1.

</domain>

<decisions>
## Implementation Decisions

### Legacy Baseline
- **D-01:** Use the old parser source command as the canonical baseline: `pnpm run parse`, which runs `tsx src/start.ts`.
- **D-02:** Create a reproducible baseline manifest recording commit, Node version, pnpm version, lockfile hash, env vars, command, worker count, log path, and output hashes.
- **D-03:** Capture two worker-count profiles: deterministic `WORKER_COUNT=1` and a legacy/default or production-like profile for performance context.
- **D-04:** Phase 1 should execute a full old-parser baseline run over the available corpus and capture logs/output hashes.
- **D-05:** Full baseline execution must be non-destructive or explicitly backed up/isolated. The plan must not overwrite current `~/sg_stats/results` or `~/sg_stats/year_results` without preserving the historical baseline first.

### Corpus Manifest
- **D-06:** Corpus manifest should include schema/profile evidence, not only counts: top-level keys, event/entity shape summaries, largest files, malformed files, and game-type distribution.
- **D-07:** Profile the full corpus before selecting fixtures, then choose fixtures from actual distribution and anomaly reports.
- **D-08:** Treat current `~/sg_stats/results` and regenerated old-parser outputs as dual evidence; classify any differences explicitly.
- **D-09:** Commit compact dossier/summary artifacts plus small deterministic fixture or index files when useful. Keep full raw corpus, bulky profiler outputs, full hashes, regenerated results, and heavy reports local/generated or ignored.

### Legacy Rules
- **D-10:** Legacy game-type filters and skip rules are owned by the parity harness, not the parser core contract.
- **D-11:** Split observed identity from legacy compatibility identity. Phase 1 maps every old identity/name rule; Phase 2 keeps observed identity raw in the contract; later aggregate parity may use a named compatibility identity layer where required.
- **D-12:** Suspected legacy bugs require a human-review gate. Do not preserve or fix suspected old bugs without explicit user approval.
- **D-13:** Phase 1 should inventory every legacy output surface/path, but detailed v1 field mapping is required only for ordinary stats. Annual/yearly nomination outputs are listed as v2-deferred references, not folded into v1 ordinary aggregates.

### Handoff and Taxonomy
- **D-14:** Old-vs-new mismatch taxonomy must include whether a diff affects only parser artifacts, `server-2` persistence/recalculation, or UI-visible public stats.
- **D-15:** Phase 1 should create interface notes for `server-2` and `web` impact, without changing adjacent apps during this phase.
- **D-16:** Phase 1 should leave separate focused documents for baseline command/runtime, corpus manifest/profile, legacy rules/output fields, and mismatch taxonomy/interface notes.
- **D-17:** Use strict verification: plans must include verification commands for every deliverable and should not close the phase if a baseline/corpus claim cannot be reproduced locally.

### the agent's Discretion
No discretionary implementation choices were delegated. Planner may choose exact filenames and local generated-artifact paths if they follow the decisions above and keep the git tree reviewable.

</decisions>

<specifics>
## Specific Ideas

- Annual/yearly statistics are a separate legacy entity, not regular aggregate results. Legacy `src/!yearStatistics` and `~/sg_stats/year_results` are historical references only in v1; product support is deferred to v2.
- Full baseline execution is allowed in Phase 1, but must preserve current historical outputs before any old-parser command can mutate `~/sg_stats/results` or `~/sg_stats/year_results`.
- `pnpm run parse` is the canonical old-parser command for parity; `pnpm run parse:dist` can be documented as secondary context but is not the primary baseline.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project scope and roadmap
- `.planning/PROJECT.md` - Project boundaries, legacy parser facts, corpus locations, v1/v2 scope decisions, and product-wide compatibility rules.
- `.planning/REQUIREMENTS.md` - Requirement traceability, v1 requirements, and v2-deferred annual nomination decision.
- `.planning/ROADMAP.md` - Phase 1 goal, success criteria, and requirement mapping.
- `.planning/STATE.md` - Current GSD state and accumulated decisions.
- `README.md` - Human-facing project status, architecture direction, data references, and workflow expectations.

### Prior research
- `.planning/research/SUMMARY.md` - Research synthesis and phase ordering rationale.
- `.planning/research/ARCHITECTURE.md` - Old-parser architecture, target architecture, and migration map.
- `.planning/research/FEATURES.md` - Feature landscape, old-parser parity needs, and anti-features.
- `.planning/research/PITFALLS.md` - Critical migration pitfalls and old-parser compatibility risks.

### Cross-application boundaries
- `gsd-briefs/replay-parser-2.md` - Parser-specific product brief and v1/v2 boundaries.
- `gsd-briefs/server-2.md` - Backend ownership of persistence, canonical identity, aggregate calculation, and public APIs.
- `gsd-briefs/web.md` - Frontend ownership and API-consumption boundaries.

### Legacy parser reference
- `/home/afgan0r/Projects/SolidGames/replays-parser/package.json` - Legacy commands, including `pnpm run parse` and `pnpm run parse:dist`.
- `/home/afgan0r/Projects/SolidGames/replays-parser/docs/architecture.md` - Legacy runtime architecture and file-backed `~/sg_stats` model.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/start.ts` - Legacy source-command entrypoint target.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/index.ts` - Main parse orchestration, game-type selection, worker pool usage, aggregation, and output generation.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/0 - utils/runtimeConfig.ts` - Legacy `WORKER_COUNT` behavior.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/0 - utils/paths.ts` - Legacy `~/sg_stats` path contract.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/1 - replays/getReplays.ts` - Legacy game-type filtering and `sgs` exclusion.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/getEntities.ts` - Entity extraction and connected-player backfill behavior.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/getKillsAndDeaths.ts` - Legacy kill, death, teamkill, and vehicle-kill behavior.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/combineSamePlayersInfo.ts` - Duplicate-slot same-name merge compatibility behavior.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/!yearStatistics` - v2-deferred yearly nomination reference only.
- `/home/afgan0r/Projects/SolidGames/replays-parser/config/excludeReplays.json` - Legacy replay exclusion input.
- `/home/afgan0r/Projects/SolidGames/replays-parser/config/includeReplays.json` - Legacy manual include/game-type input.
- `/home/afgan0r/Projects/SolidGames/replays-parser/config/excludePlayers.json` - Legacy player exclusion input.

### Historical data
- `~/sg_stats/raw_replays` - Historical OCAP raw corpus.
- `~/sg_stats/results` - Existing ordinary calculated results for golden/parity reference.
- `~/sg_stats/lists/replaysList.json` - Replay list metadata.
- `~/sg_stats/config/nameChanges.csv` - Legacy name/id compatibility input.
- `~/sg_stats/year_results` - v2-deferred yearly nomination output reference only.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Legacy `package.json` scripts provide the canonical source command and secondary compiled command for baseline documentation.
- Legacy `src/0 - utils/runtimeConfig.ts` defines default and clamped worker-count behavior that Phase 1 must record.
- Legacy `src/0 - utils/paths.ts` defines the file-backed `~/sg_stats` contract that baseline commands may mutate.
- Existing `~/sg_stats` directories provide the raw corpus, ordinary results, replay list metadata, name-change config, and v2-deferred annual nomination outputs.

### Established Patterns
- Old parser pipeline is file-backed and stage-based: discover/download, select/dispatch, per-replay parse, aggregate, output publish.
- Old parser aggregates ordinary stats from parsed replay results; annual nomination statistics live in a separate `src/!yearStatistics` pipeline and are not ordinary v1 stats.
- The new parser must preserve observed identity in normalized artifacts; any old name-change or same-name behavior belongs to a named compatibility layer for parity.
- Parser core should not own game-type selection for comparison; the parity harness owns legacy filter application.

### Integration Points
- Phase 1 docs feed Phase 2 contract planning and Phase 5 old-vs-new comparison/benchmark work.
- Interface notes must flag any Phase 1 discoveries that can affect `server-2` persistence/recalculation or `web` UI-visible stats.
- Annual/yearly nominations are deferred to v2 and should not affect v1 parser contract, server APIs, or web pages except as documented historical reference.

</code_context>

<deferred>
## Deferred Ideas

- Annual/yearly nomination statistics and nomination pages are deferred to v2. The legacy `src/!yearStatistics` pipeline and `~/sg_stats/year_results` should be preserved as references but not implemented as v1 product support.

</deferred>

---

*Phase: 01-legacy-baseline-and-corpus*
*Context gathered: 2026-04-25*
