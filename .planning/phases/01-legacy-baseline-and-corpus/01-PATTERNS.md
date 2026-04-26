# Phase 01: Legacy Baseline and Corpus - Pattern Map

**Mapped:** 2026-04-25
**Files analyzed:** 8
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `.gitignore` | config | file-I/O | `.planning/phases/01-legacy-baseline-and-corpus/01-VALIDATION.md` | exact |
| `.planning/generated/phase-01/` | generated artifact root | file-I/O | `.planning/phases/01-legacy-baseline-and-corpus/01-RESEARCH.md` | exact |
| `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` | documentation | batch/file-I/O | `01-RESEARCH.md` + legacy `package.json`/runtime files | exact |
| `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` | documentation | batch/transform | `01-RESEARCH.md` + `01-VALIDATION.md` | exact |
| `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md` | documentation | batch/transform | legacy parser source files | exact |
| `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md` | documentation | request-response/transform | `01-CONTEXT.md` + `gsd-briefs/*.md` | exact |
| `.planning/phases/01-legacy-baseline-and-corpus/fixture-index.json` | data fixture index | transform | `01-RESEARCH.md` fixture-index example | role-match |
| `README.md` | documentation | request-response | README quick task plan/summary + current README | exact |

## Pattern Assignments

### `.gitignore` (config, file-I/O)

**Analog:** `.planning/phases/01-legacy-baseline-and-corpus/01-VALIDATION.md`

**Generated artifact ignore pattern** (lines 35, 45, 65):
```markdown
| 01-00-01 | ... | generated artifacts stay out of commits | doc/git | `rg -n "\\.planning/generated" .gitignore` | no | pending |
- [ ] `.gitignore` contains `.planning/generated/`.
| T-01-05 | Large generated reports, secrets, or relay logs enter git | high | Ignore `.planning/generated/` and commit compact sanitized summaries only | `rg -n "\\.planning/generated" .gitignore && git status --short` |
```

**Apply:** add exactly `.planning/generated/` before creating baseline logs, full hashes, regenerated results, or corpus profiles.

### `.planning/generated/phase-01/` (generated artifact root, file-I/O)

**Analog:** `.planning/phases/01-legacy-baseline-and-corpus/01-RESEARCH.md`

**Directory layout pattern** (lines 201-218):
```markdown
.planning/generated/phase-01/
├── baseline-runs/<run-id>/
├── corpus-profiles/<profile-id>/
└── backups/<timestamp>/
```

**Heavy artifact rule** (lines 241-256):
```markdown
Commit summaries, source inventories, fixture indexes, and reproducible commands; leave full hashes, regenerated results, logs, and profile dumps under `.planning/generated/phase-01/`.
```

**Apply:** generated files are local evidence; committed Phase 1 docs should record paths, counts, summary hashes, and commands, not bulky raw outputs.

### `baseline-command-runtime.md` (documentation, batch/file-I/O)

**Analog:** `01-RESEARCH.md`, legacy parser runtime files.

**Committed doc responsibility** (research lines 206, 314-321):
```markdown
baseline-command-runtime.md            # committed compact command/runtime/baseline manifest
Old parser repo is clean and at commit `3392ca2f367a87f6eb59041a239e7ca2519e1ec5`.
Old `package.json` script `parse` runs `tsx src/start.ts`.
Old `.nvmrc` pins `v18.14.0`.
```

**Legacy command pattern** (`/home/afgan0r/Projects/SolidGames/replays-parser/package.json` lines 25-26):
```json
"parse": "tsx src/start.ts",
"parse:dist": "node dist/start.js"
```

**Worker-count pattern** (`src/0 - utils/runtimeConfig.ts` lines 16-29):
```typescript
export const getRuntimeConfig = (): { workerCount: number } => {
  const envWorkerCount = process.env.WORKER_COUNT;
  ...
  return { workerCount: clampWorkerCount(Math.trunc(parsedWorkerCount)) };
};
```

