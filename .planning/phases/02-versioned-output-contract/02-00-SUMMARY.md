---
phase: 02-versioned-output-contract
plan: 00
subsystem: contract
tags: [rust, cargo, serde, schemars, semver, parser-contract]
requires:
  - phase: 01-legacy-baseline-and-corpus
    provides: Legacy corpus, output-surface, identity-boundary, and mismatch-taxonomy evidence
provides:
  - Rust 2024 workspace with one Phase 2 crate at crates/parser-contract
  - Pinned Rust 1.95.0 toolchain file
  - Typed contract and parser version metadata
  - Version serialization tests proving contract_version and parser.version remain separate
affects: [phase-02, phase-03, phase-04, phase-05, phase-06, parser-contract]
tech-stack:
  added: [serde, serde_json, schemars, semver, sha2, hex, thiserror]
  patterns: [workspace-owned contract crate, semver string serialization, serde-transparent newtype]
key-files:
  created:
    - Cargo.toml
    - Cargo.lock
    - rust-toolchain.toml
    - crates/parser-contract/Cargo.toml
    - crates/parser-contract/src/lib.rs
    - crates/parser-contract/src/version.rs
    - crates/parser-contract/tests/version_contract.rs
  modified:
    - .gitignore
key-decisions:
  - "Kept `contract_version` and parser implementation version as separate public typed values."
  - "Used `#[serde(transparent)]` for `ContractVersion` so the typed wrapper serializes as a semver string."
  - "Kept Phase 2 to one Rust crate owned by `replay-parser-2`; no server-2 or web files changed."
patterns-established:
  - "Public contract modules are exported from `parser-contract` before later plans fill their domain types."
  - "Contract tests assert exact JSON field names through public crate exports."
requirements-completed: [OUT-01, OUT-06]
duration: 10m26s
completed: 2026-04-26
---

# Phase 02 Plan 00: Rust Workspace and Contract Version Summary

**Rust parser contract crate foundation with pinned toolchain, semver-backed version metadata, and tests separating contract_version from parser.version**

## Performance

- **Duration:** 10m26s
- **Started:** 2026-04-26T04:48:19Z
- **Completed:** 2026-04-26T04:58:45Z
- **Tasks:** 2
- **Files modified:** 18

## Accomplishments

- Created a Rust 2024 workspace with exactly one Phase 2 member: `crates/parser-contract`.
- Added the `parser-contract` crate dependencies and public module root for later artifact, source reference, identity, metadata, event, aggregate, diagnostic, failure, presence, schema, and version work.
- Added `ContractVersion`, `ParserInfo`, and `ParserBuildInfo` with `Serialize`, `Deserialize`, and `JsonSchema` support.
- Added integration tests proving `contract_version` serializes as `"1.0.0"` and parser implementation version remains separate under `parser.version`.

## Task Commits

1. **Task 1: Create the Rust workspace and parser-contract crate** - `16d2b36` (feat)
2. **Task 2: Add typed contract and parser version metadata** - `0688bc1` (feat)

## Files Created/Modified

- `Cargo.toml` - Workspace manifest with `crates/parser-contract`.
- `Cargo.lock` - Resolved dependency lockfile for reproducible contract crate builds.
- `rust-toolchain.toml` - Pinned Rust 1.95.0 toolchain with rustfmt and clippy.
- `.gitignore` - Ignores Cargo build output under `target/`.
- `crates/parser-contract/Cargo.toml` - Contract crate manifest and dependencies.
- `crates/parser-contract/src/lib.rs` - Public module exports for the contract crate.
- `crates/parser-contract/src/version.rs` - Contract and parser version metadata types.
- `crates/parser-contract/tests/version_contract.rs` - Behavior tests for exact version JSON fields.

## Decisions Made

- Used the plan-specified Cargo resolver 3 after repairing the local Rust 1.95.0 toolchain, rather than downgrading the workspace manifest for an older Cargo.
- Used `semver::Version` for parser implementation version directly; with the `serde` feature it serializes as a JSON string.
- Used `ContractVersion` as a transparent typed wrapper so the public contract type does not collapse into an untyped `String`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Ignored generated Cargo build output**
- **Found during:** Task 1
- **Issue:** `cargo test` created an untracked `target/` directory, which would have left generated build output in `git status`.
- **Fix:** Added `target/` to `.gitignore`.
- **Files modified:** `.gitignore`
- **Verification:** `git status --short` no longer reports `target/`.
- **Committed in:** `16d2b36`

**2. [Rule 3 - Blocking] Repaired incomplete local Rust 1.95.0 components**
- **Found during:** Task 1 and final verification
- **Issue:** Rustup reported installed components, but `cargo`, host `rust-std`, and `cargo-clippy` binaries/files were missing or unusable after interrupted downloads. This also caused the IDE `cargo metadata` error that rejected `resolver = "3"` through an older Cargo path.
- **Fix:** Reinstalled the affected rustup components for toolchain `1.95.0`: `cargo`, `rust-std-x86_64-unknown-linux-gnu`, and `clippy`.
- **Files modified:** None in the repository.
- **Verification:** `cargo --version` reports `cargo 1.95.0`; `cargo metadata`, `cargo test`, and `cargo clippy` all pass.
- **Committed in:** N/A - local toolchain repair only.

---

**Total deviations:** 2 auto-fixed (2 Rule 3 blocking issues).
**Impact on plan:** No scope change. Both fixes were required to complete verification and keep the worktree reviewable.

## Issues Encountered

- The first Rust verification attempts failed because the pinned local `1.95.0` toolchain was partially installed. Reinstalling the corrupted components fixed the issue without changing the plan's manifest requirements.
- The pre-existing `.planning/STATE.md` modification was left unstaged and unmodified, per the isolated-worktree instruction that the orchestrator owns shared state writes.

## Known Stubs

The following module files are intentional blank shells created by Task 1 so `lib.rs` can export the full Phase 2 public module surface while later plans own the actual types:

- `crates/parser-contract/src/artifact.rs:1`
- `crates/parser-contract/src/aggregates.rs:1`
- `crates/parser-contract/src/diagnostic.rs:1`
- `crates/parser-contract/src/events.rs:1`
- `crates/parser-contract/src/failure.rs:1`
- `crates/parser-contract/src/identity.rs:1`
- `crates/parser-contract/src/metadata.rs:1`
- `crates/parser-contract/src/presence.rs:1`
- `crates/parser-contract/src/schema.rs:1`
- `crates/parser-contract/src/source_ref.rs:1`

They do not block this plan's goal because Plan 00 only establishes the compileable crate shell and version foundation.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo metadata --format-version 1 --manifest-path /home/afgan0r/Projects/SolidGames/replay-parser-2/Cargo.toml --filter-platform x86_64-unknown-linux-gnu`
- `test -f Cargo.toml && test -f rust-toolchain.toml && test -f crates/parser-contract/Cargo.toml`
- `cargo test -p parser-contract version_contract`
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Next Phase Readiness

Plan 01 can build on the committed `parser-contract` crate to add the artifact envelope, status metadata, source identity, diagnostics, and success example without creating a second crate or changing adjacent applications.

## Self-Check: PASSED

- Created files listed in the summary exist.
- Task commits `16d2b36` and `0688bc1` exist in git history.
- Shared orchestrator files remain unstaged; `.planning/STATE.md` had a pre-existing local modification and was not committed.

---
*Phase: 02-versioned-output-contract*
*Completed: 2026-04-26*
