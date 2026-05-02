---
phase: quick-260502-ecp
reviewed: 2026-05-02T04:22:08Z
depth: quick
files_reviewed: 21
files_reviewed_list:
  - crates/parser-cli/tests/parse_command.rs
  - crates/parser-cli/tests/schema_command.rs
  - crates/parser-contract/examples/parse_artifact_success.v3.json
  - crates/parser-contract/examples/parse_failure.v3.json
  - crates/parser-contract/src/artifact.rs
  - crates/parser-contract/src/minimal.rs
  - crates/parser-contract/src/presence.rs
  - crates/parser-contract/src/schema.rs
  - crates/parser-contract/src/source_ref.rs
  - crates/parser-contract/tests/artifact_envelope.rs
  - crates/parser-contract/tests/failure_contract.rs
  - crates/parser-contract/tests/metadata_identity_contract.rs
  - crates/parser-contract/tests/schema_contract.rs
  - crates/parser-core/src/aggregates.rs
  - crates/parser-core/src/artifact.rs
  - crates/parser-core/tests/aggregate_projection.rs
  - crates/parser-core/tests/debug_artifact.rs
  - crates/parser-harness/benches/parser_pipeline.rs
  - crates/parser-harness/src/comparison.rs
  - crates/parser-harness/tests/comparison_report.rs
  - schemas/parse-artifact-v3.schema.json
findings:
  critical: 1
  warning: 1
  info: 0
  total: 2
status: issues_found
---

# Phase quick-260502-ecp: Code Review Report

**Reviewed:** 2026-05-02T04:22:08Z
**Depth:** quick
**Files Reviewed:** 21
**Status:** issues_found

## Summary

Reviewed the submitted compact artifact contract/core/consumer changes and ran quick pattern scans plus compile verification. The compact implementation leaves stale parser-core test consumers in the repo, so `cargo test -p parser-core --tests --no-run` does not compile. CLI, contract, and harness test targets compile, but parser-core is currently blocked.

## Critical Issues

### CR-01: BLOCKER - Parser-core Test Targets No Longer Compile

**File:** `crates/parser-core/tests/fault_injection_regressions.rs:66`
**Issue:** The compact contract removed `player_stats`, `player_id`, `bounty_eligible`, `bounty_exclusion_reasons`, and default-row `attacker_vehicle_name`, but existing parser-core tests outside the quick task file list still compile against those fields. `cargo test -p parser-core --tests --no-run` fails with E0609/E0308 errors in `fault_injection_regressions.rs`, `parser_core_api.rs`, `entity_normalization.rs`, `legacy_entity_compatibility.rs`, `combat_event_semantics.rs`, `deterministic_output.rs`, and `golden_fixture_behavior.rs`. This is a CI/build blocker, not just stale assertions.
**Fix:**
```rust
// Examples of the required migration:
assert!(artifact.players.is_empty()); // instead of artifact.player_stats
assert_eq!(teamkill.classification, KillClassification::Teamkill); // no bounty fields in default rows
assert_eq!(vehicle_kill.attacker_vehicle_class.as_deref(), Some("rhs_t72ba_tv"));
assert_eq!(unit.source_entity_id, 10); // derive entity:10 where a string player_id is needed
assert_eq!(player.compatibility_key.as_deref(), Some("legacy_name:SameName"));
```
Update all stale parser-core tests to the compact default contract or move verbose-name/bounty-reason assertions to the debug/normalized event path, then rerun `cargo test -p parser-core --tests --no-run`.

## Warnings

### WR-01: WARNING - Classification Schema Tests Mutate Removed Long Keys

**File:** `crates/parser-contract/tests/schema_contract.rs:342`
**Issue:** The invalid-classification regression tests write `classification` into compact rows, but the schema-backed compact key is `c`. These tests currently pass because `classification` is an extra closed-schema property, not because invalid `c` enum values are rejected. That can mask a regression in the actual compact classification field.
**Fix:**
```rust
success_example["kills"][0]["c"] = json!("friendly_fire");
success_example["destroyed_vehicles"][0]["c"] = json!("neutral");
```
Keep the additional-property rejection covered separately if needed, but test invalid enum values through the real compact key.

---

_Reviewed: 2026-05-02T04:22:08Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: quick_
