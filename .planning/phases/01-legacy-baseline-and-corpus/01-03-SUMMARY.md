---
phase: 01-legacy-baseline-and-corpus
plan: 03
subsystem: legacy-rules
tags: [legacy-parser, parity, identity, outputs]
requires:
  - phase: 01-00
    provides: generated artifact hygiene
provides:
  - Legacy game-type filter, skip-rule, config, identity, and output-surface dossier.
  - Parser-core versus parity-harness ownership boundary for legacy rules.
  - V2-deferred annual nomination boundary.
affects: [phase-02, phase-05, server-2, web]
tech-stack:
  added: []
  patterns: [parity-harness-boundary, compatibility-identity-layer]
key-files:
  created:
    - .planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md
    - .planning/phases/01-legacy-baseline-and-corpus/01-03-SUMMARY.md
  modified: []
key-decisions:
  - "Keep legacy selection filters and skip rules in the parity harness, not the parser core contract."
  - "Keep observed identity separate from legacy compatibility identity transforms."
  - "List annual nomination outputs as historical v2-deferred references only."
patterns-established:
  - "Legacy rule dossiers cite old-parser source paths and separate observed identity from compatibility behavior."
  - "Ordinary result surfaces are inventoried separately from yearly nomination references."
requirements-completed: [LEG-04, INT-01, INT-02, INT-04]
duration: 5min
completed: 2026-04-25
---

# Phase 01: Plan 03 Summary

**Legacy rule and output-surface inventory for parity harness ownership and identity compatibility boundaries**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-25T08:18:07Z
- **Completed:** 2026-04-25T08:23:37Z
- **Tasks:** 2
- **Files modified:** 2 committed artifacts

## Accomplishments

- Documented legacy game-type filters, `sgs` exclusion, `sm` cutoff, skip reasons, config inputs, and identity compatibility behavior.
- Inventoried ordinary output surfaces, comparable fields, rotation folders, per-player folders, and `stats.zip`.
- Recorded `src/!yearStatistics` and `~/sg_stats/year_results` as v2-deferred annual nomination references only.

## Task Commits

1. **Tasks 1-2: Legacy rules, identity boundaries, and output surfaces** - `ebfb8e1`

## Files Created/Modified

- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md` - Legacy selection, skip, config, identity, output, and yearly-reference dossier.
- `.planning/phases/01-legacy-baseline-and-corpus/01-03-SUMMARY.md` - This summary.

## Decisions Made

None beyond executing locked Phase 1 decisions D-10 through D-13.

## Deviations from Plan

None - plan executed exactly as written after the Wave 2 fallback to inline execution.

## Issues Encountered

No blocking issues. The legacy source/current results do not contain a standalone `statistics.json`; the dossier records that fact while preserving the planned `statistics.json` search term as generic shorthand.

## User Setup Required

None.

## Next Phase Readiness

Phase 2 can avoid importing legacy aggregate filters into the parser contract. Phase 5 can use this dossier to build the parity compatibility layer and compare ordinary outputs while leaving yearly nominations out of v1 scope.

## Self-Check: PASSED

- `test -f legacy-rules-output-surfaces.md` passed.
- `rg -n "mission_name\\.startsWith|startsWith\\('sgs'\\)|2023-01-01|empty_replay|mace_min_players|excludeReplays\\.json|includeReplays\\.json|excludePlayers\\.json|nameChanges\\.csv|observed identity|compatibility layer" legacy-rules-output-surfaces.md` passed.
- `rg -n "statistics\\.json|squad_statistics\\.json|squad_full_rotation_statistics\\.json|rotations_info\\.json|stats\\.zip|killsFromVehicle|vehicleKills|teamkills|kdRatio|killsFromVehicleCoef|totalPlayedGames|src/!yearStatistics|year_results|Annual/yearly nomination outputs are historical references only" legacy-rules-output-surfaces.md` passed.
- `git diff --check` passed.

---
*Phase: 01-legacy-baseline-and-corpus*
*Completed: 2026-04-25*
