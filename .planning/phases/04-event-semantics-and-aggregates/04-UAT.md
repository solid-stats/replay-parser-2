---
phase: 04-event-semantics-and-aggregates
verified_at: 2026-04-28T10:29:01+07:00
status: complete
tests_total: 7
tests_passed: 7
tests_failed: 0
tests_skipped: 0
human_needed: false
---

# Phase 04 UAT - Event Semantics and Aggregates

Phase 04 is a parser-core and parser-contract phase, so user acceptance was verified through developer-observable artifact behavior, schema validation, regression tests, and the full workspace quality gate.

## Acceptance Tests

| ID | Scenario | Evidence | Result |
|----|----------|----------|--------|
| UAT-04-01 | Contract consumers can validate typed combat events, aggregate contributions, vehicle-score evidence, and replay-side facts. | `cargo test -p parser-contract`; schema freshness check with `export_schema` and `cmp` | pass |
| UAT-04-02 | Raw killed-event accessors preserve source coordinates and malformed killed observations without panics or silent loss. | `cargo test -p parser-core raw_event_accessors` | pass |
| UAT-04-03 | Combat normalization classifies enemy kills, deaths, teamkills, suicides, null killers, vehicle victims, unknown actors, and malformed tuples with source refs and diagnostics. | `cargo test -p parser-core combat_event_semantics` | pass |
| UAT-04-04 | Aggregate projections derive legacy rows, relationship summaries, game-type/squad/rotation inputs, and bounty inputs only from auditable contribution refs. | `cargo test -p parser-core aggregate_projection` | pass |
| UAT-04-05 | Vehicle score inputs implement issue #13 category mapping, weights, source refs, denominator rows, and teamkill penalty clamp behavior. | `cargo test -p parser-core vehicle_score` | pass |
| UAT-04-06 | Winner/outcome and commander side facts represent known, unknown, conflicting, and candidate states without canonical identity. | `cargo test -p parser-core side_facts` | pass |
| UAT-04-07 | Final populated artifacts remain deterministic, schema-valid, documented, lint-clean, and within parser ownership boundaries. | `cargo test -p parser-core deterministic_output`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo doc --workspace --no-deps`; `git diff --check`; boundary grep | pass |

## Command Evidence

Passed on 2026-04-28:

- `cargo test -p parser-contract`
- `cargo test -p parser-core raw_event_accessors`
- `cargo test -p parser-core combat_event_semantics`
- `cargo test -p parser-core aggregate_projection`
- `cargo test -p parser-core vehicle_score`
- `cargo test -p parser-core side_facts`
- `cargo test -p parser-core deterministic_output`
- `cargo run -p parser-contract --example export_schema > /tmp/phase4-parse-artifact-v1.schema.json`
- `cmp /tmp/phase4-parse-artifact-v1.schema.json schemas/parse-artifact-v1.schema.json`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo doc --workspace --no-deps`
- `git diff --check`
- `rg -n "postgres|sqlx|diesel|lapin|RabbitMQ|aws_sdk_s3|canonical_player|openapi|TanStack|fetch replay|crawl" crates/parser-contract crates/parser-core README.md`

Boundary grep result: implementation scope is clean. Matches were README future-integration documentation and negative tests asserting canonical player fields are absent.

## Result

Phase 04 UAT passed. No open user-facing verification items remain before Phase 5 planning.
