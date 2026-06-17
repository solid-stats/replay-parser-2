# Quick Task: Golden Container E2E Regression Oracle — Research

**Researched:** 2026-06-17
**Domain:** testcontainers-backed Rust worker e2e (RabbitMQ + MinIO), full-byte artifact pinning
**Confidence:** HIGH on call path / wiring / coverage (code-verified); MEDIUM on testcontainers exact API (docs.rs + source, context7 unavailable this session); MEDIUM on fixtures (`~/sg_stats` missing on this machine — capture script mandatory)

## Summary

The worker's observable contract is already exercised end-to-end by `crates/parser-worker/tests/live_smoke.rs`, but only as an env-gated manual smoke against *operator-provided* infra. The oracle is that same wiring, with two upgrades: (1) the broker + object store are booted by `testcontainers-modules` (RabbitMQ + MinIO) inside the test, so no env setup; (2) the success path asserts the S3 artifact **bytes equal a committed `*.expected.json` exactly** instead of just checking existence/checksum-roundtrip. Everything else — topology declaration, worker spawn via `run_until_cancelled` + `CancellationToken`, the duplicate/checksum-mismatch/conflict scenarios, queue-empty after ack — already exists in `live_smoke.rs` and should be mirrored, not reinvented.

The real call path is fully traced below (principle 3). The worker does NOT declare its own AMQP topology — `RabbitMqClient::connect` only does `basic_qos` + `basic_consume` on a pre-existing `config.job_queue` and publishes to a pre-existing `config.result_exchange` (`amqp.rs:160-189`). So the test harness must declare the job queue, result exchange, and result-capture queues *before* spawning the worker, exactly as `prepare_broker` does (`live_smoke.rs:198-231`). Same for the MinIO bucket: the worker assumes the bucket exists (`storage.rs:212` `head_bucket` in `check_ready` is a hard readiness gate), so the test must create it first (`ensure_bucket`, `live_smoke.rs:167-182`).

**Primary recommendation:** Add a new `#[ignore]` test file `crates/parser-worker/tests/golden_container_e2e.rs` that boots `testcontainers_modules::{rabbitmq::RabbitMq, minio::MinIO}` via `AsyncRunner`, builds a `WorkerConfig` pointing at the mapped ports with `s3_force_path_style=true` and an explicit `s3_endpoint`, reuses the `live_smoke.rs` topology/spawn/assert helpers, and adds byte-exact `*.expected.json` comparison. Generate the expected baselines from `parser_core::parse_replay` (the exact bytes the processor writes: `serde_json::to_vec(&artifact)` + trailing `\n` — see `processor.rs:149-150`) and reuse them in the fast in-process golden suite. Fixtures need a capture script because `~/sg_stats` is absent here.

## User Constraints (from BRAINSTORM — locked, do not re-litigate)

### Locked Decisions
- **Full container e2e of the worker** via `testcontainers` (RabbitMQ + MinIO). Docker required on the runner.
- **Branch off `master`** (`test/golden-container-e2e-oracle`), runs as a **separate `master`-only pre-deploy gate**, NOT the fast verify/per-phase loop.
- **`#[ignore]` + `cargo test -- --ignored`**; stays out of the fast suite and the coverage gate.
- **Drop old-vs-new parity.** Pure Rust-output-vs-committed-Rust-baseline oracle. No Node/pnpm, no `crates/parser-harness`, no `compare` command (AGENTS.md bars reviving them).
- **Pin current bytes as-is.** Known tech-debt is pinned + commented, never "fixed" inside the oracle.
- **Full artifact bytes, exact compare.** One committed `*.expected.json` per fixture.
- **Drive `run_until_cancelled` + `CancellationToken`** with live `S3ObjectStore`/`RabbitMqClient`; cancel the token after the job. No signal handlers, no real timers.
- **Real OCAP fixtures from `~/sg_stats`**, gzipped, spread across success/partial/failed + one large-entity replay.
- **Reuse the same `*.expected.json` baselines in the fast in-process golden suite.**
- **Prove teeth:** an injected behavioral mutation must turn the oracle red.

### Deferred Ideas (OUT OF SCOPE)
- Shrinking the live-adapter coverage allowlist using a coverage-instrumented e2e run (the e2e runs outside coverage).
- Production deploy / live Timeweb S3 validation (deployer-run).

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| (quick task, no REQ IDs) | Container e2e oracle pinning the worker contract | Call path §1, wiring §2, baseline §3, fixtures §4, isolation §5, pitfalls §6 below |

