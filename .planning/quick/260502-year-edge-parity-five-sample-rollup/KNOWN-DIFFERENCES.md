# Known Old/New Statistics Differences

This note documents old/new parser differences accepted after five deterministic
year-edge parity samples. It is written for future engineers or agents reviewing
parity output.

## Scope

These differences apply to derived legacy statistics comparisons between the old
TypeScript parser and the new Rust parser. They do not describe raw JSON shape
differences; the new default artifact is intentionally minimal and structurally
different from the old output.

## Accepted Difference Classes

### Retained `Throw` and `Binoculars` Weapon Rows

The new parser preserves non-empty weapon names in weapon statistics. The old
parser suppresses a legacy forbidden-weapon set, including `Throw` and
`Binoculars`.

This creates `weapon_extra_in_new` rows when delayed ordnance, thrown ordnance,
or vehicle/static context is attributed while the actor's current weapon-like
source is one of those suppressed names.

The new behavior is intentional because the raw non-empty weapon name can carry
useful evidence. Public presentation can still choose to hide or group these
names later.

### Duplicate-Slot `isDeadByTeamkill` Merge

The old parser first computes death state per entity, then merges multiple
entities with the same player name. During that merge it combines
`isDeadByTeamkill` with boolean OR.

That means if one slot/entity for a player died by teamkill and a later
slot/entity for the same player died by an enemy, the old parser can still emit
`isDeadByTeamkill=true`.

The new parser follows the current product rule: for a merged player row,
`isDeadByTeamkill` reflects the latest counted death. If the latest counted
death is enemy, suicide, or null-killer, the marker is false even if an earlier
entity died by teamkill.

This difference is accepted because it models respawn/duplicate-slot play more
usefully for current statistics.

### Old `teamkillers` Merge Bug

The old parser's teamkill path merges the victim's `teamkillers` relationship
from the ordinary `killers` list instead of from the existing `teamkillers`
list. In edge cases where a player has both normal killers and teamkillers, the
old output can lose or overwrite a teamkiller relationship.

The new parser keeps the teamkiller relationship. This can appear as an extra
`teamkillers` row in new output.

This difference is accepted because reproducing the old merge bug would discard
valid teamkill relationship evidence.

## Not Accepted By Default

Any future mismatch outside the three classes above should be investigated
before treating parity as acceptable. In particular:

- new parser failures on replay files that the old parser handles;
- player kill, death, teamkill, vehicle kill, or kills-from-vehicle counter
  differences not explained by duplicate-slot teamkill-death state;
- relationship differences other than the known old `teamkillers` merge bug;
- missing weapon rows in new output;
- unexpected weapon names beyond the retained non-empty names already observed.

## Five-Sample Evidence

Across five deterministic year-edge samples:

- 364 selected replay entries were parsed by the new parser with zero failures.
- 305 old-parser artifacts were comparable; 59 old-parser rows were skipped by
  the legacy parser.
- 212 comparable stats-only results matched exactly.
- 93 comparable stats-only results mismatched only in documented known classes.

The observed mismatch detail rows were:

- 104 retained `Throw`/`Binoculars` weapon rows.
- 26 duplicate-slot `isDeadByTeamkill` rows.
- 2 old `teamkillers` merge-bug rows.
