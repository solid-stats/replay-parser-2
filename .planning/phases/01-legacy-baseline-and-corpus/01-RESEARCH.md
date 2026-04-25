# Phase 1: Legacy Baseline and Corpus - Research

**Researched:** 2026-04-25 [VERIFIED: `date +%F`]
**Domain:** Legacy TypeScript parser baseline, full-history OCAP corpus profiling, parity handoff documentation [VERIFIED: `.planning/ROADMAP.md` + `.planning/phases/01-legacy-baseline-and-corpus/01-CONTEXT.md`]
**Confidence:** MEDIUM-HIGH [VERIFIED: local old-parser/source/corpus inspection; MEDIUM because canonical source command currently fails locally]

<user_constraints>
## User Constraints (from CONTEXT.md)

Source: `.planning/phases/01-legacy-baseline-and-corpus/01-CONTEXT.md` [VERIFIED: local file read]

### Locked Decisions

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

### Claude's Discretion

No discretionary implementation choices were delegated. Planner may choose exact filenames and local generated-artifact paths if they follow the decisions above and keep the git tree reviewable.

### Deferred Ideas (OUT OF SCOPE)

- Annual/yearly nomination statistics and nomination pages are deferred to v2. The legacy `src/!yearStatistics` pipeline and `~/sg_stats/year_results` should be preserved as references but not implemented as v1 product support.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOC-01 | Keep README current with project purpose, scope, phase, architecture, validation data, commands, and workflow. | README currently says `~/sg_stats/raw_replays` has around 3,938 files, while local corpus now has 23,473 files, so Phase 1 must update README validation-data text. [VERIFIED: `README.md` + `find ~/sg_stats/raw_replays ... | wc -l`] |
| DOC-02 | README states AI + GSD-only development. | README already has a Development Workflow section stating project development uses AI agents plus GSD only. [VERIFIED: `README.md`] |
| WF-01 | Completed work leaves clean git tree with intended commits. | `commit_docs` is true and current repo `git status --short` was clean before this research file was written. [VERIFIED: `.planning/config.json` + `git status --short`] |
| WF-02 | Do not delete/revert completed work just to clean git. | AGENTS.md requires asking before unclear cleanup and forbids deleting completed work merely for status cleanliness. [VERIFIED: `AGENTS.md`] |
| WF-03 | Challenge conflicting/risky instructions. | AGENTS.md requires pushback on risky, conflicting, or disproportionate instructions. [VERIFIED: `AGENTS.md`] |
| WF-04 | Explain risk and safer alternatives when challenging. | AGENTS.md requires concrete risk explanation and safer alternatives. [VERIFIED: `AGENTS.md`] |
| WF-05 | Explicit confirmation for risky override and record decision. | AGENTS.md and REQUIREMENTS.md require explicit confirmation and recording for risky overrides. [VERIFIED: `AGENTS.md` + `.planning/REQUIREMENTS.md`] |
| INT-01 | Treat Solid Stats as parser, backend, and web apps. | `sg-replay-parser-2`, `server-2`, and `web` responsibilities are documented in AGENTS.md, README, and gsd-briefs. [VERIFIED: `AGENTS.md` + `README.md` + `gsd-briefs/*.md`] |
| INT-02 | Check cross-app compatibility before tasks. | Parser contract, identity, RabbitMQ/S3, API/data, moderation, auth, and UI-visible changes require risk-based adjacent app checks. [VERIFIED: `AGENTS.md` + `gsd-briefs/server-2.md` + `gsd-briefs/web.md`] |
| INT-03 | Apply product-wide GSD workflow rules across apps. | All three gsd-briefs repeat AI+GSD workflow, clean git, pushback, and compatibility standards. [VERIFIED: `gsd-briefs/replays-parser-2.md` + `gsd-briefs/server-2.md` + `gsd-briefs/web.md`] |
| INT-04 | Use risk-based compatibility depth. | Local-only docs can rely on local docs/briefs; contract/queue/storage/API/identity/moderation/UI-visible changes require adjacent app evidence or a user question. [VERIFIED: `AGENTS.md`] |
| LEG-01 | Developer can run and document old parser baseline. | Canonical command is known, but `pnpm run parse -- --help` currently fails under local Node 18.14.0 and 22.16.0 because `tsx src/start.ts` rejects Lodash named imports; Phase 1 planning must include an unblock step before full source-command baseline. [VERIFIED: old `package.json` + old `.nvmrc` + command preflight] |
| LEG-02 | Identify exact command, commit, runtime versions, env inputs, worker count, logs, output locations. | Old parser commit, Node/pnpm versions, lockfile hash, `WORKER_COUNT` behavior, `~/sg_stats` paths, and log/output locations were inspected locally. [VERIFIED: `git -C ... rev-parse HEAD` + old source files + local tool commands] |
| LEG-03 | Corpus manifest for raw replays, results, and replay list. | Local raw corpus has 23,473 JSON files; `replaysList.json` has `parsedReplays`/`replays` count 23,456 and `replaysListPreparedAt` `2026-04-25T04:42:54.889Z`; existing results tree has 88,485 files and a 51,717,050-byte `stats.zip`. [VERIFIED: local `find`, `node`, and `wc` commands; VERIFIED: user-provided latest replaysList count/prepared timestamp] |
| LEG-04 | Document game-type filters, skip rules, exclusions, and config inputs. | `getReplays.ts`, `index.ts`, and `parseReplayWorker.ts` define prefix filters, `sgs` exclusion, `sm` cutoff, `mace` minimum-player skip, and empty replay skip; old config files define replay/player include/exclude inputs. [VERIFIED: old parser source + config files] |
| LEG-05 | Define old-vs-new mismatch taxonomy. | CONTEXT locks taxonomy categories and cross-app impact dimensions; prior pitfalls research recommends compatible match, intentional change, old bug preserved/fixed, new bug, insufficient data, and human review. [VERIFIED: `01-CONTEXT.md` + `.planning/research/PITFALLS.md`] |
</phase_requirements>

## Summary

Phase 1 should be planned as a documentation and local evidence phase around the old parser, not as Rust parser implementation. [VERIFIED: `.planning/ROADMAP.md` + `01-CONTEXT.md`] The key implementation risk is that the locked canonical source command `pnpm run parse` is currently not runnable in the local environment under either default Node 22.16.0 or old-parser `.nvmrc` Node 18.14.0, because `tsx src/start.ts` fails on Lodash named ESM imports before help can render. [VERIFIED: command preflights in `/home/afgan0r/Projects/SolidGames/replays-parser`] The planner must include a Wave 0 unblock for canonical source-command execution or require explicit user approval before treating `parse:dist` as the primary baseline. [VERIFIED: `01-CONTEXT.md` D-01 + D-12]

The corpus is full-history scale now. [VERIFIED: local `find ~/sg_stats/raw_replays -maxdepth 1 -type f -name '*.json' | wc -l`] Current docs and prior research that mention around 3,938 files are stale for planning. [VERIFIED: `README.md` + `.planning/research/SUMMARY.md` + local file count] The latest `replaysList.json` facts to preserve in Phase 1 are `parsedReplays`/`replays` count 23,456 and `replaysListPreparedAt` `2026-04-25T04:42:54.889Z`. [VERIFIED: user-provided latest corpus update] Phase 1 should profile all 23,473 raw files, but should commit compact summaries and small indexes only; raw files, full hashes, regenerated results, logs, and heavy reports should live under a generated/local ignored path. [VERIFIED: `01-CONTEXT.md` D-06/D-09 + local `du -sh ~/sg_stats/*`]

