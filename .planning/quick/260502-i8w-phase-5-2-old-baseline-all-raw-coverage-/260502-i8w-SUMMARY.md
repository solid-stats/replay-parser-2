# Quick Task 260502-i8w Summary

## Implemented

- Removed `.planning/generated/phase-05` tracked placeholders and added benchmark-run cleanup for generated benchmark/comparison/coverage/fault-report outputs.
- Changed the v3 minimal artifact shape:
  - top-level `kills[]` removed;
  - player-authored enemy/team kill rows moved to `players[].kills`;
  - same-name slot source ids stored in `players[].eids`;
  - legacy bracket tags split into `players[].tag` with nickname in `players[].n`.
- Updated parser-core to merge same-name slots like the old parser and to remap kill victim ids through merged representative player ids.
- Updated harness comparison to read both legacy top-level `kills[]` and new nested `players[].kills`.
- Updated old selected comparison relationships to expose split `source_observed_name/source_observed_tag` and `target_observed_name/target_observed_tag`.
- Replaced the old all-raw benchmark runner with direct legacy `parseReplayInfo` over every raw file, without worker skip filters.
- Updated schema, examples, tests, README, ROADMAP, REQUIREMENTS, and STATE.

## Benchmark Result

Full benchmark command:

```bash
RUN_PHASE5_FULL_CORPUS=1 RUN_PHASE5_FULL_OLD_BASELINE=1 scripts/benchmark-phase5.sh --ci
```

The command generated a valid report but exited non-zero because acceptance gates failed, as expected from the measured values.

Selected replay:

- selected path: `/home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json`
- artifact bytes: 40780, size pass
- old wall time: 260.071354 ms
- new wall time: 92.255598 ms
- speedup: 2.8190x
- x3 status: fail
- parity status: human_review

All-raw corpus:

- attempted: old 23473, new 23473
- success: old 23469, new 23469
- errors: old 4, new 4
- skipped: old 0, new 0
- old wall time: 501274.528655 ms
- new wall time: 285716.702146 ms
- speedup: 1.7544x
- x10 status: fail
- median artifact/raw ratio: 0.02999546896239239
- p95 artifact/raw ratio: 0.12417910447761193, size gate fail
- max artifact bytes: 48313
- oversized artifacts: 0
- zero-failure status: fail because 4 raw files are malformed/non-JSON.

## Selected Replay Stats Diff

Stats-relevant selected replay values match old parser behavior:

- players: 244 vs 244
- kills: 193 vs 193
- teamkills: 18 vs 18
- killsFromVehicle: 45 vs 45
- vehicleKills: 22 vs 22
- deaths/dead players: 215 vs 215
- enemy relationship pairs: 193 vs 193
- teamkill relationship pairs: 18 vs 18

The comparison remains `human_review` because the artifact surfaces differ structurally: old selected artifact is a legacy wrapper, new selected artifact is the v3 parse contract with diagnostics and richer replay metadata.
