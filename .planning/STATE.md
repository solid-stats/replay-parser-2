---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: planning
stopped_at: Phase 7 context gathered
last_updated: "2026-05-02T17:13:39.512Z"
last_activity: 2026-05-02 -- Phase 06 completed; Phase 07 ready
progress:
  total_phases: 9
  completed_phases: 8
  total_plans: 51
  completed_plans: 51
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-01)

**Core value:** Parse OCAP JSON replays quickly and deterministically into compact server-facing statistics artifacts with enough contribution evidence for `server-2` to persist, audit, compare against golden data, and use for public statistics.
**Current focus:** Phase 07 — parallel-and-container-hardening

## Current Position

Phase: 07 (parallel-and-container-hardening) — READY
Plan: 0 of TBD
Status: Ready for Phase 07 planning/execution
Last activity: 2026-05-02 -- Phase 06 completed; Phase 07 ready
Last quick task: 2026-05-02 - Completed five deterministic year-edge `sg`/`mace`/`sm` old/new parity samples. Across 364 selected replay entries and 291 unique replay files, the new parser succeeded on all entries, the old parser produced 305 comparable artifacts and skipped 59, and no new mismatch class appeared. Remaining differences are documented accepted classes, so the Phase 05/05.2 parity follow-up is non-blocking and Phase 6 can proceed.

Progress: [█████████░] 90%

## Performance Metrics

**Velocity:**