The baseline must be non-destructive by running the old parser against an isolated fake home directory, because the old parser hard-codes `path.join(os.homedir(), 'sg_stats')` and `generateOutput` removes `resultsPath` before moving `temp_results` into place. [VERIFIED: old `paths.ts` + old `src/4 - output/index.ts` + local `HOME=/tmp/... node -e os.homedir()`] This avoids overwriting real `~/sg_stats/results` or `~/sg_stats/year_results` while still exercising the old runtime path contract. [VERIFIED: `01-CONTEXT.md` D-05 + old `paths.ts`]

**Primary recommendation:** Plan Phase 1 in four committed dossiers plus generated local artifacts: baseline command/runtime, corpus manifest/profile, legacy rules/output surfaces, and mismatch taxonomy/interface notes. [VERIFIED: `01-CONTEXT.md` D-16]

## Project Constraints (from AGENTS.md)

- This repo is the Rust replacement parser only; `server-2` owns PostgreSQL, canonical identity, APIs, auth, moderation, parse jobs, aggregate/bounty calculation, and operational visibility, while `web` owns UI/API consumption. [VERIFIED: `AGENTS.md`]
- The old parser at `/home/afgan0r/Projects/SolidGames/replays-parser` is the required v1 behavioral reference. [VERIFIED: `AGENTS.md`]
- Historical data at `~/sg_stats` is the golden/test and benchmark baseline. [VERIFIED: `AGENTS.md`]
- The new parser must preserve observed replay identity fields only; canonical player matching belongs to `server-2`. [VERIFIED: `AGENTS.md`]
- PostgreSQL persistence, public UI, Steam OAuth, correction workflow, and final bounty/reward rules are outside this parser. [VERIFIED: `AGENTS.md`]
- Keep Node/pnpm only as a development dependency for running the legacy parser baseline. [VERIFIED: `AGENTS.md`]
- Keep CLI and worker modes using the same parser core in later phases. [VERIFIED: `AGENTS.md`]
- Prove parity and determinism before optimizing for speed. [VERIFIED: `AGENTS.md`]
- Keep README current when scope, current phase, commands, architecture direction, validation data, or development workflow changes. [VERIFIED: `AGENTS.md`]
- Every completed work session must leave `git status --short` clean by committing intended results. [VERIFIED: `AGENTS.md`]
- Do not delete, revert, or discard completed work just to make the git tree clean; ask when ownership or commit intent is unclear. [VERIFIED: `AGENTS.md`]
- Cross-application compatibility depth is risk-based; parser contract, queue/storage message, artifact shape, API/data model, canonical identity, auth, moderation, or UI-visible changes require adjacent app docs/repos or a user question. [VERIFIED: `AGENTS.md`]
- No `./CLAUDE.md` exists in this repo, so there are no current-repo CLAUDE.md directives to include. [VERIFIED: `test -f CLAUDE.md`]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Legacy command/runtime capture | Legacy parser repo + local shell harness | Phase docs | The command, commit, lockfile, env, worker count, logs, and output paths come from `/home/afgan0r/Projects/SolidGames/replays-parser` and must be recorded in committed Phase 1 docs. [VERIFIED: old `package.json` + `01-CONTEXT.md` D-02] |
| Non-destructive baseline execution | Local filesystem harness | Legacy parser runtime | The old parser writes under `os.homedir()/sg_stats`, empties `temp_results`, and replaces `results`, so isolation must happen at process environment/filesystem level. [VERIFIED: old `paths.ts` + old `src/index.ts` + old `src/4 - output/index.ts`] |
| Corpus manifest/profile | Local corpus tooling | Phase docs | Counts, hashes, file sizes, malformed files, top-level keys, event/entity shapes, and game-type distribution come from `~/sg_stats`, with compact summaries committed. [VERIFIED: `01-CONTEXT.md` D-06/D-09 + local corpus commands] |
| Legacy filters/skip rules/config inventory | Legacy parser source | Parity harness notes | Game-type selection and skip rules are harness compatibility concerns, not parser-core contract behavior. [VERIFIED: `01-CONTEXT.md` D-10 + old `getReplays.ts`/`parseReplayWorker.ts`] |
| Legacy output surface inventory | Legacy output modules + existing results tree | Phase docs | Old output files/folders are produced by `src/4 - output/*` and visible in `~/sg_stats/results`. [VERIFIED: old output modules + local `find ~/sg_stats/results`] |
| Mismatch taxonomy and interface notes | Phase docs | `server-2`/`web` briefs | Diff categories must identify parser-only, backend persistence/recalculation, and UI-visible public-stat impact. [VERIFIED: `01-CONTEXT.md` D-14/D-15 + `gsd-briefs/server-2.md` + `gsd-briefs/web.md`] |
| README/workflow updates | Repo documentation | GSD state | Phase 1 owns DOC/WF/INT requirements and must correct stale validation data without implementing parser code. [VERIFIED: `.planning/ROADMAP.md` Phase 1 + README stale corpus count] |

## Standard Stack

### Core

| Tool / Library | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| Legacy parser repo | Commit `3392ca2f367a87f6eb59041a239e7ca2519e1ec5` | Behavioral reference and baseline runner | v1 behavior is required to be grounded in this old parser. [VERIFIED: `git -C /home/afgan0r/Projects/SolidGames/replays-parser rev-parse HEAD` + `AGENTS.md`] |
| `pnpm run parse` | Script maps to `tsx src/start.ts` | Canonical old-parser source baseline command | User locked this exact command as canonical in D-01. [VERIFIED: old `package.json` + `01-CONTEXT.md`] |
| Node.js | `.nvmrc` pins `v18.14.0`; default shell currently reports `v22.16.0` | Legacy parser runtime | Old parser adjacent docs identify Node 18.14.0 as platform requirement, and local default Node 22 must not be assumed compatible. [VERIFIED: old `.nvmrc` + old `CLAUDE.md` + `node --version`] |
| pnpm | Package manager field `pnpm@10.33.0`; local `pnpm --version` is `10.33.0`; registry latest is `10.33.2` modified 2026-04-23 | Legacy dependency/script runner | Baseline reproducibility should use the package-manager version declared by the old parser rather than opportunistically upgrading. [VERIFIED: old `package.json` + local `pnpm --version` + npm registry] |
| `tsx` | Installed/resolved `4.21.0`; registry latest `4.21.0` modified 2025-11-30 | Source TypeScript runner used by `pnpm run parse` | It is the command runner in the canonical source script, but current source execution fails before baseline run. [VERIFIED: old `package.json` + `pnpm exec tsx --version` + npm registry + command preflight] |
| `sha256sum` | `/usr/bin/sha256sum` | Lockfile/config/output hash capture | Baseline manifest requires lockfile and output hashes. [VERIFIED: `command -v sha256sum` + `01-CONTEXT.md` D-02] |
| `rsync` | `/usr/bin/rsync` | Non-destructive isolated baseline setup | It can copy small config/list/result references while avoiding mutation of real historical outputs. [VERIFIED: `command -v rsync` + old parser path contract] |
| `jq` | `/usr/bin/jq` | Inspect JSON manifests/results during verification | It is available for reproducible local JSON checks. [VERIFIED: `command -v jq`] |

### Supporting

