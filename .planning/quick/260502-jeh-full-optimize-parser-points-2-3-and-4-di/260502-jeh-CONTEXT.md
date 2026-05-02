# Quick Task 260502-jeh: Optimize Parser Points 2, 3, and 4 - Context

**Gathered:** 2026-05-02
**Status:** Ready for planning

<domain>
## Task Boundary

Optimize the default parser hot path for the previously identified points:

- Build the default minimal artifact directly instead of routing combat rows through full normalized event construction.
- Scan OCAP `events` once for default-path `connected` and `killed` observations.
- Use a replay-local vehicle/static weapon name index instead of linear vehicle lookup per killed event.

Do not optimize the all-raw benchmark by adding the batch runner from point 1 in this quick task.
</domain>

<decisions>
## Implementation Decisions

### Old All-Raw Baseline
- Reuse the prior old parser all-raw runtime instead of rerunning the old all-raw baseline.
- Baseline fixed from quick task `260502-i8w`: `old_wall_time_ms=501274.528655`, `attempted_count=23473`, `success_count=23469`, `error_count=4`, `skipped_count=0`.
- Save that baseline as committed evidence under `.planning/benchmarks/phase-05-old-all-raw-baseline.json`.

### Parser Hot Path
- Default parser output may bypass debug-only normalized event construction.
- Debug sidecar must keep the existing full normalized `entities`, `events`, source refs, and rule IDs.
- Preserve current v3 default artifact shape and parser ownership boundaries.

### Benchmark Policy
- New all-raw runs compare against the cached old baseline.
- `RUN_PHASE5_FULL_OLD_BASELINE=1` remains an explicit override, but this task must not use it.
</decisions>

<specifics>
## Specific Ideas

- `parse_replay` should still produce the same `players[]`, `weapons[]`, `destroyed_vehicles[]`, and `diagnostics[]` behavior.
- Existing parser-core tests are the oracle for minimal row semantics.
- All-raw acceptance may still fail `x10`; this task is an optimization step, not a Phase 6 unblock claim.
</specifics>

<canonical_refs>
## Canonical References

- `.planning/STATE.md`
- `.planning/PROJECT.md`
- `.planning/quick/260502-i8w-phase-5-2-old-baseline-all-raw-coverage-/260502-i8w-SUMMARY.md`
- `.planning/generated/phase-05/benchmarks/benchmark-report.json`
</canonical_refs>
