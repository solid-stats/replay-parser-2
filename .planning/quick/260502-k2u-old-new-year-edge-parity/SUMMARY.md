---
status: complete
quick_id: 260502-k2u
date: 2026-05-02
result: parity_failed
---

# Summary: Old/New Year-Edge Replay Parity

Ran a deterministic old-vs-new comparison for the requested sample policy:
`sg`, `mace`, and `sm` replay rows grouped by game type and year, with up to
two deterministic random picks from the first 20 and last 20 rows of each group.

## Evidence

- Full generated evidence:
  `.planning/generated/quick/260502-k2u-old-new-year-edge-parity/`
- Committed lightweight evidence:
  `.planning/quick/260502-k2u-old-new-year-edge-parity/artifacts/summary.md`
- Selected replay manifest:
  `.planning/quick/260502-k2u-old-new-year-edge-parity/artifacts/selected-replays.json`
- Runner:
  `scripts/compare-year-edge-sample.py`

## Result

- Selected replays: 73
- New parser successes: 73
- Old parser successes: 58
- Old parser skipped: 15
- Stats-only matches: 12
- Stats-only mismatches: 46
- New parser failures: 0
- All comparable statistics compatible: false

The current parser does not yet prove old-parser statistical parity on this
sample. The most common stats-only differences are relationship rows, weapon
kill rows, and player counter rows. The large exact JSON comparison also remains
`human_review` because the old and new artifact shapes intentionally differ.

## Notable Findings

- `mace`: 28 selected, 11 stats-only matches, 2 mismatches, 15 old-parser
  skipped rows.
- `sg`: 28 selected, 28 stats-only mismatches.
- `sm`: 17 selected, 1 stats-only match, 16 mismatches.

Example mismatch classes recorded in the generated per-replay reports:

- Player death/counter mismatch, for example `legacy_name:shketus` in
  `mace-2020-start-75b45469`.
- Extra/missing weapon attribution, for example `Binoculars` in
  `mace-2020-start-62c9bab1`.
- Missing/extra relationship rows, for example `Nerdan -> JustDave` in the
  generated stats-only reports.

## Verification

- `python3 -m py_compile scripts/compare-year-edge-sample.py` passed.
- `python3 scripts/compare-year-edge-sample.py --help` passed.
- `python3 scripts/compare-year-edge-sample.py` ran outside the sandbox because
  old-parser `tsx` needs an IPC socket; it exited 1 by design because the
  stats-only parity gate found 46 mismatches.