| Tool / Surface | Version / State | Purpose | When to Use |
|----------------|-----------------|---------|-------------|
| `parse:dist` | `node dist/start.js`; help works under local Node 22.16.0 | Secondary diagnostic context only | Use to inspect current compiled behavior or as a fallback only with explicit user approval, because D-01 locks source command as canonical. [VERIFIED: old `package.json` + `pnpm run parse:dist -- --help` + `01-CONTEXT.md`] |
| `nvm` | `~/.nvm/nvm.sh` present; `v18.14.0` installed | Reproduce old parser `.nvmrc` runtime | Use before source-command preflights and baseline attempts. [VERIFIED: local `nvm ls 18.14.0` + old `.nvmrc`] |
| `corepack` | `0.32.0` | Reproduce package-manager version when needed | Use only if local pnpm drifts from old `packageManager`. [VERIFIED: `corepack --version` + old `package.json`] |
| Local generated directory | Not currently present | Store bulky baseline outputs, full hashes, profiles, and logs outside committed docs | Use for all regenerated results and reports that would make git noisy. [VERIFIED: `01-CONTEXT.md` D-09] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `pnpm run parse` canonical source baseline | `pnpm run parse:dist` | `parse:dist` currently starts far enough to show help, but using it as the primary baseline contradicts D-01 unless the user explicitly approves. [VERIFIED: command preflights + `01-CONTEXT.md`] |
| Fake `HOME` isolation | Directly run against real `~/sg_stats` after backing up results | Direct run is simpler but risks replacing `~/sg_stats/results`; fake `HOME` is safer because old paths derive from `os.homedir()`. [VERIFIED: old `paths.ts` + old `src/4 - output/index.ts` + local `HOME=/tmp/... node -e os.homedir()`] |
| Commit full profile/hash outputs | Commit compact summaries plus generated local artifacts | Full raw corpus is 24G and existing results are 503M, so full artifacts are unsuitable for reviewable git commits. [VERIFIED: local `du -sh ~/sg_stats/raw_replays ~/sg_stats/results` + `01-CONTEXT.md` D-09] |

**Installation:**

```bash
# No new repo dependency is required for Phase 1 research/planning deliverables.
# Use the old parser's existing dependency install and declared package manager.
cd /home/afgan0r/Projects/SolidGames/replays-parser
corepack prepare pnpm@10.33.0 --activate
pnpm install --frozen-lockfile
```

The install command should be planned only if `node_modules` is missing or stale; `node_modules` is currently present. [VERIFIED: local `test -d old/node_modules`]

**Version verification:** npm registry checks were run for `pnpm`, `tsx`, `lodash`, `typescript`, and `fs-extra`; these checks verify current registry versions but do not justify changing the legacy baseline dependency graph. [VERIFIED: `npm view ... version time.modified`]

## Architecture Patterns

### System Architecture Diagram

```text
Historical data under ~/sg_stats
  raw_replays + lists/replaysList.json + config/nameChanges.csv + existing results
        |
        v
Read-only corpus profiler and manifest commands
        |
        +--> compact committed corpus-manifest.md / fixture-index.json
        |
        +--> heavy generated local reports under .planning/generated/phase-01/

Legacy parser repo /home/afgan0r/Projects/SolidGames/replays-parser
  package.json + source modules + config + commit + lockfile
        |
        v
Source command preflight: pnpm run parse
        |
        +--> if source command works: run isolated fake-HOME baseline
        |
        +--> if source command fails: document blocker, repair/confirm fallback before full baseline
        |
        v
Isolated fake HOME with symlinked raw/list inputs and separate results/temp/logs
        |
        v
Baseline manifest: command, versions, env, worker count, logs, output hashes
        |
        v
Mismatch taxonomy and interface notes
        |
        +--> Phase 2 contract planning
        +--> Phase 5 comparison/benchmark harness planning
        +--> server-2/web impact notes only, no adjacent app changes
```

The primary data flow enters from the historical corpus and old parser repo, branches on whether the canonical source command is executable, and exits as compact committed docs plus local generated artifacts. [VERIFIED: `01-CONTEXT.md` + local old-parser/corpus inspection]

### Recommended Project Structure

```text
.planning/phases/01-legacy-baseline-and-corpus/
├── 01-RESEARCH.md                         # this file
├── baseline-command-runtime.md            # committed compact command/runtime/baseline manifest
├── corpus-manifest.md                     # committed compact corpus profile summary
├── legacy-rules-output-surfaces.md        # committed source/config/output inventory
├── mismatch-taxonomy-interface-notes.md   # committed diff taxonomy + server/web impact notes
└── fixture-index.json                     # optional small committed fixture selector/index

.planning/generated/phase-01/
├── baseline-runs/<run-id>/                # local ignored full logs/results/hashes
├── corpus-profiles/<profile-id>/          # local ignored full profiler output
└── backups/<timestamp>/                   # local ignored safety backups, if direct runs are ever approved
```

The exact committed filenames are planner discretion, but the four focused document responsibilities are locked by D-16. [VERIFIED: `01-CONTEXT.md` D-16]

### Pattern 1: Fake-HOME Baseline Isolation

**What:** Run the old parser with `HOME` pointing at a generated run directory so `os.homedir()/sg_stats` resolves away from real `~/sg_stats`. [VERIFIED: old `paths.ts` + local `HOME=/tmp/... node -e os.homedir()`]

**When to use:** Every full baseline run unless the user explicitly approves direct mutation after backups. [VERIFIED: `01-CONTEXT.md` D-05]

**Example:**

```bash
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-wc1"
RUN_ROOT="$PWD/.planning/generated/phase-01/baseline-runs/$RUN_ID"
RUN_HOME="$RUN_ROOT/home"
mkdir -p "$RUN_HOME/sg_stats"
ln -s "$HOME/sg_stats/raw_replays" "$RUN_HOME/sg_stats/raw_replays"
ln -s "$HOME/sg_stats/lists" "$RUN_HOME/sg_stats/lists"
cp -a "$HOME/sg_stats/config" "$RUN_HOME/sg_stats/config"
HOME="$RUN_HOME" WORKER_COUNT=1 pnpm run parse
```

This pattern avoids replacing real `~/sg_stats/results` because the old parser writes to `$RUN_HOME/sg_stats/results` instead. [VERIFIED: old `paths.ts` + old `generateOutput` implementation]

### Pattern 2: Compact Dossier, Heavy Generated Artifacts

**What:** Commit summaries, source inventories, fixture indexes, and reproducible commands; leave full hashes, regenerated results, logs, and profile dumps under `.planning/generated/phase-01/`. [VERIFIED: `01-CONTEXT.md` D-09]

**When to use:** All Phase 1 deliverables, because raw corpus is 24G and existing results are 503M. [VERIFIED: local `du -sh ~/sg_stats/raw_replays ~/sg_stats/results`]

**Example:**

```bash
find "$HOME/sg_stats/results" -type f -print0 \
  | sort -z \
  | xargs -0 sha256sum \
  > .planning/generated/phase-01/current-results.sha256
```

The committed dossier should record the path and aggregate hash/count summary, not the entire per-file hash list. [VERIFIED: `01-CONTEXT.md` D-09]

### Pattern 3: Legacy Policy Inventory Before Fixture Selection

**What:** First profile the full corpus and old rules, then select fixtures from actual distribution/anomalies. [VERIFIED: `01-CONTEXT.md` D-07]

**When to use:** Before committing any fixture list or using samples for Phase 2/5 planning. [VERIFIED: `01-CONTEXT.md` D-07 + `.planning/research/PITFALLS.md`]

**Example fixture index fields:**

```json
{
  "source_file": "2021_10_31__00_13_51_ocap.json",
  "reason": ["largest-file", "mission_info", "mission_message", "killed-events"],
  "game_type_from_replays_list": "sg",
  "old_parser_skip_expected": false,
  "cross_app_relevance": ["parser-artifact", "server-recalculation", "ui-visible-public-stats"]
}
```

