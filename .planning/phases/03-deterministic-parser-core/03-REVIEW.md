---
phase: 03-deterministic-parser-core
reviewed: 2026-04-27T06:17:03Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - crates/parser-contract/src/identity.rs
  - crates/parser-contract/tests/metadata_identity_contract.rs
  - crates/parser-contract/tests/schema_contract.rs
  - crates/parser-contract/examples/parse_artifact_success.v1.json
  - schemas/parse-artifact-v1.schema.json
  - crates/parser-core/src/entities.rs
  - crates/parser-core/src/raw.rs
  - crates/parser-core/tests/entity_normalization.rs
  - crates/parser-core/tests/legacy_entity_compatibility.rs
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-04-27T06:17:03Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Reviewed the current HEAD fix commit `742580c` for the Phase 3 parser-contract and parser-core entity identity changes. The two prior blockers are resolved:

- `ObservedEntity.is_player` is now part of the Rust contract, generated schema, success example, and entity normalization path.
- Connected-player backfill now preserves a non-empty raw `observed_name` while updating aggregate-compatible `identity.nickname` from the last applicable connected event.

The remaining issue is non-blocking but still relevant to PARS-02/PARS-06 robustness: malformed or drifted connected-event data can still be silently ignored before the backfill layer can report partial data loss.

Verification run:

- `cargo test -p parser-core --test entity_normalization --test legacy_entity_compatibility`
- `cargo test -p parser-contract --test schema_contract`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Warnings

### WR-01: WARNING - Connected event schema drift is silently ignored

**File:** `crates/parser-core/src/raw.rs:180`
**Issue:** `connected_events` returns an empty list when the top-level `events` field is absent or not an array, and `filter_map` drops malformed connected tuples without diagnostics. The parser can therefore lose the only evidence needed for PARS-06 connected-player backfill while still returning a successful artifact. This is inconsistent with the Phase 3 drift handling used for metadata, entity fields, and the new `isPlayer` parsing path.
**Fix:**
```rust
pub fn connected_events(
    raw: RawReplay<'_>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Vec<ConnectedEventObservation> {
    let events = match raw.array_field("events") {
        RawField::Present { value, .. } => value,
        RawField::Absent { json_path } => {
            push_connected_event_diagnostic(
                diagnostics,
                context,
                "schema.events_absent",
                &json_path,
                "array",
                "absent",
                "skip_connected_backfill",
            );
            return Vec::new();
        }
        RawField::Drift { json_path, expected_shape, observed_shape } => {
            push_connected_event_diagnostic(
                diagnostics,
                context,
                "schema.events_shape",
                &json_path,
                expected_shape,
                &observed_shape,
                "skip_connected_backfill",
            );
            return Vec::new();
        }
    };

    events
        .iter()
        .enumerate()
        .filter_map(|(event_index, event)| {
            connected_event(event, event_index).or_else(|| {
                push_malformed_connected_tuple_if_applicable(
                    diagnostics,
                    context,
                    event,
                    event_index,
                );
                None
            })
        })
        .collect()
}
```
Add regression tests for non-array `events` and connected-like tuples with invalid frame/name/entity ID so the artifact becomes partial or at least carries a warning diagnostic instead of silently dropping the evidence.

---

_Reviewed: 2026-04-27T06:17:03Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
