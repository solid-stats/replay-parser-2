---
phase: quick-260502-ecp
fixed_at: 2026-05-02T04:36:52Z
review_path: .planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase quick-260502-ecp: Code Review Fix Report

**Fixed at:** 2026-05-02T04:36:52Z
**Source review:** .planning/quick/260502-ecp-compact-default-parser-artifact-below-10/260502-ecp-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### CR-01: BLOCKER - Parser-core Test Targets No Longer Compile

**Files modified:** `crates/parser-core/tests/parser_core_api.rs`, `crates/parser-core/tests/entity_normalization.rs`, `crates/parser-core/tests/fault_injection_regressions.rs`, `crates/parser-core/tests/deterministic_output.rs`, `crates/parser-core/tests/combat_event_semantics.rs`, `crates/parser-core/tests/legacy_entity_compatibility.rs`, `crates/parser-core/tests/golden_fixture_behavior.rs`
**Commit:** d464e37
**Applied fix:** Updated stale parser-core test assertions to the compact default artifact contract: merged player rows, compact kill classifications, `compatibility_key` as an optional value, debug-only bounty/verbose vehicle evidence, and `weapons` instead of removed `player_stats`.

### WR-01: WARNING - Classification Schema Tests Mutate Removed Long Keys

**Files modified:** `crates/parser-contract/tests/schema_contract.rs`
**Commit:** 186a3d9
**Applied fix:** Changed invalid-classification regression tests to mutate the schema-backed compact `c` key for `kills[]` and `destroyed_vehicles[]`.

---

_Fixed: 2026-05-02T04:36:52Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
