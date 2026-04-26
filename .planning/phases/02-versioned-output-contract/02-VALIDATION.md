---
phase: 02
slug: versioned-output-contract
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-26
---

# Phase 02 - Validation Strategy

Per-phase validation contract for the versioned output contract implementation.

## Test Infrastructure

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` with focused unit/integration tests |
| Config file | `Cargo.toml`, `crates/parser-contract/Cargo.toml`, `rust-toolchain.toml` |
| Quick run command | `cargo test -p parser-contract` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` |
| Estimated runtime | under 120 seconds after dependencies are cached |

## Sampling Rate

- After every task commit: run `cargo test -p parser-contract` once the workspace exists; before it exists, run `git diff --check`.
- After every plan wave: run the full suite command.
- Before `$gsd-verify-work`: run the full suite command plus schema/example validation from Plan 04.
- Max feedback latency: under 120 seconds for normal contract tests.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-00-01 | 00 | 1 | OUT-01, OUT-06 | T-02-00-01 | workspace and contract crate are reproducible | build | `test -f Cargo.toml && test -f crates/parser-contract/Cargo.toml && cargo test -p parser-contract` | no | pending |
| 02-00-02 | 00 | 1 | OUT-01 | T-02-00-02 | contract and parser versions are distinct typed values | unit | `cargo test -p parser-contract version_contract` | no | pending |
| 02-01-01 | 01 | 2 | OUT-01 | T-02-01-01 | artifact envelope serializes stable version/source/status fields | unit | `cargo test -p parser-contract artifact_envelope` | no | pending |
| 02-01-02 | 01 | 2 | OUT-01, OUT-04 | T-02-01-02 | diagnostics carry path/action/source evidence without raw snippets | unit | `cargo test -p parser-contract diagnostics_are_path_based` | no | pending |
| 02-02-01 | 02 | 3 | OUT-02, OUT-03, OUT-04 | T-02-02-01 | replay metadata and observed identity preserve unknown/null states | unit | `cargo test -p parser-contract metadata_identity_contract` | no | pending |
| 02-03-01 | 03 | 4 | OUT-05 | T-02-03-01 | events and aggregate contributions include source refs and rule IDs | unit | `cargo test -p parser-contract source_ref_contract` | no | pending |
| 02-04-01 | 04 | 5 | OUT-06, OUT-07 | T-02-04-01 | failure examples and success examples validate against generated schema | integration | `cargo test -p parser-contract schema_contract` | no | pending |
| 02-04-02 | 04 | 5 | OUT-01..OUT-07 | T-02-04-02 | final workspace is formatted, lint-clean, and documented | full | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` | no | pending |

## Wave 0 Requirements

- [ ] `Cargo.toml` - workspace manifest with `crates/parser-contract`.
- [ ] `rust-toolchain.toml` - Rust toolchain pin from project research.
- [ ] `crates/parser-contract/src/lib.rs` - public module exports.
- [ ] `crates/parser-contract/tests/` - behavior tests for public serialization and validation.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Backend accepts final artifact field names | OUT-01, OUT-06 | Only brief-level `server-2` evidence exists in this repo | Record cross-project review in a later integration phase if adjacent backend docs disagree |
| Preserve or fix suspected legacy bug reflected in contract semantics | OUT-05 | Phase 1 D-12 requires human review | Record the issue in mismatch taxonomy or phase summary before changing contract semantics |

## Security Threat Model

| ID | Threat | Severity | Mitigation | Verification |
|----|--------|----------|------------|--------------|
| T-02-00-01 | Workspace setup is not reproducible | medium | Pin toolchain and commit `Cargo.lock` once dependencies resolve | `test -f rust-toolchain.toml && test -f Cargo.lock` |
| T-02-00-02 | Contract version and parser version are conflated | high | Separate `contract_version` and `parser.version` fields | `cargo test -p parser-contract version_contract` |
| T-02-01-01 | Artifact status is ambiguous for partial/skipped/failed cases | high | Use `ParseStatus` enum with exact values `success`, `partial`, `skipped`, `failed` | `cargo test -p parser-contract artifact_envelope` |
| T-02-01-02 | Diagnostics leak raw replay snippets | medium | Store coordinates, json paths, expected/observed shape, and parser action only | `cargo test -p parser-contract diagnostics_are_path_based` |
| T-02-02-01 | Missing identity fields are serialized as ambiguous bare nulls | high | Use `FieldPresence<T>` for optional contract fields | `cargo test -p parser-contract metadata_identity_contract` |
| T-02-03-01 | Aggregates cannot be audited later | high | Require `SourceRef` and `RuleId` on events and contribution refs | `cargo test -p parser-contract source_ref_contract` |
| T-02-04-01 | Schema drifts from Rust types | high | Generate schema from `schemars` and validate committed examples in tests | `cargo test -p parser-contract schema_contract` |
| T-02-04-02 | Failure retryability is inconsistent | medium | Use explicit `Retryability` enum and stable namespaced `ErrorCode` validation | `cargo test -p parser-contract failure_contract` |

## Validation Sign-Off

- [x] All planned deliverables have automated build, unit, or schema checks.
- [x] Sampling continuity: every plan has at least one automated verification command.
- [x] Wave 0 creates the Rust test infrastructure needed by later plans.
- [x] No watch-mode flags are used.
- [x] `nyquist_compliant: true` is set in frontmatter.

Approval: pending execution.
