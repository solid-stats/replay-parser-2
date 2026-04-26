---
phase: 01-legacy-baseline-and-corpus
plan: 02
subsystem: corpus-profile
tags: [corpus, fixtures, sg-stats, legacy-parser]
requires:
  - phase: 01-00
    provides: generated artifact hygiene
provides:
  - Current full-history corpus manifest.
  - Compact fixture index derived from actual corpus profile evidence.
  - Updated project and parser brief corpus facts.
affects: [phase-02, phase-05, fixtures, benchmarks]
tech-stack:
  added: []
  patterns: [ignored-full-corpus-profile, compact-fixture-index]
key-files:
  created:
    - .planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md
    - .planning/phases/01-legacy-baseline-and-corpus/fixture-index.json
    - .planning/phases/01-legacy-baseline-and-corpus/01-02-SUMMARY.md
  modified:
    - .planning/PROJECT.md
    - gsd-briefs/replay-parser-2.md
key-decisions:
  - "Use current full-history counts instead of stale around-3,938 assumptions."
  - "Keep the full corpus profile ignored and commit only summary facts plus fixture seeds."
patterns-established:
  - "Corpus facts must be generated from local evidence and point to ignored full-profile artifacts."
  - "Fixture candidates carry old-parser skip expectation and cross-app relevance."
requirements-completed: [DOC-01, LEG-03, WF-01, WF-02]
duration: 9min
completed: 2026-04-25
---

# Phase 01: Plan 02 Summary

**Full-history corpus manifest for 23,473 raw replay JSON files with compact fixture seed coverage**

## Performance

- **Duration:** 9 min
- **Started:** 2026-04-25T08:09:03Z
- **Completed:** 2026-04-25T08:18:07Z
- **Tasks:** 2
- **Files modified:** 4 committed artifacts plus ignored generated profile

## Accomplishments

- Generated `.planning/generated/phase-01/corpus-profiles/20260425T081026Z-corpus-profile/corpus-profile.json`.
- Wrote `corpus-manifest.md` with raw/list/result/year counts, schema/shape summaries, malformed files, largest files, and game-type distribution.
- Created `fixture-index.json` with fixture candidates for large file, `sg`, `mace`, `sm`, `sgs`, `other`, raw-not-listed, and malformed coverage.
- Updated `.planning/PROJECT.md` and `gsd-briefs/replay-parser-2.md` from stale `3,938` wording to current full-history facts.

## Task Commits

1. **Tasks 1-2: Corpus profile, manifest, fixture index, and doc count updates** - `472eac6`

## Files Created/Modified

- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` - Current full-history corpus evidence summary.
- `.planning/phases/01-legacy-baseline-and-corpus/fixture-index.json` - Compact fixture seed index with skip expectations and cross-app relevance.
- `.planning/PROJECT.md` - Current corpus facts.
- `gsd-briefs/replay-parser-2.md` - Current parser brief corpus facts.

## Decisions Made

None beyond executing the plan: the fixture index is a seed list for Phase 5, not the final golden fixture suite.

## Deviations from Plan

None - plan executed as written after the Wave 2 fallback to inline execution.

## Issues Encountered

The corpus profiler found 4 malformed raw files, including a raw file named `.json`. This is recorded as corpus evidence and should feed structured failure tests later.

## User Setup Required

None.

## Next Phase Readiness

Phase 5 fixture selection can start from profile-derived candidates instead of stale count assumptions. Phase 2 contract planning can use the observed top-level key and event/entity shape summaries.

## Self-Check: PASSED

- Generated profile jq check passed for `23473`, `2026-04-25T04:42:54.889Z`, `88485`, `14`, `17`, and `0`.
- `fixture-index.json` is valid JSON and every entry has the five required keys.
- `.planning/PROJECT.md` and `gsd-briefs/replay-parser-2.md` contain `23,473`, `23,456`, and `2026-04-25T04:42:54.889Z`.
- `git diff --check` passed.

---
*Phase: 01-legacy-baseline-and-corpus*
*Completed: 2026-04-25*