**Non-destructive path pattern** (`src/0 - utils/paths.ts` lines 4-18; `src/4 - output/index.ts` lines 45-46):
```typescript
const statsPath = path.join(os.homedir(), 'sg_stats');
export const resultsPath = path.join(statsPath, 'results');
export const tempResultsPath = path.join(statsPath, 'temp_results');
...
fs.removeSync(resultsPath);
fs.moveSync(tempResultsPath, resultsPath);
```

**Apply:** document source command status, Node/pnpm/lockfile hashes, `WORKER_COUNT=1`, default worker profile, fake `HOME`, log paths, output hash summary, and any source-command blocker. Do not silently promote `parse:dist`.

### `corpus-manifest.md` (documentation, batch/transform)

**Analog:** `01-RESEARCH.md` corpus facts and validation checks.

**Corpus facts pattern** (research lines 323-334):
```markdown
`~/sg_stats/raw_replays` is 24G.
There are 17 raw replay basenames not present in `replaysList.json`.
Largest raw replay file ... `2021_10_31__00_13_51_ocap.json`.
`replaysList.json` mission-prefix counts are `sg: 2052`, `mace: 20702`, `sm: 243`, `sgs: 1`, and `other: 458`.
Existing `~/sg_stats/results` is 503M with 88,485 files.
```

**Validation pattern** (`01-VALIDATION.md` lines 63-64):
```markdown
`rg -n "23,473|23,456|88,485" .../corpus-manifest.md`
`rg -n "17|unlisted|replaysList" .../corpus-manifest.md`
```

**Apply:** include raw replay count, replay-list count/prepared timestamp, raw/list discrepancies, malformed/unlisted files, largest files, top-level key/event/entity shape evidence, game-type distribution, result count/size/hash summary, and generated profile paths.

### `legacy-rules-output-surfaces.md` (documentation, batch/transform)

**Analog:** legacy parser source.

**Game-type filter pattern** (`src/1 - replays/getReplays.ts` lines 17-23):
```typescript
const replays = uniqueReplays.filter(
  (replay) => (
    replay.mission_name.startsWith(gameType)
    && !replay.mission_name.startsWith('sgs')
  ),
);
```

**SM cutoff and orchestration pattern** (`src/index.ts` lines 23-30, 56-64, 108-134):
```typescript
if (gameType === 'sm') {
  replays = replays.filter(
    (replay) => dayjsUTC(replay.date).isAfter('2023-01-01', 'month'),
  );
}
...
const workerPool = new WorkerPool({ workerCount: getRuntimeConfig().workerCount, ... });
...
await generateOutput({ sg: { ...sgStats }, mace: { ...maceStats }, sm: { ...smStats } });
```

**Skip/error pattern** (`src/1 - replays/workers/parseReplayWorker.ts` lines 29-67):
```typescript
if (result.length === 0) return { status: 'skipped', reason: 'empty_replay', ... };
if (task.gameType === 'mace' && result.length < 10) {
  return { status: 'skipped', reason: 'mace_min_players', ... };
}
...
return { status: 'error', error: { filename, message, stack } };
```

**Yearly stats boundary** (`src/!yearStatistics/index.ts` lines 29-35, 94-96):
```typescript
/*
  This statistics includes different funny and interesting nominations
  that we usually show when it's New Year's Eve
*/
...
fs.emptyDirSync(yearResultsPath);
printOutput(result);
```

**Apply:** inventory source files, config inputs (`excludeReplays.json`, `includeReplays.json`, `excludePlayers.json`, `nameChanges.csv`), ordinary output paths/fields, skip rules, identity/name compatibility rules, and list yearly nomination outputs as v2-deferred only.

### `mismatch-taxonomy-interface-notes.md` (documentation, transform)

**Analog:** `01-CONTEXT.md`, `gsd-briefs/*.md`.

**Locked taxonomy/impact requirement** (`01-CONTEXT.md` lines 30-38):
```markdown
D-10: Legacy game-type filters and skip rules are owned by the parity harness, not the parser core contract.
D-12: Suspected legacy bugs require a human-review gate.
D-14: Old-vs-new mismatch taxonomy must include whether a diff affects only parser artifacts, `server-2` persistence/recalculation, or UI-visible public stats.
D-15: Phase 1 should create interface notes for `server-2` and `web` impact, without changing adjacent apps during this phase.
```

