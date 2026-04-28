---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 05-01-PLAN.md
last_updated: "2026-04-28T05:37:11.027Z"
last_activity: 2026-04-28
progress:
  total_phases: 7
  completed_phases: 4
  total_plans: 30
  completed_plans: 26
  percent: 87
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-28)

**Core value:** Parse OCAP JSON replays quickly and deterministically into normalized raw events plus aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public statistics.
**Current focus:** Phase 05 — cli-golden-parity-benchmarks-and-coverage-gates

## Current Position

Phase: 05 (cli-golden-parity-benchmarks-and-coverage-gates) — EXECUTING
Plan: 3 of 6
Status: Ready to execute
Last activity: 2026-04-28

Progress: [█████████░] 87%

## Performance Metrics

**Velocity:**

- Total plans completed: 26
- Average duration: N/A
- Total execution time: 0.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 | 5 | - | - |
| 02 | 6 | - | - |
| 03 | 6 | 62m23s | 10m24s |
| 04 | 7/7 | 96m40s | 13m49s |
| 05 | 2/6 | 35m | 17m30s |

**Recent Trend:**

- Last 5 plans: N/A
- Trend: N/A

*Updated after each plan completion*
| Phase 02 P00 | 10m26s | 2 tasks | 18 files |
| Phase 02 P01 | 5m22s | 3 tasks | 10 files |
| Phase 02 P02 | 3m51s | 3 tasks | 4 files |
| Phase 02 P03 | 4m53s | 3 tasks | 5 files |
| Phase 02 P04 | 8m47s | 4 tasks | 11 files |
| Phase 02 P05 | planned | 4 tasks | 16 files |
| Phase 02 P05 | 7m24s | 4 tasks | 16 files |
| Phase 03 P00 | 11m44s | 2 tasks | 5 files |
| Phase 03 P01 | 6m39s | 2 tasks | 9 files |
| Phase 03 P02 | 14m | 2 tasks | 9 files |
| Phase 03 P03 | 11m | 2 tasks | 8 files |
| Phase 03 P04 | 7m | 3 tasks | 8 files |
| Phase 03 P05 | 12m | 4 tasks | 7 files |
| Phase 04 P00 | 14m | 4 tasks | 17 files |
| Phase 04 P01 | 5m31s | 3 tasks | 4 files |
| Phase 04 P02 | 8m27s | 3 tasks | 5 files |
| Phase 04 P03 | 11m45s | 4 tasks | 5 files |
| Phase 04 P04 | 8m27s | 3 tasks | 5 files |
| Phase 04 P05 | 8m30s | 4 tasks | 6 files |
| Phase 04 P06 | 40m | 4 tasks | 14 files |
| Phase 05 P00 | 22m | 4 tasks | 7 files |
| Phase 05 P01 | 13min | 3 tasks | 5 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- V1 behavior must be grounded in the old TypeScript parser at `/home/afgan0r/Projects/SolidGames/replays-parser`.
- `~/sg_stats` historical data is the golden/test baseline, not a production import source.
- Parser output preserves observed identifiers only; canonical player matching and PostgreSQL persistence belong to `server-2`.
- Vehicle score from GitHub issue #13 is in Phase 4 event/aggregate semantics.
- README.md must stay current and explicitly state that project development uses only AI agents plus GSD workflow.
- Completed work must leave the git tree clean by committing intended results; never delete completed work just to make status clean, and ask when unclear.
- AI agents must challenge requests that conflict with project logic, architecture, quality, maintainability, or proportional scope; they should explain the risk, offer safer alternatives, and ask for explicit confirmation before a risky override.
- Solid Stats consists of `replays-fetcher`, `replay-parser-2`, `server-2`, and `web`; tasks must be checked for compatibility with adjacent application contracts and ownership before execution.
- GSD workflow rules apply product-wide across all four apps; compatibility checks are risk-based, escalating from local docs/briefs to adjacent app docs/repos or a user question when cross-app risk exists.
- Frontend API typing should use `openapi-typescript` generated from the `server-2` OpenAPI schema; parser contract changes that surface in APIs must account for this type-generation flow.
- README.md should primarily serve humans and developers as the repository entry point; AI/GSD workflow rules belong in a dedicated development section, not as the whole document.
- `replays-fetcher` owns production replay discovery/fetching, S3 `raw/` object writes, and ingestion staging/outbox records; `server-2` promotes staged records into `replays` and `parse_jobs`.
- Successful parser-worker results are stored as S3 `artifacts/` objects and reported to `server-2` by artifact reference over RabbitMQ.
- Parser-core is now pure and transport-free: it accepts bytes plus caller metadata, decodes OCAP JSON with `serde_json`, normalizes replay metadata and observed entity facts, emits capped diagnostics/partial status for schema drift, and keeps `produced_at` unset for deterministic output.
- Connected-player backfill and duplicate-slot same-name legacy behavior are preserved as auditable observed facts/hints; parser-core still does not perform canonical player matching.
- Phase 4 planning splits event/aggregate work into seven execution plans: contract extensions, raw killed-event accessors, combat normalization, aggregate projections/bounty inputs, vehicle score, commander/outcome side facts, and final schema/README/quality gates.
- Phase 4 Plan 00 added schema-visible combat payloads, aggregate contribution helper schemas, vehicle score evidence payloads, and replay-side commander/outcome facts without introducing canonical identity, persistence, queue/storage, API, or UI ownership changes.
- Phase 4 Plan 01 added tolerant raw killed-event tuple observations and event-coordinate source refs without adding semantic counters, aggregate projections, or parser artifact event population.
- Phase 4 Plan 02 normalized source killed tuples into typed combat events with bounty eligibility/exclusion metadata, legacy counter effects, source refs, and data-loss diagnostics for unknown actor cases.
- Phase 4 Plan 03 derives auditable per-replay legacy, relationship, game-type, squad, rotation, and bounty projections from normalized combat events without canonical player IDs or downstream totals.
- Phase 4 Plan 04 emits issue #13 vehicle score award/penalty inputs, denominator eligibility rows, raw/applied teamkill penalty weights, and source refs without computing final cross-replay score.
- Phase 4 Plan 05 emits typed replay-side outcome facts and commander candidates with conservative known/unknown semantics, confidence, rule IDs, and source refs without canonical commander truth.
- Phase 4 review fixes add legacy player eligibility across combat and aggregate projections, zero-counter rows for eligible players, raw vehicle-class category mapping, friendly vehicle/static teamkill penalties, vehicle-score entity source refs, typed aggregate payload schema conditionals, conflicting outcome diagnostics, tokenized commander matching, and malformed killed-event diagnostics.
- Phase 4 verification passed with UAT, security, validation, schema freshness, full Cargo quality gate, and boundary grep evidence. Phase 5 can plan CLI, golden parity, benchmarks, and coverage gates on top of verified parser-core artifacts.
- [Phase 05]: Plan 00 locks replay-parser-2 as the public local binary with parse, schema, and reserved compare subcommands. — Matches Phase 5 CLI command contract and keeps old sg-replay-parser name as legacy baseline context only.
- [Phase 05]: Plan 00 CLI parse computes local SHA-256 and writes parser-core ParseArtifact JSON for success and parser failures. — Keeps filesystem/stderr concerns in the CLI adapter and leaves parser-core pure.
- [Phase 05]: Plan 00 schema command exports parser_contract::schema::parse_artifact_schema as the source of truth. — CLI schema output is byte-checked against the committed parse-artifact schema.
- [Phase 05]: Plan 01 reuses existing compact Phase 3/4 focused fixtures via golden manifest links. — Avoids duplicating OCAP payloads while keeping fixture coverage traceable and executable.
- [Phase 05]: Plan 01 golden behavior tests assert public parser-core artifacts through parse_replay. — Keeps tests behavior-oriented and avoids production-only exports.

