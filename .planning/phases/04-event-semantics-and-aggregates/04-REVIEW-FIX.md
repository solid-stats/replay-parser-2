---
phase: 04-event-semantics-and-aggregates
review_path: .planning/phases/04-event-semantics-and-aggregates/04-REVIEW.md
fixed_at: 2026-04-28T10:10:19+07:00
fix_scope: critical_warning
findings_in_scope: 11
fixed: 11
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 04 Review Fix Report

All Phase 04 review blockers and warnings were fixed in the parser-contract/parser-core scope. No parser-owned persistence, queue/storage, API, canonical identity, UI, or replay discovery behavior was added.

## Fixed Findings

| Finding | Status | Fix |
|---------|--------|-----|
| BL-01 | fixed | Added shared legacy player eligibility and used it for combat, aggregate projection, and commander candidates. Non-player units no longer feed legacy counters, bounty inputs, vehicle score, or commander candidates. |
| BL-02 | fixed | Legacy per-replay rows are initialized from every eligible player group, so zero-counter participants still emit `totalPlayedGames = 1`. |
| BL-03 | fixed | Vehicle score class mapping now handles real raw class evidence such as `rhs_t72ba_tv`, BTR/BMP/APC, UAZ/HMMWV/offroad, heli, plane, truck, and static weapon patterns while preserving raw class strings. |
| BL-04 | fixed | Same-side vehicle/static destruction from a vehicle now emits vehicle-score penalty inputs with teamkill clamp semantics instead of award inputs. |
| BL-05 | fixed | JSON Schema now constrains `AggregateContributionRef.value` by contribution `kind` for legacy counter, relationship, bounty input, and vehicle score payloads. |
| BL-06 | fixed | Vehicle-score contribution source refs include the event evidence plus attacker vehicle name/class/category, attacker vehicle entity, and target category source refs. |
| BL-07 | fixed | Outcome normalization now detects conflicting recognized `winner`/`winningSide`/`outcome` values, emits data-loss diagnostics, and returns unknown outcome. |
| BL-08 | fixed | Raw killed-event accessors now preserve killed observations with malformed frames or malformed kill-info shapes so combat normalization emits unknown events and diagnostics. |
| WR-01 | fixed | Commander keyword matching now uses token boundaries for `ks`, `commander`, and `командир`, avoiding broad substring matches such as `Maksim` or `Marksman`. |
| WR-02 | fixed | Winner-side parsing now trims and case-folds aliases such as `BLUFOR`, `opfor`, `West`, and padded labels. |
| WR-03 | fixed | README and STATE now describe Phase 4 implementation/review fixes as complete while phase-level GSD verification remains pending. |

## Regression Coverage

- `aggregate_projection` covers non-player unit exclusion and zero-counter eligible player rows.
- `vehicle_score` covers raw Arma class mapping, friendly static/vehicle penalty clamp, denominator exclusion for penalties, and entity source refs on vehicle-score inputs.
- `combat_event_semantics` and `raw_event_accessors` cover malformed killed tuple preservation and emitted diagnostics.
- `side_facts` covers conflicting outcome diagnostics, trimmed/case-insensitive winner aliases, and commander substring false positives.
- `schema_contract` covers kind-specific aggregate contribution payload rejection through the committed schema.

## Verification

Passed:

- `cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v1.schema.json`
- `cargo test -p parser-core`
- `cargo test -p parser-contract schema_contract`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo doc --workspace --no-deps`
- `git diff --check`

## Notes

Two subagents were started after explicit user permission. The raw/side-facts agent disconnected before completion, and the schema agent was shut down after overlapping schema work was completed inline. All final edits were integrated and verified in this workspace.
