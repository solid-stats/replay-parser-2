# Testing Patterns

**Analysis Date:** 2026-06-13

## Test Framework

**Runner:**
- `cargo test` (built-in Rust test harness)
- No external test runner; uses standard Rust `#[test]` attribute
- Integration tests in `crates/*/tests/` directory structure
- Unit tests inline in source modules using `#[cfg(test)] mod tests { }`

**Assertion Library:**
- Standard Rust `assert!`, `assert_eq!`, `assert_ne!` macros
- No external assertion library; semantic clarity through assertion messages

**Run Commands:**
```bash
cargo test -p parser-contract              # Test contract crate
cargo test -p parser-core                  # Test core parser logic
cargo test -p parser-cli                   # Test CLI and command-line behavior
cargo test -p parser-quality               # Test quality gates
cargo test -p parser-worker                # Test worker integration
cargo test --workspace                     # All crates
cargo test --workspace -- --nocapture      # Show println! output
scripts/coverage-gate.sh --check            # Check coverage compliance
COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict  # Strict coverage
scripts/fault-report-gate.sh                # Validate fault-injection regressions
```

## Test File Organization

**Location:**
- `crates/parser-core/tests/` - Integration tests for parser core; files at `tests/*.rs`
- `crates/parser-cli/tests/` - Integration tests for CLI command behavior
- `crates/parser-worker/tests/` - Integration tests for worker mode
- `crates/parser-contract/tests/` - Integration tests for contract schema and serialization
- `crates/parser-quality/tests/` - Integration tests for quality gates

**Naming:**
- Semantic prefixes: `parser_core_api.rs`, `golden_fixture_manifest.rs`, `deterministic_output.rs`, `fault_injection_regressions.rs`, `combat_event_semantics.rs`, `entity_normalization.rs`, `raw_event_accessors.rs`, `schema_drift_status.rs`, `aggregate_projection.rs`, `metadata_normalization.rs`, `side_facts.rs`, `legacy_entity_compatibility.rs`
- Fixture directory: `crates/parser-core/tests/fixtures/` contains OCAP JSON test data

**Structure:**
```
crates/parser-core/
  tests/
    parser_core_api.rs
    golden_fixture_manifest.rs
    deterministic_output.rs
    ...
    fixtures/
      invalid-json.ocap.json
      aggregate-combat.ocap.json
      combat-events.ocap.json
      ...
      golden/
        manifest.json
        [individual fixture subdirectories]
```

## Test Structure

**Suite Organization:**
From `crates/parser-core/tests/parser_core_api.rs`:
```rust
//! Parser-core public API behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::...;
use parser_core::...;

const INVALID_JSON_FIXTURE: &[u8] = include_bytes!("fixtures/invalid-json.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

#[test]
fn parser_core_api_should_return_success_shell_when_root_object_is_valid() {
    let input = parser_input(br#"{"missionName":"Operation Copper","entities":[]}"#);
    let artifact = parse_replay(input);
    
    assert_eq!(artifact.status, ParseStatus::Success);
    // ... more assertions
}
```

**Patterns:**
- Fixture constants using `include_bytes!()` or `include_str!()` for embedded JSON test data
- Helper function per logical test context: `parser_info()`, `replay_source()`, `parser_input()`, `parse_fixture()`
- Test names follow semantic pattern: `{function}_should_{expected_behavior}_when_{condition}`
- `#![allow(clippy::expect_used, reason = "...")]` at test-file top level to allow expect() calls with descriptive messages
- Each test is fully self-contained with its own setup helpers

## Mocking

**Framework:** No explicit mocking library (like `mockito` or `mockall`)

**Patterns:**
- Test fixtures provide real OCAP JSON data: embedded bytes in test files or files in `fixtures/` directory
- Golden fixture manifest (`fixtures/golden/manifest.json`) declares test data, categories, and expected outcomes
- Fixtures organized by testing strategy: `normal`, `malformed`, `partial_schema_drift`, `old_shape`, `winner_present`, `winner_missing`, `vehicle_kill`, `teamkill`, `commander_side`, `null_killer`, `duplicate_slot_same_name`, `connected_player_backfill`
- Builder-style test data construction: separate functions for `parser_info()`, `replay_source()`, `source_ref()` instead of mocking

**What to Mock:**
- Not typically mocked; parser core accepts `ParserInput<'_>` with caller-provided `bytes`, `source`, `parser`, `options`
- Worker tests use real RabbitMQ/S3 interaction patterns; tower middleware in dev dependencies for HTTP testing

**What NOT to Mock:**
- Core parser logic; use real OCAP JSON fixtures instead
- Deterministic contract serialization; tests verify exact byte-for-byte output
- Artifact construction; all parsing is tested with actual replay data

## Fixtures and Factories

**Test Data:**
```rust
fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-0001".to_string()),
        source_file: "fixtures/replay-0001.ocap.json".to_string(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect("test checksum should be valid"),
            source: None,
        },
    }
}

fn parser_input(bytes: &[u8]) -> ParserInput<'_> {
    ParserInput {
        bytes,
        source: replay_source(),
        parser: parser_info(),
        options: ParserOptions::default(),
    }
}

const AGGREGATE_FIXTURE: &[u8] = include_bytes!("fixtures/aggregate-combat.ocap.json");
const INVALID_JSON_FIXTURE: &[u8] = include_bytes!("fixtures/invalid-json.ocap.json");
```

