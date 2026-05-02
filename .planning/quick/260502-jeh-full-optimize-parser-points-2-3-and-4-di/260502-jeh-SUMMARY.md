---
status: complete
quick_id: 260502-jeh
date: 2026-05-02
code_commit: 3176abb
---

# Quick Task 260502-jeh Summary

**Date:** 2026-05-02
**Code commit:** `3176abb`
**Status:** Implemented and verified; Phase 6 remains blocked by benchmark acceptance.

## What Changed

- Default `parse_replay` now collects relevant raw events once and derives v3 minimal rows directly from `connected` and `killed` observations.
- Debug parsing still uses the full normalized entity/event path for source refs, rule IDs, and internal provenance.
- Default entity normalization can consume pre-collected connected events, avoiding a second event scan.
- Default killed-event classification now uses a replay-local vehicle/static observed-name index instead of repeated linear lookup.
- The previous old all-raw runtime is committed at `.planning/benchmarks/phase-05-old-all-raw-baseline.json`.
- `scripts/benchmark-phase5.sh` reuses the cached old all-raw baseline unless `RUN_PHASE5_FULL_OLD_BASELINE=1` is explicitly set.

## Benchmark Evidence

The old all-raw parser was not rerun for this task. The benchmark report reused the cached baseline:

- Cached old all-raw wall time: `501274.528655ms`
- New all-raw wall time: `235598.648803ms`
- All-raw speedup: `2.127663x`
- Previous all-raw new wall time from quick `260502-i8w`: `285716.702146ms`
- Wall-time improvement vs previous new run: about `50118ms` faster, about `17.5%` lower wall time
- All-raw attempted/success/failed/skipped: `23473/23469/4/0`

Remaining benchmark blockers:

- Selected x3/parity was not rerun successfully in this benchmark report because the old selected `tsx` baseline hit sandbox `EPERM`; statuses are `unknown` and `not_run`.
- All-raw x10 still fails at `2.127663x`.
- All-raw size gate still fails because p95 artifact/raw ratio is `0.12417910447761193`.
- All-raw zero-failure still fails on the same 4 malformed/non-JSON raw files.
