---
status: complete
quick_id: 260502-ecp
completed: 2026-05-02
base_commit: c21ce8a0cda1f61ed7a029e1ce0e500fb93465b7
commits:
  - 70a0ebf
  - 710079f
  - 2bf586f
  - 2d7d6f6
  - 4716f51
  - b238642
  - d464e37
  - 186a3d9
  - bbebfcb
selected_large_artifact_bytes: 40042
artifact_size_limit_bytes: 100000
---

# Quick Task 260502-ecp: Compact Default Parser Artifact Summary

## Outcome

Implemented the compact default parser artifact shape and regenerated the selected large artifact at 40,042 bytes, below the 100,000-byte hard limit. Verbose provenance remains available through the debug sidecar path, while default artifacts omit verbose evidence and removed legacy duplication.

Full Phase 5.2 acceptance was not expanded in this quick task. The existing benchmark report still records broader selected/all-raw gates as open or failing until the full Phase 5.2 benchmark workflow is rerun.

## Changes

- Merged non-zero player counters into compact `players[]` rows and removed default `player_stats`.
- Added deterministic compact `weapons[]` dictionary rows.
- Replaced verbose kill and destroyed-vehicle default rows with compact entity/classification refs.
- Omitted null, empty, and zero default success fields from default artifact serialization.
- Preserved debug sidecar provenance and normalized detail coverage.
- Updated parser-core aggregate projection tests and debug artifact tests.
- Updated CLI parse tests for recursive absence of verbose/default-removed fields.
- Updated comparison harness derivation to read compact rows and reconstruct legacy review surfaces deterministically.
- Regenerated `schemas/parse-artifact-v3.schema.json` and contract examples.
- Regenerated `.planning/generated/phase-05/benchmarks/selected-large-artifact.json`.

## Commits

- `70a0ebf` - `test(quick-260502-ecp): add compact contract red tests`
- `710079f` - `feat(quick-260502-ecp): compact parser contract rows`
- `2bf586f` - `test(quick-260502-ecp): add compact parser-core red tests`
- `2d7d6f6` - `feat(quick-260502-ecp): build compact parser-core rows`
- `4716f51` - `test(quick-260502-ecp): add compact consumer red tests`
- `b238642` - `feat(quick-260502-ecp): update consumers for compact artifacts`
- `d464e37` - `fix(260502-ecp): CR-01 update parser-core compact tests`
- `186a3d9` - `fix(260502-ecp): WR-01 test compact classification keys`
- `bbebfcb` - `fix(260502-ecp): satisfy compact artifact clippy gate`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Omitted nested optional nulls**

- **Found during:** Task 3 consumer verification
- **Issue:** After root compact serialization was implemented, nested optional source/reference fields still serialized as `null`, weakening the compact default artifact goal.
- **Fix:** Added `skip_serializing_if = "Option::is_none"` to nested optional source/reference fields and updated metadata identity expectations.
- **Files modified:** `crates/parser-contract/src/presence.rs`, `crates/parser-contract/src/source_ref.rs`, `crates/parser-contract/tests/metadata_identity_contract.rs`
- **Commit:** `b238642`

**2. [Rule 3 - Blocking Issue] Updated direct consumers of removed `player_stats`**

- **Found during:** Task 3 consumer verification
- **Issue:** Schema command tests and the parser pipeline benchmark still referenced the removed `player_stats` table.
- **Fix:** Updated those consumers to assert/use compact `weapons[]` instead.
- **Files modified:** `crates/parser-cli/tests/schema_command.rs`, `crates/parser-harness/benches/parser_pipeline.rs`
- **Commit:** `b238642`

### Code Review Fixes

**1. CR-01: Parser-core test targets no longer compiled**

- **Found during:** Quick code review
- **Issue:** Some parser-core tests still referenced removed default fields such as `player_stats`, `bounty_eligible`, and `attacker_vehicle_name`.
- **Fix:** Updated stale tests to assert compact default rows and debug-only verbose evidence.
- **Commit:** `d464e37`

**2. WR-01: Schema regression tests used removed long key**

- **Found during:** Quick code review
- **Issue:** Two schema regression tests mutated removed `classification` keys, so they were no longer testing the compact schema surface.
- **Fix:** Changed those tests to mutate compact `c` classification keys.
- **Commit:** `186a3d9`

### Final Quality-Gate Fix

**1. Clippy gate cleanup**

- **Found during:** Final orchestrator verification
- **Issue:** `cargo clippy --workspace --all-targets -- -D warnings` rejected a Serde skip predicate, a collapsible `if`, and a test helper `panic!`.
- **Fix:** Added a narrow Clippy expectation for the Serde predicate signature, collapsed the weapon dictionary branch, and rewrote the null-check helper to return a boolean before asserting.
- **Commit:** `bbebfcb`

## Verification

- `cargo test -p parser-contract --test schema_contract` - passed, 19 tests.
- `cargo test -p parser-core --test aggregate_projection` - passed, 8 tests.
- `cargo test -p parser-core --test debug_artifact` - passed, 4 tests.
- `cargo test -p parser-core --tests --no-run` - passed after code review fixes.
- `cargo test -p parser-core --tests` - passed, 100 tests.
- `cargo test -p parser-cli --test parse_command` - passed, 11 tests.
- `cargo test -p parser-harness --test comparison_report` - passed, 14 tests.
- `cargo clippy --workspace --all-targets -- -D warnings` - passed after final quality-gate fix.
- `cargo run -q -p parser-cli --bin replay-parser-2 --release -- parse /home/afgan0r/sg_stats/raw_replays/2021_10_31__00_13_51_ocap.json --output .planning/generated/phase-05/benchmarks/selected-large-artifact.json` - passed.
- `python3 -c 'from pathlib import Path; import json; p=Path(".planning/generated/phase-05/benchmarks/selected-large-artifact.json"); n=p.stat().st_size; data=json.loads(p.read_text()); blob=p.read_text(); print(f"selected_large_artifact_bytes={n}"); assert n <= 100000; assert "player_stats" not in data; assert "bounty_eligible" not in blob; assert "source_refs" not in blob'` - passed, `selected_large_artifact_bytes=40042`.
- `cargo run -q -p parser-harness --bin benchmark-report-check -- --report .planning/generated/phase-05/benchmarks/benchmark-report.json --mode structural` - passed structural validation.

Benchmark report status from structural validation:

- `benchmark_report_valid=true`
- `phase=05.2`
- `artifact_size_limit_bytes=100000`
- `selected_x3_status=Unknown`
- `selected_parity_status=NotRun`
- `selected_artifact_size_status=Fail`
- `all_raw_x10_status=Unknown`
- `all_raw_size_gate_status=Unknown`
- `all_raw_zero_failure_status=Unknown`

## Remaining Phase 6 Blockers

- The selected-large artifact generated by this quick task passes the hard byte proof at 40,042 bytes.
- The committed existing benchmark report still shows broader Phase 5.2 evidence as open/failing because the full benchmark workflow was intentionally not changed or rerun in this quick task.
- Phase 6 should still wait for the existing Phase 5.2 selected x3/parity and all-raw x10/zero-failure/size gates to be regenerated and accepted through their normal workflow.

## Known Stubs

None found in files created or modified for this quick task.

## Threat Flags

None beyond the planned parser-core, parser artifact, CLI filesystem output, and comparison harness trust surfaces.

## Self-Check: PASSED

- Summary file exists at `.planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-SUMMARY.md`.
- All nine task, review-fix, and quality-gate commits resolve in git.
- `git status --short` shows only uncommitted quick-task docs and `STATE.md`, as expected for orchestrator-managed docs.
