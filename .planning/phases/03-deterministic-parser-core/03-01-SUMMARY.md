---
phase: 03-deterministic-parser-core
plan: 01
subsystem: parser-core
tags: [rust, serde-json, parser-core, parser-contract, structured-failures, deterministic-output]

requires:
  - phase: 03-deterministic-parser-core
    provides: "Plan 03-00 observed entity contract extensions and parser-contract artifact/failure types"
  - phase: 02-versioned-output-contract
    provides: "ParseArtifact, ParseFailure, ReplaySource, SourceRefs, ContractVersion, ParserInfo"
provides:
  - "parser-core workspace crate with pure parse_replay(ParserInput<'_>) API"
  - "Deterministic success artifact shell for valid JSON object roots"
  - "Structured failed artifacts for invalid JSON and non-object JSON roots"
  - "Focused parser-core API/failure tests and malformed JSON fixture"
affects: [03-deterministic-parser-core, parser-core, parser-contract-consumers]

tech-stack:
  added: [parser-core crate, serde, serde_json, thiserror]
  patterns:
    - "Pure parser-core boundary receives bytes and caller-supplied source/parser metadata"
    - "Parser-core returns Phase 2 parser-contract artifacts directly"
    - "Parser-core leaves produced_at unset for deterministic output"

key-files:
  created:
    - .planning/phases/03-deterministic-parser-core/03-01-SUMMARY.md
    - crates/parser-core/Cargo.toml
    - crates/parser-core/src/lib.rs
    - crates/parser-core/src/input.rs
    - crates/parser-core/src/artifact.rs
    - crates/parser-core/src/diagnostics.rs
    - crates/parser-core/tests/parser_core_api.rs
    - crates/parser-core/tests/fixtures/invalid-json.ocap.json
  modified:
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "parser-core exposes only a pure bytes-plus-metadata API; transport, storage, database, and timestamp concerns remain adapter-owned."
  - "Invalid JSON and non-object roots return ParseStatus::Failed with parser-contract ParseFailure instead of panics."
  - "Valid object roots currently return deterministic empty replay/entity/event/aggregate shells so later Phase 3 plans can populate metadata and entities incrementally."

patterns-established:
  - "Artifact shell construction uses ContractVersion::current(), caller ParserInfo/ReplaySource, produced_at: None, empty deterministic collections, and BTreeMap extensions."
  - "Root failure source refs use json_path '$' with stable rule IDs failure.json.decode and failure.schema.root_object."
  - "Tests exercise the public parser_core::parse_replay API rather than private helper functions."

requirements-completed: [OUT-08, PARS-01, PARS-02]

duration: 6m39s
completed: 2026-04-27
---

# Phase 3 Plan 01: Parser-Core Crate Foundation Summary

**Pure parser-core crate with deterministic artifact shells and structured JSON/root failures**

## Performance

- **Duration:** 6m39s
- **Started:** 2026-04-27T04:55:04Z
- **Completed:** 2026-04-27T05:01:43Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added `parser-core` as a workspace member with `parse_replay(ParserInput<'_>)`, `ParserInput`, `ParserOptions`, and `DiagnosticPolicy`.
- Implemented deterministic success shells for valid JSON object roots with `produced_at: None`, empty diagnostics/entities/events, default aggregates, and no failure.
- Implemented structured failed artifacts for invalid JSON and non-object roots using Phase 2 `ParseFailure`, `ErrorCode`, `Retryability`, `SourceRefs`, and stable failure rule IDs.
- Added parser-core API tests covering success shell, invalid JSON failure, non-object root failure, and no parser-core timestamp population.

## Task Commits

1. **Task 1: Add parser-core workspace crate and public modules** - `ef72e3c` (feat)
2. **Task 2: Build deterministic artifact shell and structured failure path** - `a99eeba` (feat)

## Files Created/Modified

