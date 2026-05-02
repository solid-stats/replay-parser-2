---
status: complete
quick_id: 260502-k2u
date: 2026-05-02
result: parity_improved_with_latest_teamkill_marker
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
the same deterministic sample first improved to 55 matches and 3 mismatches.
After the follow-up product decision to keep non-empty weapon names such as
`Binoculars` and `Throw` in the minimal artifact instead of applying the old
parser's forbidden-weapon filter, the sample reported 40 matches and 18
mismatches. After the additional product decision that `isDeadByTeamkill`
should represent the latest counted death state, not whether the player was
ever teamkilled earlier in the replay, the current run reports:

- Selected replays: 73
- New parser successes: 73
- Old parser successes: 58
- Old parser skipped: 15
- Stats-only matches: 42
- Stats-only mismatches: 16
- New parser failures: 0
- All comparable statistics compatible: false

Implemented compatibility fixes:

- Normalized legacy tag handling for duplicate-slot compatibility keys so
  `[TAG]Name` compares as `legacy_name:Name`.
- Avoided applying stale duplicate-slot keys when connected-event nicknames
  disagree with the original duplicate entity name.
- Compared old vehicle statistics together with old weapon statistics, matching
  the new minimal artifact's compact weapon dictionary surface.
- Preserved non-empty weapon statistic names, including old-parser forbidden
  names such as `throw`, `binoculars`, `бинокль`, `pdu`, and `vector`, because
  they can carry raw delayed ordnance/context evidence even when the old parser
  suppresses them from public weapon stats.
- Added a compact latest-death teamkill marker so `isDeadByTeamkill` is
  overwritten by later enemy, suicide, or null-killer deaths after a respawn.
- Preserved `unknown_deaths` diagnostics without counting unknown deaths as
  ordinary v1 deaths.
- Recorded old-parser skip reasons in generated old-runner results.

## Original Notable Findings

- `mace`: 28 selected, 11 stats-only matches, 2 mismatches, 15 old-parser
  skipped rows.
- `sg`: 28 selected, 28 stats-only mismatches.
- `sm`: 17 selected, 1 stats-only match, 16 mismatches.

## Follow-up Notable Findings

- `mace`: 28 selected, 12 stats-only matches, 1 mismatch, 15 old-parser
  skipped rows.
- `sg`: 28 selected, 17 stats-only matches, 11 mismatches.
- `sm`: 17 selected, 13 stats-only matches, 4 mismatches.

Current stats-only mismatches are mostly intentional weapon-surface differences
caused by retaining names the old parser suppresses:

- 21 `weapon_extra_in_new` rows, mostly `Throw` and `Binoculars`.
- 1 extra `teamkillers` relationship caused by old-parser merge behavior.

The previous `isDeadByTeamkill` edge cases now match the old-parser overwrite
semantics:

- `sg-2025-end-4ff4d4a0`: `Loza.isDeadByTeamkill` now becomes false after a
  later non-teamkill death.
- `sm-2026-start-747cf162`: `Likvidator.isDeadByTeamkill` now becomes false
  for the same later non-teamkill overwrite behavior.

The only remaining non-weapon mismatch is an old-parser teamkill relationship
merge edge case:

- `sg-2026-end-31556f1b`: `Zero.teamkillers` differs because the old parser's
  teamkill path merges from `killers` instead of existing `teamkillers`.

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
  because the stats-only parity gate found 18 mismatches after preserving
  old-parser forbidden weapon names.
- Latest-death teamkill marker verification passed:
  `cargo fmt --all --check`,
  `python3 -m py_compile scripts/compare-year-edge-sample.py`,
  `cargo test -p parser-core --test combat_event_semantics`,
  `cargo test -p parser-harness --test comparison_report`,
  `cargo test -p parser-contract --test schema_contract`,
  `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings`.
- Final `python3 scripts/compare-year-edge-sample.py` ran outside the sandbox
  because old-parser `tsx` needs an IPC socket; it exited 1 by design because
  the stats-only parity gate now finds 16 mismatches: 21 retained
  `Throw`/`Binoculars` weapon rows and 1 retained `teamkillers` relationship
  row across 58 comparable old-parser successes.
