---
status: planned
mode: quick-full
must_haves:
  truths:
    - Strict quality enforcement uses stable Rust tooling only.
    - Workspace lints apply to current and future crates through Cargo inheritance.
    - Current code passes the new lint, format, doc, and test gates.
    - No parser contract, queue/storage, identity, persistence, or UI data shape changes are introduced.
  artifacts:
    - Cargo workspace lint policy.
    - Stable rustfmt config.
    - Cargo quality aliases.
    - Updated README validation commands.
    - Quick task summary and verification artifacts.
  key_links:
    - Cargo.toml
    - crates/parser-contract/Cargo.toml
    - rustfmt.toml
    - .cargo/config.toml
    - README.md
---

# Quick Task 260426-joq: Strict Quality Rules - Plan

## Task 1: Add Strict Stable Tooling Policy

**Files:** `Cargo.toml`, `crates/parser-contract/Cargo.toml`, `rustfmt.toml`, `.cargo/config.toml`, `README.md`

**Action:** Add workspace-level package metadata, strict Rust and Clippy lint policy, stable rustfmt settings, cargo aliases, and README quality commands.

**Verify:** Tooling commands recognize the new configuration without nightly-only option errors.

**Done:** The repo has a single documented quality gate that future crates inherit.

## Task 2: Fix Current Code Under the New Rules

**Files:** `crates/parser-contract/src/**/*.rs`, `crates/parser-contract/tests/*.rs`, `crates/parser-contract/examples/*.rs`

**Action:** Add public API docs, `#[must_use]`, `const fn`, stronger derives, and small test changes required by the strict lint policy.

**Verify:** `cargo clippy --workspace --all-targets -- -D warnings` passes.

**Done:** Strict lints pass without broad allowlists.

## Task 3: Verify and Record the Quick Task

**Files:** `.planning/quick/260426-joq-strict-quality-rules/*`, `.planning/STATE.md`

**Action:** Run format, lint, docs, and tests. Write summary and verification artifacts. Update `STATE.md`.

**Verify:** `git status --short` is clean after commits.

**Done:** Quick task artifacts and intended code/config changes are committed.