- `Cargo.toml` - Adds `crates/parser-core` to the workspace members.
- `Cargo.lock` - Records the new `parser-core` package entry.
- `crates/parser-core/Cargo.toml` - Defines the parser-core crate and contract/Serde/error dependencies.
- `crates/parser-core/src/lib.rs` - Exposes public modules and the pure `parse_replay` API.
- `crates/parser-core/src/input.rs` - Defines `ParserInput<'a>` and default `ParserOptions`.
- `crates/parser-core/src/artifact.rs` - Builds success shells and structured JSON/schema failure artifacts.
- `crates/parser-core/src/diagnostics.rs` - Adds the minimal diagnostic policy wrapper.
- `crates/parser-core/tests/parser_core_api.rs` - Verifies public API success/failure behavior.
- `crates/parser-core/tests/fixtures/invalid-json.ocap.json` - Provides a truncated malformed OCAP JSON fixture.

## Verification

| Command | Result |
| --- | --- |
| `test -f crates/parser-core/Cargo.toml` | PASS |
| `test -f crates/parser-core/src/lib.rs` | PASS |
| `rg -n "crates/parser-core\|parse_replay\|ParserInput\|ParserOptions" Cargo.toml crates/parser-core/src` | PASS |
| `rg -n "name = \"parser-core\"\|parser-contract" crates/parser-core/Cargo.toml` | PASS |
| `rg -n "diagnostic_limit: 100" crates/parser-core/src/input.rs` | PASS |
| `cargo check -p parser-core --all-targets` | PASS |
| `cargo test -p parser-core parser_core_api` | PASS - 2 filtered tests passed |
| `cargo test -p parser-core parser_core_failure` | PASS - 2 filtered tests passed |
| `cargo test -p parser-contract artifact_envelope` | PASS - matching filtered contract tests passed |
| `cargo test --workspace` | PASS - 54 workspace tests passed |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |

## Decisions Made

- Kept parser-core transport-free and deterministic: no file reads, queue publishing, S3 access, database writes, or wall-clock timestamps.
- Used `serde_json::Value` only for root validation in this plan; typed metadata/entity normalization is deferred to later Phase 3 plans.
- Preserved caller-provided `ReplaySource` and `ParserInfo` in both success and failure artifacts rather than deriving or mutating identity inside parser-core.
- Derived Serde traits for `ParserOptions` so the plan-mandated `serde` dependency is directly used without adding adapter behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed strict clippy blockers in new parser-core code**
- **Found during:** Task 2 verification
- **Issue:** Workspace clippy gates rejected a pass-by-value helper, an unnecessary `Result` wrapper, non-const helper methods, and an unnecessary raw string hash in a test.
- **Fix:** Borrowed `FailureSpec`, returned `SourceRef` directly, made simple helper methods `const`, and changed `br#"{}"#` to `br"{}"`.
- **Files modified:** `crates/parser-core/src/artifact.rs`, `crates/parser-core/tests/parser_core_api.rs`
- **Verification:** `cargo clippy --workspace --all-targets -- -D warnings` passed.
- **Committed in:** `a99eeba`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** No scope change. The fix was required to satisfy the existing strict Rust quality gate.

## Issues Encountered

- The local `node_modules/@gsd-build/sdk` CLI path was absent and the `gsd-sdk query` command on PATH did not support query mode. This did not block execution because the user requested normal git commits and explicitly assigned central `STATE.md`/`ROADMAP.md` progress updates to the orchestrator.

## Known Stubs

None. The `None`, `Vec::new()`, `BTreeMap::new()`, and default aggregate values in `parser-core` are intentional deterministic artifact-shell outputs required by Plan 03-01; later Phase 3 plans populate replay metadata and entities.

## Threat Flags

None. The plan added no network endpoints, auth paths, file access patterns, queue/S3/database behavior, canonical identity fields, or parser-owned timestamp population.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03-02 can build tolerant OCAP root decode and replay metadata normalization on top of the pure `parser-core` API and structured failure shell introduced here.

## Self-Check: PASSED

- Found `.planning/phases/03-deterministic-parser-core/03-01-SUMMARY.md`.
- Found `crates/parser-core/Cargo.toml`.
- Found `crates/parser-core/src/lib.rs`.
- Found `crates/parser-core/src/artifact.rs`.
- Found `crates/parser-core/tests/parser_core_api.rs`.
- Found task commits `ef72e3c` and `a99eeba` in git history.

---
*Phase: 03-deterministic-parser-core*
*Completed: 2026-04-27*
