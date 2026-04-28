---
status: issues-found
phase: 05-cli-golden-parity-benchmarks-and-coverage-gates
source: [05-00-SUMMARY.md, 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md, 05-04-SUMMARY.md, 05-05-SUMMARY.md]
started: 2026-04-28T09:50:39Z
updated: 2026-04-28T10:15:28Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Kill any running server/service. Clear ephemeral state (temp DBs, caches, lock files). Start the application from scratch. Server boots without errors, any seed/migration completes, and a primary query (health check, homepage load, or basic API call) returns live data.
result: pass

### 2. Local Parse Command
expected: Running `replay-parser-2 parse <input> --output <path>` on a valid compact OCAP fixture writes a pretty JSON `ParseArtifact` to the requested output path, records the local source path and SHA-256 checksum, exits successfully, and stays deterministic across repeated runs.
result: pass

### 3. Structured Parse Failure
expected: Running `replay-parser-2 parse <bad-input> --output <path>` on malformed or unsupported replay JSON writes a structured failed `ParseArtifact`, emits only a concise human stderr summary, and exits non-zero without panicking.
result: pass

### 4. Contract Schema Export
expected: Running `replay-parser-2 schema` prints the current `ParseArtifact` JSON Schema, and running it with an output path writes schema bytes that match the committed contract schema source of truth.
result: pass

### 5. Golden Fixture Regression Coverage
expected: The golden fixture manifest is small, traceable, and executable; parser-core golden tests cover malformed, partial, old-shape, winner present/missing, vehicle-kill, teamkill, commander-side, null-killer, duplicate-slot, and connected-player cases through public `parse_replay` behavior.
result: pass

### 6. Selected Comparison Reports
expected: Running `replay-parser-2 compare` on saved artifacts or a selected replay produces a structured report over status, replay, events, legacy projections, relationships, bounty inputs, and vehicle score inputs, with mismatch categories and parser/server/web impact fields; ambiguous inputs are rejected.
result: pass

### 7. Strict Coverage Gate
expected: Running `scripts/coverage-gate.sh` generates coverage evidence under `.planning/generated/phase-05/coverage/`, validates the exact-line allowlist, and reports zero unallowlisted uncovered production locations for reachable Rust code.
result: pass

### 8. Fault Report Gate
expected: Running `scripts/fault-report-gate.sh` uses `cargo mutants` when available or the deterministic fallback otherwise, writes a validated fault report under `.planning/generated/phase-05/fault-report/`, and blocks high-risk missed cases unless they have accepted non-applicable rationale.
result: pass

### 9. Benchmark Report Gate
expected: Running `scripts/benchmark-phase5.sh --ci` writes a validated benchmark report with workload identity, parser-stage throughput, old baseline profile, parity status, RSS note, and explicit 10x status; when the old parser and `~/sg_stats` are available, it also records a curated old-vs-new comparison result instead of leaving parity unrun.
result: issue

### 10. README and Scope Handoff
expected: The README documents implemented Phase 5 commands and gates using `replay-parser-2 parse`, `schema`, and `compare`, reflects the current Phase 5 verification gap, and keeps RabbitMQ/S3 worker integration, server persistence, canonical identity, replay discovery, UI, and yearly nomination behavior outside Phase 5 scope.
result: pass

## Summary

total: 10
passed: 9
issues: 1
pending: 0
skipped: 0
blocked: 0

## Evidence

- Test 1 accepted by user response: `pass`.
- Tests 2-4 verified with direct `replay-parser-2 parse` and `schema` command runs against compact fixtures and committed schema.
- Test 5 verified with `cargo test -p parser-core golden_fixture_manifest` and `cargo test -p parser-core golden_fixture_behavior`.
- Test 6 verified with `cargo test -p parser-cli compare_command`.
- Test 7 verified with `scripts/coverage-gate.sh`; result included `production_files=25`, `allowlisted_locations=386`, `uncovered_locations=0`.
- Test 8 verified with `scripts/fault-report-gate.sh`; deterministic fallback reported `total_cases=6`, `high_risk_missed=0`.
- Test 9 verified with `scripts/benchmark-phase5.sh --ci`; latest generated report is valid but includes `ten_x_status=Fail`, `parity_status=Some(HumanReview)`, so the benchmark gate exposes a Phase 5 gap instead of passing the 10x target.
- Test 10 verified by README scope grep for implemented commands, Phase 5 gap status, and Phase 6/adjacent-app boundaries.

## Gaps

### Test 9: Curated Old/New Benchmark Fails 10x And Needs Parity Review

- truth: Benchmark evidence must compare old and new parser results on an equivalent selected workload before claiming Phase 5 performance readiness.
- status: failed
- severity: major
- root_cause: The Phase 5 benchmark gate now runs a curated selected old-parser `runParseTask` sample against the Rust release CLI. The generated report validates structurally, but the selected run remains well below the `10x` target and every compared surface is classified `human_review`.
- evidence: `.planning/generated/phase-05/benchmarks/benchmark-report.json`, `.planning/generated/phase-05/comparison/comparison-report.json`, `scripts/benchmark-phase5.sh`.
- missing: Accepted parity decisions for the seven selected comparison surfaces, targeted performance work or an explicitly approved benchmark model/full-corpus benchmark, and a regenerated report that records `ten_x_status=pass` or an accepted gap.