These fields match locked corpus/profile and mismatch-impact decisions. [VERIFIED: `01-CONTEXT.md` D-06/D-14]

### Anti-Patterns to Avoid

- **Running `pnpm run parse` against real `~/sg_stats`:** old output generation removes `resultsPath` and moves `temp_results`, so direct execution can destroy current historical outputs. [VERIFIED: old `src/4 - output/index.ts`]
- **Treating `parse:dist` success as source baseline success:** compiled help works, but source command currently fails; D-01 locks the source command as canonical. [VERIFIED: command preflights + `01-CONTEXT.md` D-01]
- **Selecting fixtures from stale 3,938-file research:** current raw corpus count is 23,473 files, and old count references are stale. [VERIFIED: local `find` + `README.md` + `.planning/research/SUMMARY.md`]
- **Folding yearly nominations into v1 ordinary stats:** yearly nomination outputs are v2-deferred references only. [VERIFIED: `01-CONTEXT.md` Deferred Ideas + `.planning/REQUIREMENTS.md` FUT-06]
- **Letting parser core own legacy game-type filters:** D-10 assigns these rules to the parity harness, not the parser core contract. [VERIFIED: `01-CONTEXT.md` D-10]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Old behavior oracle | A new ad hoc interpretation of OCAP stats | The old parser repo at commit `3392ca2f...` plus source inventories | v1 parity is defined by old parser behavior, not by fresh guesses from raw JSON. [VERIFIED: `AGENTS.md` + old git commit] |
| Non-destructive isolation | Manual backup-and-hope process around real `~/sg_stats` | Fake `HOME` run directory with symlinked read-only inputs | Old parser writes under `os.homedir()/sg_stats`, so environment-level isolation is the cleanest control. [VERIFIED: old `paths.ts` + local `HOME` check] |
| Hashing | Custom JS hash formats without stable commands | `sha256sum` over sorted file lists | Baseline manifests require reproducible output hashes. [VERIFIED: `01-CONTEXT.md` D-02 + `command -v sha256sum`] |
| JSON inspection | Regex/string matching over replay JSON | `jq` or Node JSON parsing with try/catch and size-aware output summaries | Corpus profile must distinguish malformed JSON, top-level keys, event/entity shapes, and distribution. [VERIFIED: `01-CONTEXT.md` D-06 + local corpus inspection] |
| Cross-app impact classification | A parser-only pass/fail diff | Locked taxonomy with parser-only, `server-2`, and UI-visible impact dimensions | Diffs can affect backend recalculation and public stats even when parser artifacts look local. [VERIFIED: `01-CONTEXT.md` D-14 + `gsd-briefs/server-2.md` + `gsd-briefs/web.md`] |

