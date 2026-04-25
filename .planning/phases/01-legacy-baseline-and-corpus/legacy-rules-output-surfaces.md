---
phase: 01
artifact: legacy-rules-output-surfaces
status: draft
---

# Legacy Rules and Output Surfaces

## Game-Type Selection

Legacy replay selection is a parity-harness concern, not a parser core contract rule. Phase 2 should keep normalized parser artifacts based on observed replay contents; Phase 5 can apply the legacy selection policy when comparing old and new aggregate outputs.

Source references:
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/1 - replays/getReplays.ts:20` filters replay-list rows with `replay.mission_name.startsWith(gameType)`.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/1 - replays/getReplays.ts:23` excludes Solid Games Squad missions with `!replay.mission_name.startsWith('sgs')`.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/index.ts:26` applies an extra `sm` selection rule: `dayjsUTC(replay.date).isAfter('2023-01-01', 'month')`.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/1 - replays/getReplays.ts:19` deduplicates replay-list rows by filename before filtering.

The new parser should not hide or drop replay content based on these legacy filters. The filters are used to reproduce the old aggregate baseline and should be named as such in the comparison harness.

## Skip Rules

Legacy parse workers can return successful skips rather than parsed replay data.

Source references:
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/1 - replays/workers/parseReplayWorker.ts:29` returns skipped status with reason `empty_replay` when parsed replay info has no player results.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/1 - replays/workers/parseReplayWorker.ts:38` returns skipped status with reason `mace_min_players` when a `mace` replay has fewer than `10` parsed players.
- Other read/parse failures return `status: 'error'` with filename, message, and stack; they are not the same as the two explicit skip reasons.

These skip reasons are legacy aggregate/parity behavior. A normalized parser artifact can still represent source references and parse failures without treating the legacy aggregate skip as canonical parser semantics.

## Config Inputs

Legacy parity depends on four config inputs:

| Input | Observed state | Legacy role |
|---|---:|---|
| `/home/afgan0r/Projects/SolidGames/replays-parser/config/excludeReplays.json` | 16 replay path entries, including one duplicate | Replay exclusion input used by legacy preparation/selection logic. |
| `/home/afgan0r/Projects/SolidGames/replays-parser/config/includeReplays.json` | 3 manual mission includes: Red Dawn, Unorthodox Methods, Nuclear Danger | Manual mission-to-game-type include input. |
| `/home/afgan0r/Projects/SolidGames/replays-parser/config/excludePlayers.json` | 5 player rules: scandal, mayson, exile, mooniverse, jm0t | Legacy player exclusion input with optional date bounds. |
| `~/sg_stats/config/nameChanges.csv` | 74 lines including header | Legacy name/id compatibility input. |

The new parser should preserve source-observed replay identity fields. Config-driven exclusions, manual includes, and name compatibility belong in the parity harness or aggregate compatibility layer unless a later phase explicitly moves a specific rule into a server-owned workflow.

## Identity Compatibility Rules

Observed identity and compatibility identity are separate.

Source references:
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/getEntities.ts:27` creates player records from unit entities using observed `id`, `name`, and `side`.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/getEntities.ts:55` backfills player records from `connected` events when the entity is not a vehicle and the connected name is present.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/combineSamePlayersInfo.ts:13` merges duplicate player records by equal `name`, combining kills, vehicle stats, teamkills, death flags, weapons, vehicles, and other-player references.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/0 - utils/namesHelper/prepareNamesList.ts` reads `nameChanges.csv` and prepares legacy name history for aggregate compatibility.

Phase 2 parser artifacts must keep observed identity raw enough for `server-2` to own canonical player matching later. Old same-name combining, accepted name-change history, and display-name compatibility are compatibility layer behavior for parity and aggregate comparison, not parser-core identity normalization.

## Harness Ownership Boundary

The parity harness owns:
- Replay-list filtering by `mission_name.startsWith(gameType)`.
- `startsWith('sgs')` exclusion.
- `sm` date cutoff after `2023-01-01` by month.
- Legacy skip classification for `empty_replay` and `mace_min_players`.
- Config application for `excludeReplays.json`, `includeReplays.json`, `excludePlayers.json`, and `nameChanges.csv`.
- Compatibility layer identity transforms needed to compare old aggregate output with new derived output.

The parser core owns:
- Deterministic reading of OCAP JSON replay files.
- Source references and parse diagnostics.
- Normalized events and observed identity fields.
- Contract-stable serialization that `server-2` can persist and recalculate from later.

