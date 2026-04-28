---
phase: 05-cli-golden-parity-benchmarks-and-coverage-gates
plan: 00
subsystem: cli
tags: [rust, clap, assert_cmd, parser-core, parser-contract, sha256, json-schema]

# Dependency graph
requires:
  - phase: 04-event-semantics-and-aggregates
    provides: verified parser-core ParseArtifact generation and contract schema source
provides:
  - public local replay-parser-2 binary with parse and schema commands
  - structured CLI parse failure artifact behavior
  - command-level parse/schema tests
  - reserved compare command surface for Phase 5 Plan 02
affects: [phase-05-plan-01, phase-05-plan-02, phase-05-plan-05, README]

# Tech tracking
tech-stack:
  added: [clap, assert_cmd, sha2, hex]
  patterns: [thin CLI adapter around parser-core, contract-backed schema export, behavior-level CLI tests]

key-files:
  created:
    - crates/parser-cli/Cargo.toml
    - crates/parser-cli/src/main.rs
    - crates/parser-cli/tests/parse_command.rs
    - crates/parser-cli/tests/schema_command.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - README.md

key-decisions:
  - "Public local binary is replay-parser-2 with parse, schema, and reserved compare subcommands."
  - "Local parse computes SHA-256 from bytes and writes parser-core ParseArtifact JSON for success and parser failures."
  - "Schema export is generated from parser-contract::schema::parse_artifact_schema()."
  - "Compare remains a planned non-zero stub until Phase 5 Plan 02."

patterns-established:
  - "CLI adapters own filesystem/stdout/stderr behavior while parser-core remains pure and transport-free."
  - "Command tests use assert_cmd against the public replay-parser-2 binary and assert observable JSON/stderr behavior."

requirements-completed: [CLI-01, CLI-02, CLI-04, TEST-03, TEST-08, TEST-09, TEST-10, TEST-11]

# Metrics
duration: 22min
completed: 2026-04-28
---

# Phase 5 Plan 00: Public CLI Binary Summary

**Local `replay-parser-2` CLI with deterministic parse artifacts, contract-backed schema export, and command-level failure/determinism tests**

## Performance

- **Duration:** 22 min
- **Started:** 2026-04-28T04:54:03Z
- **Completed:** 2026-04-28T05:15:55Z
- **Tasks:** 4
- **Files modified:** 7

## Accomplishments

- Added `crates/parser-cli` workspace binary package with public bin target `replay-parser-2`.
- Implemented `parse`, `schema`, and reserved `compare` CLI surface without adding worker, database, replay discovery, UI, or canonical identity behavior.
- `parse` reads local replay bytes, computes SHA-256, populates `ReplaySource`, calls `parser_core::parse_replay`, writes pretty JSON artifacts, and exits non-zero for failed artifacts.
- `schema` emits the current contract schema from `parser_contract::schema::parse_artifact_schema()` to stdout or a requested file.
- Added `assert_cmd` integration tests for success parse artifacts, malformed-input failure artifacts, deterministic repeated output, schema stdout/file modes, and committed schema freshness.
- Updated README to document implemented local CLI commands and keep `compare`/worker scope explicit.

## Task Commits

1. **Task 1: Add parser-cli workspace binary crate** - `bafaa66` (feat)
2. **Task 2: Implement parse command with automatic checksum and structured artifacts** - `4011073` (test)
3. **Task 3: Implement schema command from parser-contract source of truth** - `345f41a` (test)
4. **Task 4: Run CLI quality and boundary checks** - `5af9794` (fix)
5. **AGENTS.md README adjustment** - `fe219cc` (docs)

## Files Created/Modified

- `Cargo.toml` - Added `crates/parser-cli` to workspace members.
- `Cargo.lock` - Locked CLI dependencies.
- `crates/parser-cli/Cargo.toml` - Defined `parser-cli` package, `replay-parser-2` binary, dependencies, and lints.
- `crates/parser-cli/src/main.rs` - Implemented CLI command parsing, local parse artifact writing, schema export, concise stderr summaries, and compare planned stub.
- `crates/parser-cli/tests/parse_command.rs` - Covered valid parse, invalid JSON failure artifact, stderr summary, and repeated-output determinism.
- `crates/parser-cli/tests/schema_command.rs` - Covered schema stdout/file modes and byte-for-byte committed schema freshness.
- `README.md` - Documented implemented `replay-parser-2 parse` and `schema` commands plus reserved compare/worker scope.

