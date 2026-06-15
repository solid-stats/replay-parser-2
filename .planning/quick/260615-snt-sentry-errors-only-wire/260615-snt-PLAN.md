---
quick_id: 260615-snt
slug: sentry-errors-only-wire
status: complete
date: 2026-06-15
---

# Quick Task 260615-snt: Wire errors-only Sentry

Wire an errors-only Sentry/GlitchTip SDK into the `replay-parser-2` binary per
`plans/replay-parser-2/briefs/sentry-wire.md`.

## Task

1. Add the `sentry` crate to `parser-cli` with a minimal, errors-only feature set
   (`backtrace`, `contexts`, `panic`, `reqwest`, `rustls`; `default-features = false`) —
   no `tracing`/`profiling` features, no performance tracing.
   - files: `crates/parser-cli/Cargo.toml`
   - verify: `cargo build`
   - done: dependency present, builds clean.

2. Initialize the Sentry guard at the very top of `fn main()` in the single binary
   (`parser-cli/src/main.rs`), bound to `let _sentry` so it lives for the whole process and
   flushes on drop. DSN from `SENTRY_DSN` (empty → disabled no-op client). Set
   `environment = "staging"` and `release = sentry::release_name!()`; never set
   `traces_sample_rate`.
   - files: `crates/parser-cli/src/main.rs`
   - verify: `cargo build && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cargo test`
   - done: guard bound before `run()`, outlives the worker's tokio runtime; gate green.

## must_haves

- truths: errors-only (no traces_sample_rate); guard held for process lifetime; empty DSN is a no-op.
- artifacts: `crates/parser-cli/Cargo.toml`, `crates/parser-cli/src/main.rs`.
- key_links: `plans/replay-parser-2/briefs/sentry-wire.md`.
