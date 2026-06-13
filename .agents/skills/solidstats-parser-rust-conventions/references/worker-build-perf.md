# Worker, build, supply-chain & performance

The durable worker runtime and the build/CI gates around it. The pure `parser-core` logic stays sync;
this file governs the binaries and the toolchain.

## §H. Async & worker

Applies to `parser-worker` (the `parser-core` logic stays sync and pure).

- **tokio discipline**: never `std::thread::sleep` on an async path; never hold a lock across
  `.await`; honor `Send` bounds; bound concurrency with a semaphore / `JoinSet` rather than
  unbounded spawning.
- **Durability**: the worker coordinates through durable `parse_jobs` state — never fire-and-forget.
  Honor ack/nack semantics, make processing **idempotent** (a redelivered message must not double
  apply), and support **graceful shutdown** (`CancellationToken` / the existing `shutdown.rs`).
- **Poison messages → dead-letter, never requeue-loop.** A parse/validation failure nacks with
  `requeue=false` to a dead-letter exchange (or relies on a quorum-queue `delivery-limit`); only
  *transient* failures requeue. An always-failing message must not redeliver forever.
- **Bound prefetch.** Set a small `basic_qos(prefetch_count)` (e.g. 10–50) so unacked deliveries
  don't stream unbounded into worker memory — the queue-level counterpart to the input-size cap
  (`parsing-types-errors.md` §F). Prefetch is **delivery backpressure, not task concurrency**: it
  caps unacked deliveries, not how many parse tasks run at once — cap real concurrency with a
  bounded semaphore or bounded channel (the tokio-discipline bullet above).
- **Drain on shutdown, don't just signal.** Graceful shutdown both *tells* (CancellationToken) and
  *waits*: collect in-flight tasks in a `JoinSet` and `join_next` them (ack the in-flight delivery)
  before exit, so SIGTERM doesn't drop a mid-parse message.
- **Handle consumer cancellation / recovery.** A broker `basic.cancel` (queue deleted, failover) ends
  the consumer stream — treat stream-end as reconnect, and enable connection recovery: turn on
  lapin's automatic connection + topology recovery via
  `ConnectionProperties::default().enable_auto_recover()` and call
  `channel.wait_for_recovery(error).await` on recoverable errors instead of tearing the worker down.
- **Bound S3.** aws-sdk-s3 retries by default but sets **no** operation timeout — configure explicit
  `operation_timeout` + `operation_attempt_timeout` so a stalled read can't hang a worker task.
- **Instrument with `tracing`** — the log-hygiene rules (structured fields, level semantics, span
  hierarchy, `replay_id` / `job_id` correlation) live in `observability-and-lifecycle.md` §K;
  diagnosability in §L.
- *(Optional, once an OTLP collector exists)* propagate `traceparent` through the RabbitMQ message and
  export worker spans via `tracing-opentelemetry` / OTLP, so a parse is followable from server-2's
  dispatch span through the worker in one distributed trace.

## §I. Docs, API hygiene & performance

- `missing_docs` is denied → every public item has a `///` doc (what/how); `//` comments explain
  **why**. `TODO(#issue)` only, never a bare `TODO`.
- `#[must_use]` on functions whose result must not be ignored; `#[non_exhaustive]` on public enums
  expected to grow.
- Static dispatch (generics) by default; `dyn` only for genuinely heterogeneous collections, boxed
  at the boundary.
- **Performance**: profile before optimizing; benchmark only in `--release`; prefer iterators, avoid
  needless `.collect()` and clones in loops, box large enum variants. The `parser-quality`
  cargo-budget gate guards build/size — respect it.

## §J. Build, supply-chain & CI gates

Beyond the lint floor, the parser's build and dependency graph are gated — a deterministic parser
that publishes a versioned artifact needs an audited, reproducible toolchain.

- **`cargo-deny`** runs all four checks in CI: `advisories` (RustSec DB), `licenses` (an allowlist),
  `bans` (reject duplicate major versions of core deps), and `sources` (crates only from crates.io +
  any trusted private registry — blocks silent git/registry substitution).
- **`cargo-audit`** runs on a nightly schedule against the pinned `Cargo.lock`, catching advisories
  disclosed *between* dependency bumps — complementary to `cargo-deny`'s PR-time check.
- **MSRV** is declared via `rust-version` in `Cargo.toml` and CI-enforced (a job pinned to exactly
  that toolchain); never raised in a patch release.
- **`overflow-checks = true`** in `[profile.release]` (see determinism §C). Consider
  **`panic = "abort"`** for the worker binary *only if* no `Drop`-based cleanup must run on panic — it
  removes unwind machinery but also skips destructors.
- **Reproducible binary** *(when it matters)* — build with `cargo build --locked` and
  `RUSTFLAGS=--remap-path-prefix=$PWD=.` so the artifact-producing binary embeds neither absolute
  paths nor a drifting dependency graph.
- **Benchmark budgets** — the `parser-quality` cargo-budget gate guards size/build; for runtime,
  compare `criterion` baselines across commits (critcmp / Bencher) and fail on a regression beyond a
  threshold.
