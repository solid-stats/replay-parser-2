---
status: complete
must_haves:
  truths:
    - Default parser output preserves the v3 minimal artifact shape and behavior.
    - Default parser no longer derives minimal tables from full normalized combat events.
    - Default event observation uses one relevant-event scan for connected and killed events.
    - Vehicle/static weapon lookup uses a replay-local name index in the optimized default path.
    - Old all-raw baseline is reused from committed cached evidence, not rerun.
  artifacts:
    - crates/parser-core/src/artifact.rs
    - crates/parser-core/src/aggregates.rs
    - crates/parser-core/src/raw_compact.rs
    - crates/parser-core/src/raw.rs
    - crates/parser-core/src/entities.rs
    - scripts/benchmark-phase5.sh
    - .planning/benchmarks/phase-05-old-all-raw-baseline.json
  key_links:
    - .planning/quick/260502-i8w-phase-5-2-old-baseline-all-raw-coverage-/260502-i8w-SUMMARY.md
---

# Quick Task 260502-jeh Plan

## Task 1: Optimize Default Parser Hot Path

**Files:** `crates/parser-core/src/artifact.rs`, `crates/parser-core/src/aggregates.rs`, `crates/parser-core/src/raw_compact.rs`, `crates/parser-core/src/raw.rs`, `crates/parser-core/src/entities.rs`

**Action:** Add a one-pass relevant-event collector, allow default entity normalization to consume pre-collected connected events, and derive minimal rows directly from killed observations using a vehicle/static name index. Keep `parse_replay_debug` on the full normalized debug path.

**Verify:** `cargo test -p parser-core -- --nocapture`

**Done:** Parser-core behavior tests pass and default rows still omit debug-only event/source fields.

## Task 2: Reuse Cached Old All-Raw Baseline

**Files:** `scripts/benchmark-phase5.sh`, `.planning/benchmarks/phase-05-old-all-raw-baseline.json`

**Action:** Commit the previous old all-raw runtime evidence and update the Phase 5 benchmark script to read it when `RUN_PHASE5_FULL_OLD_BASELINE` is not explicitly `1`.

**Verify:** Run `RUN_PHASE5_FULL_CORPUS=1 scripts/benchmark-phase5.sh --ci` without `RUN_PHASE5_FULL_OLD_BASELINE=1`; confirm report old wall time comes from the cache and no old all-raw command is executed.

**Done:** Benchmark report compares new all-raw runtime against cached `501274.528655ms` old baseline.

## Task 3: Quality Gates And Handoff

**Files:** quick task docs, `.planning/STATE.md`

**Action:** Run formatting, clippy, workspace tests, benchmark report validation, write summary/verification, update state, and commit intended results.

**Verify:**

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p parser-harness --bin benchmark-report-check -- --report .planning/generated/phase-05/benchmarks/benchmark-report.json --mode structural`

**Done:** Verification artifacts record pass/fail evidence and remaining Phase 6 blockers honestly.
