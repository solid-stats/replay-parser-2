# Phase 02: Versioned Output Contract - Pattern Map

**Mapped:** 2026-04-26
**Files analyzed:** planning docs, Phase 1 dossiers, legacy parser TypeScript source, repo root docs
**Analogs found:** 12 / 12

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `Cargo.toml` | workspace config | build | `.planning/research/SUMMARY.md` stack recommendation | role-match |
| `rust-toolchain.toml` | toolchain config | build | `.planning/research/SUMMARY.md` Rust version recommendation | role-match |
| `crates/parser-contract/Cargo.toml` | crate config | build | `.planning/research/SUMMARY.md` contract crate recommendation | role-match |
| `crates/parser-contract/src/lib.rs` | public API root | library | Phase 1 dossier module-style handoff docs | role-match |
| `crates/parser-contract/src/version.rs` | contract version types | serialize | Phase 2 D-02 | exact |
| `crates/parser-contract/src/artifact.rs` | artifact envelope | serialize | Phase 2 D-01, D-03, D-14 | exact |
| `crates/parser-contract/src/presence.rs` | explicit unknown/null states | serialize | Phase 2 D-05 through D-08 | exact |
| `crates/parser-contract/src/metadata.rs` | replay metadata contract | serialize | legacy `ReplayInfo` type and corpus profile | exact |
| `crates/parser-contract/src/identity.rs` | observed identity contract | serialize | legacy `getEntities.ts` and D-11 from Phase 1 | exact |
| `crates/parser-contract/src/source_ref.rs` | source coordinates/rule IDs | audit | Phase 2 D-09 through D-13 | exact |
| `crates/parser-contract/src/events.rs` and `aggregates.rs` | event/contribution skeleton | audit | legacy `getKillsAndDeaths.ts`, Phase 4 handoff | role-match |
| `crates/parser-contract/src/failure.rs`, `diagnostic.rs`, `schema.rs` | failure/schema/diagnostics | validation | legacy worker response types, Phase 2 D-16 through D-19 | exact |

## Pattern Assignments

### Workspace and crate setup

Use the project research architecture: a Rust 2024 workspace with a pure parser core later and a first `parser-contract` crate now. Phase 2 should not create CLI, worker, or parser-core behavior yet.

Apply:

- Workspace root `Cargo.toml` with `members = ["crates/parser-contract"]`.
- `rust-toolchain.toml` pin matching project research.
- `crates/parser-contract` as the only crate in this phase.

### Legacy metadata source

Legacy `ReplayInfo` defines the first metadata facts: `missionName`, `worldName`, `missionAuthor`, `playersCount`, `captureDelay`, `endFrame`, `entities`, `events`, `Markers`, and usually `EditorMarkers`.

Apply:

- Use `snake_case` output names: `mission_name`, `world_name`, `mission_author`, `players_count`, `capture_delay`, `end_frame`.
- Keep time/frame boundaries explicit and nullable through `FieldPresence<T>`.
- Do not parse raw OCAP in Phase 2.

### Observed identity boundary

Legacy `getEntities.ts` creates players from unit entities and connected events. Legacy `combineSamePlayersInfo.ts` merges same-name slots for aggregate compatibility.

Apply:

- Preserve observed entity IDs and identity fields in the contract.
- Represent SteamID and missing identity fields through `FieldPresence<T>`.
- Do not canonicalize player identity and do not implement same-name merging in contract types.

### Source reference and rule ID model

Phase 2 D-09 through D-13 require source refs and rule IDs without raw replay snippets.

Apply:

- `SourceRef` should support `replay_id`, `source_file`, `checksum`, `frame`, `event_index`, `entity_id`, `json_path`, and `rule_id` when available.
- `RuleId` should be a stable string newtype or validated wrapper, used by derived/inferred values and aggregate contributions.
- Diagnostics should describe expected/observed shape and parser action by path, not embed raw source payloads.

### Failure contract pattern

Legacy worker responses distinguish `success`, `skipped`, and `error`; Phase 2 expands this into parser-owned status plus structured failure fields.

Apply:

- Use `ParseStatus` values `success`, `partial`, `skipped`, and `failed`.
- Use `ParseFailure` with `job_id`, `replay_id`, `source_file`, `checksum`, `stage`, `error_code`, `message`, `retryability`, `source_cause`, and `source_refs`.
- Use namespaced error-code families such as `io.*`, `json.*`, `schema.*`, `unsupported.*`, and `internal.*`.

### Test pattern

Use behavior tests as executable contract documentation.

Apply:

- Assert exact serialized JSON keys and enum states.
- Use builders for success artifact, missing SteamID, null killer, skipped artifact, and parse failure.
- Validate committed example artifacts against generated schema after schema support exists.

## Shared Verification Commands

```bash
cargo test -p parser-contract
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

## Pattern Mapping Complete

Plans should use these assignments and avoid adding parser behavior or transport behavior during Phase 2.
