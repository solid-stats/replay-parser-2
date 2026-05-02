---
status: complete
quick_id: 260502-k2u
date: 2026-05-02
result: parity_improved
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

## Original Result

- Selected replays: 73
- New parser successes: 73
- Old parser successes: 58
- Old parser skipped: 15
- Stats-only matches: 12
- Stats-only mismatches: 46
- New parser failures: 0
- All comparable statistics compatible: false

The original run did not prove old-parser statistical parity on this sample.
The most common stats-only differences were relationship rows, weapon kill rows,
and player counter rows. The large exact JSON comparison also remains
`human_review` because the old and new artifact shapes intentionally differ.

## Follow-up Fix Result

After tightening the stats-only comparator and parser v1 compatibility rules,
the same deterministic sample now reports:

- Selected replays: 73
- New parser successes: 73
- Old parser successes: 58
- Old parser skipped: 15
- Stats-only matches: 55
- Stats-only mismatches: 3
- New parser failures: 0
- All comparable statistics compatible: false

Implemented compatibility fixes:

- Normalized legacy tag handling for duplicate-slot compatibility keys so
  `[TAG]Name` compares as `legacy_name:Name`.
- Avoided applying stale duplicate-slot keys when connected-event nicknames
  disagree with the original duplicate entity name.
- Compared old vehicle statistics together with old weapon statistics, matching
  the new minimal artifact's compact weapon dictionary surface.
- Filtered old-parser forbidden weapon statistic names such as `throw`,
  `binoculars`, `бинокль`, `pdu`, and `vector`.
- Preserved `unknown_deaths` diagnostics without counting unknown deaths as
  ordinary v1 deaths.
- Recorded old-parser skip reasons in generated old-runner results.

## Original Notable Findings

- `mace`: 28 selected, 11 stats-only matches, 2 mismatches, 15 old-parser
  skipped rows.
- `sg`: 28 selected, 28 stats-only mismatches.
- `sm`: 17 selected, 1 stats-only match, 16 mismatches.

## Follow-up Notable Findings

- `mace`: 28 selected, 13 stats-only matches, 15 old-parser skipped rows.
- `sg`: 28 selected, 26 stats-only matches, 2 mismatches.
- `sm`: 17 selected, 16 stats-only matches, 1 mismatch.

Remaining stats-only mismatches are limited to three old-parser teamkill edge
cases:

- `sg-2025-end-4ff4d4a0`: `Loza.isDeadByTeamkill` differs because the old parser
  later overwrites the prior teamkill-death flag with a non-teamkill death.
- `sg-2026-end-31556f1b`: `Zero.teamkillers` differs because the old parser's
  teamkill path merges from `killers` instead of existing `teamkillers`.
- `sm-2026-start-747cf162`: `Likvidator.isDeadByTeamkill` differs for the same
  later non-teamkill overwrite behavior.

Original example mismatch classes recorded in the generated per-replay reports:

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
- Follow-up targeted tests passed:
  `cargo test -p parser-core --test combat_event_semantics` and
  `cargo test -p parser-core --test legacy_entity_compatibility`.
- Follow-up `python3 scripts/compare-year-edge-sample.py` ran outside the
  sandbox because old-parser `tsx` needs an IPC socket; it exited 1 by design
  because the stats-only parity gate now finds 3 mismatches.