`server-2` owns canonical identity, persistence, recalculation jobs, moderation/correction workflows, and public API shape. `web` owns UI-visible rendering through `server-2` APIs. Phase 1 creates notes only and does not change either adjacent app.

## Ordinary Output Surfaces

Legacy ordinary outputs are produced by `/home/afgan0r/Projects/SolidGames/replays-parser/src/4 - output`.

Source references:
- `src/4 - output/consts.ts` defines `global_statistics.json`, `squad_statistics.json`, `squad_full_rotation_statistics.json`, `rotations_info.json`, `all_time`, `weapons_statistics`, `weeks_statistics`, and `other_players_statistics`.
- `src/4 - output/archiveFiles.ts` writes `stats.zip` under the temp results root before publishing results.
- `src/4 - output/index.ts` writes per-game-type folders and moves `temp_results` to `results`.
- `src/4 - output/json.ts` writes global, squad, optional full-rotation squad, per-player weapon, per-player week, and per-player other-player JSON files.
- `src/4 - output/rotationsJSON.ts` writes `rotation_N` folders for rotation-specific output.

Observed ordinary result surfaces in `~/sg_stats/results` include:
- Root `stats.zip`.
- `mace/global_statistics.json`, `mace/squad_statistics.json`, and per-player `mace/weapons_statistics`, `mace/weeks_statistics`, and `mace/other_players_statistics`.
- `sm/global_statistics.json`, `sm/squad_statistics.json`, and per-player `sm/weapons_statistics`, `sm/weeks_statistics`, and `sm/other_players_statistics`.
- `sg/rotations_info.json`.
- `sg/all_time/global_statistics.json`, `sg/all_time/squad_statistics.json`, and all-time per-player folders when generated.
- `sg/rotation_1` through `sg/rotation_20` folders with `global_statistics.json`, `squad_statistics.json`, and `squad_full_rotation_statistics.json`.

No standalone `statistics.json` file was found in the legacy source or current results. For Phase 5, treat `statistics.json` as a generic shorthand only if a future harness names an output class; the concrete legacy filenames above are the comparable v1 ordinary surfaces.

## Ordinary Comparable Fields

Ordinary v1 aggregate comparisons should focus on fields produced by the global, weekly, squad, and rotation statistics code:

- Player and squad counters: `kills`, `killsFromVehicle`, `vehicleKills`, `teamkills`, `deaths`, `score`, `totalScore`, and `totalPlayedGames`.
- Ratios and coefficients: `kdRatio` and `killsFromVehicleCoef`.
- Weekly fields: `week`, `startDate`, and `endDate`.
- Rotation fields: `startDate` and `endDate` in `rotations_info.json` and rotation-specific stats.
- Per-player side outputs: weapon stats under `weapons_statistics`, week stats under `weeks_statistics`, and other-player references under `other_players_statistics`.

Detailed Phase 5 comparison should record whether each diff affects only parser artifact evidence, `server-2` persistence/recalculation, or UI-visible public stats.

## Annual Nomination Reference Only

Legacy annual nomination code lives under `/home/afgan0r/Projects/SolidGames/replays-parser/src/!yearStatistics`.

Observed yearly outputs live under `~/sg_stats/year_results` and currently include 14 `nomination.txt` files in numbered folders.

Annual/yearly nomination outputs are historical references only for v1 and must not be folded into ordinary player, squad, rotation, weekly, bounty, or parser contract support in Phase 1.

This is the D-13/FUT-06 boundary: yearly nominations remain listed for historical context and future v2 planning, but Phase 1 does not implement yearly nomination product support, `server-2` API support, or `web` pages.

## Phase 2 And Phase 5 Handoff

Phase 2 contract planning should use this document to keep parser-core output focused on observed replay data:
- Preserve observed identity rather than canonicalizing players.
- Preserve enough source references to explain skipped, malformed, or failed parse outcomes.
- Keep game-type filters outside the parser contract.
- Avoid importing yearly nomination requirements into the parser schema.

Phase 5 comparison planning should use this document to build the parity compatibility layer:
- Apply legacy replay selection and skip behavior before comparing derived ordinary aggregates.
- Apply name-change and same-name compatibility identity only inside the compatibility layer.
- Compare concrete ordinary output surfaces: `global_statistics.json`, `squad_statistics.json`, `squad_full_rotation_statistics.json`, `rotations_info.json`, per-player folders, rotation folders, and `stats.zip` contents.
- Classify suspected legacy bugs through the human-review gate before preserving or fixing behavior.
