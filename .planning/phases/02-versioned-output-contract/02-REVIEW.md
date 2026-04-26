---
phase: 02-versioned-output-contract
reviewed: 2026-04-26T05:51:05Z
depth: standard
files_reviewed: 27
files_reviewed_list:
  - .gitignore
  - Cargo.toml
  - README.md
  - rust-toolchain.toml
  - schemas/parse-artifact-v1.schema.json
  - crates/parser-contract/Cargo.toml
  - crates/parser-contract/examples/export_schema.rs
  - crates/parser-contract/examples/parse_artifact_success.v1.json
  - crates/parser-contract/examples/parse_failure.v1.json
  - crates/parser-contract/src/aggregates.rs
  - crates/parser-contract/src/artifact.rs
  - crates/parser-contract/src/diagnostic.rs
  - crates/parser-contract/src/events.rs
  - crates/parser-contract/src/failure.rs
  - crates/parser-contract/src/identity.rs
  - crates/parser-contract/src/lib.rs
  - crates/parser-contract/src/metadata.rs
  - crates/parser-contract/src/presence.rs
  - crates/parser-contract/src/schema.rs
  - crates/parser-contract/src/source_ref.rs
  - crates/parser-contract/src/version.rs
  - crates/parser-contract/tests/artifact_envelope.rs
  - crates/parser-contract/tests/failure_contract.rs
  - crates/parser-contract/tests/metadata_identity_contract.rs
  - crates/parser-contract/tests/schema_contract.rs
  - crates/parser-contract/tests/source_ref_contract.rs
  - crates/parser-contract/tests/version_contract.rs
findings:
  critical: 4
  warning: 2
  info: 0
  total: 6
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-04-26T05:51:05Z
**Depth:** standard
**Files Reviewed:** 27
**Status:** issues_found

## Summary

Reviewed the Phase 2 contract crate, generated schema, committed examples, README/workspace files, and behavior tests. The workspace checks pass, but the contract currently accepts artifacts that violate the phase's own success criteria around structured failures, source integrity, and auditability.

Validation run:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p parser-contract`

`Cargo.lock` was read as requested but excluded from `files_reviewed_list` per workflow lockfile filtering.

## Critical Issues

### CR-01: BLOCKER - Failed Artifacts Can Omit Structured Failure Details

**File:** `crates/parser-contract/src/artifact.rs:32`
**Issue:** `ParseArtifact.status` and `ParseArtifact.failure` are independent fields, so Rust can construct `status: ParseStatus::Failed` with `failure: None`, or a success artifact with a populated failure object. The generated schema also does not require `failure` at the top level, so a failed artifact without `stage`, `error_code`, `retryability`, or source cause can still validate. That violates OUT-07 and leaves `server-2` without the retry and diagnosis data it needs.
**Fix:**

```rust
impl ParseArtifact {
    pub fn validate_status_payload(&self) -> Result<(), ParseArtifactError> {
        match (self.status, self.failure.as_ref()) {
            (ParseStatus::Failed, Some(_)) => Ok(()),
            (ParseStatus::Failed, None) => Err(ParseArtifactError::MissingFailure),
            (_, Some(_)) => Err(ParseArtifactError::UnexpectedFailure),
            _ => Ok(()),
        }
    }
}
```

Also add schema validation tests proving `status: "failed"` requires a non-null `failure`, and non-failed statuses reject a populated `failure`.

### CR-02: BLOCKER - Failure Envelope Requires a Checksum Even When One Cannot Exist

**File:** `crates/parser-contract/src/source_ref.rs:8`
**Issue:** Every `ParseArtifact` requires `source.checksum`, but several required failure modes occur before a replay checksum can be computed, such as unreadable input, missing object, permission failure, or interrupted read. `ParseFailure.checksum` is optional, but the full failed artifact still must invent a top-level checksum to satisfy `ReplaySource`. This makes checksum/input failures impossible to represent truthfully.
**Fix:** Make top-level source checksum explicit about availability, for example `FieldPresence<SourceChecksum>` or separate `expected_checksum` and `observed_checksum` fields. Require an observed checksum only for success/partial artifacts where bytes were read, and add tests for an input-stage failure with no computed checksum.

### CR-03: BLOCKER - Source References Can Be Empty While Still Passing the Contract

**File:** `crates/parser-contract/src/source_ref.rs:17`
**Issue:** `SourceRef` has all fields optional and derives `Default`, while `NormalizedEvent.source_refs` and `AggregateContributionRef.source_refs` are plain `Vec<SourceRef>`. The schema requires the array fields but allows `[]`, and also allows source ref objects with no replay/file/frame/event/entity/path/rule evidence. This breaks the Phase 2 requirement that normalized events and aggregate contributions remain auditable through source references.
**Fix:** Introduce a non-empty source-reference wrapper and reject empty coordinate objects.

```rust
pub struct SourceRefs(Vec<SourceRef>);

impl SourceRef {
    pub fn has_evidence(&self) -> bool {
        self.replay_id.is_some()
            || self.source_file.is_some()
            || self.frame.is_some()
            || self.event_index.is_some()
            || self.entity_id.is_some()
            || self.json_path.is_some()
            || self.rule_id.is_some()
    }
}
```

Use `SourceRefs` for events, aggregate contributions, diagnostics, and failures, then add schema tests requiring `minItems: 1` for event/contribution references where audit evidence is mandatory.

### CR-04: BLOCKER - Source Checksums Are Unvalidated Free-Form Strings

**File:** `crates/parser-contract/src/source_ref.rs:11`
**Issue:** `SourceChecksum.algorithm` and `SourceChecksum.value` accept any string, and the schema only says both are strings. Artifacts with `{"algorithm":"sha256","value":"not-a-hash"}` or `{"algorithm":"md5","value":"x"}` can validate. Since checksum is part of source identity and later worker integrity checks, this is a data-integrity hole in the machine-checkable contract.
**Fix:** Replace free strings with validated types, at least for `sha256` in v1.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumAlgorithm {
    Sha256,
}

pub struct ChecksumValue(String); // validate 64 lowercase hex chars for Sha256
```

Add negative tests for wrong length, non-hex values, uppercase drift if not accepted, and unsupported algorithms.

## Warnings

### WR-01: WARNING - Error Code Validation Cannot Represent Some Defined Failure Stages

**File:** `crates/parser-contract/src/failure.rs:47`
**Issue:** `ParseStage` includes `checksum` and `output`, but `ErrorCode` only accepts families `io.`, `json.`, `schema.`, `unsupported.`, and `internal.`. A natural checksum failure such as `checksum.mismatch` is rejected even though checksum is a first-class stage. The same validator also accepts empty dotted segments such as `json..decode`, unlike `RuleId`.
**Fix:** Align error-code families with parse stages or use the same segment validation pattern as `RuleId`.

### WR-02: WARNING - Inferred Confidence Values Are Unbounded

**File:** `crates/parser-contract/src/presence.rs:24`
**Issue:** `FieldPresence::Inferred` exposes `confidence: Option<f32>` with no finite or range constraint. The generated schema accepts any number, including negative values and values above 1.0, which makes confidence metadata ambiguous for future winner/commander/source inference consumers.
**Fix:** Use a validated `Confidence` newtype with a finite `0.0..=1.0` range and generate schema minimum/maximum constraints. Add tests rejecting `-0.1`, `1.1`, and non-finite values before serialization.

---

_Reviewed: 2026-04-26T05:51:05Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
