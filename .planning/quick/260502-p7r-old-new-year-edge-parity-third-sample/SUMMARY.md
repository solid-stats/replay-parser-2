---
status: complete
quick_id: 260502-p7r
date: 2026-05-02
result: third_sample_compared
---

# Summary: Old/New Year-Edge Replay Parity Third Sample

Ran a third deterministic old-vs-new comparison for the same year-edge sample
policy, with seed `260502-p7r`.

## Evidence

- Full generated evidence:
  `.planning/generated/quick/260502-p7r-old-new-year-edge-parity-third-sample/`
- Committed lightweight evidence:
  `.planning/quick/260502-p7r-old-new-year-edge-parity-third-sample/artifacts/summary.md`
- Selected replay manifest:
  `.planning/quick/260502-p7r-old-new-year-edge-parity-third-sample/artifacts/selected-replays.json`
- Runner:
  `scripts/compare-year-edge-sample.py --sample-seed 260502-p7r`

## Result

- Selected replays: 73
- Overlap with first sample: 9 replays
- Overlap with second sample: 9 replays
- Overlap with any previous sample: 16 replays
- New unique replays versus previous samples: 57
- New parser successes: 73
- Old parser successes: 61
- Old parser skipped: 12
- Stats-only matches: 40
- Stats-only mismatches: 21
- New parser failures: 0
- All comparable statistics compatible: false

By game type:

- `mace`: 14 matches, 2 mismatches, 12 old-parser skipped rows.
- `sg`: 15 matches, 13 mismatches.
- `sm`: 11 matches, 6 mismatches.

## Findings

The third sample did not introduce a new mismatch class:

- 28 `weapon_extra_in_new` rows, all retained `Throw` or `Binoculars`.
- 5 `isDeadByTeamkill` rows where old reports `true` and new reports `false`.

The `isDeadByTeamkill` rows are the same old-parser merge-OR pattern seen in
the second sample. A prior entity for the same compatibility key died from a
teamkill, while a later merged entity died from an enemy kill. The new parser
follows the current product rule that the merged player's latest counted death
controls `isDeadByTeamkill`; the old baseline keeps the flag true because its
same-player merge uses boolean OR.

Checked `isDeadByTeamkill` rows:

- `sg-2020-start-a6fb7ed8`: `JustDave`.
- `sg-2020-start-fc7dae27`: `Felix`.
- `sg-2024-end-825d864c`: `Metabo`.
- `sg-2026-end-f2b5a9c5`: `Midas`.
- `sm-2024-start-682f3bd6`: `varekai`.

## Verification

- `python3 scripts/compare-year-edge-sample.py --output-root .planning/generated/quick/260502-p7r-old-new-year-edge-parity-third-sample --sample-seed 260502-p7r`
  ran outside the sandbox because old-parser `tsx` needs an IPC socket; it
  exited 1 by design because the stats-only parity gate found 21 mismatches.
- Stats-only mismatch classification found only retained weapon rows and the
  known old-parser `isDeadByTeamkill` merge-OR class.
