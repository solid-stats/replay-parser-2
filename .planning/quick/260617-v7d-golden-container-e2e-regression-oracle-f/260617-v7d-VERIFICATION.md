---
phase: quick-260617-v7d
verified: 2026-06-17T17:30:00Z
status: passed
score: 7/7
behavior_unverified: 0
overrides_applied: 0
---

# Quick Task 260617-v7d: Golden Container E2E Regression Oracle — Verification Report

**Task Goal:** A golden container-e2e regression oracle — a testcontainers worker e2e (ephemeral RabbitMQ + MinIO) that pins the full observable contract byte-for-byte, asserts idempotency + failure paths, runs as an #[ignore] master-only pre-deploy gate, mirrored by a fast in-process byte-exact consumer of the SAME baseline, with teeth proven by an injected mutation turning it red.
**Verified:** 2026-06-17T17:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `cargo test -p parser-worker -- --ignored` with Docker boots ephemeral RabbitMQ + MinIO, drives the real worker via `run_until_cancelled` + `CancellationToken`, and the worker-written S3 artifact bytes equal a committed `*.expected.json` byte-for-byte | VERIFIED | `golden_container_e2e.rs:229-284`: `#[ignore]` test, `spawn_worker` calls `run_until_cancelled` (common/mod.rs:161), `assert_eq!(fetched, EXPECTED_BASELINE)` at line 118. TEETH-PROOF.md records green end-to-end run under Docker. |
| 2 | The e2e asserts the full observable contract: `parse.completed`/`parse.failed` message shape, S3 artifact key+checksum+size, idempotency (duplicate redelivery → terminal once, single artifact), checksum-mismatch → `parse.failed`, artifact-conflict → `parse.failed` (`output.artifact_conflict` / `ParseStage::Output`) | VERIFIED | `assert_success_contract` (lines 100-137) asserts key, checksum, size, job_id, replay_id, source_checksum. `assert_idempotency` (lines 140-170) publishes twice, checks single artifact via `count_artifact_keys`. `assert_checksum_mismatch_failure` (lines 173-193) asserts `"checksum.mismatch"` / `ParseStage::Checksum` / `NotRetryable`. `assert_artifact_conflict_failure` (lines 199-226) pre-seeds conflicting bytes and asserts `"output.artifact_conflict"` / `ParseStage::Output`. |
| 3 | `cargo test --workspace` (no `--ignored`) stays green with Docker absent: the e2e is `#[ignore]` and skips cleanly | VERIFIED | Ran `cargo test --workspace` — result: all ok, `1 ignored` in parser-worker. Ran `cargo test -p parser-worker --test golden_container_e2e` — output: `test result: ok. 0 passed; 0 failed; 1 ignored` with message "requires Docker; boots ephemeral RabbitMQ + MinIO via testcontainers". |
| 4 | The same committed `*.expected.json` is byte-compared by a fast in-process parser-core golden test, so parser drift fails immediately without containers | VERIFIED | `golden_artifact_bytes.rs:36` uses `include_bytes!("fixtures/golden/expected/valid-minimal.expected.json")` — the SAME file as the e2e (`include_bytes!("../../parser-core/tests/fixtures/golden/expected/valid-minimal.expected.json")`). Both `include!` the same `golden_identity.rs`. `cargo test -p parser-core --test golden_artifact_bytes` passes: `1 passed; 0 failed`. |
| 5 | A documented mutation (one file-byte in `*.expected.json`, one behavioral in parse output) turns the oracle red — teeth proven on the record | VERIFIED | `TEETH-PROOF.md` records Mutation A: flipped one byte in `valid-minimal.expected.json` (`"Altis"→"Altiz"`) — both fast consumer and container e2e went RED. Mutation B: renamed `"worldName"` to `"worldNameX"` in `metadata.rs:36` — fast consumer went RED. Both reverted; tree clean. |
| 6 | The strict coverage gate is unaffected: `coverage-gate.sh` runs `llvm-cov` without `--ignored`, so the e2e never executes under coverage | VERIFIED | `grep --ignored scripts/coverage-gate.sh` returns no output — `--ignored` is never passed. The `verify` CI job step at `cd.yml:45` runs `cargo +"$RUST_VERSION" test --workspace` (no `--ignored`). The new `golden-container-e2e` job is a SEPARATE job. |
| 7 | A deterministic capture script lets a human add real `~/sg_stats` replays later without breaking verify | VERIFIED | `scripts/capture-golden-replays.sh` exists (84 lines, syntax-clean via `sh -n`). Lines 27-32: `if [ ! -d "$SG_STATS_DIR" ]; then echo ... exit 0; fi` — no-ops with presence note when `~/sg_stats` absent. Sorted glob + fixed indices for deterministic selection; size guard at 256 KiB; gzip into `crates/parser-worker/tests/fixtures/real/`. |