### Pending Todos

None yet.

### Blockers/Concerns

None yet.

### Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| 260425-fd2 | Added mandatory 100% test coverage requirements using `unit-tests-philosophy` | 2026-04-25 | docs-only |  | [260425-fd2-replay-parser-2-100-unit-tests-philos](./quick/260425-fd2-replay-parser-2-100-unit-tests-philos/) |
| 260425-fgb | Added README maintenance and AI+GSD development workflow requirements | 2026-04-25 | docs-only |  | [260425-fgb-readme-md-gsd](./quick/260425-fgb-readme-md-gsd/) |
| 260425-fj0 | Added clean git tree completion requirements | 2026-04-25 | docs-only |  | [260425-fj0-git](./quick/260425-fj0-git/) |
| 260425-fln | Added AI pushback and safer-alternative workflow requirements | 2026-04-25 | docs-only |  | [260425-fln-ai-pushback-policy](./quick/260425-fln-ai-pushback-policy/) |
| 260425-fnz | Added multi-project product compatibility requirements | 2026-04-25 | docs-only |  | [260425-fnz-replay-parser-2-server-2-web](./quick/260425-fnz-replay-parser-2-server-2-web/) |
| 260425-fro | Clarified product-wide GSD rules and risk-based compatibility checks | 2026-04-25 | docs-only |  | [260425-fro-clarify-product-wide-gsd-rules-and-risk-](./quick/260425-fro-clarify-product-wide-gsd-rules-and-risk-/) |
| 260425-fxa | Added `openapi-typescript` API typing guidance to all project briefs | 2026-04-25 | docs-only |  | [260425-fxa-add-openapi-typescript-to-web-typing-bri](./quick/260425-fxa-add-openapi-typescript-to-web-typing-bri/) |
| 260425-g0r | Rewrote README as a human-facing project entry point | 2026-04-25 | docs-only |  | [260425-g0r-rewrite-readme-for-humans-and-developers](./quick/260425-g0r-rewrite-readme-for-humans-and-developers/) |
| 260426-eja | Renamed project identity to `replay-parser-2` | 2026-04-26 | docs-only |  | [260426-eja-rename-project-to-replay-parser-2](./quick/260426-eja-rename-project-to-replay-parser-2/) |
| 260426-joq | Added strict stable Rust lint, format, docs, and type-safety gates | 2026-04-26 | 7ad4af4 | Verified | [260426-joq-strict-quality-rules](./quick/260426-joq-strict-quality-rules/) |
| 260426-rfs | Added `replays-fetcher` product boundary and S3 artifact-reference result policy | 2026-04-26 | docs-only | Verified | [260426-rfs-replays-fetcher-boundary](./quick/260426-rfs-replays-fetcher-boundary/) |

## Deferred Items

Items acknowledged and carried forward from previous milestone close:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

## Session Continuity

Last session: 2026-04-28T05:37:11.023Z
Stopped at: Completed 05-01-PLAN.md
Resume file: None

**Completed Phase:** 01 (Legacy Baseline and Corpus) — 5 plans — 2026-04-25
**Completed Phase:** 02 (Versioned Output Contract) — 6 plans — 2026-04-26
**Completed Phase:** 03 (Deterministic Parser Core) — 6 plans — 2026-04-27
**Completed Phase:** 04 (Event Semantics and Aggregates) — 7 plans — 2026-04-28
**Next Step:** Phase 5 planning — CLI, golden parity, benchmarks, and coverage gates
