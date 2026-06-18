---
phase: quick-260617-v7d
plan: 01
subsystem: parser-worker-testing
tags: [testcontainers, golden-oracle, e2e, determinism, byte-exact, ci]
status: complete
requires:
  - parser_worker::runner::run_until_cancelled
  - parser_worker::config::WorkerConfigOverrides
  - parser_core::public_parse_replay
  - testcontainers-modules 0.15.0 (rabbitmq, minio)
provides:
  - golden_container_e2e regression oracle (#[ignore], full worker contract byte-exact)
  - shared parser-worker tests/common/mod.rs wiring helpers
  - committed valid-minimal.expected.json baseline (one baseline, two consumers)
  - fast in-process golden_artifact_bytes.rs consumer
  - scripts/capture-golden-replays.sh
  - master-only cd.yml golden-container-e2e CI job
affects:
  - .github/workflows/cd.yml (new master-only job; verify job + coverage gate untouched)
tech-stack:
  added:
    - "testcontainers-modules 0.15.0 (dev-dep, features: rabbitmq, minio)"
  patterns:
    - "drive the real worker via run_until_cancelled + CancellationToken (no signals/timers)"
    - "test declares AMQP topology + creates MinIO bucket before spawning the worker"
    - "one *.expected.json baseline consumed byte-exact by both the e2e and a fast in-process test"
    - "Docker / fixture / credential skip-guards keep cargo test --workspace green w/o Docker"
key-files:
  created:
    - crates/parser-worker/tests/golden_container_e2e.rs
    - crates/parser-worker/tests/common/mod.rs
    - crates/parser-core/tests/golden_artifact_bytes.rs
    - crates/parser-core/tests/common/golden_identity.rs
    - crates/parser-core/tests/fixtures/golden/expected/valid-minimal.expected.json
    - scripts/capture-golden-replays.sh
    - .planning/quick/260617-v7d-golden-container-e2e-regression-oracle-f/TEETH-PROOF.md
  modified:
    - crates/parser-worker/Cargo.toml
    - Cargo.lock
    - crates/parser-core/tests/fixtures/golden/manifest.json
    - .github/workflows/cd.yml
decisions:
  - "Worker writes public_parse_replay output (provenance-stripped); baseline + fast test must use public_parse_replay, not parse_replay — the e2e caught this byte mismatch during construction"
  - "MinIO creds passed via process env (AWS_ACCESS_KEY_ID/SECRET=minioadmin) not std::env::set_var, because the workspace forbids unsafe and set_var is unsafe on edition 2024; e2e skip-guards if creds absent"
  - "Manifest entry uses existing category 'normal' (free String, not an enum) with a byte_exact_artifact_baseline feature tag, not an invented category"
metrics:
  tasks_completed: 3
  files_created: 7
  files_modified: 4
  completed: 2026-06-17
---

# Quick Task 260617-v7d: Golden Container E2E Regression Oracle Summary

A testcontainers-backed `#[ignore]` worker e2e that boots ephemeral RabbitMQ + MinIO,
drives the real worker through `run_until_cancelled` + `CancellationToken`, and pins the
full observable contract byte-for-byte (S3 artifact bytes == committed baseline,
`parse.completed`/`parse.failed` shape, key/checksum/size, idempotency, checksum-mismatch,
artifact-conflict), with the same baseline reused by a fast in-process parser-core golden
test so parser drift fails without containers.

## What Was Built

- **`golden_container_e2e.rs`** — one `#[tokio::test(multi_thread)]` `#[ignore]` test that
  boots `MinIO` + `RabbitMq` via testcontainers-modules 0.15.0, builds a `WorkerConfig`
  pointed at the mapped ports (`s3_force_path_style=true`, `probes_enabled=false`),
  declares the AMQP topology + creates the bucket BEFORE spawning the worker, and asserts:
  byte-exact artifact == committed baseline; `completed` message contract (job_id,
  replay_id, source_checksum, key, artifact_checksum, size); idempotency (duplicate
  redelivery → both completed, single artifact); checksum-mismatch → `parse.failed`
  (`checksum.mismatch` / `ParseStage::Checksum` / `NotRetryable`); artifact-conflict →
  `parse.failed` (`output.artifact_conflict` / `ParseStage::Output`). Loop ends only via
  `shutdown.cancel()` + a 10s join timeout — no signal handlers, no real timers.
- **`tests/common/mod.rs`** — shared worker-test wiring promoted from `live_smoke.rs`
  (`s3_client`, `ensure_bucket`, `put_raw_object`, `prepare_broker`, `spawn_worker`,
  `stop_worker`, `publish_job`, `wait_for_completed`/`wait_for_failed`,
  `fetch_artifact_bytes`, `count_artifact_keys`, `put_conflicting_artifact`,
  `assert_queue_empty`) — one definition, no duplication.
- **`valid-minimal.expected.json`** — committed byte-exact baseline = the exact bytes the
  worker writes (`serde_json::to_vec(&public_parse_replay(..))` + trailing `b'\n'`),
  consumed by BOTH the container e2e and the fast in-process test.
- **`golden_artifact_bytes.rs`** — fast in-process parser-core test: parse the seed fixture
  through `public_parse_replay`, serialize +`\n`, byte-compare against the same baseline.
- **`golden_identity.rs`** — single source-of-truth pinned identity (replay_id, object_key,
  checksum hex, parser name) `include!`d by both crates so the embedded bytes always agree.
- **`scripts/capture-golden-replays.sh`** — deterministic `~/sg_stats` capture (sorted +
  pinned selection, gzip, size guard, baseline regen note); no-ops with a clear message
  when `~/sg_stats` is absent (it is, on this machine).
- **`cd.yml`** — master-only `needs: verify` job runs the `#[ignore]` e2e with `--ignored`
  under Docker (ubuntu-latest), exporting MinIO creds; the existing `verify` job and the
  coverage gate are untouched.

## Verification

- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo test --workspace` (no Docker path) — green; the e2e is `#[ignore]` and skips
  cleanly; the fast `golden_artifact_bytes` byte-compare passes.
- `cargo test -p parser-worker --test golden_container_e2e -- --ignored` with Docker —
  **green** end-to-end against real RabbitMQ + MinIO (run on the authoring machine).
- `sh -n scripts/capture-golden-replays.sh` parses; runs as a no-op with `~/sg_stats` absent.
- `cd.yml` is valid YAML; the master-only e2e job is `needs: verify`.
- TEETH-PROOF.md records Mutation A (baseline byte flip → fast + container e2e RED) and
  Mutation B (behavioral parse-path change → fast RED), both reverted; tree clean.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Baseline pinned to the wrong parser entrypoint**
- **Found during:** Task 3 (first green run of the container e2e)
- **Issue:** The plan said to generate the baseline from `parser_core::parse_replay` and
  noted the worker uses `public_parse_replay` "same artifact". They are NOT byte-identical:
  `parse_replay` retains per-field `source` provenance, while the worker writes
  `public_parse_replay` output (provenance stripped, processor.rs:136). The container e2e's
  S3-bytes==baseline assertion failed, surfacing the mismatch — exactly the drift the oracle
  exists to catch.
- **Fix:** Regenerated `valid-minimal.expected.json` via `public_parse_replay` and switched
  `golden_artifact_bytes.rs` to `public_parse_replay` so both consumers match the real worker
  output byte-for-byte.
- **Files modified:** `crates/parser-core/tests/fixtures/golden/expected/valid-minimal.expected.json`, `crates/parser-core/tests/golden_artifact_bytes.rs`
- **Commit:** 56c2605

**2. [Rule 3 - Blocking] MinIO credentials without `unsafe`**
- **Found during:** Task 1 (clippy/build)
- **Issue:** The plan suggested setting AWS creds via `std::env::set_var`, but the workspace
  forbids `unsafe` and `set_var` is `unsafe` on edition 2024.
- **Fix:** The e2e reads creds from the process env (the runner / CI job exports
  `AWS_ACCESS_KEY_ID=minioadmin`, `AWS_SECRET_ACCESS_KEY=minioadmin`) and adds a clean
  skip-guard when they are absent. The cd.yml job sets them via `env:`.
- **Files modified:** `crates/parser-worker/tests/golden_container_e2e.rs`, `.github/workflows/cd.yml`
- **Commit:** 28274e8 / b85905c

### Environment note (not a deviation)
The container e2e initially failed pulling `rabbitmq:3.8.22-management` with a registry TLS
handshake timeout (transient, VPN-adjacent). Pre-pulling the image resolved it; subsequent
runs are green. The CI job has a generous `timeout-minutes: 30` for first-run image pulls.

## Commits

- `28274e8` test(quick-260617-v7d): testcontainers worker e2e + shared helpers
- `b85905c` test(quick-260617-v7d): fast byte-exact consumer + capture script + CI job
- `56c2605` fix(quick-260617-v7d): pin baseline to worker output + prove oracle teeth

## Self-Check: PASSED

All created files exist on disk; all three commit hashes are in `git log`. Working tree
contains only pre-existing untracked files (BRAINSTORM/RESEARCH docs, `test_float_cmp`) that
are outside this task's code scope.
