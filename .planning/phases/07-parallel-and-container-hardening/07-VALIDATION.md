---
phase: 07
slug: parallel-and-container-hardening
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-03
---

# Phase 07 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust Cargo tests, async Tokio integration tests, assert_cmd CLI tests, Docker Compose live smoke |
| **Config file** | `Cargo.toml`, `crates/parser-worker/Cargo.toml`, `Dockerfile`, `docker-compose.worker-smoke.yml`, `scripts/worker-smoke.sh`, `scripts/coverage-gate.sh`, `scripts/fault-report-gate.sh` |
| **Quick run command** | `cargo test -p parser-worker -p parser-cli` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps && scripts/coverage-gate.sh --check && scripts/fault-report-gate.sh && scripts/benchmark-phase5.sh --ci && scripts/worker-smoke.sh` |
| **Estimated runtime** | ~1200 seconds including Docker smoke; full-corpus benchmark may dominate runtime |

---

## Sampling Rate

- **After every task commit:** Run the task's narrow `cargo test` or script command from its `<verify>` block.
- **After every plan wave:** Run `cargo test -p parser-worker -p parser-cli`.
- **Before `$gsd-verify-work`:** Full suite and Docker smoke must be green, or any missing external Timeweb credentials must be documented as manual-only.
- **Max feedback latency:** 1200 seconds for full phase gates; normal task feedback should stay under 180 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 07-00-01 | 00 | 1 | WORK-08 | T-07-00-01 | Concurrent workers cannot overwrite a deterministic artifact key without compare/reuse/conflict handling | storage | `cargo test -p parser-worker storage conditional_put artifact_write_existing_match artifact_write_existing_conflict` | W0 | pending |
| 07-00-02 | 00 | 1 | WORK-08 | T-07-00-02 | Duplicate/redelivered jobs reuse matching artifacts and republish durable completion | processor | `cargo test -p parser-worker processor_duplicate_redelivery processor_artifact_conflict` | W0 | pending |
| 07-01-01 | 01 | 1 | WORK-09 | T-07-01-01 | Probe config validates bind address/port and does not leak secrets | config/CLI | `cargo test -p parser-worker config health && cargo test -p parser-cli worker_command` | W0 | pending |
| 07-01-02 | 01 | 1 | WORK-09 | T-07-01-02 | `/readyz` is false on startup/dependency failure/shutdown and true only after AMQP/S3 readiness | async health | `cargo test -p parser-worker health runner_readiness shutdown` | W0 | pending |
| 07-02-01 | 02 | 2 | WORK-08/WORK-09 | T-07-02-01 | Stable logs include worker/job/stage/artifact/result/ack fields and exclude secrets | log taxonomy | `cargo test -p parser-worker log_taxonomy config` | W0 | pending |
| 07-02-02 | 02 | 2 | WORK-08/WORK-09 | T-07-02-02 | Workers use `REPLAY_PARSER_WORKER_ID` or hostname identity in logs/probes | config/logs | `cargo test -p parser-worker worker_identity log_taxonomy` | W0 | pending |
| 07-03-01 | 03 | 3 | WORK-08/WORK-09 | T-07-03-01 | Docker image runs non-root with health wiring and two worker instances pass probes | container smoke | `scripts/worker-smoke.sh` | W0 | pending |
| 07-03-02 | 03 | 3 | WORK-08 | T-07-03-02 | Two-worker smoke proves duplicate/redelivery and artifact conflict behavior | live smoke | `scripts/worker-smoke.sh` | W0 | pending |
| 07-03-03 | 03 | 3 | WORK-08/WORK-09 | T-07-03-03 | Timeweb endpoint/path-style/signature config is documented and conditional write support has fallback | provider compatibility | `rg -n "Timeweb|s3.twcstorage.ru|REPLAY_PARSER_S3_FORCE_PATH_STYLE|conditional" README.md scripts/worker-smoke.sh .planning/phases/07-parallel-and-container-hardening` | W0 | pending |
| 07-04-01 | 04 | 4 | WORK-08/WORK-09 | T-07-04-01 | Final gates prove Phase 7 deliverables without parser-core/contract transport leakage or secret leakage | full gate | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps && scripts/coverage-gate.sh --check && scripts/fault-report-gate.sh && scripts/benchmark-phase5.sh --ci && scripts/worker-smoke.sh && git diff --check` | W0 | pending |

*Status: pending, green, red, flaky*

---

## Wave 0 Requirements

Existing Rust test infrastructure, worker fake adapters, and live smoke infrastructure cover all phase requirements. Plan 01 adds the HTTP probe server dependency and tests before probe behavior is consumed by Plan 03.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live Timeweb Cloud S3 conditional-write capability | WORK-08/WORK-09 | Requires real Timeweb bucket credentials and must not be committed or logged | Run the Timeweb smoke mode or documented AWS CLI command with `REPLAY_PARSER_S3_ENDPOINT=https://s3.twcstorage.ru`, `REPLAY_PARSER_S3_FORCE_PATH_STYLE=true`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and a disposable bucket/object key. Confirm whether conditional put returns success/412/409-compatible behavior; if not, record that compare/reuse/conflict fallback is the supported deployment path. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 1200s for normal full gates
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-03
