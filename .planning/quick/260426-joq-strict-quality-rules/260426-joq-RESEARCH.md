# Quick Task 260426-joq: Strict Quality Rules - Research

**Mode:** quick-task
**Date:** 2026-04-26

## Findings

- The repository is a Rust 2024 Cargo workspace pinned to Rust `1.95.0`.
- Cargo workspace lints are the right stable mechanism for enforcing Rust and Clippy policy across current and future crates.
- `rustfmt.toml` can enforce stable formatting choices without requiring nightly. Unstable options such as import grouping should be avoided for this task.
- `clippy::all`, `clippy::pedantic`, `clippy::nursery`, and `clippy::cargo` provide a strict but maintainable baseline. The `restriction` group should not be enabled wholesale because it includes intentionally contradictory style lints, but high-signal restriction lints can be denied directly.
- Public parser contract types should tolerate a high documentation bar because this crate is an integration boundary for `server-2`.
- `clippy::cargo` requires package metadata such as `description`, `repository`, `readme`, `keywords`, and `categories`.

## Recommended Implementation

1. Add workspace package metadata and inherited package lint policy.
2. Add `[workspace.lints.rust]` with `warnings = deny`, `missing_docs = deny`, `unsafe_code = forbid`, Rust idiom/compatibility groups, and type-safety lints such as elided lifetimes, unused dependencies, trivial casts, and unused results.
3. Add `[workspace.lints.clippy]` with `all`, `pedantic`, `nursery`, and `cargo` denied, plus targeted restriction lints for `unwrap`, `panic`, `todo`, `dbg`, `print`, wildcard imports, and unchecked conversions.
4. Add `rustfmt.toml` with stable deterministic formatting rules.
5. Add `.cargo/config.toml` aliases for `cargo quality`, `cargo lint`, `cargo fmt-check`, and docs checks.
6. Fix current public API docs and lint violations so the strict gate passes now.

## Verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo doc --workspace --no-deps`

## Research Complete

Research written to `.planning/quick/260426-joq-strict-quality-rules/260426-joq-RESEARCH.md`.
