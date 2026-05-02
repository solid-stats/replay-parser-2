# Quick Task 260502-gn2: Phase 5.2 Full Benchmark and Old-vs-New Stats Diff - Research

**Researched:** 2026-05-02
**Domain:** Phase 5.2 benchmark acceptance and selected replay parity analysis
**Confidence:** HIGH for local benchmark/comparison gate behavior; MEDIUM for final pass/fail until the full run completes.

## Findings

### Existing Benchmark Workflow

`scripts/benchmark-phase5.sh --ci` is the canonical Phase 5.2 benchmark workflow. It always:

- builds `target/release/replay-parser-2`;
- selects the largest raw replay under `~/sg_stats/raw_replays`;
- parses that replay with the release CLI;
- optionally runs an old-parser selected baseline when metadata and old parser prerequisites are available;
- optionally runs all raw replay files when `RUN_PHASE5_FULL_CORPUS=1`;
- optionally runs the old full baseline when `RUN_PHASE5_FULL_OLD_BASELINE=1`;
- writes `.planning/generated/phase-05/benchmarks/benchmark-report.json`;
- validates the report in acceptance mode.

The script can exit non-zero after writing valid evidence when acceptance gates fail. That non-zero exit is useful evidence, not necessarily a tooling failure.

### Benchmark Gates

Selected replay acceptance requires:

- `speedup >= 3.0`;
- `parity_status = passed`;
- `artifact_bytes <= 100000`.

All-raw corpus acceptance requires:

- `speedup >= 10.0`;
- `median_artifact_raw_ratio <= 0.05`;
- `p95_artifact_raw_ratio <= 0.10`;
- `max_artifact_bytes <= 100000`;
- `oversized_artifact_count = 0`;
- `failed_count = 0` and `skipped_count = 0`, unless a user-approved allowlist exists.

### Comparison Surfaces

The selected comparison harness derives old-vs-new parity over:

- `status`;
- `replay`;
- `legacy.player_game_results`;
- `legacy.relationships`;
- `bounty.inputs`.

The stats-impacting surfaces are `legacy.player_game_results`, `legacy.relationships`, and `bounty.inputs`. Formatting-only artifact differences are less important than differences in counters, identity grouping, enemy/teamkill classification, deaths, vehicle counters, score formulas, and bounty candidates.

### Cross-App Boundary

This task is local validation/reporting. It may update generated benchmark evidence, tracked selected artifact evidence, quick-task docs, and `STATE.md`. It must not change parser contract behavior, server persistence, canonical identity, replay fetching, public APIs, or UI behavior.

## Pitfalls

- The full old baseline may take substantial wall time and may fail if legacy parser prerequisites are missing.
- The all-raw Rust run writes many ignored generated artifacts under `.planning/generated/phase-05/benchmarks/all-raw-artifacts/`.
- The benchmark report can be structurally valid but fail acceptance. The summary must distinguish "workflow ran" from "x3/x10 passed".
- Selected parity can remain `human_review` even when speed passes; this blocks selected x3 acceptance by current rules.

## Verification Strategy

1. Run the full benchmark command with both full-corpus and old-baseline flags.
2. Run structural report validation if acceptance validation fails, to confirm evidence shape.
3. Read `benchmark-report.json`, `all-raw-summary.json`, failures/oversized files, and selected comparison report.
4. Generate an additional focused stats-diff summary from old/new selected artifacts so the user can see what differs in calculation-relevant data.
5. Record final evidence in `SUMMARY.md` and `VERIFICATION.md`.
