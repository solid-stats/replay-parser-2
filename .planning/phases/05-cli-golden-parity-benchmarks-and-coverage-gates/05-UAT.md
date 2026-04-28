---
status: complete
phase: 05-cli-golden-parity-benchmarks-and-coverage-gates
source: [05-00-SUMMARY.md, 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md, 05-04-SUMMARY.md, 05-05-SUMMARY.md]
started: 2026-04-28T09:50:39Z
updated: 2026-04-28T09:55:29Z
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
expected: Running `scripts/benchmark-phase5.sh --ci` writes a validated benchmark report with workload identity, parser-stage throughput, old baseline profile, parity status, RSS note, and explicit 10x status; portable CI reports `ten_x_status=unknown` with triage instead of claiming a pass.
result: pass

### 10. README and Scope Handoff
expected: The README documents implemented Phase 5 commands and gates using `replay-parser-2 parse`, `schema`, and `compare`, marks Phase 5 ready for verification, and keeps RabbitMQ/S3 worker integration, server persistence, canonical identity, replay discovery, UI, and yearly nomination behavior outside Phase 5 scope.
result: pass

## Summary

total: 10
passed: 10
issues: 0
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
- Test 9 verified with `scripts/benchmark-phase5.sh --ci`; result included `benchmark_report_valid=true`, `ten_x_status=Unknown`, `parity_status=Some(NotRun)`.
- Test 10 verified by README scope grep for implemented commands, Phase 5 status, and Phase 6/adjacent-app boundaries.

## Gaps

[none yet]