- Total plans completed: 31
- Average duration: N/A
- Total execution time: 0.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 | 5 | - | - |
| 02 | 6 | - | - |
| 03 | 6 | 62m23s | 10m24s |
| 04 | 7/7 | 96m40s | 13m49s |
| 05 | 6/6 | 221m | 36m50s |
| 05.1 | 8/8 | executed; acceptance gap | - |
| 06 | 6/6 | executed | - |

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
| Phase 05 P02 | 65min | 4 tasks | 11 files |
| Phase 05 P03 | 73min | 3 tasks | 22 files |
| Phase 05 P04 | 14min | 3 tasks | 7 files |
| Phase 05 P05 | 34min | 4 tasks | 8 files |
| Phase 05.2 P00 | 1m | 2 tasks | 2 files |
| Phase 05.2 P01 | 10 min | 3 tasks | 18 files |
| Phase 05.2 P02 | 22m04s | 3 tasks | 21 files |
| Phase 05.2 P03 | 9m22s | 3 tasks | 5 files |
| Phase 05.2 P04 | 14m18s | 3 tasks | 4 files |
| Phase 05.2 P05 | 15m | 3 tasks | 6 files |
| Phase 05.2 P06 | 14m48s | 3 tasks | 16 files |
| Phase 06 P05 | 49m26s | 2 tasks | 23 files |

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
- [Phase 05]: Plan 02 comparison reports use Phase 1 mismatch categories plus parser/server/web impact dimensions. — Keeps selected-input parity review explicit without moving adjacent app behavior into parser-core.
- [Phase 05]: Plan 02 compare command supports saved artifacts and selected replay parsing only. — Full-corpus parity remains generated harness/CI output under .planning/generated/phase-05/.
- [Phase 05]: Plan 03 uses `cargo llvm-cov --json` plus a parser-harness postprocessor for strict reachable production coverage. — Stable Rust cannot subtract narrow source-line exclusions natively, so the committed gate fails on any unallowlisted production uncovered region and records generated evidence under `.planning/generated/phase-05/coverage/`.
- [Phase 05]: Plan 03 coverage exclusions are exact source lines with committed allowlist metadata and production-file `coverage-exclusion:` markers. — Blanket non-generated exclusions remain rejected, and behavior tests stay public-API oriented.
- [Phase 05]: Plan 04 adds a mutation/equivalent fault-report gate with deterministic fallback. — `scripts/fault-report-gate.sh` prefers `cargo mutants` when installed and otherwise generates a validated `deterministic-fault-injection` report covering vehicle score, combat events, aggregate source refs, and failure paths.
- [Phase 05]: Plan 04 fault report validation blocks high-risk `missed` cases without accepted non-applicable rationale. — Report validation remains in parser-harness and does not introduce parser-core test-only hooks.
- [Phase 05]: Plan 05 benchmark reports carry workload identity, old baseline profile, throughput, parity status, RSS note, and 10x status. — The benchmark gate now runs a curated selected old/new comparison when the old parser and `~/sg_stats` are available; the latest evidence records `ten_x_status=fail`, `parity_status=human_review`.
- [Phase 05]: Final local gates pass without adding worker, database, API, replay discovery, canonical identity, UI, or yearly nomination behavior, but UAT escalated the benchmark/parity gap into a parser artifact and performance redesign. Phase 6 still owns RabbitMQ/S3 worker integration after Phase 5.1.
- [Phase 05.1]: Inserted urgent redesign phase after UAT rejected the current parser direction. — The default server-facing artifact must become compact; full normalized event/entity dumps move out of ordinary ingestion; comparison reports must become summary-first; performance work should use selective OCAP extraction instead of optimizing a large JSON-to-JSON roundtrip.
- [Phase 05.1]: Annual/yearly nomination statistics should not force a large v1 side artifact. — Future yearly work can reprocess raw OCAP files and compare against `~/sg_stats/year_results`, matching the old-parser model more closely than carrying a heavy default document through every parse.
- [Phase 05.1]: Planning produced 8 execution plans. — Wave 1 records `server-2` compatibility and approval state; Wave 2 replaces the public contract with compact `participants`, `facts`, and `summaries`; Wave 3 replaces the normal parser hot path with selective OCAP extraction; Wave 4 preserves combat/aggregate/side semantics in compact output; Wave 5 updates CLI/schema and summary-first comparison; Wave 6 adds compact artifact-size and whole-list/corpus benchmark evidence; Wave 7 runs final quality gates and handoff docs.
- [Phase 05.1]: User accepted the server-2 compatibility gate before compact contract implementation. — `05.1-SERVER-COMPATIBILITY.md` records accepted compact artifact delivery with observed identity boundaries and no parser-owned persistence/API/UI/canonical identity behavior.
- [Phase 05.1]: The default public artifact now uses compact `participants`, `facts`, `summaries`, `side_facts`, diagnostics, status/failure, source, parser, and contract metadata. — Top-level full `entities`, `events`, and `aggregates` are no longer default serialized output.
- [Phase 05.1]: Parser-core normal input now passes through a selective root boundary instead of `serde_json::Value` full-DOM decode in the checked hot-path files. — Internal normalized semantics are still used before compact mapping where needed for behavior preservation.
- [Phase 05.1]: Comparison output is summary-first Markdown by default with explicit JSON details through `--detail-output` or `--format json`. — Compact surfaces replace old top-level event/entity comparison surfaces.
- [Phase 05.1]: Benchmark reports now require raw input bytes, compact artifact bytes, artifact/raw ratio, selected evidence, and whole-list/corpus evidence or a concrete unavailable reason. — Its benchmark gap was superseded by Phase 05.2 minimal-artifact work and the 2026-05-02 benchmark acceptance update.
- [Phase 05.2]: Discussion locked minimal default artifact decisions. — Default v1 output should be minified compact `players[]`, `weapons[]`, `destroyed_vehicles[]`, and `diagnostics[]`; player-authored enemy/team kills live under the killer `players[].kills`; no frame/time/source refs/rule IDs/event indexes/entity snapshots in default rows; full normalized detail belongs only behind an internal `--debug-artifact`-style sidecar.
- [Phase 05.2]: Discussion locked performance and issue #13 decisions. — Remove issue #13 vehicle score implementation from v1; use automatic large selected replay for x3; use sequential `WORKER_COUNT=1` old baseline and sequential new artifact writing for all-raw x10; require zero failed/skipped full-corpus artifacts unless an explicit allowlist is later approved.
- [Phase 05.2]: Planning produced 7 execution plans. — Wave 1 records minimal artifact server compatibility acceptance; Wave 2 replaces the public contract with v3 minimal flat tables and removes vehicle-score contract surfaces; Wave 3 updates parser-core minimal rows and debug sidecar; Wave 4 updates CLI/schema/README command behavior; Wave 5 derives old-vs-new parity from minimal tables; Wave 6 implements selected x3, all-raw x10, zero-failure, and artifact-size benchmark gates; Wave 7 runs final quality gates and handoff docs.
- [Phase 05.2]: Plan 00 recorded product-owner approval for minimal flat artifact implementation. — Brief-level downstream evidence is sufficient after explicit approval; server-2 will adapt later.
- [Phase 05.2]: Plan 01 cuts the parser contract to v3.0.0 minimal tables: players, weapons, nested player kills, destroyed_vehicles, and diagnostics. — Matches Phase 5.2 minimal artifact direction before parser-core construction.
- [Phase 05.2]: Issue 13 vehicle score contract types, schema helpers, projection keys, and tests are no longer active v1 parser-contract surfaces. — Ordinary vehicleKills, killsFromVehicle, weapon, and attacker vehicle context remain for current stats and future raw replay reprocessing.
- [Phase 05.2]: side_facts remains in ParseArtifact but is defaultable for minimal v3 examples. — This preserves the retained contract field while keeping success examples free of rule/source provenance fields.
- [Phase 05.2]: parser-core parse_replay now emits v3 minimal players, weapons, nested player kills, and destroyed_vehicles by default.
- [Phase 05.2]: Issue 13 vehicle score parser-core implementation was removed while ordinary vehicleKills, killsFromVehicle, attacker vehicle context, and destroyed_vehicles remain covered.
- [Phase 05.2]: Full normalized entities, events, source refs, rule IDs, side facts, and diagnostics are available only through parser-core parse_replay_debug.
- [Phase 05.2]: CLI parse now writes minified v3 minimal JSON by default; human-readable JSON requires --pretty.
- [Phase 05.2]: CLI debug sidecar output is explicit internal tooling through --debug-artifact <path> and uses parser-core parse_replay_debug only when requested.
- [Phase 05.2]: Public schema and README references now use schemas/parse-artifact-v3.schema.json and v3 example artifacts.
- [Phase 05.2]: Old-vs-new parity compares a derived legacy view from v3 minimal tables. — This preserves Phase 5.2 minimal default artifacts while keeping v1 human-review parity meaningful.
- [Phase 05.2]: Selected comparison surfaces exclude vehicle_score, compact facts, aggregate contributions, participants, and summary projections. — Final selected parity is limited to status, replay, legacy player rows, legacy relationships, and bounty inputs.
- [Phase 05.2]: Phase 05.2 benchmark acceptance must enforce max default artifact bytes <= 100_000 per successful artifact. — The benchmark report now carries explicit size evidence placeholders while Wave 6 owns the hard gate.
- [Phase 05.2]: Phase 05.2 benchmark reports use report_version 2 with selected_large_replay, all_raw_corpus, allowlist, rss_note, and artifact_size_limit_bytes.
- [Phase 05.2]: The default artifact hard limit is exactly 100000 bytes; selected size passes only with artifact_bytes <= 100000.
- [Phase 05.2]: All-raw size originally required median artifact/raw ratio <= 0.05 and p95 <= 0.10, but the 2026-05-02 acceptance update supersedes this: size acceptance now focuses on max_artifact_bytes <= 100000 and oversized_artifact_count == 0.
- [Phase 05.2]: scripts/benchmark-phase5.sh --ci emits structurally valid smoke reports with unknown statuses when full prerequisites are absent; full acceptance now also honors the 2026-05-02 user-accepted performance, max-size, and malformed-file parity policy.
- [Phase 05.2]: Plan 06 final gates passed structurally, but Phase 6 remained blocked because selected artifact_bytes=203683 exceeded the hard 100000-byte limit and full-corpus gates were unknown.
- [Phase 05.2]: Fault report gates now target parser-core::minimal_artifact and the stale active v2 vehicle-score schema was removed from maintained schema surfaces.
- [Quick 260502-ecp]: The default selected-large artifact was compacted to 40042 bytes by merging player counters into `players[]`, using numeric refs and a weapon dictionary, and omitting null/empty/zero default fields. This resolves the selected hard-size blocker only; selected x3/parity and all-raw x10/zero-failure/size acceptance still require the normal Phase 05.2 benchmark workflow.
- [Quick 260502-gn2]: Full Phase 5.2 benchmark evidence was regenerated after replacing the non-parsing old WorkerPool all-raw path with a generated direct `runParseTask` runner. Selected artifact size passes at 40042 bytes, selected x3 fails at 2.4996x, selected parity remains `human_review`, all-raw x10 is `unknown` because old coverage is incomplete, all-raw size fails p95 ratio, and all-raw zero-failure fails on 4 new-parser raw failures.
- [Quick 260502-i8w]: The generated Phase 5 benchmark directory was fully cleaned, tracked generated placeholders were removed, same-name slots now merge like the old parser, legacy tags are split from observed nicknames, player-authored kill rows now live under `players[].kills`, and the old all-raw baseline now attempts every raw file. Full evidence records selected x3 fail at 2.8190x, selected parity `human_review`, all-raw x10 fail at 1.7544x, all-raw old/new attempted_count=23473, all-raw size p95 fail at 0.12417910447761193, and zero-failure fail on the same 4 malformed raw files.
- [Quick 260502-jeh]: Default `parse_replay` now derives minimal rows directly from one-pass connected/killed observations and a replay-local vehicle/static name index, while debug parsing keeps full normalized events. Old all-raw baseline is cached at `.planning/benchmarks/phase-05-old-all-raw-baseline.json` and benchmark runs now reuse it unless `RUN_PHASE5_FULL_OLD_BASELINE=1`. Full new all-raw evidence records old cached wall `501274.528655ms`, new wall `235598.648803ms`, and speedup `2.1277x`; all-raw x10/size/zero-failure remain failed, and selected parity/x3 was not run because the old selected `tsx` baseline hit sandbox `EPERM`.
- [Phase 05.2 Acceptance Update 2026-05-02]: Product owner accepted the current benchmark performance, so historical selected x3 and all-raw x10 statuses remain reported but no longer block Phase 6 by themselves.
- [Phase 05.2 Acceptance Update 2026-05-02]: Product owner accepted p95 artifact/raw ratio above `0.10`; all-raw size acceptance now focuses on hard max artifact size `<= 100000` and `oversized_artifact_count == 0`, while median/p95 remain trend evidence.
- [Phase 05.2 Acceptance Update 2026-05-02]: Product owner accepted the 4 malformed/non-JSON all-raw failures when the old cached baseline reports the same failure count and new failure paths match `.planning/benchmarks/phase-05-all-raw-accepted-failures.json`.
- [Quick 260502-k2u]: Deterministic year-edge old/new parity sample selected 73 replays across `sg`, `mace`, and `sm`. New parser produced 73 artifacts, old parser produced 58 successful artifacts and 15 skipped rows. Compatibility fixes first improved stats-only parity from 12 matches / 46 mismatches to 55 matches / 3 mismatches among comparable old-parser successes. After the follow-up decision to preserve old-parser forbidden weapon names such as `Throw` and `Binoculars`, stats-only parity was 40 matches / 18 mismatches. After adding a compact latest-death teamkill marker for `isDeadByTeamkill`, current stats-only parity is 42 matches / 16 mismatches; 21 mismatch rows are expected `weapon_extra_in_new` differences, with one remaining old teamkill relationship merge edge row. Full evidence is under `.planning/generated/quick/260502-k2u-old-new-year-edge-parity/`, with lightweight summary artifacts committed under `.planning/quick/260502-k2u-old-new-year-edge-parity/`.
- [Quick 260502-nx9]: A second deterministic year-edge old/new parity sample used the same policy as `260502-k2u` but with seed `260502-nx9`. It selected 72 replays, overlapped the first sample by 12 replays, and added 60 unique replays. New parser succeeded on all 72, old parser succeeded on 67 and skipped 5. Stats-only parity is 52 matches / 15 mismatches; mismatch rows are 12 retained `Throw`/`Binoculars` weapon rows and 6 `isDeadByTeamkill` rows in duplicate-slot/respawn cases where the new parser follows the latest counted merged-player death while the old baseline leaves the teamkill flag true. Full evidence is under `.planning/generated/quick/260502-nx9-old-new-year-edge-parity-second-sample/`, with lightweight summary artifacts committed under `.planning/quick/260502-nx9-old-new-year-edge-parity-second-sample/`.
- [Quick 260502-p7r]: A third deterministic year-edge old/new parity sample used seed `260502-p7r`. It selected 73 replays, overlapped previous samples by 16 replays, and added 57 unique replays. New parser succeeded on all 73, old parser succeeded on 61 and skipped 12. Stats-only parity is 40 matches / 21 mismatches; mismatch rows are 28 retained `Throw`/`Binoculars` weapon rows and 5 `isDeadByTeamkill` rows in the same old-parser merge-OR class. Full evidence is under `.planning/generated/quick/260502-p7r-old-new-year-edge-parity-third-sample/`, with lightweight summary artifacts committed under `.planning/quick/260502-p7r-old-new-year-edge-parity-third-sample/`.
- [Quick 260502-q4m]: A fourth deterministic year-edge old/new parity sample used seed `260502-q4m`. It selected 73 replays. New parser succeeded on all 73, old parser succeeded on 56 and skipped 17. Stats-only parity is 33 matches / 23 mismatches; mismatch rows are 25 retained `Throw`/`Binoculars` weapon rows, 6 duplicate-slot `isDeadByTeamkill` rows, and 1 known old-parser `teamkillers` merge-bug row. Full evidence is under `.planning/generated/quick/260502-q4m-old-new-year-edge-parity-fourth-sample/`, with lightweight summary artifacts committed under `.planning/quick/260502-q4m-old-new-year-edge-parity-fourth-sample/`.
- [Quick 260502-v8n]: A fifth deterministic year-edge old/new parity sample used seed `260502-v8n`. It selected 73 replays. New parser succeeded on all 73, old parser succeeded on 63 and skipped 10. Stats-only parity is 45 matches / 18 mismatches; mismatch rows are 18 retained `Throw`/`Binoculars` weapon rows and 9 duplicate-slot `isDeadByTeamkill` rows. Full evidence is under `.planning/generated/quick/260502-v8n-old-new-year-edge-parity-fifth-sample/`, with lightweight summary artifacts committed under `.planning/quick/260502-v8n-old-new-year-edge-parity-fifth-sample/`.
- [Quick 260502-rollup]: Five-sample year-edge parity rollup covered 364 selected replay entries and 291 unique replay files. New parser succeeded on all selected entries; old parser produced 305 comparable artifacts and skipped 59. The 93 comparable stats-only mismatches contained only documented known classes: 104 retained `Throw`/`Binoculars` weapon detail rows, 26 duplicate-slot `isDeadByTeamkill` rows, and 2 old-parser `teamkillers` merge-bug rows. Phase 05/05.2 parity follow-up is non-blocking; accepted differences are documented in `.planning/quick/260502-year-edge-parity-five-sample-rollup/KNOWN-DIFFERENCES.md`.
- [Phase 06]: Worker request/result contracts are typed in `parser-contract`, generated into `schemas/parse-job-v1.schema.json` and `schemas/parse-result-v1.schema.json`, and backed by parse job/completed/failed examples.
- [Phase 06]: `replay-parser-2 worker` is implemented as a single-worker RabbitMQ/S3 adapter. It consumes jobs with `job_id`, `replay_id`, `object_key`, `checksum`, and `parser_contract_version`; downloads raw objects; verifies local SHA-256; writes minimal v3 artifacts; and publishes `parse.completed` or `parse.failed`.
- [Phase 06]: Worker artifact keys are deterministic `artifacts/v3/{encoded_replay_id}/{source_sha256}.json` paths and use the raw source checksum segment, not the artifact checksum.
- [Phase 06]: RabbitMQ manual ack happens only after confirmed completed/failed result publication. Publish failures nack for retry, and default prefetch stays `1`.
- [Phase 06]: Parser-core and parser-contract remain transport-free. Worker/CLI contain runtime integration; debug sidecar parsing remains explicit CLI tooling and is not used by worker result artifacts.
- [Phase 06]: WORK-01 through WORK-07 are complete. WORK-08 and WORK-09 remain Phase 7 pending for multi-worker safety, structured operations logs, container probes, and hardening.

