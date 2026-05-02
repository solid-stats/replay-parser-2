---
status: complete
quick_id: 260502-rollup
date: 2026-05-02
result: phase_05_parity_unblocked
---

# Summary: Five-Sample Year-Edge Old/New Parity Rollup

Five deterministic year-edge samples were run across `sg`, `mace`, and `sm`.
No new old/new statistics mismatch class appeared in samples 4 and 5, and all
remaining mismatch classes are documented in `KNOWN-DIFFERENCES.md`.

## Aggregate Result

- Sample runs: 5
- Selected replay entries, including repeats: 364
- Unique selected replay files: 291
- New parser successes: 364
- Old parser successes: 305
- Old parser skipped: 59
- Comparable stats-only matches: 212
- Comparable stats-only mismatches: 93
- New parser failures: 0

## Mismatch Classes

Across all five samples, mismatch detail rows fell into these known classes:

- 104 retained weapon rows, caused by preserving non-empty `Throw` and
  `Binoculars` weapon names that the old parser suppresses.
- 26 `isDeadByTeamkill` rows, caused by the old parser's duplicate-slot merge
  using boolean OR while the new parser follows the latest counted death for
  the merged player.
- 2 `teamkillers` relationship rows, caused by the old parser's teamkiller path
  merging from the ordinary `killers` list instead of the existing
  `teamkillers` list.

## Decision

The five-sample parity follow-up does not block Phase 05.2/Phase 5 readiness.
Phase 05.2 remains accepted and Phase 6 work can proceed. Future parity work
should investigate only mismatch classes outside the documented known
differences, or a regression in new parser success/failure behavior.

## Evidence

- Sample 1: `.planning/quick/260502-k2u-old-new-year-edge-parity/`
- Sample 2: `.planning/quick/260502-nx9-old-new-year-edge-parity-second-sample/`
- Sample 3: `.planning/quick/260502-p7r-old-new-year-edge-parity-third-sample/`
- Sample 4: `.planning/quick/260502-q4m-old-new-year-edge-parity-fourth-sample/`
- Sample 5: `.planning/quick/260502-v8n-old-new-year-edge-parity-fifth-sample/`
- Known differences: `KNOWN-DIFFERENCES.md`

## Reader-Test

A cold reader can classify future parity mismatches by checking whether they are
retained `Throw`/`Binoculars` weapon rows, duplicate-slot `isDeadByTeamkill`
merge-OR rows, or the known old `teamkillers` merge bug. Anything else remains
investigation material.
