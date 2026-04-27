---
phase: 04-event-semantics-and-aggregates
status: complete
mapped: 2026-04-27
---

# Phase 04: Event Semantics and Aggregates - Pattern Map

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/parser-contract/src/events.rs` | combat event contract | serialize/audit | Existing `NormalizedEvent`, `EventActorRef`, `SourceRefs`, `RuleId` | exact |
| `crates/parser-contract/src/aggregates.rs` | contribution/projection contract | serialize/audit | Existing `AggregateContributionRef`, `AggregateSection` | exact |
| `crates/parser-contract/src/side_facts.rs` | commander/outcome contract | serialize/audit | `metadata.rs` + `presence.rs` explicit unknown/inferred states | role-match |
| `crates/parser-contract/src/artifact.rs` | artifact envelope | serialize | Current `events`, `aggregates`, `extensions` envelope | exact |
| `crates/parser-contract/src/lib.rs` | module export | library | Current contract module export style | exact |
| `crates/parser-contract/tests/combat_event_contract.rs` | event contract tests | test | `source_ref_contract.rs`, `metadata_identity_contract.rs` | exact |
| `crates/parser-contract/tests/aggregate_contract.rs` | aggregate contract tests | test | `schema_contract.rs`, `artifact_envelope.rs` | exact |
| `crates/parser-contract/tests/replay_side_facts_contract.rs` | side facts contract tests | test | `metadata_identity_contract.rs` | role-match |
| `crates/parser-contract/examples/parse_artifact_success.v1.json` | example artifact | schema | Current committed success example update pattern | exact |
| `schemas/parse-artifact-v1.schema.json` | generated schema | schema | `cargo run -p parser-contract --example export_schema` | exact |
| `crates/parser-core/src/raw.rs` | tolerant event accessors | parse | Existing `connected_events` helper pattern | exact |
| `crates/parser-core/src/events.rs` | combat normalization | normalize | Existing `entities.rs` normalization + diagnostics pattern | role-match |
| `crates/parser-core/src/aggregates.rs` | aggregate derivation | project | Existing `AggregateSection` contract + legacy `getKillsAndDeaths.ts` | role-match |
| `crates/parser-core/src/vehicle_score.rs` | issue #13 scoring evidence | project | Contract contribution refs + project matrix | role-match |
| `crates/parser-core/src/side_facts.rs` | commander/outcome extraction | normalize | Metadata/identity presence-state patterns | role-match |
| `crates/parser-core/src/artifact.rs` | artifact assembly | normalize | Current metadata/entity assembly point | exact |
| `crates/parser-core/src/lib.rs` | module export | library | Current parser-core module export style | exact |
| `crates/parser-core/tests/*.rs` | behavior tests | test | Existing parser-core integration tests | exact |
| `crates/parser-core/tests/fixtures/*.ocap.json` | focused OCAP fixtures | test data | Existing minimal entity/compatibility fixtures | exact |
| `README.md` | human-facing status | docs | Phase 3 README handoff update | exact |

## Existing Patterns to Reuse

### Source Reference Construction

Use `SourceContext::source_ref` and entity/event-specific helper functions to
carry `replay_id`, `source_file`, checksum, frame, event index, entity ID, JSON
path, and rule ID. Source-backed events and aggregate contributions must use
`SourceRefs`, never empty vectors.

### Raw Accessors

`raw.rs` keeps OCAP tuple quirks at the boundary. Add `KilledEventObservation`
beside `ConnectedEventObservation` and preserve:

- original event index
- frame
- killed entity ID
- nullable killer ID state
- weapon string state
- distance
- JSON path `$.events[<index>]`

Malformed killed tuples should not panic. They should either be skipped with a
diagnostic or represented as unknown events where enough coordinates exist.

### Diagnostics and Partial Status

Use `DiagnosticAccumulator` and `DiagnosticImpact`:

- schema drift that drops or unauditably changes event/aggregate evidence:
  `DiagnosticImpact::DataLoss`
- expected missing commander/winner: no diagnostic or non-loss informational
  diagnostic, and status can remain `success`
- unknown vehicle score taxonomy with a source event: diagnostic plus no score
  contribution unless the contribution remains auditable

### Deterministic Output

Follow current stable ordering:

- sort normalized entities by source entity ID
- sort combat events by source event index, then event ID
- sort contributions by contribution ID
- use `BTreeMap` for projection maps and structured dynamic attributes
- keep `produced_at` as `None` in parser-core

### Contract Schema Export

After contract changes:

```bash
cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v1.schema.json
cargo test -p parser-contract schema_contract
```

The committed success example must include at least one combat event,
aggregate contribution, vehicle score input, and replay-side facts object.

## Legacy Oracle Patterns

Legacy `getKillsAndDeaths.ts` behavior to preserve:

```text
null killer + player victim => victim dead, no killer counters
enemy player kill => killer kills + victim death + killed/killers relations
same-side non-suicide => killer teamkills + victim death by teamkill + team relations
suicide => victim death, not teamkill
player killer + vehicle victim => killer vehicleKills += 1
weapon string matching a vehicle name => killsFromVehicle += 1
```

Legacy aggregate formulas to preserve in projections:

```text
kdRatio = round((kills - teamkills) / abs(deaths.total - deaths.byTeamkills), 2)
score = round((kills - teamkills) / (totalPlayedGames - deaths.byTeamkills), 2)
killsFromVehicleCoef = round(killsFromVehicle / kills, 2)
```

Use old formulas in namespaced legacy projections. Do not emit full
multi-replay all-time, weekly, squad, or rotation outputs in Phase 4.

## Suggested Module Layout

```text
crates/parser-contract/src/side_facts.rs
crates/parser-contract/tests/combat_event_contract.rs
crates/parser-contract/tests/aggregate_contract.rs
crates/parser-contract/tests/replay_side_facts_contract.rs

crates/parser-core/src/events.rs
crates/parser-core/src/aggregates.rs
crates/parser-core/src/vehicle_score.rs
crates/parser-core/src/side_facts.rs
crates/parser-core/tests/raw_event_accessors.rs
crates/parser-core/tests/combat_event_semantics.rs
crates/parser-core/tests/aggregate_projection.rs
crates/parser-core/tests/vehicle_score.rs
crates/parser-core/tests/side_facts.rs
```

## Shared Verification Commands

```bash
cargo test -p parser-contract combat_event_contract aggregate_contract replay_side_facts_contract schema_contract
cargo test -p parser-core raw_event_accessors
cargo test -p parser-core combat_event_semantics
cargo test -p parser-core aggregate_projection
cargo test -p parser-core vehicle_score
cargo test -p parser-core side_facts
cargo test -p parser-core deterministic_output
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
git diff --check
```

## Pattern Mapping Complete

Phase 4 plans should keep normalized events primary, derive all aggregate
outputs from event-backed contribution refs, preserve observed identity, and
avoid moving `server-2`, `web`, `replays-fetcher`, CLI, worker, benchmark, or
full parity responsibilities into this phase.
