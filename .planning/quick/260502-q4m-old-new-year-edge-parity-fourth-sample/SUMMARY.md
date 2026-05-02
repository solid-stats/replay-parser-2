---
status: complete
quick_id: 260502-q4m
date: 2026-05-02
result: fourth_sample_compared
---

# Summary: Old/New Year-Edge Replay Parity Fourth Sample

Ran a fourth deterministic old-vs-new comparison with seed `260502-q4m`.

## Evidence

- Full generated evidence:
  `.planning/generated/quick/260502-q4m-old-new-year-edge-parity-fourth-sample/`
- Committed lightweight evidence:
  `.planning/quick/260502-q4m-old-new-year-edge-parity-fourth-sample/artifacts/summary.md`
- Selected replay manifest:
  `.planning/quick/260502-q4m-old-new-year-edge-parity-fourth-sample/artifacts/selected-replays.json`

## Result

- Selected replays: 73
- New parser successes: 73
- Old parser successes: 56
- Old parser skipped: 17
- Stats-only matches: 33
- Stats-only mismatches: 23
- New parser failures: 0
- All comparable statistics compatible: false

By game type:

- `mace`: 10 matches, 1 mismatch, 17 old-parser skipped rows.
- `sg`: 11 matches, 17 mismatches.
- `sm`: 12 matches, 5 mismatches.

## Findings

No new mismatch class appeared:

- 25 `weapon_extra_in_new` rows: retained `Throw` or `Binoculars`.
- 6 `isDeadByTeamkill` rows: old-parser duplicate-slot merge-OR behavior.
- 1 `teamkillers` relationship row: the known old-parser teamkiller merge bug.

## Verification

- `python3 scripts/compare-year-edge-sample.py --output-root .planning/generated/quick/260502-q4m-old-new-year-edge-parity-fourth-sample --sample-seed 260502-q4m`
  ran outside the sandbox because old-parser `tsx` needs an IPC socket; it
  exited 1 by design because the stats-only parity gate found 23 mismatches.
- Mismatch classification found only known classes.