**Score:** 7/7 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Min Lines | Actual Lines | Status | Details |
|----------|----------|-----------|-------------|--------|---------|
| `crates/parser-worker/tests/golden_container_e2e.rs` | `#[ignore]` testcontainers worker e2e with all 4 assertion paths | 130 | 284 | VERIFIED | All assertion functions present and called; `#[ignore]` confirmed; Docker/fixture/creds skip-guards present |
| `crates/parser-worker/tests/common/mod.rs` | Shared helpers mirrored from `live_smoke.rs` | 80 | 329 | VERIFIED | `s3_client`, `ensure_bucket`, `put_raw_object`, `prepare_broker`, `spawn_worker`, `stop_worker`, `publish_job`, `wait_for_completed`, `wait_for_failed`, `fetch_artifact_bytes`, `count_artifact_keys`, `put_conflicting_artifact`, `assert_queue_empty` all present |
| `crates/parser-core/tests/fixtures/golden/expected/valid-minimal.expected.json` | Committed byte-exact baseline | — | 826 bytes | VERIFIED | File exists (826 bytes), consumed via `include_bytes!` by both consumers |
| `crates/parser-core/tests/golden_artifact_bytes.rs` | Fast in-process golden test, `public_parse_replay`, byte-compare | 30 | 85 | VERIFIED | Uses `public_parse_replay`, shares `golden_identity.rs` via `include!`, byte-compare passes |
| `scripts/capture-golden-replays.sh` | Deterministic capture script, no-op when `~/sg_stats` absent | 20 | 84 | VERIFIED | Syntax-clean; no-op exit 0 when `~/sg_stats` absent; sorted selection; size guard |
| `.github/workflows/cd.yml` | Master-only pre-deploy CI job `needs: verify`; `verify` job untouched | — | — | VERIFIED | `golden-container-e2e` job at line 47: `needs: verify`, `if: ... github.ref == 'refs/heads/master'`, runs `-- --ignored`; `verify` job at line 24 unchanged |
| `.planning/quick/260617-v7d-golden-container-e2e-regression-oracle-f/TEETH-PROOF.md` | Recorded mutations turning oracle red | — | 115 lines | VERIFIED | Mutation A (byte flip) and Mutation B (behavioral) documented; Docker run recorded; both reverted; tree clean |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `golden_container_e2e.rs` | `runner.rs` | `run_until_cancelled(config, shutdown)` via `common::spawn_worker` | VERIFIED | `common/mod.rs:161`: `tokio::spawn(async move { run_until_cancelled(config, shutdown).await })` |
| `golden_container_e2e.rs` | `processor.rs` | S3 artifact bytes == committed baseline (`serde_json::to_vec(&artifact)+\n`) | VERIFIED | `assert_eq!(fetched, EXPECTED_BASELINE)` at e2e line 118; `golden_artifact_bytes.rs` uses `public_parse_replay` matching `processor.rs:136` path |
| `golden_container_e2e.rs` | `config.rs` | `WorkerConfig::from_env_and_overrides` with container ports | VERIFIED | Lines 83-96: `WorkerConfig::from_env_and_overrides(|_| None, WorkerConfigOverrides { amqp_url, s3_endpoint, s3_force_path_style: Some(true), probes_enabled: Some(false), ... })` |
| `golden_artifact_bytes.rs` | `valid-minimal.expected.json` | `include_bytes!` byte-compare against committed baseline | VERIFIED | `include_bytes!("fixtures/golden/expected/valid-minimal.expected.json")` at line 36; `assert_eq!(produced, EXPECTED_BASELINE)` at line 80 |
| `cd.yml` | `golden_container_e2e.rs` | master-only job: `cargo ... test -p parser-worker --test golden_container_e2e -- --ignored` | VERIFIED | `cd.yml:72`: exact command; job gated on `refs/heads/master`, `needs: verify` |

