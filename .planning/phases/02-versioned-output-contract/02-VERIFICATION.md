---
phase: 02-versioned-output-contract
verified: 2026-04-26T05:58:07Z
status: gaps_found
score: "1/4 must-haves verified"
overrides_applied: 0
gaps:
  - truth: "Developer can validate a current ParseArtifact JSON document that includes parser version, contract version, replay/source identifiers, checksum, and parse status metadata."
    status: partial
    reason: "The artifact exists and schema validates basic fields, but source checksum is free-form and the schema cannot reject invalid checksum algorithms or values. The top-level source checksum is also required for all artifacts, including input failures where no observed checksum may exist."
    artifacts:
      - path: "crates/parser-contract/src/source_ref.rs"
        issue: "ReplaySource requires checksum, while SourceChecksum.algorithm and value are unconstrained strings."
      - path: "schemas/parse-artifact-v1.schema.json"
        issue: "SourceChecksum has no enum/pattern constraints; ReplaySource requires checksum unconditionally."
    missing:
      - "Validated checksum algorithm/value types or schema constraints for v1 sha256."
      - "A truthful availability model for checksum on pre-read/pre-checksum failures."
  - truth: "Developer can trace normalized events and aggregate contributions back to replay, frame, event index, entity ID, and rule ID where available."
    status: failed
    reason: "The contract has source_refs fields, but arrays may be empty and SourceRef objects may contain no evidence. Schema-valid events and aggregate contributions can therefore be untraceable."
    artifacts:
      - path: "crates/parser-contract/src/source_ref.rs"
        issue: "SourceRef derives Default and all coordinate fields are optional."
      - path: "crates/parser-contract/src/events.rs"
        issue: "NormalizedEvent.source_refs is Vec<SourceRef> with no non-empty/evidence invariant."
      - path: "crates/parser-contract/src/aggregates.rs"
        issue: "AggregateContributionRef.source_refs is Vec<SourceRef> with no non-empty/evidence invariant."
      - path: "schemas/parse-artifact-v1.schema.json"
        issue: "source_refs arrays have no minItems and SourceRef has no required coordinate fields."
    missing:
      - "Non-empty source-reference wrapper or validation for events and aggregate contributions."
      - "A SourceRef evidence invariant requiring at least one meaningful coordinate or rule id."
      - "Schema constraints that expose those auditability requirements to server-2."
  - truth: "Developer can validate structured ParseFailure output with job/replay/file identifiers, stage, error code, message, retryability, and source cause."
    status: failed
    reason: "ParseArtifact.status and failure are independent, and the generated schema does not require failure when status is failed or reject failure on non-failed statuses. Structured failure identifiers and source_cause are optional, so schema-valid failures can omit required diagnostic context."
    artifacts:
      - path: "crates/parser-contract/src/artifact.rs"
        issue: "status: ParseStatus and failure: Option<ParseFailure> have no validator or type-level invariant."
      - path: "crates/parser-contract/src/failure.rs"
        issue: "job_id, replay_id, source_file, checksum, and source_cause are optional in ParseFailure."
      - path: "schemas/parse-artifact-v1.schema.json"
        issue: "failure is not top-level required and no conditional schema ties status failed to a populated failure object."
    missing:
      - "Status/payload validation for failed and non-failed artifacts."
      - "Schema tests proving failed artifacts require structured failure details."
      - "A clear required/optional policy for job, replay, file, checksum, and source cause fields by failure stage."
---

# Phase 2: Versioned Output Contract Verification Report

**Phase Goal:** `server-2` and parser tooling can rely on a stable, machine-checkable parse artifact and failure contract.  
**Verified:** 2026-04-26T05:58:07Z  
**Status:** gaps_found  
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Developer can validate a current `ParseArtifact` JSON document that includes parser version, contract version, replay/source identifiers, checksum, and parse status metadata. | PARTIAL | `ParseArtifact` exists in `crates/parser-contract/src/artifact.rs:27`, examples validate, and schema export is byte-identical. Gap: `SourceChecksum` is unconstrained free-form strings at `source_ref.rs:11`, and `ReplaySource.checksum` is required for every artifact at `source_ref.rs:8`, including failures where no checksum may exist. |
| 2 | Server integrator can consume normalized replay metadata, observed identity fields, and explicit unknown/null states without canonical player matching. | VERIFIED | `ReplayMetadata` covers mission/world/author/player-count/capture/end/frame/time fields in `metadata.rs:6`; `ObservedIdentity` covers nickname, SteamID, side/faction/group/squad/role/description in `identity.rs:33`; `FieldPresence<T>` provides present, explicit_null, unknown, inferred, and not_applicable states in `presence.rs:6`. No canonical identity fields found. |
| 3 | Developer can trace normalized events and aggregate contributions back to replay, frame, event index, entity ID, and rule ID where available. | FAILED | `NormalizedEvent.source_refs` and `AggregateContributionRef.source_refs` exist, but both are plain arrays with no non-empty invariant (`events.rs:41`, `aggregates.rs:24`). `SourceRef` has all optional fields and derives `Default` (`source_ref.rs:17`). Schema has no `minItems` or required evidence fields. |
| 4 | Developer can validate structured `ParseFailure` output with job/replay/file identifiers, stage, error code, message, retryability, and source cause. | FAILED | `ParseFailure` exists at `failure.rs:77`, but `ParseArtifact.status` and `failure` are independent (`artifact.rs:32`, `artifact.rs:39`), and schema does not require `failure` when `status` is `failed`. Several required diagnostic fields are optional. |