## Decisions Made

- The CLI source file coordinate uses the input path as displayed by the command-line `PathBuf`.
- Parser metadata uses `{ name: "replay-parser-2", version: env!("CARGO_PKG_VERSION") }`.
- File-operation failures are concise stderr-only command failures; parser-core failures write structured JSON artifacts before returning non-zero.
- The compare command intentionally returns a non-zero planned message until Plan 02 implements comparison reports.

## Verification

- `cargo check -p parser-cli --all-targets` - passed
- `cargo run -p parser-cli -- --help` - passed; help lists `parse`, `schema`, and `compare`
- `cargo test -p parser-cli parse_command` - passed
- `cargo test -p parser-cli schema_command` - passed
- `cargo test -p parser-contract schema_contract` - passed
- `cargo fmt --all -- --check` - passed
- `cargo clippy -p parser-cli --all-targets -- -D warnings` - passed
- `cargo test -p parser-cli` - passed
- `cargo test --workspace` - passed
- `rg -n "postgres|sqlx|diesel|lapin|RabbitMQ|aws_sdk_s3|S3|canonical_player|openapi|TanStack|fetch replay|crawl" crates/parser-cli` - no matches
- `git diff --check` - passed

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed formatter and clippy gate blockers**
- **Found during:** Task 4 (Run CLI quality and boundary checks)
- **Issue:** `cargo fmt --all -- --check` and clippy found formatting/API cleanup issues in the new CLI adapter and tests.
- **Fix:** Applied `cargo fmt`, changed helper path arguments from by-value `PathBuf` to `&Path`, used `map_or_else`, and simplified schema output branching.
- **Files modified:** `crates/parser-cli/src/main.rs`, `crates/parser-cli/tests/parse_command.rs`, `crates/parser-cli/tests/schema_command.rs`
- **Verification:** `cargo fmt --all -- --check`, `cargo clippy -p parser-cli --all-targets -- -D warnings`, `cargo test --workspace`
- **Committed in:** `5af9794`

**2. [Rule 2 - AGENTS.md] Updated README for implemented command surface**
- **Found during:** Post-task AGENTS.md enforcement
- **Issue:** README still said the CLI was not implemented and used the legacy `sg-replay-parser` planned command name.
- **Fix:** Documented implemented `replay-parser-2 parse` and `schema`, clarified `compare` is reserved for Plan 02, and kept worker mode as Phase 6 scope.
- **Files modified:** `README.md`
- **Verification:** `git diff --check`
- **Committed in:** `fe219cc`

---

**Total deviations:** 2 auto-fixed (1 blocking quality issue, 1 AGENTS.md documentation requirement)
**Impact on plan:** Both changes were required to satisfy repository quality and documentation rules. No parser-core, worker, server, fetcher, web, or canonical identity behavior was added.

## Issues Encountered

- `cargo clippy` initially hit sandbox DNS while downloading a missing transitive crate; reran with network escalation and completed successfully.
- Git staging/commits required escalation because sandboxed git could not create `.git/index.lock`.

## Known Stubs

| File | Line | Stub | Reason |
|------|------|------|--------|
| `crates/parser-cli/src/main.rs` | 104 | `compare command is planned in Phase 5 Plan 02` | Intentional reserved command surface required by this plan; implementation belongs to Phase 5 Plan 02. |

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 01 can build golden fixture coverage on top of the public CLI and existing parser-core fixtures. Plan 02 can replace the reserved compare stub with selected-input comparison reports without changing the public command name.

## Self-Check: PASSED

- Verified created/modified files exist.
- Verified task and README adjustment commits exist: `bafaa66`, `4011073`, `345f41a`, `5af9794`, `fe219cc`.
- Verified summary whitespace with `git diff --check`.

---
*Phase: 05-cli-golden-parity-benchmarks-and-coverage-gates*
*Completed: 2026-04-28*