### Manifest Entry Verification

| Field | Requirement | Actual Value | Status |
|-------|------------|-------------|--------|
| `category` | `"normal"` (existing vocabulary) | `"normal"` | VERIFIED |
| `fixture_strategy` | `"linked_existing_focused_fixture"` | `"linked_existing_focused_fixture"` | VERIFIED |
| `expected_status` | `"success"` | `"success"` | VERIFIED |
| `expected_features` | contains `byte_exact_artifact_baseline` | `["byte_exact_artifact_baseline", "deterministic_serialization", "shared_golden_identity", "container_e2e_oracle"]` | VERIFIED |
| `cross_app_impact` | all three fields non-empty | `parser_artifact`, `server_2`, `web` all non-empty | VERIFIED |
| `decisions` | non-empty | `["D-08", "D-13"]` | VERIFIED |
| `source.notes` | non-empty, states shared baseline purpose | Non-empty; explains dual-consumer usage | VERIFIED |
| `fixture` path | existing file | `tests/fixtures/valid-minimal.ocap.json` (exists) | VERIFIED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| e2e is `#[ignore]` and skips without Docker | `cargo test -p parser-worker --test golden_container_e2e` | `1 ignored`, exit 0 | PASS |
| Fast golden test passes in-process | `cargo test -p parser-core --test golden_artifact_bytes` | `1 passed; 0 failed`, exit 0 | PASS |
| Workspace test suite green without Docker | `cargo test --workspace` | All ok, 0 failed | PASS |
| Capture script syntax-clean | `sh -n scripts/capture-golden-replays.sh` | exit 0, no errors | PASS |
| `cd.yml` valid YAML | `python3 yaml.safe_load(...)` | `cd.yml YAML valid` | PASS |
| Coverage gate never passes `--ignored` | `grep --ignored scripts/coverage-gate.sh` | no output | PASS |

### Anti-Patterns Found

None. No `TBD`, `FIXME`, or `XXX` markers in any of the 7 created/modified files.

### Human Verification Required

None. All must-haves are verified programmatically. The Docker-required e2e path is adequately documented by TEETH-PROOF.md (which records a real green Docker run on the authoring machine) and will be exercised by the master-only CI job on the first push to master.

### Notable Deviations from Plan (Auto-Fixed, No Gaps)

1. **Baseline uses `public_parse_replay` not `parse_replay`**: The PLAN specified `parse_replay` but the worker actually calls `public_parse_replay` (provenance stripped). The executor caught this via the container e2e byte-mismatch assertion and fixed it. The deviation is correct and the baseline now matches real worker output.

2. **MinIO creds via process env, not `std::env::set_var`**: The 2024 edition forbids `unsafe`, and `set_var` is unsafe. The e2e reads from the process env and adds a skip-guard when creds are absent; the CI job sets them via `env:`. This is strictly correct.

---

_Verified: 2026-06-17T17:30:00Z_
_Verifier: Claude (gsd-verifier)_