**Score:** 1/4 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `Cargo.toml` | Rust workspace with `crates/parser-contract` | VERIFIED | Workspace member exists; Rust 2024 resolver 3 present. |
| `rust-toolchain.toml` | Pinned Rust 1.95.0 toolchain | VERIFIED | Contains `channel = "1.95.0"` and rustfmt/clippy components. |
| `crates/parser-contract/src/version.rs` | Separate contract/parser version types | VERIFIED | `ContractVersion`, `ParserInfo`, `ParserBuildInfo`, and current version are implemented. |
| `crates/parser-contract/src/artifact.rs` | Unified parse artifact envelope | PARTIAL | Envelope exists, but status/failure invariants are missing. |
| `crates/parser-contract/src/source_ref.rs` | Source identity, checksums, source refs, rule IDs | PARTIAL | Fields exist, but checksum and source-reference evidence are not machine-checkable. |
| `crates/parser-contract/src/metadata.rs` | Replay metadata contract | VERIFIED | Required Phase 2 metadata fields use explicit presence semantics. |
| `crates/parser-contract/src/identity.rs` | Observed identity contract | VERIFIED | Observed fields are present and canonical identity is excluded. |
| `crates/parser-contract/src/events.rs` | Normalized event skeleton | PARTIAL | Event skeleton exists, but audit refs can be empty/hollow. |
| `crates/parser-contract/src/aggregates.rs` | Aggregate contribution refs | PARTIAL | Contribution skeleton exists, but audit refs can be empty/hollow. |
| `crates/parser-contract/src/failure.rs` | Structured parse failure | PARTIAL | Failure fields exist, but top-level artifact and schema do not require failure details for failed status. |
| `crates/parser-contract/src/schema.rs` and `schemas/parse-artifact-v1.schema.json` | Generated machine-readable schema | PARTIAL | Fresh export matches committed schema, but schema omits critical conditional/integrity constraints. |
| `crates/parser-contract/examples/*.json` | Success/failure examples | VERIFIED | Both examples deserialize and validate against the committed schema. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `version.rs` | Phase 2 D-02 | Separate contract and parser versions | VERIFIED | `gsd-sdk verify.key-links` found `contract_version`; tests assert separate serialized fields. |
| `artifact.rs` | Phase 2 D-01/D-14 | Unified envelope and status model | PARTIAL | Envelope and statuses exist, but D-01's "either parsed data sections or structured failure details" is not enforced. |
| `identity.rs` | Phase 1 identity boundary | Observed identity only | VERIFIED | Identity types omit canonical player/account fields. |
| `aggregates.rs` | Phase 2 D-12 | Aggregate contribution reference shape | PARTIAL | Shape exists, but audit refs are not enforceably non-empty. |
| `schemas/parse-artifact-v1.schema.json` | Rust `ParseArtifact` type | Generated from Rust types | VERIFIED | `cargo run -p parser-contract --example export_schema > /tmp/... && cmp ...` passed. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `schemas/parse-artifact-v1.schema.json` | Schema JSON | `parse_artifact_schema()` in `schema.rs`, exported by `examples/export_schema.rs` | Yes, from Rust `ParseArtifact` type | FLOWING |
| `parse_artifact_success.v1.json` | Example artifact | Static committed example, validated by `schema_contract.rs` | Yes for example review only | FLOWING |
| `parse_failure.v1.json` | Failure example | Static committed example, validated by `schema_contract.rs` | Yes for example review only | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Formatting gate | `cargo fmt --all -- --check` | Exit 0 | PASS |
| Lint gate | `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 | PASS |
| Workspace tests | `cargo test --workspace` | 28 tests passed | PASS |
| Schema export drift | `cargo run -p parser-contract --example export_schema > /tmp/parse-artifact-v1.schema.json && cmp /tmp/parse-artifact-v1.schema.json schemas/parse-artifact-v1.schema.json` | Exit 0 | PASS |
| Success/failure example jq assertions | `jq` assertions for success and failure examples | Both returned `true` | PASS |
| Schema invariant inspection | `jq` assertion checking missing `failure` requirement, missing checksum pattern/enum, missing source_refs `minItems`, and missing `SourceRef.required` | Returned `true`, confirming the gaps | FAIL |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| OUT-01 | 02-00, 02-01, 02-04 | Stable JSON `ParseArtifact` with parser/contract version, replay/source IDs, checksum, status metadata | BLOCKED | Envelope and examples exist, but checksum fields are unconstrained and required even for failures where no checksum can exist. |
| OUT-02 | 02-02, 02-04 | Normalized replay metadata | SATISFIED | `ReplayMetadata` covers mission/world/author/player-count/capture/end-frame/time/frame boundaries. |
| OUT-03 | 02-02, 02-04 | Observed identity fields without canonical matching | SATISFIED | `ObservedIdentity` includes observed identity fields; no canonical player/account fields found. |
| OUT-04 | 02-01, 02-02, 02-04 | Explicit unknown/null states | SATISFIED | `FieldPresence<T>` models explicit null, unknown, inferred, and not applicable; tests cover missing SteamID and null killer. |
| OUT-05 | 02-01, 02-03, 02-04 | Source references link events/contributions to replay/frame/event/entity/rule evidence | BLOCKED | Source-ref fields exist, but empty arrays and empty `SourceRef` objects remain schema-valid. |
| OUT-06 | 02-00, 02-04 | JSON Schema generation/equivalent machine-readable validation | BLOCKED | Schema is generated and committed, but it does not machine-check critical status/failure, checksum, or source-reference invariants. |
| OUT-07 | 02-04 | Structured `ParseFailure` output with identifiers, stage, error code, message, retryability, source cause | BLOCKED | `ParseFailure` exists, but failed artifacts can omit it and several expected diagnostic context fields are optional/schema-unconditional. |

No orphaned Phase 2 requirements found: ROADMAP and REQUIREMENTS list OUT-01 through OUT-07 for Phase 2, and all are claimed by Phase 2 plans.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---:|---|---|---|
| `crates/parser-contract/src/artifact.rs` | 32 | Independent `status` and `failure` fields | BLOCKER | A schema-valid failed artifact can omit structured failure details. |
| `crates/parser-contract/src/source_ref.rs` | 8 | Required top-level checksum for all artifacts | BLOCKER | Input/read failures may need to invent a checksum to fit the envelope. |
| `crates/parser-contract/src/source_ref.rs` | 11 | Free-form checksum algorithm/value strings | BLOCKER | Schema-valid artifacts can carry unusable checksum identity. |
| `crates/parser-contract/src/source_ref.rs` | 17 | `SourceRef` derives `Default`; all evidence fields optional | BLOCKER | Audit references can be hollow while still validating. |
| `crates/parser-contract/src/events.rs` | 41 | `Vec<SourceRef>` without non-empty invariant | BLOCKER | Events can be untraceable. |
| `crates/parser-contract/src/aggregates.rs` | 24 | `Vec<SourceRef>` without non-empty invariant | BLOCKER | Aggregate contributions can be untraceable. |
| `crates/parser-contract/src/failure.rs` | 46 | Error-code validation not aligned with checksum/output stages and permits empty dotted segments | WARNING | Some valid stages lack natural error-code families; malformed codes can deserialize. |
| `crates/parser-contract/src/presence.rs` | 24 | Unbounded inferred confidence | WARNING | Confidence metadata can be outside a meaningful 0.0 to 1.0 range. |

### Human Verification Required

None. This phase is a Rust contract/schema phase and the blocking gaps are programmatically observable.

### Gaps Summary

Phase 2 is not ready to proceed as achieved. The crate, examples, tests, and generated schema exist, and the metadata/identity/unknown-state portion is solid. The missed goal is the stronger contract promise: `server-2` and tooling cannot fully rely on the artifact as machine-checkable because schema-valid artifacts can still violate failure, checksum, and auditability requirements.

Blocking gaps:

1. **OUT-07 / OUT-06:** Failed artifacts are not required to contain structured failure details. Evidence: `ParseArtifact.status` and `failure` are independent fields in `artifact.rs`, and the schema omits conditional status/failure validation.
2. **OUT-01 / OUT-06 / OUT-07:** Source checksum handling is not reliable. Evidence: `ReplaySource.checksum` is mandatory for every artifact, but checksum values are unconstrained strings and may be unavailable for input-stage failures.
3. **OUT-05 / OUT-06:** Source references can be hollow. Evidence: events and aggregate contributions use plain `Vec<SourceRef>`, `SourceRef` has no required evidence fields, and the schema has no `minItems` or equivalent invariant.

The advisory review findings CR-01 through CR-04 are accepted as phase-goal blockers. WR-01 and WR-02 are retained as warnings that should be considered while closing the blockers.

---

_Verified: 2026-04-26T05:58:07Z_  
_Verifier: the agent (gsd-verifier)_
