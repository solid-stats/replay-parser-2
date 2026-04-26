---
phase: 02-versioned-output-contract
verified: 2026-04-26T06:35:26Z
status: passed
score: "4/4 must-haves verified"
overrides_applied: 0
gaps: []
human_verification: []
---

# Phase 2: Versioned Output Contract Verification Report

**Phase Goal:** `server-2` and parser tooling can rely on a stable, machine-checkable parse artifact and failure contract.  
**Verified:** 2026-04-26T06:35:26Z  
**Status:** passed  
**Re-verification:** Yes - after `02-05` gap closure

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Developer can validate a current `ParseArtifact` JSON document that includes parser version, contract version, replay/source identifiers, checksum, and parse status metadata. | VERIFIED | `ParseArtifact` and exact status values are in `crates/parser-contract/src/artifact.rs`; `ReplaySource.checksum` uses `FieldPresence<SourceChecksum>` in `source_ref.rs`; schema includes sha256 enum/pattern constraints; examples validate. |
| 2 | Server integrator can consume normalized replay metadata, observed identity fields, and explicit unknown/null states without canonical player matching. | VERIFIED | `ReplayMetadata`, `ObservedIdentity`, and `FieldPresence<T>` cover Phase 2 metadata/identity states. `rg canonical_player\|canonical_id\|account_id` found only negative test assertions, not contract fields. |
| 3 | Developer can trace normalized events and aggregate contributions back to replay, frame, event index, entity ID, and rule ID where available. | VERIFIED | `NormalizedEvent.source_refs`, `AggregateContributionRef.source_refs`, `Diagnostic.source_refs`, and `ParseFailure.source_refs` use `SourceRefs`; `SourceRef` rejects hollow evidence; schema has `SourceRefs.minItems: 1` and `SourceRef.anyOf`. |
| 4 | Developer can validate structured `ParseFailure` output with job/replay/file identifiers, stage, error code, message, retryability, and source cause. | VERIFIED | `ParseFailure` uses explicit presence for job/replay/file/checksum/source cause; `ParseArtifact::validate_status_payload()` rejects missing/unexpected failure payloads; schema conditionals enforce failed/non-failed failure shape. |

**Score:** 4/4 truths verified

## Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `Cargo.toml` | Rust workspace with `crates/parser-contract` | VERIFIED | Workspace exists and tests run under Rust 1.95.0. |
| `rust-toolchain.toml` | Pinned Rust 1.95.0 toolchain | VERIFIED | Toolchain file exists with rustfmt/clippy components. |
| `crates/parser-contract/src/version.rs` | Separate contract/parser version types | VERIFIED | Tests assert `contract_version` and parser implementation version remain separate. |
| `crates/parser-contract/src/artifact.rs` | Unified parse artifact envelope | VERIFIED | Envelope exists and status/failure validator closes the prior gap. |
| `crates/parser-contract/src/source_ref.rs` | Source identity, validated checksums, source refs, rule IDs | VERIFIED | `ChecksumAlgorithm`, `ChecksumValue`, `SourceChecksum`, `SourceRef`, and `SourceRefs` are implemented. |
| `crates/parser-contract/src/metadata.rs` | Replay metadata contract | VERIFIED | Required metadata fields use explicit presence semantics. |
| `crates/parser-contract/src/identity.rs` | Observed identity contract | VERIFIED | Observed fields are present and canonical identity is excluded. |
| `crates/parser-contract/src/events.rs` | Normalized event skeleton | VERIFIED | Event skeleton includes non-empty source refs and rule IDs. |
| `crates/parser-contract/src/aggregates.rs` | Aggregate contribution refs | VERIFIED | Contribution refs use non-empty source refs and deterministic projections. |
| `crates/parser-contract/src/failure.rs` | Structured parse failure | VERIFIED | Required diagnostic context is explicit and error codes cover checksum/output families. |
| `crates/parser-contract/src/schema.rs` and `schemas/parse-artifact-v1.schema.json` | Generated machine-readable schema | VERIFIED | Fresh export is byte-identical and includes the added conditionals/ranges/patterns/minItems. |
| `crates/parser-contract/examples/*.json` | Success/failure examples | VERIFIED | Both examples deserialize into `ParseArtifact` and validate against committed schema. |

## Requirements Coverage

| Requirement | Status | Evidence |
|---|---|---|
| OUT-01 | SATISFIED | Stable `ParseArtifact` envelope includes contract/parser/source/status fields and validated checksum identity. |
| OUT-02 | SATISFIED | `ReplayMetadata` covers mission, world, author, player counts, capture delay, end frame, and time/frame bounds. |
| OUT-03 | SATISFIED | `ObservedIdentity` covers observed nickname, SteamID, side/faction, group/squad, role/description, and source entity ID without canonical matching. |
| OUT-04 | SATISFIED | `FieldPresence<T>` models present, explicit null, unknown, inferred, and not applicable states; confidence is bounded. |
| OUT-05 | SATISFIED | Source refs are non-empty and non-hollow for events, diagnostics, failures, and aggregate contributions. |
| OUT-06 | SATISFIED | `parse_artifact_schema()` generates a schema that validates examples and rejects gap-regression fixtures. |
| OUT-07 | SATISFIED | `ParseFailure` includes identifiers, stage, error code, message, retryability, source cause, and source refs. |

## Behavioral Checks

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS - 36 tests passed |
| `cargo run -p parser-contract --example export_schema > /tmp/parse-artifact-v1.schema.json` | PASS |
| `cmp /tmp/parse-artifact-v1.schema.json schemas/parse-artifact-v1.schema.json` | PASS |
| Success/failure example `jq` assertions | PASS |
| Schema invariant `jq` assertions for checksum enum/pattern, `SourceRefs.minItems`, and `allOf` conditionals | PASS |
| `git diff --check` during plan validation | PASS |
| Code review gate | PASS - `02-REVIEW.md` status `clean` |
| Regression gate | SKIPPED - Phase 1 verification had no runnable prior test files |
| Schema drift gate | PASS - `drift_detected: false` |
| Codebase drift gate | SKIPPED - `no-structure-md` |

## Prior Gap Closure

| Prior Gap | Status |
|---|---|
| OUT-01 / OUT-06 / OUT-07 checksum constraints and checksum availability | CLOSED |
| OUT-05 / OUT-06 empty or hollow source references | CLOSED |
| OUT-07 / OUT-06 status/failure payload invariants | CLOSED |
| Error-code family mismatch for checksum/output stages | CLOSED |
| Unbounded inferred confidence | CLOSED |

## Human Verification Required

None. Phase 2 is a Rust contract/schema phase and all success criteria are verified with automated checks.

## Final Assessment

Phase 2 achieved its goal. The parser contract is stable enough for Phase 3 parser-core work and for `server-2` integration planning, while still keeping canonical identity, persistence, APIs, and UI behavior outside this repository's ownership.

---
_Verified: 2026-04-26T06:35:26Z_  
_Verifier: Codex inline phase verifier_
