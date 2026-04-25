---
phase: 01-legacy-baseline-and-corpus
plan: 01
subsystem: legacy-baseline
tags: [legacy-parser, baseline, pnpm, node, parity]
requires:
  - phase: 01-00
    provides: source-command gate and generated-artifact hygiene
provides:
  - Reproducible old-parser command/runtime dossier.
  - Two non-destructive isolated baseline profiles.
  - Current-versus-regenerated result comparison classified under D-08.
affects: [phase-02, phase-05, server-2, web]
tech-stack:
  added: []
  patterns: [fake-home-baseline-isolation, relative-result-digests]
key-files:
  created:
    - .planning/phases/01-legacy-baseline-and-corpus/01-01-SUMMARY.md
  modified:
    - .planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md
key-decisions:
  - "Use the repaired source command at legacy commit 5e639fc0af222d198a4d20c402f2c8edb0bdc90d for baseline reproduction."
  - "Classify current-vs-regenerated output differences as human review until a future diff explains them."
patterns-established:
  - "Full old-parser baseline runs use fake HOME and never attach real results/year_results."
  - "Relative hash lists are used for path-independent output comparison."
requirements-completed: [LEG-01, LEG-02, LEG-05, WF-01, WF-02]
duration: 28min
completed: 2026-04-25
---

# Phase 01: Plan 01 Summary

**Reproducible legacy parser baseline dossier with two isolated full-corpus runs and D-08 comparison evidence**

## Performance

- **Duration:** 28 min
- **Started:** 2026-04-25T07:41:34Z
- **Completed:** 2026-04-25T08:09:03Z
- **Tasks:** 3
- **Files modified:** 1 committed dossier plus ignored generated evidence

## Accomplishments

- Ran `pnpm run parse` twice under fake HOME: `WORKER_COUNT=1` and default worker count.
- Verified real `~/sg_stats/results` and `~/sg_stats/year_results` aggregate digests were unchanged after both runs.
- Recorded output file counts, sizes, digest paths, malformed replay warnings, and D-08 comparison results in `baseline-command-runtime.md`.

## Task Commits

1. **Tasks 1-3: Baseline profiles, D-08 comparison, and manifest finalization** - `4e75d4b`

## Files Created/Modified

- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` - Final command/runtime manifest, generated paths, output hash summary, and current-vs-regenerated classification.
- `.planning/generated/phase-01/baseline-runs/20260425T074853Z-isolated-baseline-*` - Ignored full logs, regenerated outputs, hash lists, and comparison JSON.

## Decisions Made

Current historical results differ from both regenerated old-parser outputs, and the two regenerated profiles also differ by relative digest/size. These are recorded as `human review`; Phase 1 does not decide whether the difference is an old bug to preserve, old bug to fix, or environmental output drift.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Subagent execution fallback**
- **Found during:** Wave 2 dispatch
- **Issue:** Background executor agents produced no completion signal or filesystem work.
- **Fix:** Closed the idle agents and executed this plan inline/sequentially per the execute-phase fallback rule.
- **Files modified:** None beyond the plan deliverables.
- **Verification:** Plan acceptance commands passed after inline execution.
- **Committed in:** `4e75d4b`

---

**Total deviations:** 1 auto-fixed blocking workflow issue.
**Impact on plan:** Execution mode changed, but deliverables and verification stayed aligned with the plan.

## Issues Encountered

- The first baseline shell attempt failed before running the parser because `set -u` tripped an environment hook while loading Node tooling. It was rerun without nounset.
- Both full runs reported the same three malformed replay warnings; these are preserved as baseline evidence.

## User Setup Required

None.

## Next Phase Readiness

Phase 2 and Phase 5 can now rely on a reproducible old-parser source command, generated baseline output locations, and explicit D-08 comparison categories. The human-review differences must be investigated before claiming parity.

## Self-Check: PASSED

- `rg -n "WORKER_COUNT=1|default worker|fake HOME|results.sha256|real ~/sg_stats/results.*unchanged" baseline-command-runtime.md` passed.
- `rg -n "## Current Results Comparison|D-08|human review|server-2 recalculation|UI-visible public stats" baseline-command-runtime.md` passed.
- `rg -n "pnpm@10\\.33\\.0|v18\\.14\\.0|sha256" baseline-command-runtime.md` passed.
- `git diff --check` passed.

---
*Phase: 01-legacy-baseline-and-corpus*
*Completed: 2026-04-25*
