---
status: complete
quick_id: 260502-nx9
date: 2026-05-02
result: second_sample_compared
---

# Summary: Old/New Year-Edge Replay Parity Second Sample

Ran a second deterministic old-vs-new comparison for the same sample policy as
`260502-k2u`, but with seed `260502-nx9`.

## Evidence

- Full generated evidence:
  `.planning/generated/quick/260502-nx9-old-new-year-edge-parity-second-sample/`
- Committed lightweight evidence:
  `.planning/quick/260502-nx9-old-new-year-edge-parity-second-sample/artifacts/summary.md`
- Selected replay manifest:
  `.planning/quick/260502-nx9-old-new-year-edge-parity-second-sample/artifacts/selected-replays.json`
- Runner:
  `scripts/compare-year-edge-sample.py --sample-seed 260502-nx9`

## Result

- Selected replays: 72
- Overlap with first sample: 12 replays
- New unique replays versus first sample: 60
- New parser successes: 72
- Old parser successes: 67
- Old parser skipped: 5
- Stats-only matches: 52
- Stats-only mismatches: 15
- New parser failures: 0
- All comparable statistics compatible: false

By game type:

- `mace`: 22 matches, 1 mismatch, 5 old-parser skipped rows.
- `sg`: 16 matches, 12 mismatches.
- `sm`: 14 matches, 2 mismatches.

## Findings

The second sample confirms the retained weapon-name mismatch class:

- 12 `weapon_extra_in_new` rows, all `Throw` or `Binoculars`.

It also surfaced 6 `isDeadByTeamkill` rows where old reports `true` and new
reports `false`:

- `sg-2022-start-bd4b69ea`: `Bear`, `MarolFox`.
- `sg-2024-end-825d864c`: `Metabo`.
- `sg-2024-end-bb447216`: `Grace`, `beda`.
- `sg-2025-start-1de393ce`: `Death`.

Spot checks show these are duplicate-slot/respawn cases. A prior entity for the
same player was killed by a teammate, then a later duplicate entity for the same
player was killed by an enemy. The new parser merges the duplicate-slot player
row and applies the current product rule that `isDeadByTeamkill` follows the
latest counted death. The old baseline keeps `isDeadByTeamkill=true` in these
cases, so this is a likely old-parser duplicate-slot merge edge rather than the
simple "latest death overwrites" behavior.

## Verification

- `python3 -m py_compile scripts/compare-year-edge-sample.py` passed.
- `cargo fmt --all --check` passed.
- `python3 scripts/compare-year-edge-sample.py --help` passed and shows
  `--sample-seed`.
- `python3 scripts/compare-year-edge-sample.py --output-root .planning/generated/quick/260502-nx9-old-new-year-edge-parity-second-sample --sample-seed 260502-nx9`
  ran outside the sandbox because old-parser `tsx` needs an IPC socket; it
  exited 1 by design because the stats-only parity gate found 15 mismatches.