---

## 1. Real Worker Call Path (principle 3 — mandatory)

Entry: `parser_worker::runner::run_until_cancelled(config, shutdown)` (`runner.rs:46-51`) → `run_with_shutdown(config, shutdown, listen_for_ctrl_c=false)` (`runner.rs:53`). With `listen_for_ctrl_c=false` **no ctrl-c listener is spawned** (`runner.rs:65-68`) — shutdown is *only* via the passed `CancellationToken`. This is the seam the test drives.

Startup sequence (all before any job is consumed):

| Step | Op | Code | Notes for the test |
|------|----|------|--------------------|
| 1 | `config.validate()` | `runner.rs:60` | fail-fast on bad config |
| 2 | `spawn_probe_server` (HTTP /livez /readyz) | `runner.rs:76` | bound by `config.probe_port`; **set `probes_enabled=false`** to avoid port collisions across parallel tests |
| 3 | `S3ObjectStore::from_config` | `runner.rs:85`, `storage.rs:184-201` | builds aws-sdk-s3 client: `endpoint_url(config.s3_endpoint)`, `force_path_style(config.s3_force_path_style)`, region `config.s3_region` |
| 4 | `store.check_ready()` → **S3 `head_bucket`** | `runner.rs:94`, `storage.rs:211-226` | **hard gate** — bucket must already exist or worker returns `WorkerError` and exits |
| 5 | `RabbitMqClient::connect` → AMQP connect + 2 channels + `basic_qos(prefetch)` + `basic_consume(config.job_queue, "replay-parser-2-worker", no_ack:false)` + publish channel `confirm_select` | `runner.rs:103`, `amqp.rs:160-189` | **worker does NOT declare the queue/exchange** — they must pre-exist |

Per-delivery loop `consume_until_shutdown` (`runner.rs:142-198`):

- `tokio::select!` over `shutdown.cancelled()` vs `rabbit.consumer_mut().next()` (`runner.rs:157-163`). Cancelling the token breaks the loop cleanly — this is the seam (principle 9). After processing each delivery it re-checks `shutdown.is_cancelled()` (`runner.rs:186`).
- `process_job_body(&delivery.data, config, store, rabbit, parser)` (`runner.rs:182-183`) then `apply_lapin_delivery_action` (ack/nack) (`runner.rs:184`).

`process_job_body` → `process_decoded_job` (`processor.rs:66-183`) — every external read/write, in order, keyed by trigger:

