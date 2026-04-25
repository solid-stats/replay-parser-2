---
phase: 01-legacy-baseline-and-corpus
plan: 00
subsystem: legacy-baseline
tags: [gsd, legacy-parser, pnpm, node, baseline]
requires: []
provides:
  - Generated Phase 1 evidence directories are ignored by git.
  - Canonical old-parser source command preflight is documented.
  - Source-command failure was repaired and re-verified before full baseline execution.
affects: [phase-01, phase-05, legacy-parser-baseline]
tech-stack:
  added: []
  patterns: [ignored-generated-evidence, source-command-gate]
key-files:
  created:
    - .gitignore
    - .planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md
  modified:
    - /home/afgan0r/Projects/SolidGames/replays-parser/src
key-decisions:
  - "workflow.auto_advance selected repair-source-command at the blocking decision checkpoint."
  - "The legacy parser source command remains canonical; parse:dist is diagnostic only unless explicitly approved later."
patterns-established:
  - "Phase 1 bulky logs, hashes, profiles, regenerated outputs, and backups live under ignored .planning/generated/."
  - "A failing canonical legacy command must be repaired or explicitly overridden before any full baseline run."
requirements-completed: [WF-01, WF-02, LEG-01, LEG-02]
duration: 8min
completed: 2026-04-25
---

# Phase 01: Plan 00 Summary

**Generated evidence hygiene plus a repaired canonical legacy source-command preflight for Phase 1 baseline work**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-25T07:31:43Z
- **Completed:** 2026-04-25T07:38:49Z
- **Tasks:** 3
- **Files modified:** 3 tracked artifacts in this repo, plus one committed compatibility repair in the legacy parser repo

## Accomplishments

- Added `.planning/generated/` to `.gitignore` and created ignored local Phase 1 evidence directories.
- Recorded the canonical `pnpm run parse` -> `tsx src/start.ts` preflight, runtime target, lockfile hash, log paths, and initial failure.
- Auto-selected `repair-source-command`, committed the legacy parser Lodash import compatibility repair at `5e639fc0af222d198a4d20c402f2c8edb0bdc90d`, and re-ran the canonical preflight successfully.

## Task Commits

1. **Task 1: Add generated artifact hygiene** - `5b0914c`
2. **Task 2: Record the canonical source-command preflight gate** - `c3ec98f`
3. **Task 3: Resolve source-command blocker before baseline execution** - `4e41b46`

**External compatibility repair:** `/home/afgan0r/Projects/SolidGames/replays-parser` commit `5e639fc0af222d198a4d20c402f2c8edb0bdc90d`.

## Files Created/Modified

- `.gitignore` - Ignores generated GSD evidence artifacts.
- `.planning/generated/phase-01/` - Local ignored evidence directories for baseline runs, corpus profiles, and backups.
- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` - Source-command gate, preflight evidence, fallback warning, repair decision, and post-repair pass.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src` - Mechanical Lodash import compatibility repair committed in the legacy parser repo.

## Decisions Made

`parse:dist` remains a secondary diagnostic only. The canonical baseline path is still `pnpm run parse`; it is now unblocked by the committed old-parser compatibility repair.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired legacy source-command runtime failure**
- **Found during:** Task 3 (Resolve source-command blocker before baseline execution)
- **Issue:** `pnpm run parse -- --help` failed because Node ESM could not provide named exports from CommonJS `lodash`.
- **Fix:** Replaced legacy parser Lodash named ESM imports with default imports plus destructuring, preserving the same Lodash functions.
- **Files modified:** `/home/afgan0r/Projects/SolidGames/replays-parser/src`
- **Verification:** `pnpm run parse -- --help` passes under Node `v18.14.0`.
- **Committed in:** `5e639fc0af222d198a4d20c402f2c8edb0bdc90d`

---

**Total deviations:** 1 auto-fixed blocking issue.
**Impact on plan:** The repair was required to satisfy D-01 without promoting `parse:dist` to canonical baseline. No full old-parser baseline was run.

## Issues Encountered

`pnpm run tsc` in the legacy repo still fails on pre-existing NodeNext/package typing issues, including missing explicit relative import extensions and `@types/node`/DOM `AbortSignal` conflicts. This is documented as a validation note, but it does not block Plan 00 because the plan gate is the source-command `--help` preflight.

## User Setup Required

None.

## Next Phase Readiness

Wave 2 can run baseline and corpus plans against the canonical source command. Any full old-parser run must still use the isolated/generated paths documented by Phase 1 planning and must not mutate real `~/sg_stats/results` or `~/sg_stats/year_results`.

## Self-Check: PASSED

- `rg -n "^\\.planning/generated/$" .gitignore` passed.
- `pnpm run parse -- --help` passed in `/home/afgan0r/Projects/SolidGames/replays-parser` under Node `v18.14.0`.
- `rg -n "Gate decision:|Source command status: PASS" baseline-command-runtime.md` passed.
- `git diff --check` passed in this repo.

---
*Phase: 01-legacy-baseline-and-corpus*
*Completed: 2026-04-25*