**Cross-app boundary pattern** (`gsd-briefs/replay-parser-2.md` lines 13, 146, 150):
```markdown
`replay-parser-2` owns the parsing engine and parsing result contract.
`server-2` owns persistence into PostgreSQL and aggregate publication.
Parser output contract changes that affect API payloads must be coordinated with `server-2` schema changes before `web` consumes new fields.
```

**Apply:** include categories: compatible, intentional change, old bug preserved, old bug fixed, new bug, insufficient data, human review. Each category must carry impact dimensions: parser artifact only, `server-2` persistence/recalculation, UI-visible public stats, and whether user approval is required.

### `fixture-index.json` (data fixture index, transform)

**Analog:** `01-RESEARCH.md` fixture example.

**JSON shape pattern** (research lines 263-274):
```json
{
  "source_file": "2021_10_31__00_13_51_ocap.json",
  "reason": ["largest-file", "mission_info", "mission_message", "killed-events"],
  "game_type_from_replays_list": "sg",
  "old_parser_skip_expected": false,
  "cross_app_relevance": ["parser-artifact", "server-recalculation", "ui-visible-public-stats"]
}
```

**Apply:** only create after full corpus profiling. Keep it compact and deterministic; sort entries by stable key such as `source_file`.

### `README.md` (documentation, request-response)

**Analog:** current `README.md` and quick-task 260425-g0r.

**README structure pattern** (`README.md` lines 7-18, 58-71, 105-132):
```markdown
## Current Status
...
## Data and References
...
## Development Workflow
Project development is performed only by AI agents using the GSD workflow.
...
## README Maintenance
Last updated: 2026-04-25.
```

**Quick-task verification pattern** (`260425-g0r-PLAN.md` lines 21-25):
```markdown
- Ensure README does not imply runnable code exists before implementation starts.
- Ensure README still explicitly states AI + GSD-only development.
- Run markdown whitespace validation with `git diff --check`.
```

**Apply:** update stale corpus language to full-history facts, keep "no runnable Rust workspace yet" accurate, preserve AI+GSD-only workflow, and keep README human-facing rather than turning it into a phase dump.

## Shared Patterns

### Verification Commands

**Source:** `01-VALIDATION.md` lines 20-26, 35-41.

```bash
git diff --check
git status --short
test -f .planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md
test -f .planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md
test -f .planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md
test -f .planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md
rg -n "\\.planning/generated" .gitignore
```

### Baseline Safety

**Source:** `01-RESEARCH.md` fake-HOME pattern and legacy `paths.ts`/output replacement code.

```bash
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-wc1"
RUN_ROOT="$PWD/.planning/generated/phase-01/baseline-runs/$RUN_ID"
RUN_HOME="$RUN_ROOT/home"
HOME="$RUN_HOME" WORKER_COUNT=1 pnpm run parse
```

Apply fake `HOME` for full baseline runs because the legacy parser writes to `os.homedir()/sg_stats` and replaces `results`.

### Documentation Style

**Source:** `01-CONTEXT.md`, `01-RESEARCH.md`, `01-VALIDATION.md`.

Use concise Markdown with frontmatter when the artifact is a strategy/plan, tables for facts and checks, and explicit `[VERIFIED: ...]` evidence when recording local command/source findings. Keep generated artifact paths and exact verification commands next to the claim they support.

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| Dedicated corpus profiler script, if planner creates one | utility | batch/transform/file-I/O | No current repo implementation exists yet; use research command patterns and Node/jq JSON parsing guidance instead. |

## Metadata

**Analog search scope:** repo planning docs, README, quick-task artifacts, `gsd-briefs`, and selected legacy parser source files.
**Files scanned:** planning/docs plus legacy command/runtime/filter/output files.
**Pattern extraction date:** 2026-04-25