**Location:**
- Fixture functions defined at test-file scope (above test functions)
- Embedded OCAP JSON files in `crates/parser-core/tests/fixtures/` (referenced by `include_bytes!()`)
- Golden fixture metadata in `crates/parser-core/tests/fixtures/golden/manifest.json` declaring requirements, decisions, expected status, expected features, cross-app impact

## Coverage

**Requirements:** 100% reachable-code statement, branch, function, and line coverage as a v1.0 release gate

**Coverage scripts:**
```bash
scripts/coverage-gate.sh --check          # Check coverage compliance (default behavior)
COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict  # Strict coverage with heavy reporting
```

**View Coverage:**
- Coverage gates check line, branch, and function coverage via script in `scripts/coverage-gate.sh`
- Defensive branches reviewed and allowlisted: marked with `// coverage-exclusion: reviewed Phase 05 [behavior] branches are allowlisted by exact source line`
- Heavy coverage reporting available with `COVERAGE_ALLOW_HEAVY=1` environment variable

## Test Types

**Unit Tests:**
- Inline modules with `#[cfg(test)]` in source files (rare in this codebase; most tests are integration tests)
- Scope: Individual functions, internal helpers
- Example: Error type serialization, schema validation in `crates/parser-contract/`

**Integration Tests:**
- Primary testing strategy; files in `crates/*/tests/` directory
- Scope: Public API behavior, full parser execution, artifact determinism, CLI command execution
- Example files:
  - `crates/parser-core/tests/parser_core_api.rs` - tests public `parse_replay()`, `public_parse_artifact()`, `public_parse_replay()` functions
  - `crates/parser-core/tests/deterministic_output.rs` - verifies same input produces identical JSON across runs
  - `crates/parser-core/tests/fault_injection_regressions.rs` - tests regression cases for high-risk semantics
  - `crates/parser-cli/tests/parse_command.rs` - tests CLI command execution with real subprocess

**E2E Tests:**
- CLI tests use `assert_cmd` crate to spawn and verify CLI process behavior
- Example from `crates/parser-cli/tests/parse_command.rs`:
  ```rust
  use assert_cmd::Command;
  
  fn run_parse(input: &PathBuf, output: &PathBuf) -> Output {
      Command::cargo_bin("replay-parser-2")
          .expect("replay-parser-2 binary should build")
          .arg("parse")
          .arg(input)
          .arg("--output")
          .arg(output)
          .output()
          .expect("parse command should run")
  }
  ```
- Worker tests verify RabbitMQ/S3 integration patterns with real message passing

## Common Patterns

**Async Testing:**
Not used in this codebase; parser core is synchronous. Worker uses `tokio::test` for async operations.

**Error Testing:**
```rust
#[test]
fn parser_core_api_should_report_json_decode_failure_when_input_is_invalid_json() {
    let input = parser_input(INVALID_JSON_FIXTURE);
    
    let artifact = parse_replay(input);
    
    assert_eq!(artifact.status, ParseStatus::Failed);
    assert!(artifact.failure.is_some());
    if let Some(failure) = artifact.failure {
        assert_eq!(failure.stage, ParseStage::JsonDecode);
        assert!(failure.error_code.is_some());
    }
}
```

**Determinism Testing:**
```rust
#[test]
fn deterministic_output_should_serialize_identically_when_same_input_is_parsed_twice() {
    let first_artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let second_artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    
    let first_serialized = serde_json::to_string(&first_artifact)
        .expect("first artifact should serialize");
    let second_serialized = serde_json::to_string(&second_artifact)
        .expect("second artifact should serialize");
    
    assert_eq!(first_serialized, second_serialized);
}
```

**Semantic Behavior Testing:**
```rust
#[test]
fn fault_injection_regressions_should_catch_same_side_kills_counted_as_enemy_kills() {
    let artifact = parse_fixture(COMBAT_EVENTS_FIXTURE, "fixtures/combat-events.ocap.json");
    let teamkill = player_kill_rows(&artifact)
        .find(|row| row.classification == KillClassification::Teamkill)
        .expect("same-side teamkill row should exist");
    
    assert_eq!(teamkill.victim_source_entity_id, Some(3));
    assert_eq!(teamkill.classification, KillClassification::Teamkill);
}
```

**Golden Fixture Manifest Testing:**
```rust
#[test]
fn golden_fixture_manifest_should_include_every_required_phase_5_category() {
    let entries = manifest_entries();
    let categories = entries.iter()
        .map(|entry| entry.category.as_str())
        .collect::<BTreeSet<_>>();
    
    for category in REQUIRED_CATEGORIES {
        assert!(categories.contains(category), "manifest should contain category {category}");
    }
}
```

## Test Organization Principles

- **Isolated fixtures**: Each test receives complete `ParserInput` constructed from helpers
- **Deterministic data**: Test data is embedded or in committed fixture files; no randomness
- **Semantic naming**: Test names describe behavior, not implementation
- **Comprehensive coverage**: 100% reachable-code coverage enforced by CI gates
- **Regression-focused**: Dedicated fault-injection and regression test suites for high-risk areas
- **Behavioral verification**: Tests check artifact content, determinism, and error states rather than implementation details
- **Golden data**: Phase 5 fixtures reference requirements, decisions, and cross-app impact; manifest verifies all categories are covered

---

*Testing analysis: 2026-06-13*
