---
phase: 03-deterministic-parser-core
status: complete
mapped: 2026-04-26
---

# Phase 03: Deterministic Parser Core - Pattern Map

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `Cargo.toml` | workspace config | build | Phase 2 workspace manifest | exact |
| `crates/parser-contract/src/identity.rs` | entity contract extension | serialize | Existing `ObservedEntity`, `SourceRefs`, `FieldPresence` patterns | exact |
| `crates/parser-contract/tests/metadata_identity_contract.rs` | contract regression tests | test | Existing metadata/identity contract tests | exact |
| `crates/parser-contract/tests/schema_contract.rs` | schema regression tests | test | Existing schema gap-regression tests | exact |
| `crates/parser-contract/examples/parse_artifact_success.v1.json` | committed example | serialize | Existing success artifact example | exact |
| `schemas/parse-artifact-v1.schema.json` | generated schema | schema | `parse_artifact_schema()` export pattern | exact |
| `crates/parser-core/Cargo.toml` | crate config | build | `crates/parser-contract/Cargo.toml` | exact |
| `crates/parser-core/src/lib.rs` | public parser API | library | `parser-contract/src/lib.rs` module docs style | role-match |
| `crates/parser-core/src/input.rs` | parse input/options | adapter boundary | Phase 3 D-01/D-02 context decisions | exact |
| `crates/parser-core/src/artifact.rs` | artifact builder/failure shell | normalize | `ParseArtifact` constructors in contract tests | role-match |
| `crates/parser-core/src/raw.rs` | tolerant OCAP decode | parse | legacy `ReplayInfo` type shape and corpus manifest | exact |
| `crates/parser-core/src/metadata.rs` | metadata normalization | normalize | `parser-contract/src/metadata.rs` plus top-level OCAP fields | exact |
| `crates/parser-core/src/entities.rs` | entity normalization | normalize | legacy `getEntities.ts` | exact |
| `crates/parser-core/src/diagnostics.rs` | capped diagnostics/status policy | normalize | `parser-contract/src/diagnostic.rs` | exact |
| `crates/parser-core/tests/*.rs` | behavior tests | test | Existing integration tests under `parser-contract/tests` | exact |
| `crates/parser-core/tests/fixtures/*.ocap.json` | focused OCAP fixtures | test data | Phase 1 fixture index and old parser source references | role-match |

## Established Patterns to Reuse

### Workspace and crate manifest

The root workspace currently lists one member:

```toml
members = ["crates/parser-contract"]
```

Phase 3 should update this to:

```toml
members = ["crates/parser-contract", "crates/parser-core"]
```

`crates/parser-core/Cargo.toml` should inherit workspace edition, rust version, license, repository, readme, keywords, categories, and lints just like `parser-contract`.

### Public API docs and strict lints

The workspace denies missing docs, unsafe code, `unwrap`, `expect`, stdout/stderr macros, and warnings. New public parser-core APIs need doc comments and test helpers should use explicit `expect` messages only inside test files where the current clippy policy allows them.

### Contract optional facts

Use `FieldPresence<T>` for optional or inferred values. Do not use bare `Option<T>` for replay facts that can be absent, null, unknown, inferred, or not applicable in the artifact.

### Source refs

Use `SourceRefs` where auditability is mandatory. For entity source refs, Phase 3 should move from `Vec<SourceRef>` to `SourceRefs`. The JSON shape remains an array, but Rust/schema validation prevents empty arrays.

### Deterministic maps

Use `BTreeMap` for any serialized dynamic object. The existing `ParseArtifact.extensions` and `AggregateSection.projections` establish this pattern.

### Contract schema export

After changing `parser-contract`, regenerate the committed schema with:

```bash
cargo run -p parser-contract --example export_schema > schemas/parse-artifact-v1.schema.json
```

Then compare or test with:

```bash
cargo test -p parser-contract schema_contract
```

### Legacy entity extraction

Legacy `getEntities.ts` behavior to preserve in spirit:

```ts
if (entity.type === 'unit' && entity.isPlayer && entity.description.length && entity.name) {
  players[id] = { ...defaultPlayerInfo, id, name, side };
}

if (entity.type === 'vehicle') {
  vehicles[id] = { id, name, class: vehicleClass, positions };
}

if (eventType === 'connected') {
  const [, , name, id] = event;
  const entityInfo = entities.find((entity) => entity.id === id);
  if (isUndefined(entityInfo) || entityInfo.type === 'vehicle' || !name) return;
  players[id] = { ...defaultPlayerInfo, id, name, side: entityInfo.side };
}
```

The Rust implementation should expose observed facts and provenance, not old aggregate counters.

### Duplicate-slot compatibility

Legacy `combineSamePlayersInfo.ts` merges players by equal name during aggregate projection. Phase 3 should not reproduce that merge in normalized entities. It should emit compatibility hints that preserve the candidate entity IDs and source refs for Phase 4/5.

## Suggested Parser-Core Module Layout

```text
crates/parser-core/src/lib.rs
crates/parser-core/src/input.rs
crates/parser-core/src/artifact.rs
crates/parser-core/src/raw.rs
crates/parser-core/src/metadata.rs
crates/parser-core/src/entities.rs
crates/parser-core/src/diagnostics.rs
```

Keep module ownership narrow:

- `input.rs`: public `ParserInput`, `ParserOptions`, `ParseArtifactMetadata`.
- `artifact.rs`: artifact shell, failure artifact construction, source-ref helpers.
- `raw.rs`: `serde_json` root decode, top-level field lookup, raw entity/event shape helpers.
- `metadata.rs`: `ReplayMetadata` population only.
- `entities.rs`: observed entity facts, connected-player backfill, same-name hint detection.
- `diagnostics.rs`: diagnostic cap, status escalation, stable diagnostic codes.

## Shared Verification Commands

```bash
cargo test -p parser-contract metadata_identity_contract schema_contract
cargo test -p parser-core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
git diff --check
```

## Pattern Mapping Complete

Phase 3 plans should use these assignments, keep parser-core transport-free, and avoid pulling Phase 4 combat/event aggregate semantics into the parser foundation.
