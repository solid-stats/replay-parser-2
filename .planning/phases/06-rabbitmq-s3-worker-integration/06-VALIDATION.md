---
phase: 06
slug: rabbitmq-s3-worker-integration
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-02
---

# Phase 06 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust Cargo tests, assert_cmd CLI tests, JSON Schema tests, fake async worker integration tests |
| **Config file** | `Cargo.toml`, `rust-toolchain.toml`, `scripts/coverage-gate.sh`, `scripts/fault-report-gate.sh`, `scripts/benchmark-phase5.sh` |
| **Quick run command** | `cargo test -p parser-contract -p parser-worker -p parser-cli` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps && scripts/coverage-gate.sh --check && scripts/fault-report-gate.sh && scripts/benchmark-phase5.sh --ci` |
| **Estimated runtime** | ~900 seconds without live RabbitMQ/S3 services |

---

## Sampling Rate

- **After every task commit:** Run the narrow package tests named in that task's `<verify>` block.
- **After every plan wave:** Run `cargo test -p parser-contract -p parser-worker -p parser-cli` once `parser-worker` exists.
- **Before `$gsd-verify-work`:** Full suite must be green.
- **Max feedback latency:** 900 seconds for code waves, excluding existing benchmark smoke time.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-00-01 | 00 | 1 | WORK-01/WORK-05/WORK-06 | T-06-00-01 | Message contracts reject missing/invalid fields and preserve structured failure data | contract/schema | `cargo test -p parser-contract worker_message_contract schema_contract` | W0 | pending |
| 06-00-02 | 00 | 1 | WORK-01/WORK-05/WORK-06 | T-06-00-02 | Worker schemas are generated from Rust types, not hand-written docs | schema | `cargo run -p parser-contract --example export_worker_schemas -- --output-dir schemas && cargo test -p parser-contract schema_contract` | W0 | pending |
| 06-01-01 | 01 | 2 | WORK-01/WORK-02/WORK-07 | T-06-01-01 | Worker config exposes required settings without leaking secrets | config/CLI | `cargo test -p parser-worker config && cargo test -p parser-cli worker_command` | W0 | pending |
| 06-01-02 | 01 | 2 | WORK-01/WORK-02 | T-06-01-02 | Runtime adapter remains outside parser-core | boundary | `cargo check -p parser-worker --all-targets && ! rg -n "lapin|aws_sdk_s3|tokio::signal" crates/parser-core crates/parser-contract` | W0 | pending |
| 06-02-01 | 02 | 3 | WORK-02/WORK-03/WORK-04 | T-06-02-01 | Raw checksum mismatch cannot produce a successful artifact | storage | `cargo test -p parser-worker storage checksum_mismatch artifact_key` | W0 | pending |
| 06-02-02 | 02 | 3 | WORK-04 | T-06-02-02 | Existing deterministic S3 key is reused only when stored bytes match | storage | `cargo test -p parser-worker artifact_write_existing_match artifact_write_existing_conflict` | W0 | pending |
| 06-03-01 | 03 | 3 | WORK-01/WORK-05/WORK-06/WORK-07 | T-06-03-01 | Manual ack occurs only after confirmed completed/failed result publication | AMQP | `cargo test -p parser-worker amqp_publish_confirm ack_after_confirm nack_on_publish_failure` | W0 | pending |
| 06-03-02 | 03 | 3 | WORK-01/WORK-07 | T-06-03-02 | Default prefetch is 1 and topology names are configurable | AMQP/config | `cargo test -p parser-worker amqp_config prefetch_defaults` | W0 | pending |
| 06-04-01 | 04 | 4 | WORK-01/WORK-02/WORK-03/WORK-04/WORK-05/WORK-06/WORK-07 | T-06-04-01 | Successful job writes minimal artifact and publishes artifact reference proof | processor | `cargo test -p parser-worker processor_success worker_artifact_matches_cli_minimal_bytes` | W0 | pending |
| 06-04-02 | 04 | 4 | WORK-03/WORK-06/WORK-07 | T-06-04-02 | Handled failures publish parse.failed and ack; publish failures requeue | processor | `cargo test -p parser-worker processor_failures ack_policy shutdown` | W0 | pending |
| 06-05-01 | 05 | 5 | WORK-01/WORK-02/WORK-03/WORK-04/WORK-05/WORK-06/WORK-07 | T-06-05-01 | Final gates and docs prove Phase 6 deliverables without Phase 7 scope creep | full gate | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps && scripts/coverage-gate.sh --check && scripts/fault-report-gate.sh && scripts/benchmark-phase5.sh --ci && git diff --check` | W0 | pending |

*Status: pending, green, red, flaky*

---

## Wave 0 Requirements

Existing Rust test infrastructure covers all phase requirements. Plan 01 creates the `parser-worker` crate before worker-specific tests run.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live RabbitMQ/S3 deployment names | WORK-01/WORK-02/WORK-05/WORK-06 | Real `server-2` deployment topology is intentionally config-owned and may differ from local defaults | Review README config table and override env/flags in a staging deployment later; Phase 6 automated tests use fakes and default names only. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 900s for normal code gates
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-02