### Roadmap Evolution

- Phase 5.1 inserted after Phase 5 (URGENT): Compact Artifact and Selective Parser Redesign. Reason: Phase 5 UAT found that the current parser artifact is too large, benchmark speedup is too weak, and comparison output is not reviewable; Phase 6 worker integration must wait for a compact artifact and selective parser direction.
- Phase 5.1 planned with 8 plans and 7 execution waves on 2026-04-29.
- Phase 5.1 executed all 8 plans on 2026-04-29; its benchmark/parity gap was superseded by Phase 05.2 and later accepted benchmark policy.
- Phase 05.2 inserted after Phase 5: Minimal Artifact and Performance Acceptance (URGENT)
- Phase 05.2 discussed on 2026-05-01; context written to `.planning/phases/05.2-minimal-artifact-and-performance-acceptance/05.2-CONTEXT.md`.

### Pending Todos

None yet.

### Blockers/Concerns

Active: Phase 6 worker integration is complete. Current final-gate all-raw
evidence records cached old all-raw wall `501274.528655ms`, new all-raw wall
`272233.457364ms`, speedup `1.8413x`, all-raw old/new
`attempted_count=23473`, new success/failure/skip counts `23469/4/0`, p95
artifact/raw ratio `0.12432307336264753`, max artifact bytes `48270`, and zero
oversized artifacts. The 4 malformed/non-JSON failures are accepted only while
new failure paths match `.planning/benchmarks/phase-05-all-raw-accepted-failures.json`
and the old cached baseline keeps `error_count=4`, `skipped_count=0`.
Phase 7 is ready to plan/execute WORK-08 and WORK-09.

