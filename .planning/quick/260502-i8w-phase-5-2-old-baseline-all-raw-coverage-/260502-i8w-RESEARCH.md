# Quick Task 260502-i8w Research

## Legacy Parser Behavior

Old parser reference:

- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/index.ts`
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/combineSamePlayersInfo.ts`
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/0 - utils/getPlayerName.ts`
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/1 - replays/workers/parseReplayWorker.ts`

Findings:

- `combineSamePlayersInfo` merges player rows by exact `player.name` and keeps the later entity id while summing counters and merging relationship arrays.
- `getPlayerName` splits the first bracket tag as `prefix` and removes all bracket groups from the visible nickname.
- The old worker skips empty results and small MACE games. That is correct production-worker behavior but wrong for the Phase 5.2 all-raw benchmark, because it prevents old/new attempted-count parity.
- Direct `parseReplayInfo` can be run over every raw file and count JSON decode/parser errors without applying worker skip filters.

## Benchmark Findings

The old all-raw runner must not parse raw JSON while constructing its task list. The corpus includes malformed/non-JSON files such as `.json` and HTML payloads. Those should count as per-file `error_count`, not abort the whole old baseline run.

After the fix, old and new both attempted 23473 raw files.

## Selected Replay Comparison

Selected replay: `/home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json`

Stats-relevant totals match between old and new:

- players: 244 vs 244
- kills: 193 vs 193
- teamkills: 18 vs 18
- killsFromVehicle: 45 vs 45
- vehicleKills: 22 vs 22
- deaths/dead players: 215 vs 215
- enemy relationship pairs: 193 vs 193
- teamkill relationship pairs: 18 vs 18

Non-blocking for current stat math but visible in comparison:

- old status is `success`, new status is `partial` because the new parser emits 10 `event.killed_actor_unknown` diagnostics for non-player/unknown killed events;
- old replay metadata is a small benchmark wrapper, new replay metadata is the v3 parse artifact metadata object;
- comparison remains `human_review` because surfaces are not byte-identical even though selected stats totals and relationship pair counts match.
