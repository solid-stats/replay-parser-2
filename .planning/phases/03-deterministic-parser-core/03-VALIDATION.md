---
phase: 03
slug: deterministic-parser-core
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-26
---

# Phase 03 - Validation Strategy

## Test Infrastructure

| Property | Value |
|----------|-------|
| Framework | Rust integration and unit tests through Cargo |
| Config file | `Cargo.toml`, `.cargo/config.toml`, `clippy.toml`, `rustfmt.toml` |
| Quick run command | `cargo test -p parser-core` |
| Contract run command | `cargo test -p parser-contract metadata_identity_contract schema_contract` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps` |
| Estimated runtime | under 60 seconds before full corpus work exists |

## Sampling Rate

- After every task commit: run the plan-specific cargo test command named in `<verify>`.
- After every plan wave: run `cargo test --workspace`.
- Before `$gsd-verify-work`: run the full suite command.
- Max feedback latency: one plan task before a targeted automated test runs.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-00-01 | 03-00 | 1 | PARS-05 | T-03-00-01 | Entity contract preserves observed name/class without canonical identity | contract | `cargo test -p parser-contract metadata_identity_contract` | no | pending |
| 03-00-02 | 03-00 | 1 | OUT-08 | T-03-00-02 | Schema rejects unauditable entity source refs | schema | `cargo test -p parser-contract schema_contract` | no | pending |
| 03-01-01 | 03-01 | 2 | PARS-01 | T-03-01-01 | Parser-core API has no file, queue, S3, or DB side effects | unit | `cargo test -p parser-core parser_core_api` | no | pending |
| 03-01-02 | 03-01 | 2 | PARS-02 | T-03-01-02 | Malformed JSON yields structured failure artifact | unit | `cargo test -p parser-core parser_core_failure` | no | pending |
| 03-02-01 | 03-02 | 3 | PARS-03 | T-03-02-01 | Metadata fields are normalized with explicit source refs and unknowns | integration | `cargo test -p parser-core metadata_normalization` | no | pending |
| 03-03-01 | 03-03 | 4 | PARS-04/PARS-05 | T-03-03-01 | Unit, vehicle, and static weapon facts are deterministic and auditable | integration | `cargo test -p parser-core entity_normalization` | no | pending |
| 03-04-01 | 03-04 | 5 | OUT-08/PARS-02 | T-03-04-01 | Repeated parse output is byte-stable and drift escalates status correctly | integration | `cargo test -p parser-core deterministic_output schema_drift_status` | no | pending |
| 03-05-01 | 03-05 | 6 | PARS-06/PARS-07 | T-03-05-01 | Legacy compatibility hooks preserve raw observations and provenance | integration | `cargo test -p parser-core legacy_entity_compatibility` | no | pending |

## Wave 0 Requirements

- No separate Wave 0 is required. Existing Cargo, rustfmt, clippy, docs, and parser-contract test infrastructure are already present.
- Plan 03-01 creates `crates/parser-core/tests/fixtures/` before any parser-core behavior depends on fixtures.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cross-application compatibility | INT-02 | Adjacent apps are not implemented in this repo | Confirm no plan writes parser results to server-2 tables, changes RabbitMQ/S3 messages, or adds canonical identity fields. |

## Validation Sign-Off

- [x] All plans include automated verification commands.
- [x] Sampling continuity: no plan wave is more than one task away from a targeted cargo test.
- [x] Existing infrastructure covers Phase 3 requirements.
- [x] No watch-mode flags are required.
- [x] Full suite includes format, clippy, tests, and docs.

Approval: pending execution