| # | Op (tier) | Trigger | Code | Outcome |
|---|-----------|---------|------|---------|
| A | `serde_json::from_slice::<ParseJobMessage>` | always | `processor.rs:73` | malformed → `publish_failed` (`json.decode`/`schema.parse_job`), ACK |
| B | field-empty validation | decoded | `processor.rs:91`, `validate_job_fields:256` | empty job_id/replay_id/object_key → `parse.failed` `schema.parse_job`, ACK |
| C | contract-version check | decoded | `processor.rs:104` | mismatch → `parse.failed` `unsupported_contract_version`, ACK |
| D | **S3 GET** `store.download_raw(job.object_key)` → `get_object` + collect body, **then computes SHA-256 of downloaded bytes** | always reaches here on valid job | `processor.rs:116`, `storage.rs:111-117` + `243-245` + `320-362` | not-found → `parse.failed` `io.s3_read` (ObjectNotFound); other → `io.s3_read` |
| E | **checksum verify** `verify_source_checksum(downloaded.bytes, job.checksum)` | after download | `processor.rs:124`, `checksum.rs:verify_source_checksum` | mismatch → `parse.failed` `checksum.mismatch`, stage `Checksum`, `NotRetryable` |
| F | `parser_core::public_parse_replay(ParserInput{bytes, source, parser, options})` (pure, no I/O) | checksum ok | `processor.rs:136-141` | `ParseStatus::Failed` → `parse.failed` with parser failure (`processor.rs:144-147`) |
| G | serialize artifact: `serde_json::to_vec(&artifact)` **then push `b'\n'`** | parse success/partial | `processor.rs:149-150` | **these are the exact bytes written to S3 and pinned** |
| H | build **S3 artifact key** `artifact_key(config.artifact_prefix, job.replay_id, job.checksum)` = `{prefix}/{pct-encoded replay_id}/{source_sha256}.json` | after serialize | `processor.rs:151`, `artifact_key.rs` | err → `parse.failed` `output.artifact_key` |
| I | **S3 PUT (conditional)** `store.write_artifact_if_absent_or_matching(key, bytes)` | after key | `processor.rs:160`, `storage.rs:120-168` + `281-316` | uses `put_object().if_none_match("*")`. Created → write. Precondition-failed/409/412 → **S3 GET existing**, compare bytes; equal → `reused_existing`; differ → `parse.failed` `output.artifact_conflict` stage `Output`. `NotImplemented`/`InvalidRequest if-none-match` → fallback GET-then-PUT |
| J | **AMQP publish** `publisher.publish_completed(ParseCompletedMessage)` to `config.result_exchange` / `config.completed_routing_key`, confirmed | write ok | `processor.rs:172-182`, `amqp.rs:216-235` | publish failure → `NackRequeue` |
| (J') | `publish_failed(ParseFailedMessage)` → `config.failed_routing_key`, confirmed | any failure branch above | `amqp.rs:242-258` | |
| K | ack/nack the original delivery | after publish | `runner.rs:184`, `delivery_action_after_publish` | confirmed publish → ACK |

**Checksum locations to remember:** the *source* checksum is computed on the downloaded raw bytes inside `download_raw` (`storage.rs:114`) and verified at step E; the *artifact* checksum + size are computed inside `write_artifact_if_absent_or_matching` (`storage.rs:126-127`) and travel into `ParseCompletedMessage`. **Artifact key** is built at `processor.rs:151`.

**What the fakes/asserts must cover (every tier):**
- **AMQP in:** publish `ParseJobMessage` to the job queue (real broker). Mirror `publish_job` (`live_smoke.rs:266-297`).
- **S3 in:** pre-seed the raw object at `job.object_key` (real MinIO). Mirror `live_smoke.rs:61-68`.
- **S3 out:** GET the artifact back at the deterministic key and assert **bytes == committed `*.expected.json`**, plus `artifact_checksum`/`artifact_size_bytes` from the completed message match those bytes. Extends `assert_artifact_exists` (`live_smoke.rs:376-391`) with byte-equality.
- **AMQP out:** consume from a bound result queue, assert `ParseCompletedMessage`/`ParseFailedMessage` contract (job_id, replay_id, source_checksum, artifact.key, artifact_checksum, size, error_code/stage/retryability). Mirror `wait_for_completed`/`wait_for_failed` (`live_smoke.rs:337-357`).
- **Idempotency:** publish the same job twice → both completed, same key/checksum/size, single object at the key. Mirror `assert_duplicate_completed_results` + `assert_single_artifact_key` (`live_smoke.rs:299-316`, `393-410`), queue empty after ack (`assert_queue_empty`, `live_smoke.rs:459-469`).
- **Failure tiers:** checksum-mismatch job (`checksum.mismatch`/`Checksum`/`NotRetryable`) and pre-seeded conflicting artifact (`output.artifact_conflict`/`Output`). Both already in `live_smoke.rs:108-135`.

## 2. testcontainers Wiring

**Stack** `[VERIFIED: crates.io API]`: `testcontainers-modules` **0.15.0** (latest stable, published 2026-02-21). Modules gated behind features `rabbitmq` and `minio`. Add as dev-dep:

```toml
# crates/parser-worker/Cargo.toml [dev-dependencies]
testcontainers-modules = { version = "0.15", features = ["rabbitmq", "minio"] }
# (testcontainers core is re-exported as testcontainers_modules::testcontainers; AsyncRunner lives there)
```

**Module facts** `[CITED: docs.rs/testcontainers-modules + github source main]` (MEDIUM — context7 was unavailable; verified against docs.rs and the repo `src/{minio,rabbitmq}/mod.rs`. Pin the *actual* resolved version's constants when implementing, since image tags move):

| Module | Struct | Image | Tag (main) | Port | Ready (WaitFor) | Creds |
|--------|--------|-------|------------|------|-----------------|-------|
| MinIO | `testcontainers_modules::minio::MinIO` | `minio/minio` | `RELEASE.2025-02-28T09-55-16Z` | 9000 (API), 9001 (console) | `message_on_stderr("API:")` | access `minioadmin` / secret `minioadmin` (MinIO defaults) |
| RabbitMQ | `testcontainers_modules::rabbitmq::RabbitMq` | `rabbitmq` | `4.2-management` | 5672 (AMQP) | `"Server startup complete"` on stdout | default `guest`/`guest` |

**Async boot + port mapping** `[CITED: docs.rs MinIO example]`:

```rust
use testcontainers_modules::{minio::MinIO, rabbitmq::RabbitMq,
    testcontainers::runners::AsyncRunner};

let minio = MinIO::default().start().await?;          // ContainerAsync<MinIO>
let s3_host = minio.get_host().await?;
let s3_port = minio.get_host_port_ipv4(9000).await?;
let s3_endpoint = format!("http://{s3_host}:{s3_port}");

let rabbit = RabbitMq::default().start().await?;
let amqp_port = rabbit.get_host_port_ipv4(5672).await?;
let amqp_url = format!("amqp://guest:guest@{}:{amqp_port}/%2f",
    rabbit.get_host().await?);
```

Hold the `ContainerAsync` handles in scope for the whole test — drop = container stop. The `multi_thread` tokio runtime in `live_smoke.rs:44` is the right flavor.

**Flowing endpoints into the worker** — build `WorkerConfig` via `from_env_and_overrides` (avoids env mutation, matches the unit tests at `runner.rs:326`):

```rust
let config = WorkerConfig::from_env_and_overrides(|_| None, WorkerConfigOverrides {
    amqp_url: Some(amqp_url),
    s3_bucket: Some("solid-replays".to_owned()),
    s3_endpoint: Some(s3_endpoint),
    s3_force_path_style: Some(true),     // MinIO requires path-style
    s3_region: Some("us-east-1".to_owned()),
    probes_enabled: Some(false),         // avoid probe port collisions
    prefetch: Some(1),
    worker_id: Some("e2e-oracle".to_owned()),
    ..Default::default()
})?;
```

**AWS creds:** the SDK reads `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY` from its standard chain (`config.rs:1-6` docstring) — set them to `minioadmin`/`minioadmin` for the test process (or via a credentials provider on the client). `S3ObjectStore::from_config` (`storage.rs:188-200`) already wires `endpoint_url` + `force_path_style` from config exactly as `s3_client` in `live_smoke.rs:150-161`.

**Topology the test must declare before spawning the worker** (worker declares none — `amqp.rs` confirmed): job queue (durable), result exchange (direct, durable), and two result queues bound on `completed_routing_key`/`failed_routing_key`. **Reuse `prepare_broker`/`declare_result_queue` verbatim** (`live_smoke.rs:198-257`). Bucket: reuse `ensure_bucket` (`live_smoke.rs:167-182`).

**Spawn/stop the worker** — reuse `spawn_worker` (`live_smoke.rs:259-264`: `tokio::spawn(run_until_cancelled(config, shutdown))`) and the clean-shutdown block (`live_smoke.rs:140-147`: `shutdown.cancel()` then `timeout(10s, worker)`).

## 3. Expected-Artifact Baseline

The exact bytes written to S3 (step G) are `serde_json::to_vec(&artifact)` **plus a trailing `\n`** (`processor.rs:149-150`). The committed `*.expected.json` must equal those bytes byte-for-byte (including the newline). Determinism is guaranteed by parser-core (convention §C; `deterministic_output.rs` proves re-serialization equality).

**Public entry to regenerate baselines:** `parser_core::parse_replay(ParserInput{ bytes, source, parser, options })` (used by `golden_fixture_behavior.rs:75`; the worker uses `public_parse_replay`, same artifact). The baseline generator must reproduce the worker's `ParserInput`:
- `source = ReplaySource { replay_id: Some(job.replay_id), source_file: job.object_key, checksum: Present(job.checksum) }` (`processor.rs:129-133`)
- `parser = ParserInfo { name:"replay-parser-2", version: CARGO_PKG_VERSION }` (`runner.rs:283-289`)
- `options = ParserOptions::default()`

⚠ **The artifact embeds `parser.version` and the source identity (replay_id/source_file/checksum).** The expected bytes therefore depend on those exact values. Pin the fixture's `replay_id`/`object_key`/`checksum` as constants shared between the baseline generator, the committed `*.expected.json`, and the e2e job message — otherwise the bytes drift on every version bump. Recommend a `#[ignore]`-able `--regenerate` test or a small `xtask`/script that writes the baselines so refresh is one command (mirrors `insta` review ergonomics from the tests skill).

**Reuse in the fast suite (no duplication):** add a manifest entry per new real fixture in `crates/parser-core/tests/fixtures/golden/manifest.json` (the existing pattern: entries *link* fixtures, never duplicate payloads — see all 12 current entries) and a `golden_fixture_behavior.rs` assertion that parses through `parse_replay` and compares serialized-bytes-plus-newline to the *same* committed `*.expected.json`. One baseline file, consumed by both layers. This is what makes parser drift fail immediately during Phases 9-12 without containers.

## 4. Real Fixtures from ~/sg_stats

**`~/sg_stats` does NOT exist on this machine** `[VERIFIED: test -d ~/sg_stats → MISSING]`. So the capture-script + presence-skip-guard (brief principle 8) is **mandatory**, not a fallback. `verify` must stay green without the fixtures.

Existing fixture convention (`crates/parser-core/tests/fixtures/`): tiny hand-focused `*.ocap.json` (16 B–1.7 KB), linked from `golden/manifest.json` with `fixture_strategy: "linked_existing_focused_fixture"`. The oracle adds a new *category* of fixture: **real OCAP** captured from `~/sg_stats`, gzipped.

**Recommended capture approach:**
1. A deterministic capture script (e.g. `scripts/capture-golden-replays.sh`) the human runs once with `~/sg_stats` present. It selects a small spread — one success, one partial (schema drift), one failed/malformed, one large-entity replay — by a *pinned, sorted, deterministic* selection (e.g. specific named files or a sorted glob + fixed indices), gzips each, and writes them under a new dir like `crates/parser-worker/tests/fixtures/real/` with a small `manifest.json` (real source filename, expected status, byte size). Determinism of selection matters so re-capture reproduces the same set.
2. The script then regenerates each `*.expected.json` baseline (§3) so the pair (gzipped input + expected output) is committed together.
3. **Size guard:** PROJECT.md notes every artifact ≤100 KB but raw replays vary widely. Gzip-commit only those under a threshold (e.g. ≤256 KB compressed); for any too large to commit, the script leaves a note and the e2e skip-guard covers absence.
4. **Presence skip-guard** in the e2e: if a real fixture (or Docker) is absent, skip cleanly with a clear message rather than fail — keeps the fast suite and `verify` green. Pattern: early `return` with an `eprintln!`/log when the fixture file or `DOCKER_HOST`/daemon is unavailable, same spirit as the env-gate in `live_smoke.rs:47-51`.

The fixtures load via gzip decode at test time (add `flate2` as a dev-dep, or commit a tiny loader). The large-entity replay is the one that catches ordering/allocation drift the tiny fixtures miss.

## 5. Isolation & Coverage

**`#[ignore]` keeps it out of the fast suite:** `cargo test` skips ignored tests; only `cargo test -- --ignored` (or `--include-ignored`) runs them. The pre-deploy gate runs `cargo test -p parser-worker -- --ignored` with Docker up (BRAINSTORM verification plan; matches `live_smoke.rs` which is `#[ignore]`).

**Why it adds zero coverage obligation** `[VERIFIED: scripts/coverage-gate.sh:159,195]`: the gate invokes
```
cargo llvm-cov ... --workspace --bins --tests --examples --no-cfg-coverage ...
```
with **no `--ignored` / `--include-ignored`**. `cargo test`/`llvm-cov` compile ignored tests but do not execute them, so their lines never register as covered AND the e2e's own code is test-code (not production) so it is not subject to the production-line allowlist. The strict gate then runs `parser-quality` `coverage-check` against `coverage/allowlist.toml` (`coverage-gate.sh:197`). The worker's live adapters (`storage.rs`, `amqp.rs`, `runner.rs`) stay allowlisted exactly as today (`coverage/allowlist.toml:65,89,97,113`) — the oracle does not change that (deferred: shrinking the allowlist via an instrumented e2e run is explicitly out of scope).

**No new coverage entries needed.** Do not run the e2e under coverage.

**Master-only pre-deploy CI job shape (recommended):** a dedicated job (separate from the fast verify job) that (1) is gated to `master` / pre-deploy, (2) has a Docker daemon (services or DinD), (3) `cargo build -p parser-worker --tests` then `cargo test -p parser-worker --test golden_container_e2e -- --ignored`, (4) generous per-job timeout for first-run image pulls, (5) caches the cargo + Docker image layers. Fast `verify` (lint/typecheck/unit/coverage/build) stays untouched (principle 11).

## 6. Pitfalls & Seams

- **Docker is a hard dependency.** No daemon → the test must skip with a clear message (presence guard), never false-fail. Confirm the deploy runner has Docker (BRAINSTORM assumption HIGH, user-confirmed).
- **Clean loop termination = `CancellationToken` only.** `run_until_cancelled` runs with `listen_for_ctrl_c=false` (`runner.rs:50,65-68`) so there is no signal handler and no real timer in the worker path. End the loop with `shutdown.cancel()` then `timeout(10s, worker)` join (`live_smoke.rs:140-147`). Never register signal handlers or `sleep`-pace the test loop (principle 9). The only `tokio::time::sleep` acceptable is the **result-polling** `basic_get` retry in `wait_for_delivery` (`live_smoke.rs:359-374`), which is test-side polling of the result queue, not a worker timer.
- **MinIO bucket must pre-exist.** `check_ready` does `head_bucket` and the worker exits if it fails (`storage.rs:211-226`, `runner.rs:94-100`). Create the bucket before spawning the worker (`ensure_bucket`).
- **RabbitMQ topology must pre-exist.** Worker declares nothing (`amqp.rs:160-189`); declare job queue + result exchange + result queues first (`prepare_broker`).
- **S3 path-style + custom endpoint.** Set `s3_force_path_style=true` and `s3_endpoint=http://host:port`; the SDK wiring in `storage.rs:188-200` already honors both. Without path-style, the SDK builds virtual-host URLs MinIO can't serve.
- **Conditional PUT semantics on MinIO.** The artifact write uses `if_none_match("*")` (`storage.rs:294`). MinIO supports this and returns 412/PreconditionFailed on conflict (`classify_conditional_put_error`, `storage.rs:420-445`). The duplicate-job idempotency path (`AlreadyExists` → compare bytes) is therefore exercised for real on MinIO — good, that is the contract.
- **Expected-bytes drift on version bump.** The artifact embeds `parser.version` (CARGO_PKG_VERSION) and source identity (§3). Pin fixture identity constants and regenerate baselines via one script when the version changes. Document this in the fixture manifest README.
- **Proving teeth (mandatory).** Demonstrate the oracle goes red by a temporary behavioral mutation. Cheapest reliable mutation: change a single field in a committed `*.expected.json` (the byte-exact compare must fail) AND, separately, inject a parser-core behavioral change (e.g. flip an aggregate or drop a field) to prove the *parse path*, not just the file, is pinned — reuse the `fault_injection_regressions.rs` / `scripts/fault-report-gate.sh` mutation ideas. Document the run once (BRAINSTORM acceptance).

## Architecture Patterns

- **Extend, don't reinvent.** `live_smoke.rs` is the wiring template; promote its helpers (`prepare_broker`, `declare_result_queue`, `spawn_worker`, `publish_job`, `wait_for_completed`, `wait_for_failed`, `assert_*`, `ensure_bucket`, `s3_client`) into a shared `tests/common/` module or copy-mirror them in the new file. Prefer a shared module to avoid duplication (principle 9 / tests skill "no duplication").
- **One baseline, two consumers.** `*.expected.json` is the single artifact compared by both the container e2e and the fast `golden_fixture_behavior` assertion.
- **Tests skill obligations:** tests return `Result`/use `?` where practical, assert error *variants* with `matches!`, every new behavior gets a manifest entry, determinism stays first-class. The e2e is the integration layer the tests skill places under `crates/<crate>/tests/`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Ephemeral broker/object store | docker-compose scripts, manual env | `testcontainers-modules` rabbitmq + minio | self-contained, mapped ports, ready-waits |
| AMQP topology declaration | new declare code | `prepare_broker`/`declare_result_queue` from `live_smoke.rs` | already correct + matches worker assumptions |
| Worker spawn/shutdown | signal handlers, sleeps | `run_until_cancelled` + `CancellationToken` | the designed seam, no leaks |
| Expected artifact bytes | hand-written JSON | generate from `parse_replay` + trailing `\n` | byte-exact to what the worker writes |
| S3 client for MinIO | custom HTTP | `S3ObjectStore::from_config` / `aws-sdk-s3` with endpoint+path-style | identical to production path |

## Package Legitimacy Audit

| Package | Registry | Age | Source Repo | Verdict | Disposition |
|---------|----------|-----|-------------|---------|-------------|
| `testcontainers-modules` (0.15) | crates.io | published 2026-02-21, established crate | github.com/testcontainers/testcontainers-rs-modules-community | OK | Approved (dev-dep) |
| `flate2` (gzip loader, if used) | crates.io | mature, ubiquitous | github.com/rust-lang/flate2-rs | OK | Approved (dev-dep) |

Both are well-known, maintained crates with real source repos. (`package-legitimacy check` seam not run this session — crates are ecosystem-standard; confirm with `cargo add --dry-run` during plan.)

## Environment Availability

| Dependency | Required By | Available | Notes |
|------------|------------|-----------|-------|
| Docker daemon | testcontainers boot | unverified here | hard requirement on the pre-deploy runner; e2e skips cleanly if absent |
| `~/sg_stats` real OCAP | real fixtures | ✗ MISSING on this machine | capture script + presence skip-guard mandatory |
| `cargo`/Rust 2024 toolchain | build | ✓ | workspace `rust-version = 1.95.0` |
| `cargo-llvm-cov` | coverage gate | (unchanged) | e2e excluded from it |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | testcontainers-modules 0.15 MinIO tag `RELEASE.2025-02-28...`, creds `minioadmin`/`minioadmin`, port 9000, wait `message_on_stderr("API:")` | §2 | wrong tag/creds → container won't auth/ready; pin from the resolved version's source at impl time |
| A2 | RabbitMq module tag `4.2-management`, guest/guest, port 5672, wait "Server startup complete" | §2 | wrong → AMQP connect fails; verify resolved version |
| A3 | MinIO honors `if_none_match("*")` conditional PUT returning 412 | §6 | if not → conflict path takes the `UnsupportedConditionalWrite` fallback (still handled, `storage.rs:434-441`) — assertion still valid but via a different branch |
| A4 | Docker available on pre-deploy runner | §5,§6 | no Docker → e2e can't run; skip-guard keeps verify green, but the gate provides no signal |

*(context7 MCP was not available this session; testcontainers facts are CITED from docs.rs + the repo source `main` branch, tagged MEDIUM. Re-confirm constants against the exact resolved crate version during planning/implementation.)*

## Sources

### Primary (HIGH — code-verified this session)
- `crates/parser-worker/src/runner.rs`, `processor.rs`, `storage.rs`, `amqp.rs`, `config.rs`, `artifact_key.rs`, `checksum.rs` — full call path, config knobs, topology assumptions
- `crates/parser-worker/tests/live_smoke.rs` — the wiring to extend
- `crates/parser-core/tests/golden_fixture_behavior.rs`, `deterministic_output.rs`, `fixtures/golden/manifest.json` — fast golden suite + fixture convention
- `scripts/coverage-gate.sh:159,195` — coverage invocation (no `--ignored`)
- `coverage/allowlist.toml:65-113` — worker live-adapter allowlist entries
- `test -d ~/sg_stats` → MISSING

### Secondary (MEDIUM — docs.rs + GitHub source, context7 unavailable)
- crates.io API: testcontainers-modules 0.15.0 latest (2026-02-21)
- docs.rs/testcontainers-modules MinIO + RabbitMQ module pages
- github.com/testcontainers/testcontainers-rs-modules-community `src/minio/mod.rs`, `src/rabbitmq/mod.rs` (main)

## Metadata

**Confidence breakdown:**
- Call path / wiring / coverage: HIGH — read directly from source this session.
- testcontainers API constants: MEDIUM — docs.rs + repo source; pin exact tags at impl time.
- Fixtures: MEDIUM — `~/sg_stats` absent here; capture script + skip-guard required.

**Research date:** 2026-06-17
**Valid until:** ~2026-07-17 (testcontainers image tags move; re-verify constants before implementation)
