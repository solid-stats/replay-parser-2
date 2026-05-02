---
status: complete
quick_id: 260502-v8n
date: 2026-05-02
result: fifth_sample_compared
---

# Summary: Old/New Year-Edge Replay Parity Fifth Sample

Ran a fifth deterministic old-vs-new comparison with seed `260502-v8n`.

## Evidence

- Full generated evidence:
  `.planning/generated/quick/260502-v8n-old-new-year-edge-parity-fifth-sample/`
- Committed lightweight evidence:
  `.planning/quick/260502-v8n-old-new-year-edge-parity-fifth-sample/artifacts/summary.md`
- Selected replay manifest:
  `.planning/quick/260502-v8n-old-new-year-edge-parity-fifth-sample/artifacts/selected-replays.json`

## Result

- Selected replays: 73
- New parser successes: 73
- Old parser successes: 63
- Old parser skipped: 10
- Stats-only matches: 45
- Stats-only mismatches: 18
- New parser failures: 0
- All comparable statistics compatible: false

By game type:

- `mace`: 16 matches, 2 mismatches, 10 old-parser skipped rows.
- `sg`: 18 matches, 10 mismatches.
- `sm`: 11 matches, 6 mismatches.

## Findings

No new mismatch class appeared:

- 18 `weapon_extra_in_new` rows: retained `Throw` or `Binoculars`.
- 9 `isDeadByTeamkill` rows: old-parser duplicate-slot merge-OR behavior.

## Verification

- `python3 scripts/compare-year-edge-sample.py --output-root .planning/generated/quick/260502-v8n-old-new-year-edge-parity-fifth-sample --sample-seed 260502-v8n`
  ran outside the sandbox because old-parser `tsx` needs an IPC socket; it
  exited 1 by design because the stats-only parity gate found 18 mismatches.
- Mismatch classification found only known classes.