Resolved: Phase 5.1 replaced the default artifact with compact
`participants`/`facts`/`summaries`, removed full top-level `entities` and
`events` from default output, added a selective parser boundary, and made
comparison reports summary-first with explicit JSON detail output.

Resolved: The 05-03 stable Rust coverage blocker was resolved by the custom
`cargo llvm-cov --json` postprocessor documented in
`.planning/phases/05-cli-golden-parity-benchmarks-and-coverage-gates/05-03-BLOCKER.md`.

- Phase 6 completed on 2026-05-02 and is no longer blocked by Phase 05.2 performance, p95 size, known malformed-file gates, or worker integration gates.

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
| 260429-bench-scope | Clarified that one-replay benchmark evidence is insufficient and whole-list/corpus parsing must be measured | 2026-04-29 | docs-only | Verified |  |
| 260502-ecp | Compacted selected default parser artifact below 100 KB | 2026-05-02 | bbebfcb | Verified | [260502-ecp-compact-default-parser-artifact-below-10](./quick/260502-ecp-compact-default-parser-artifact-below-10/) |
| 260502-gn2 | Ran full Phase 5.2 benchmark and selected old-vs-new stats diff after fixing the legacy all-raw baseline runner | 2026-05-02 | 998c799 | Verified | [260502-gn2-phase-5-2-x3-x10-old-vs-new](./quick/260502-gn2-phase-5-2-x3-x10-old-vs-new/) |
| 260502-i8w | Fixed old all-raw coverage, merged same-name players, split legacy tags, nested player kills under players, cleaned generated phase output, and reran x10 evidence | 2026-05-02 | committed | Verified | [260502-i8w-phase-5-2-old-baseline-all-raw-coverage-](./quick/260502-i8w-phase-5-2-old-baseline-all-raw-coverage-/) |
| 260502-jeh | Optimized default minimal parser hot path and reused cached old all-raw baseline for new benchmark comparisons | 2026-05-02 | 3176abb | Verified | [260502-jeh-full-optimize-parser-points-2-3-and-4-di](./quick/260502-jeh-full-optimize-parser-points-2-3-and-4-di/) |
| 260502-k2u | Compared deterministic year-edge old/new replay statistics; parity failed on comparable stats | 2026-05-02 | committed | Verified - parity failed | [260502-k2u-old-new-year-edge-parity](./quick/260502-k2u-old-new-year-edge-parity/) |
| 260502-nx9 | Ran a second deterministic year-edge old/new replay statistics sample with a different seed | 2026-05-02 | committed | Verified - parity failed | [260502-nx9-old-new-year-edge-parity-second-sample](./quick/260502-nx9-old-new-year-edge-parity-second-sample/) |
| 260502-p7r | Ran a third deterministic year-edge old/new replay statistics sample with a different seed | 2026-05-02 | committed | Verified - parity failed | [260502-p7r-old-new-year-edge-parity-third-sample](./quick/260502-p7r-old-new-year-edge-parity-third-sample/) |
| 260502-q4m | Ran a fourth deterministic year-edge old/new replay statistics sample with a different seed | 2026-05-02 | committed | Verified - known differences only | [260502-q4m-old-new-year-edge-parity-fourth-sample](./quick/260502-q4m-old-new-year-edge-parity-fourth-sample/) |
| 260502-v8n | Ran a fifth deterministic year-edge old/new replay statistics sample with a different seed | 2026-05-02 | committed | Verified - known differences only | [260502-v8n-old-new-year-edge-parity-fifth-sample](./quick/260502-v8n-old-new-year-edge-parity-fifth-sample/) |
| 260502-rollup | Documented five-sample year-edge parity result and accepted old/new difference classes | 2026-05-02 | committed | Verified - Phase 05 parity follow-up non-blocking | [260502-year-edge-parity-five-sample-rollup](./quick/260502-year-edge-parity-five-sample-rollup/) |

## Deferred Items

Items acknowledged and carried forward from previous milestone close:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

## Session Continuity

Last session: 2026-05-02T17:13:39.451Z
Stopped at: Phase 7 context gathered
Resume file: .planning/phases/07-parallel-and-container-hardening/07-CONTEXT.md

**Completed Phase:** 01 (Legacy Baseline and Corpus) — 5 plans — 2026-04-25
**Completed Phase:** 02 (Versioned Output Contract) — 6 plans — 2026-04-26
**Completed Phase:** 03 (Deterministic Parser Core) — 6 plans — 2026-04-27
**Completed Phase:** 04 (Event Semantics and Aggregates) — 7 plans — 2026-04-28
**Completed Phase:** 06 (RabbitMQ/S3 Worker Integration) — 6 plans — 2026-05-02
**Next Step:** Start Phase 07 parallel and container hardening.
