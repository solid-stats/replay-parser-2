---
quick_id: 260615-snt
slug: sentry-errors-only-wire
status: complete
date: 2026-06-15
---

# Quick Task 260615-snt — Summary

Wired errors-only Sentry/GlitchTip reporting into the `replay-parser-2` binary.

## Changes

- `crates/parser-cli/Cargo.toml` — added `sentry = "0.48.2"` with `default-features = false`
  and `features = ["backtrace", "contexts", "panic", "reqwest", "rustls"]`. `rustls` matches the
  worker's existing TLS stack; `panic` kept, `tracing`/`profiling` omitted.
- `crates/parser-cli/src/main.rs` — `let _sentry = sentry::init(...)` as the first statement of
  `main()`, bound so the guard outlives the worker's tokio runtime and flushes on drop. DSN from
  `SENTRY_DSN` (empty → disabled no-op). `environment = "staging"`, `release = release_name!()`,
  no `traces_sample_rate` (errors only).
- `Cargo.lock` — dependency resolution.

## Gate

- cargo build: ✅
- cargo test: ✅ (all suites pass)
- cargo clippy --all-targets -- -D warnings: ✅
- cargo fmt --check: ✅

## Review

`solidstats-parser-rust-code-review`: APPROVE. Contract N/A (binary-only change); determinism and
lint gates clean. No critical/high findings; two informational notes (handled CLI errors not
reported to Sentry by design; guard correctly outlives the runtime).

## Notes / assumptions

- The brief references `src/main.rs`; the workspace's single binary entrypoint is
  `crates/parser-cli/src/main.rs` (the worker runs via the `Worker` subcommand inside it), so the
  guard there covers both CLI and worker process lifetimes.
- Forced-error GlitchTip smoke test from the brief is operator-run with a live `SENTRY_DSN`; not
  performed here.
- Explicit `sentry::capture_error` on handled `CliError` paths is out of scope (panics are
  auto-captured); can be added later if operators want handled errors in GlitchTip.
