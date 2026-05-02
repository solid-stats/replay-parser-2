# Quick Task 260502-gn2: Phase 5.2 x3/x10 Benchmark and Old-vs-New Stats Diff - Context

**Gathered:** 2026-05-02
**Status:** Ready for planning

<domain>
## Task Boundary

Run the existing Phase 5.2 benchmark workflow in its full local mode, check the selected large replay x3 gate and all-raw corpus x10 gate, then compare the selected old-vs-new parsing results.

The comparison should prioritize differences that can affect future statistics calculation: legacy player game rows, kill/teamkill/death counters, vehicle counters, relationship rows, and bounty input rows. Parser-local benchmark evidence must not introduce canonical identity matching, PostgreSQL persistence, public API/UI behavior, replay discovery/fetching, or worker message changes.

</domain>

<decisions>
## Implementation Decisions

### Benchmark Scope

- Use the existing `scripts/benchmark-phase5.sh --ci` acceptance workflow.
- Enable `RUN_PHASE5_FULL_CORPUS=1` to attempt every raw replay under `~/sg_stats/raw_replays`.
- Enable `RUN_PHASE5_FULL_OLD_BASELINE=1` so the report can evaluate all-raw x10 when the old parser baseline succeeds.
- Keep generated all-raw artifacts under `.planning/generated/phase-05/benchmarks/all-raw-artifacts/`; these remain ignored generated evidence unless a tracked evidence file is already part of the repository.

### Comparison Scope

- Use the benchmark-generated selected old artifact, selected new artifact, and comparison report.
- Treat `status`, `replay`, `legacy.player_game_results`, `legacy.relationships`, and `bounty.inputs` as the relevant surfaces.
- Explain data differences by their impact on future `server-2` calculations, not by artifact formatting alone.

### Acceptance Interpretation

- `x3` passes only when selected speedup is at least `3.0`, selected parity is `passed`, and selected artifact size is at most `100000` bytes.
- `x10` passes only when all-raw speedup is at least `10.0`, the all-raw size gate passes, and zero-failure status passes.
- If any benchmark gate fails or remains unknown, record concrete evidence and do not claim Phase 6 readiness.

</decisions>

<specifics>
## Relevant Existing Evidence

- Previous quick task `260502-ecp` reduced the selected large default artifact to `40042` bytes, but the broader benchmark report was stale and still recorded unknown selected x3/parity and all-raw gates.
- Current selected replay policy is "largest .json by byte size under `~/sg_stats/raw_replays`; tie-break lexicographic path".
- Current selected replay path in existing evidence is `/home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json`.

</specifics>

<canonical_refs>
## Canonical References

- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/research/SUMMARY.md`
- `.planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-SUMMARY.md`
- `scripts/benchmark-phase5.sh`
- `crates/parser-harness/src/benchmark_report.rs`
- `crates/parser-harness/src/comparison.rs`

</canonical_refs>
