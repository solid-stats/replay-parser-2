---
status: complete
quick_id: 260502-gn2
completed: 2026-05-02
base_commit: 009c48223c12cbe4973e01b893abefd40818c51a
selected_x3_status: fail
selected_parity_status: human_review
selected_artifact_size_status: pass
all_raw_x10_status: unknown
all_raw_size_gate_status: fail
all_raw_zero_failure_status: fail
---

# Quick Task 260502-gn2: Phase 5.2 Full Benchmark and Old-vs-New Stats Diff Summary

## Outcome

Fixed the benchmark runner so old selected metadata derives `game_type` from mission name and the old all-raw baseline uses a generated direct `runParseTask` runner instead of the legacy WorkerPool path that was only emitting `ERR_UNKNOWN_FILE_EXTENSION ".ts"`.

Fresh full benchmark evidence is structurally valid, but Phase 5.2 acceptance still fails:

- Selected replay x3: `fail` - old `250.819333 ms`, new `100.343921 ms`, speedup `2.499596692060698`, below `3.0`.
- Selected parity: `human_review` - five selected surfaces differ.
- Selected artifact size: `pass` - `40042` bytes, below `100000`.
- All-raw x10: `unknown` - old baseline did not cover every raw replay file.
- All-raw size gate: `fail` - median ratio `0.02947` passes, p95 ratio `0.121999` exceeds `0.10`; max artifact `50029`, oversized count `0`.
- All-raw zero-failure: `fail` - new parser attempted `23473`, succeeded `23469`, failed `4`, skipped `0`.

Phase 6 remains blocked.

## Benchmark Evidence

Fresh report: `.planning/generated/phase-05/benchmarks/benchmark-report.json`

Selected large replay:

- Path: `/home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json`
- Raw bytes: `19706937`
- Artifact bytes: `40042`
- New artifact status: `partial`
- New artifact rows: `245` players, `33` weapons, `224` kills, `22` destroyed vehicles, `10` diagnostics.
- New kill classifications: `193` enemy kills, `18` teamkills, `4` suicides, `9` unknown.

All-raw new parser:

- Attempted: `23473`
- Success: `23469`
- Failed: `4`
- Skipped: `0`
- New wall time: `238172.821205 ms`
- Median artifact/raw ratio: `0.02947196770570225`
- p95 artifact/raw ratio: `0.12199904605437491`
- Max artifact bytes: `50029`
- Oversized artifacts: `0`

Old direct all-raw baseline:

- Attempted: `22996`
- Success: `18414`
- Error: `3`
- Skipped: `4579`
- Wall time: `320065.322367 ms`
- Coverage mismatch: old baseline uses replay-list/game-type filtering and cannot cover all `23473` raw files as required by the all-raw x10 gate.

## Stats-Impacting Selected Diff

Calculation-relevant totals match on the selected replay:

| Metric | Old | New |
|---|---:|---:|
| kills | 193 | 193 |
| killsFromVehicle | 45 | 45 |
| vehicleKills | 22 | 22 |
| teamkills | 18 | 18 |
| dead players | 215 | 215 |
| dead by teamkill | 18 | 18 |

Differences that matter for future statistics:

1. `status`: old is `success`; new is `partial` because it emits 10 warnings for unknown/non-player killed-event actor cases. The counted player stats still match, but server ingestion policy must decide whether `partial` is acceptable for stats publication.
2. `legacy.player_game_results`: old has `244` rows, new has `245`. The only row-level mismatch is `[WD]German`: old merged same-name slots into one row with `teamkills=2`; new keeps two observed entity rows with the same `legacy_name:[WD]German` compatibility key, one zero row and one `teamkills=2` row. If `server-2` groups by compatibility key, totals stay stable; if it counts rows as games directly, this can overcount participation.
3. `legacy.relationships`: bucket totals match (`193` killed, `193` killers, `18` teamkilled, `18` teamkillers), but 406 relationship entries differ by identity text. Old target names are legacy-normalized names like `FJey`; new uses observed names with clan tags like `[31st]FJey`. This affects relationship pages and grouping unless `server-2` canonicalizes/normalizes relationship participants.
4. `bounty.inputs`: old benchmark side is always empty; new derives `193` enemy-kill bounty input rows. This is intentionally useful for future bounty calculation, but current parity cannot pass until old-side bounty absence is classified as intentional or the comparison harness stops treating it as old-vs-new mismatch.

## Fixes Made

- `scripts/benchmark-phase5.sh` now infers selected `game_type` from mission name prefixes `sg`, `mace`, and `sm` when replay-list metadata lacks a game type.
- `scripts/benchmark-phase5.sh` now runs old all-raw evidence through generated `run-old-all-raw.ts`, which calls old `runParseTask` directly and records attempted/success/error/skipped counts.
- The old all-raw report now fails coverage honestly instead of waiting on or accepting the old WorkerPool `.ts` loading error path.

## Verification

- `bash -n scripts/benchmark-phase5.sh` - passed.
- `env RUN_PHASE5_FULL_CORPUS=0 RUN_PHASE5_FULL_OLD_BASELINE=0 scripts/benchmark-phase5.sh --ci` - generated selected old baseline and comparison report; acceptance failed as expected because selected x3/parity and all-raw gates do not pass.
- `env RUN_PHASE5_FULL_CORPUS=1 RUN_PHASE5_FULL_OLD_BASELINE=1 scripts/benchmark-phase5.sh --ci` - generated full evidence; acceptance failed as expected with selected x3 fail, selected parity human review, all-raw x10 unknown, all-raw size fail, all-raw zero-failure fail.
- `cargo run -q -p parser-harness --bin benchmark-report-check -- --report .planning/generated/phase-05/benchmarks/benchmark-report.json --mode structural` - passed.
- `cargo test -p parser-harness benchmark_report` - passed, 15 tests.
- `git diff --check` - passed.

## Remaining Blockers

- Selected x3 is below target: `2.499596692060698x`, not `>= 3x`.
- Selected parity remains `human_review`.
- All-raw x10 remains `unknown` because old direct baseline covers `22996` replay-list rows, not every `23473` raw file, and includes `4579` legacy skips.
- All-raw size gate fails on p95 ratio `0.121999 > 0.10`.
- All-raw zero-failure gate fails due 4 malformed/non-JSON raw files.