**Key insight:** Phase 1 is evidence capture and compatibility framing; hand-rolled replacement semantics belong to later phases only after the old source, corpus, outputs, and mismatch categories are documented. [VERIFIED: `.planning/ROADMAP.md` + `01-CONTEXT.md`]

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | `~/sg_stats/raw_replays` has 23,473 JSON files and 24G; `~/sg_stats/results` has 88,485 files and 503M; `~/sg_stats/lists/replaysList.json` is 5,905,859 bytes; `~/sg_stats/config/nameChanges.csv` is 9,924 bytes. [VERIFIED: local `find`, `du`, `wc`] | Do not mutate real stored data; run baseline in fake `HOME`; record current hashes/counts before regenerated outputs. [VERIFIED: `01-CONTEXT.md` D-05/D-08] |
| Live service config | No external live-service configuration was needed for Phase 1 baseline/corpus work; old parser `.env` exists and `.env.sample` documents relay URL/token variables. [VERIFIED: local `find old/.env*` + `.env.sample`] | Do not commit `.env` values; document only variable names and presence. [VERIFIED: old `.env.sample` + AGENTS clean-review rules] |
| OS-registered state | `pm2` is not installed on PATH; user crontab has only `notify-send "poe anti afk"`; user systemd services matching `sg|replay|parser|solid` were not found. [VERIFIED: `command -v pm2`; `crontab -l`; `systemctl --user ... | rg`] | No OS service changes are required for Phase 1; do not register scheduler/pm2 processes. [VERIFIED: Phase 1 scope in `.planning/ROADMAP.md`] |
| Secrets/env vars | Old parser `.env` exists (269 bytes) and `~/sg_stats/relay_logs/issued-tokens.log` exists (743 bytes); values were not read into research. [VERIFIED: local `wc -c` + `find`] | Treat these as local secrets/logs; exclude from committed artifacts and sanitize command logs. [CITED: OWASP ASVS project guidance on secure development requirements at https://owasp.org/www-project-application-security-verification-standard/] |
| Build artifacts | Old parser `dist/` exists and `parse:dist -- --help` works; source command `pnpm run parse -- --help` fails; `node_modules` exists. [VERIFIED: local `find old/dist`, command preflights, `test -d old/node_modules`] | Do not use `dist` as canonical unless approved; plan a source-command unblock or record source-command blocker before full baseline. [VERIFIED: `01-CONTEXT.md` D-01/D-12] |

**Nothing found in category:** No current-repo test/build artifacts exist because this repository does not yet contain a runnable Rust workspace, CLI, worker, or test suite. [VERIFIED: `README.md` + `rg --files` test/config scan]

## Legacy Baseline Facts To Plan Around

| Fact | Planning Impact |
|------|-----------------|
| Old parser repo is clean and at commit `3392ca2f367a87f6eb59041a239e7ca2519e1ec5`. [VERIFIED: old `git status --short` + `git rev-parse HEAD`] | Record this commit in the baseline manifest before any old-parser toolchain repair. [VERIFIED: `01-CONTEXT.md` D-02] |
| Old `package.json` script `parse` runs `tsx src/start.ts`. [VERIFIED: old `package.json`] | This is the canonical source command; planning must not silently substitute compiled output. [VERIFIED: `01-CONTEXT.md` D-01] |
| Old `package.json` script `parse:dist` runs `node dist/start.js`. [VERIFIED: old `package.json`] | It can be documented as secondary context only. [VERIFIED: `01-CONTEXT.md` specifics] |
| Old `.nvmrc` pins `v18.14.0`. [VERIFIED: old `.nvmrc`] | Baseline manifest must include the Node version actually used. [VERIFIED: `01-CONTEXT.md` D-02] |
| Old runtime config clamps `WORKER_COUNT` to integer range 1..64 and defaults to CPU count minus one. [VERIFIED: old `runtimeConfig.ts`] | Phase 1 must capture `WORKER_COUNT=1` plus default/legacy profile. [VERIFIED: `01-CONTEXT.md` D-03] |
| Old paths are rooted at `os.homedir()/sg_stats` and include `raw_replays`, `lists`, `results`, `temp_results`, `year_results`, `logs`, and `config/nameChanges.csv`. [VERIFIED: old `paths.ts`] | Fake `HOME` can isolate baseline writes while preserving runtime path semantics. [VERIFIED: local `HOME` check] |
| Old `generateOutput` removes `resultsPath` and moves `tempResultsPath` to `resultsPath`. [VERIFIED: old `src/4 - output/index.ts`] | Direct full baseline runs can overwrite current historical results and must be isolated or explicitly backed up. [VERIFIED: `01-CONTEXT.md` D-05] |
| Source-command help currently fails with `SyntaxError: The requested module 'lodash' does not provide an export named 'isEmpty'` under Node 18.14.0 and 22.16.0. [VERIFIED: local command preflights] | Plan Wave 0 must unblock the canonical source command before LEG-01/LEG-02 can close, or request approval to use `parse:dist` as a temporary baseline. [VERIFIED: `01-CONTEXT.md` D-01/D-12/D-17] |

## Corpus Facts To Plan Around

| Fact | Planning Impact |
|------|-----------------|
| `~/sg_stats/raw_replays` currently has 23,473 `*.json` files. [VERIFIED: local `find ... | wc -l`] | All profiler and fixture-selection tasks must assume full-history scale, not 3,938-file scale. [VERIFIED: user update + local count] |
| `~/sg_stats/raw_replays` is 24G. [VERIFIED: `du -sh`] | Full JSON profile outputs should be generated local/ignored, and profiler implementation should be resumable or summary-oriented. [VERIFIED: `01-CONTEXT.md` D-09] |
| `~/sg_stats/lists/replaysList.json` has `parsedReplays`/`replays` count 23,456, top-level keys `replaysListPreparedAt`, `parsedReplays`, `replays`, and `problematicReplays`, and prepared timestamp `2026-04-25T04:42:54.889Z`. [VERIFIED: local Node JSON inspection for keys/row count; VERIFIED: user-provided latest parsedReplays/prepared timestamp] | Manifest must explicitly reconcile 23,473 raw files versus 23,456 listed replay rows and record the prepared timestamp. [VERIFIED: local raw/list comparison; VERIFIED: user-provided latest prepared timestamp] |
| There are 17 raw replay basenames not present in `replaysList.json`, including `.json` with invalid HTML content. [VERIFIED: local Node raw/list comparison + sample parse] | Manifest must include unlisted/malformed files and should not assume replay-list completeness. [VERIFIED: `01-CONTEXT.md` D-06] |
| Largest raw replay file found by metadata is `2021_10_31__00_13_51_ocap.json` at 19,706,937 bytes. [VERIFIED: local `find -printf '%s %f' | sort -nr | head`] | Fixture selection should include largest files and high-event/entity cases. [VERIFIED: `01-CONTEXT.md` D-06/D-07] |
| Sample valid replays have top-level keys `EditorMarkers`, `Markers`, `captureDelay`, `endFrame`, `entities`, `events`, `missionAuthor`, `missionName`, `playersCount`, and `worldName`. [VERIFIED: local Node sample inspection] | Corpus profile should verify key-set distribution across the full corpus before Phase 2 contract decisions. [VERIFIED: `01-CONTEXT.md` D-06] |
| `replaysList.json` mission-prefix counts are `sg: 2052`, `mace: 20702`, `sm: 243`, `sgs: 1`, and `other: 458`. [VERIFIED: local Node replay-list inspection] | Game-type distribution and filter rules must be documented separately from raw parser contract behavior. [VERIFIED: `01-CONTEXT.md` D-10] |
| Existing `~/sg_stats/results` is 503M with 88,485 files; `stats.zip` is 51,717,050 bytes. [VERIFIED: local `du`, `find`, and `find ... stats.zip -printf`] | Current results are too large to commit and should be represented by compact hashes/counts in the dossier. [VERIFIED: `01-CONTEXT.md` D-09] |
| Existing ordinary result surfaces include `sg/all_time`, `sg/rotation_1` through `sg/rotation_20`, `mace`, and `sm` folders. [VERIFIED: local `find ~/sg_stats/results -maxdepth 2 -type d`] | Output-surface inventory should cover all-time, rotation, weapons, weeks, other-player, global, squad, and archive outputs. [VERIFIED: old output modules + local results tree] |
| `~/sg_stats/year_results` is 204K with 14 nomination text files. [VERIFIED: local `du` + `find ... | wc -l`] | Yearly outputs should be listed as v2-deferred references only. [VERIFIED: `01-CONTEXT.md` Deferred Ideas] |

## Legacy Rules And Output Surfaces To Inventory

| Area | Files / Evidence | Required Phase 1 Inventory |
|------|------------------|----------------------------|
| Game type selection | `src/1 - replays/getReplays.ts`, `src/index.ts`, `src/0 - consts/gameTypesArray.ts` [VERIFIED: old source read] | Prefix filters for `sg`, `mace`, `sm`; `sgs` exclusion; unique-by-filename behavior; `sm` date cutoff after `2023-01-01` by month. [VERIFIED: old source read] |
| Replay-level skip rules | `src/1 - replays/workers/parseReplayWorker.ts` [VERIFIED: old source read] | `empty_replay` when parsed result length is 0; `mace_min_players` when `gameType === 'mace'` and result length is less than 10. [VERIFIED: old source read] |
| Config inputs | old `config/excludeReplays.json`, `config/includeReplays.json`, `config/excludePlayers.json`, `~/sg_stats/config/nameChanges.csv` [VERIFIED: local file reads/hashes] | Include/exclude replay/player counts, hashes, date-bound player exclusion behavior, and name-change source for compatibility identity. [VERIFIED: old config + old `add.ts` + old names helper paths] |
| Observed entity extraction | `src/2 - parseReplayInfo/getEntities.ts` [VERIFIED: old source read] | Unit/player conditions: `type === 'unit'`, `isPlayer`, non-empty `description`, and non-empty `name`; vehicle entity capture; connected-event player backfill. [VERIFIED: old source read] |
| Kill/death semantics | `src/2 - parseReplayInfo/getKillsAndDeaths.ts` [VERIFIED: old source read] | Null killer death handling; same-side teamkill; suicide special case; weapon-as-vehicle-name `killsFromVehicle`; killed vehicle `vehicleKills`; relationship arrays. [VERIFIED: old source read] |
| Duplicate-slot merge | `src/2 - parseReplayInfo/combineSamePlayersInfo.ts` [VERIFIED: old source read] | Same-name entity merge behavior and field union/sum rules must be mapped as compatibility behavior. [VERIFIED: old source read + `01-CONTEXT.md` D-11] |
| Aggregate formulas | `src/3 - statistics/global/add.ts`, `calculateScore.ts`, `calculateKDRatio.ts`, `calculateVehicleKillsCoef.ts` [VERIFIED: old source read] | Ordinary stats field formulas and compatibility identity lookup must be documented for later aggregate parity. [VERIFIED: old source read] |
| Output files | `src/4 - output/json.ts`, `rotationsJSON.ts`, `archiveFiles.ts`, `consts.ts` [VERIFIED: old source read] | `global_statistics.json`, `squad_statistics.json`, `squad_full_rotation_statistics.json`, `rotations_info.json`, per-player weapons/weeks/other-player folders, rotation folders, and `stats.zip`. [VERIFIED: old source read + local results tree] |
| Annual nominations | `src/!yearStatistics/*`, `~/sg_stats/year_results` [VERIFIED: old source file listing + local data] | Preserve as v2-deferred historical reference only, not v1 ordinary aggregate mapping. [VERIFIED: `01-CONTEXT.md` Deferred Ideas + `.planning/REQUIREMENTS.md` FUT-06] |

## Common Pitfalls

### Pitfall 1: Canonical Source Command Is Documented But Not Currently Executable
**What goes wrong:** Planner schedules full baseline run but `pnpm run parse` fails before any baseline data is produced. [VERIFIED: command preflights]  
**Why it happens:** `tsx src/start.ts` imports old TypeScript source as ESM and fails on Lodash named exports under the tested Node runtimes. [VERIFIED: command error output]  
**How to avoid:** Add a Wave 0 source-command preflight/unblock task and require the source command to pass before full baseline execution; use `parse:dist` only as explicitly labeled secondary context or with user approval. [VERIFIED: `01-CONTEXT.md` D-01/D-12/D-17]  
**Warning signs:** The plan treats `parse:dist -- --help` as proof that `pnpm run parse` is ready, or closes LEG-01 without source-command output. [VERIFIED: command preflights]

### Pitfall 2: Accidentally Replacing Historical Results
**What goes wrong:** A full old-parser run removes current `~/sg_stats/results` and replaces it with regenerated output. [VERIFIED: old `generateOutput` implementation]  
**Why it happens:** Old runtime paths are hard-coded to `os.homedir()/sg_stats`, and output publication removes `resultsPath` before moving `temp_results`. [VERIFIED: old `paths.ts` + old output module]  
**How to avoid:** Run baseline with fake `HOME`, symlink read-only raw/list inputs, copy config, and keep generated results/logs under `.planning/generated/phase-01/`. [VERIFIED: local `HOME` check + `01-CONTEXT.md` D-05/D-09]  
**Warning signs:** Any command line starts with `cd /home/.../replays-parser && WORKER_COUNT=1 pnpm run parse` without fake `HOME` or backup proof. [VERIFIED: old path contract]

### Pitfall 3: Stale Corpus Assumptions
**What goes wrong:** Fixture selection and manifest sizing use 3,938-file assumptions and miss full-history distribution. [VERIFIED: stale README/prior research vs local count]  
**Why it happens:** Prior docs and research were written before the user updated `~/sg_stats` to the latest full-history corpus. [VERIFIED: README/prior research dates + user update + local count]  
**How to avoid:** Start Phase 1 with a fresh read-only count/profile and update README/current docs to 23,473-file full-history language. [VERIFIED: DOC-01 + local count]  
**Warning signs:** Plans say "around 3,938 raw replay files" or select fixtures before profiling current `~/sg_stats/raw_replays`. [VERIFIED: README stale text + `01-CONTEXT.md` D-07]

### Pitfall 4: Treating Replay List As Complete Corpus Truth
**What goes wrong:** Raw files outside `replaysList.json`, malformed raw files, or stale list rows are omitted from the manifest. [VERIFIED: local raw/list comparison]  
**Why it happens:** `replaysList.json` has 23,456 rows while raw files count is 23,473, with 17 raw basenames not listed. [VERIFIED: local Node comparison]  
**How to avoid:** Manifest must include both raw-directory inventory and replay-list inventory, then classify discrepancies. [VERIFIED: `01-CONTEXT.md` D-06]  
**Warning signs:** Corpus manifest has one count and no "raw not listed" / "listed missing raw" section. [VERIFIED: local corpus discrepancy]

### Pitfall 5: Copying Legacy Identity Into Parser Contract
**What goes wrong:** Old `nameChanges.csv`, same-name merge, and prefix handling become normalized parser identity instead of compatibility projection behavior. [VERIFIED: old identity-related source + AGENTS identity constraint]  
**Why it happens:** Old aggregate outputs are already compatibility-identity based, but new parser scope preserves observed identity only. [VERIFIED: `AGENTS.md` + old `getPlayerName`/`getPlayerId` usage]  
**How to avoid:** Phase 1 maps every old identity/name rule and labels compatibility behavior separately for Phase 2/5. [VERIFIED: `01-CONTEXT.md` D-11]  
**Warning signs:** A Phase 2 contract proposal includes canonical player IDs or uses name-change IDs as raw observed identity. [VERIFIED: `AGENTS.md` + `gsd-briefs/server-2.md`]

### Pitfall 6: Mismatch Reports Without Cross-App Impact
**What goes wrong:** A diff is classified as parser-only even though it changes backend recalculation or public stats. [VERIFIED: `01-CONTEXT.md` D-14 + gsd briefs]  
**Why it happens:** Parser artifacts, `server-2` persistence/recalculation, and `web` UI-visible stats are separate responsibility tiers. [VERIFIED: `gsd-briefs/server-2.md` + `gsd-briefs/web.md`]  
**How to avoid:** Every mismatch category must carry impact dimensions: parser artifact only, `server-2` persistence/recalculation, UI-visible public stats, and human-review requirement. [VERIFIED: `01-CONTEXT.md` D-14/D-15]  
**Warning signs:** Taxonomy only has pass/fail or old/new labels with no `server-2`/`web` notes. [VERIFIED: INT requirements]

## Code Examples

Verified patterns from local sources:

### Source Command Preflight

```bash
cd /home/afgan0r/Projects/SolidGames/replays-parser
source "$HOME/.nvm/nvm.sh"
nvm use --silent v18.14.0
pnpm run parse -- --help
```

This currently fails on Lodash named ESM imports and should be a Wave 0 gate before full baseline execution. [VERIFIED: local command preflight]

### Secondary Compiled Help Check

```bash
cd /home/afgan0r/Projects/SolidGames/replays-parser
pnpm run parse:dist -- --help
```

This currently prints old parser help, but it is not the canonical baseline command. [VERIFIED: local command preflight + `01-CONTEXT.md` D-01]

### Read-Only Corpus Count And Size

```bash
find "$HOME/sg_stats/raw_replays" -maxdepth 1 -type f -name '*.json' | wc -l
du -sh "$HOME/sg_stats/raw_replays" "$HOME/sg_stats/results" "$HOME/sg_stats/year_results"
```

These commands verified the 23,473-file, 24G raw corpus and large generated result surfaces. [VERIFIED: local command output]

### Raw/List Discrepancy Check

```bash
node -e "const fs=require('fs');const dir=process.env.HOME+'/sg_stats/raw_replays';const raw=new Set(fs.readdirSync(dir).filter(f=>f.endsWith('.json')).map(f=>f.replace(/\\.json$/,'')));const rs=JSON.parse(fs.readFileSync(process.env.HOME+'/sg_stats/lists/replaysList.json','utf8')).replays||[];const listed=new Set(rs.map(r=>r.filename));console.log(JSON.stringify({rawFiles:raw.size,listedFiles:listed.size,listedRows:rs.length,rawNotListedCount:[...raw].filter(f=>!listed.has(f)).length,listedMissingRawCount:[...listed].filter(f=>!raw.has(f)).length},null,2))"
```

This command verified 17 raw basenames not present in `replaysList.json` and zero listed filenames missing from raw files. [VERIFIED: local command output]

### Non-Destructive Baseline Skeleton

```bash
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-wc1"
RUN_ROOT="$PWD/.planning/generated/phase-01/baseline-runs/$RUN_ID"
RUN_HOME="$RUN_ROOT/home"
mkdir -p "$RUN_HOME/sg_stats"
ln -s "$HOME/sg_stats/raw_replays" "$RUN_HOME/sg_stats/raw_replays"
ln -s "$HOME/sg_stats/lists" "$RUN_HOME/sg_stats/lists"
cp -a "$HOME/sg_stats/config" "$RUN_HOME/sg_stats/config"
HOME="$RUN_HOME" WORKER_COUNT=1 pnpm run parse
```

This skeleton depends on first unblocking the canonical source command. [VERIFIED: command preflight + old path contract]

## State of the Art

| Old Approach | Current Phase 1 Approach | When Changed | Impact |
|--------------|--------------------------|--------------|--------|
| Treat `~/sg_stats/results` as the output location for parser runs | Treat real `~/sg_stats/results` as historical evidence and run regenerated baselines in an isolated fake `HOME` | Locked in Phase 1 context on 2026-04-25 | Prevents accidental destructive replacement while preserving old path semantics. [VERIFIED: `01-CONTEXT.md` + old output module] |
| Rely on stale corpus count around 3,938 files | Use current full-history corpus count of 23,473 raw JSON files | User update and local verification on 2026-04-25 | Changes manifest/profiler scale, artifact policy, and fixture selection. [VERIFIED: user update + local count] |
| Fold annual nomination outputs into broad "results" thinking | List yearly nomination outputs as v2-deferred references only | Requirements updated 2026-04-25 | Keeps Phase 1/Phase 2 v1 scope focused on ordinary stats. [VERIFIED: `.planning/REQUIREMENTS.md` + `01-CONTEXT.md`] |
| Use old parser behavior informally | Pin command/runtime/commit/env/log/output hashes in a baseline manifest | Locked in Phase 1 context on 2026-04-25 | Makes parity and benchmarks reproducible. [VERIFIED: `01-CONTEXT.md` D-02] |

**Deprecated/outdated:**
- "Around 3,938 raw replay files" is outdated for this repo because current `~/sg_stats/raw_replays` has 23,473 files. [VERIFIED: `README.md` + `.planning/research/SUMMARY.md` + local count]
- Treating `parse:dist` as equivalent to `pnpm run parse` is outdated for Phase 1 planning because the source command is explicitly locked as canonical and currently fails locally. [VERIFIED: `01-CONTEXT.md` D-01 + command preflight]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| - | No `[ASSUMED]` claims are used in this research. | All | All listed factual claims are tied to local files, local command output, npm registry output, or cited official docs. [VERIFIED: source review] |

## Open Questions - RESOLVED

1. **RESOLVED: How should Phase 1 unblock the canonical source command?** [VERIFIED: command preflight]
   - What we know: `pnpm run parse -- --help` fails under Node 18.14.0 and Node 22.16.0 with a Lodash named export error. [VERIFIED: command preflight]
   - Resolution: [01-00-PLAN.md](./01-00-PLAN.md) creates the Plan 00 source-command gate, records the canonical `pnpm run parse` preflight, and blocks downstream baseline execution unless the source command is working or an explicit fallback override is recorded. [VERIFIED: `01-CONTEXT.md` D-01/D-12/D-17]

2. **RESOLVED: What exact full-corpus profiler implementation should Phase 1 commit?** [VERIFIED: local 24G corpus size]
   - What we know: corpus is 24G and a naive full JSON parse can be slow enough that research did not rely on it for full event/entity distribution. [VERIFIED: local `du` + research command behavior]
   - Resolution: [01-02-PLAN.md](./01-02-PLAN.md) requires a generated `.planning/generated/phase-01/corpus-profiles/<UTC-profile-id>/corpus-profile.json` plus compact committed `corpus-manifest.md` and `fixture-index.json` summaries, keeping bulky profiler evidence ignored per D-09. [VERIFIED: old parser Node/pnpm dependency + `01-CONTEXT.md` D-06/D-07/D-09]

3. **RESOLVED: Should `.planning/generated/phase-01/` be added to `.gitignore`?** [VERIFIED: no `.gitignore` content found]
   - What we know: Phase 1 requires local heavy artifacts and current repo has no `.gitignore` entries. [VERIFIED: `.gitignore` read + `01-CONTEXT.md` D-09]
   - Resolution: [01-00-PLAN.md](./01-00-PLAN.md) adds `.planning/generated/` to `.gitignore` before generating full hashes, regenerated results, logs, backups, or profiles. [VERIFIED: `01-CONTEXT.md` D-09 + WF clean-tree requirements]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Old parser repo | LEG-01/LEG-02 | yes | commit `3392ca2f367a87f6eb59041a239e7ca2519e1ec5` | none; required reference [VERIFIED: old git commands + AGENTS.md] |
| Node default | Old parser and corpus tooling | yes | `v22.16.0` | Use nvm Node 18.14.0 for old-parser `.nvmrc` reproduction. [VERIFIED: `node --version` + old `.nvmrc`] |
| Node via nvm | Old parser source baseline | yes | `v18.14.0` installed | Source still fails; needs baseline unblock. [VERIFIED: `nvm ls 18.14.0` + command preflight] |
| pnpm | Old parser scripts | yes | local `10.33.0`; old packageManager `pnpm@10.33.0` | Use corepack if local pnpm drifts. [VERIFIED: local `pnpm --version` + old `package.json`] |
| npm | Registry/version checks | yes | `10.9.2` | none needed [VERIFIED: `npm --version`] |
| corepack | pnpm version reproduction | yes | `0.32.0` | use installed pnpm 10.33.0 [VERIFIED: `corepack --version`] |
| nvm | Node version reproduction | yes | `nvm.sh` present; nvm v0.37.2 output observed | direct installed Node only, but not enough for source baseline [VERIFIED: local nvm commands] |
| `sha256sum` | Hash manifests | yes | `/usr/bin/sha256sum` | Node `crypto` if needed [VERIFIED: `command -v sha256sum`] |
| `rsync` | Isolated run setup | yes | `/usr/bin/rsync` | `cp -a` for small trees and symlinks for raw/list directories [VERIFIED: `command -v rsync`] |
| `jq` | JSON inspection | yes | `/usr/bin/jq` | Node JSON scripts [VERIFIED: `command -v jq`] |
| `pm2` | Old production scheduler context | no | - | Not required for Phase 1; do not register scheduler processes. [VERIFIED: `command -v pm2` + Phase 1 scope] |

**Missing dependencies with no fallback:**
- None for research/dossier writing; canonical source baseline execution is blocked by runtime/source compatibility, not by a missing executable. [VERIFIED: environment audit + command preflight]

**Missing dependencies with fallback:**
- `pm2` is missing, but Phase 1 does not need scheduler registration. [VERIFIED: `command -v pm2` + `.planning/ROADMAP.md` Phase 1]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | None in current repo; this repository has no runnable Rust workspace, CLI, worker, or test suite yet. [VERIFIED: README + `rg --files` scan] |
| Config file | none; no `Cargo.toml`, `package.json`, Jest, Vitest, or pytest config found in current repo. [VERIFIED: `rg --files` scan] |
| Quick run command | `git diff --check` plus targeted shell/doc preflights. [VERIFIED: prior quick-task validation pattern + current docs-only scope] |
| Full suite command | No full suite exists; Phase 1 verification should be command-based deliverable checks. [VERIFIED: README + no test config] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| DOC-01 | README reflects current phase and full-history corpus facts | doc check | `rg -n "23,473|full-history|Phase 1" README.md` | yes, README exists [VERIFIED: README] |
| DOC-02 | README states AI + GSD-only workflow | doc check | `rg -n "AI agents.*GSD|GSD workflow" README.md` | yes, README exists [VERIFIED: README] |
| WF-01 | Work session commits intended results | git check | `git status --short` | yes, git repo exists [VERIFIED: local git status] |
| WF-02 | No destructive cleanup to force clean tree | doc review | `rg -n "delete|discard|git tree|clean" AGENTS.md README.md .planning/REQUIREMENTS.md` | yes, docs exist [VERIFIED: local rg] |
| WF-03/WF-04/WF-05 | Risky instruction pushback workflow documented | doc review | `rg -n "challenge|risky|confirmation|safer" AGENTS.md README.md .planning/REQUIREMENTS.md` | yes, docs exist [VERIFIED: local rg] |
| INT-01/INT-03 | Multi-project product and product-wide GSD rules documented | doc review | `rg -n "server-2|web|product-wide" README.md AGENTS.md gsd-briefs/*.md` | yes, docs exist [VERIFIED: local rg] |
| INT-02/INT-04 | Risk-based compatibility checks documented | doc review | `rg -n "risk-based|compatibility|adjacent" README.md AGENTS.md gsd-briefs/*.md` | yes, docs exist [VERIFIED: local rg] |
| LEG-01 | Canonical old parser source command can run | command smoke | `cd /home/afgan0r/Projects/SolidGames/replays-parser && source ~/.nvm/nvm.sh && nvm use --silent v18.14.0 && pnpm run parse -- --help` | no, currently fails Wave 0 [VERIFIED: command preflight] |
| LEG-02 | Baseline manifest records commit/runtime/env/outputs | artifact check | `test -f .planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` | no, Wave 0/1 [VERIFIED: file absent] |
| LEG-03 | Corpus manifest covers raw/results/replay list | artifact check | `test -f .planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` | no, Wave 0/1 [VERIFIED: file absent] |
| LEG-04 | Legacy filters/config inputs documented | artifact check | `test -f .planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md` | no, Wave 0/1 [VERIFIED: file absent] |
| LEG-05 | Mismatch taxonomy documented | artifact check | `test -f .planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md` | no, Wave 0/1 [VERIFIED: file absent] |

### Sampling Rate

- **Per task commit:** run `git diff --check`, `git status --short`, and the deliverable-specific `test -f` or `rg` command. [VERIFIED: docs-only workflow + WF requirements]
- **Per wave merge:** rerun all Phase 1 artifact checks plus source-command preflight after any old-parser/tooling changes. [VERIFIED: LEG-01/LEG-02 requirements]
- **Phase gate:** require source baseline command status to be resolved, compact docs present, generated artifact paths documented, and no real `~/sg_stats/results` mutation unaccounted for. [VERIFIED: `01-CONTEXT.md` D-05/D-17]

### Wave 0 Gaps

- [ ] Source-command unblock for `pnpm run parse`; current preflight fails before full baseline can run. [VERIFIED: command preflight]
- [ ] `.gitignore` entry for `.planning/generated/` or equivalent generated artifact path. [VERIFIED: empty/no-op `.gitignore` read + `01-CONTEXT.md` D-09]
- [ ] `baseline-command-runtime.md` file. [VERIFIED: file absent]
- [ ] `corpus-manifest.md` file. [VERIFIED: file absent]
- [ ] `legacy-rules-output-surfaces.md` file. [VERIFIED: file absent]
- [ ] `mismatch-taxonomy-interface-notes.md` file. [VERIFIED: file absent]
- [ ] README stale corpus count update. [VERIFIED: README stale text + current count]

## Security Domain

### Applicable ASVS Categories

OWASP ASVS 5.0.0 is the latest stable ASVS release listed by OWASP as of the page opened during research. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| Authentication | no | Phase 1 has no user login/session surface. [VERIFIED: `.planning/ROADMAP.md` Phase 1] |
| Session Management | no | Phase 1 has no browser or service session state. [VERIFIED: `.planning/ROADMAP.md` Phase 1] |
| Access Control | no | Phase 1 changes local docs/artifacts only and must not change adjacent app authorization. [VERIFIED: INT scope docs] |
| Input Validation | yes | Corpus/profile scripts must parse untrusted/malformed JSON with try/catch and classify failures instead of crashing or executing content. [VERIFIED: `.planning/REQUIREMENTS.md` PARS-02 future requirement + local `.json` malformed sample] |
| Cryptography / Integrity | yes | Use standard SHA-256 tooling for manifest/output integrity; do not implement custom hash algorithms. [VERIFIED: `sha256sum` availability + `01-CONTEXT.md` D-02] |
| Configuration / Secrets | yes | Exclude `.env`, relay token logs, and generated logs containing secrets from committed artifacts. [VERIFIED: old `.env.sample` + local secret/log file presence] |

### Known Threat Patterns for Phase 1 Tooling

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Accidental overwrite of historical results | Tampering | Fake `HOME` isolation and pre-run current-results hash capture. [VERIFIED: old output module + `01-CONTEXT.md` D-05] |
| Committing `.env` or token logs | Information Disclosure | Commit only variable names and sanitized summaries; keep `.env` and relay logs local. [VERIFIED: old `.env.sample` + local file presence] |
| Shell command injection through replay filenames in generated commands | Tampering | Use null-delimited `find -print0`/`xargs -0` and quote paths in documented commands. [VERIFIED: local command patterns] |
| Malformed raw replay crashing profiler | Denial of Service | Catch JSON parse errors and record malformed-file entries in generated profile. [VERIFIED: local `.json` malformed sample + `01-CONTEXT.md` D-06] |
| Oversized generated artifacts entering git | Availability / Repudiation | Add ignored generated path and commit compact summaries only. [VERIFIED: 24G raw corpus + 503M results + `01-CONTEXT.md` D-09] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/01-legacy-baseline-and-corpus/01-CONTEXT.md` - locked Phase 1 decisions, deferred scope, canonical references. [VERIFIED: local file read]
- `.planning/REQUIREMENTS.md` - Phase 1 requirement IDs and v1/v2 boundaries. [VERIFIED: local file read]
- `.planning/ROADMAP.md` - Phase 1 goal and success criteria. [VERIFIED: local file read]
- `.planning/PROJECT.md`, `.planning/STATE.md`, `README.md`, `AGENTS.md` - project constraints, stale/current context, workflow requirements. [VERIFIED: local file reads]
- `gsd-briefs/replays-parser-2.md`, `gsd-briefs/server-2.md`, `gsd-briefs/web.md` - cross-application responsibilities and compatibility rules. [VERIFIED: local file reads]
- `/home/afgan0r/Projects/SolidGames/replays-parser/package.json`, `.nvmrc`, `CLAUDE.md`, `docs/architecture.md`, `src/start.ts`, `src/index.ts`, `src/0 - utils/runtimeConfig.ts`, `src/0 - utils/paths.ts`, `src/1 - replays/getReplays.ts`, `src/1 - replays/workers/parseReplayWorker.ts`, `src/2 - parseReplayInfo/*`, `src/3 - statistics/*`, `src/4 - output/*`, `config/*.json`. [VERIFIED: local file reads]
- Local corpus commands over `~/sg_stats/raw_replays`, `~/sg_stats/results`, `~/sg_stats/year_results`, `~/sg_stats/lists/replaysList.json`, and `~/sg_stats/config/nameChanges.csv`. [VERIFIED: local command output]
- npm registry checks for `pnpm`, `tsx`, `lodash`, `typescript`, and `fs-extra`. [VERIFIED: npm registry]
- OWASP ASVS project page for ASVS purpose and latest stable version. [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Secondary (MEDIUM confidence)

- `.planning/research/SUMMARY.md`, `ARCHITECTURE.md`, `FEATURES.md`, `PITFALLS.md` - prior project research, used only where consistent with current corpus facts. [VERIFIED: local file reads]

### Tertiary (LOW confidence)

- None. [VERIFIED: no unverified web-search-only claims retained]

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM-HIGH - old parser and local tools are verified, but canonical source command currently fails and needs a plan unblock. [VERIFIED: local file/command evidence]
- Architecture: HIGH - Phase 1 artifact split, fake-HOME isolation, and harness ownership follow locked decisions and old source behavior. [VERIFIED: `01-CONTEXT.md` + old source]
- Corpus facts: HIGH for counts/sizes/list discrepancy verified by local commands; MEDIUM for full event/entity distribution because a dedicated full profiler remains a Phase 1 deliverable. [VERIFIED: local commands]
- Pitfalls: HIGH - destructive output behavior, source-command failure, stale corpus count, identity boundary, and cross-app impact are all directly evidenced. [VERIFIED: old source + command preflights + planning docs]

**Research date:** 2026-04-25 [VERIFIED: `date +%F`]
**Valid until:** 2026-05-02 for tooling/runtime facts and corpus counts; re-run preflights if old parser dependencies or `~/sg_stats` changes. [VERIFIED: fast-moving local corpus/tooling]
