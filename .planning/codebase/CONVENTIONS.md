# Coding Conventions

**Analysis Date:** 2026-06-13

## Naming Patterns

**Files:**
- Rust module files use `snake_case` corresponding to module names
- Test files in `crates/*/tests/` use descriptive `snake_case` with semantic prefixes: `parser_core_api.rs`, `golden_fixture_manifest.rs`, `deterministic_output.rs`, `fault_injection_regressions.rs`, `combat_event_semantics.rs`
- Main binary is named explicitly in `[bin]` section: `replay-parser-2` in `crates/parser-cli/src/main.rs`

**Functions:**
- Public functions use `snake_case` and must include `#[must_use]` unless they have intentional side effects
- Functions returning `Result` or `Option` follow standard Rust naming: `parse_replay()`, `normalize_entities()`, `public_parse_artifact()`
- Helper/internal functions are private (`fn` without `pub`)
- Factory/builder functions use descriptive names: `parser_info()`, `replay_source()`, `source_ref()` in tests

**Variables:**
- Local variables use `snake_case`
- Constants use `SCREAMING_SNAKE_CASE`: `MANIFEST`, `REQUIRED_CATEGORIES`, `PHASE5_REQUIREMENTS`, `INVALID_JSON_FIXTURE`, `AGGREGATE_FIXTURE`, `COMBAT_EVENTS_FIXTURE`
- Test data construction functions are lowercase: `parser_info()`, `replay_source()`, `parser_input()`, `temp_output_path()`
- Iterators naming follows convention: `entity_ids`, `entry.category`, `builder.field_name`

**Types:**
- Struct names use `PascalCase`: `ParserInput<'a>`, `ParserOptions`, `ParseArtifact`, `DiagnosticPolicy`, `DiagnosticAccumulator`, `SourceContext`, `ObservedEntity`
- Enum names use `PascalCase`: `ParseStatus`, `DiagnosticSeverity`, `EntityKind`, `EntitySide`, `ParseStage`
- Lifetime parameters use single lowercase letters: `<'a>`, `<'static>`

## Code Style

**Formatting:**
- Uses `cargo fmt` with default Rust formatting
- 80-character line width is preferred but not strictly enforced (some lines go to 100+)
- Imports are organized by workspace crate first, then external crates, then std
- Trailing commas in multiline structures

**Linting:**
- Workspace-level lints enabled in `Cargo.toml` at `[workspace.lints]`
- Clippy lints at `allow = "all"`, `deny = "all"` with priority levels
- Specific denies: `unsafe_code = "forbid"`, `missing_docs = "deny"`, `expect_used = "deny"`, `unwrap_used = "deny"`, `panic = "deny"`, `todo = "deny"`
- Tests can allow specific clippy lints: `#![allow(clippy::expect_used, reason = "integration tests use expect messages as assertion context")]`
- Lint configuration at `[workspace.lints.rust]`, `[workspace.lints.rustdoc]`, `[workspace.lints.clippy]`

**Examples from codebase:**
- From `crates/parser-core/src/lib.rs`: Functions decorated with `#[must_use]` pub attribute
- From `crates/parser-core/src/artifact.rs`: Module documentation comment starting with `//! ` followed by comments for defensive branches: `// coverage-exclusion: reviewed Phase 05 defensive artifact construction branches...`
- From `crates/parser-core/src/entities.rs`: Allow-statements with reasons: `#[allow(clippy::trivially_copy_pass_by_ref, reason = "the plan requires normalize_entities to accept a borrowed RawReplay")]`

## Import Organization

**Order:**
1. Standard library (`std::`)
2. External dependencies (from workspace dependencies, then external crates)
3. Workspace internal crates (`parser_contract`, `parser_core`, etc.)
4. Module declarations (`pub mod`, `pub use`)

**Examples from codebase:**
- From `crates/parser-core/tests/deterministic_output.rs`:
  ```rust
  use parser_contract::{
      artifact::ParseArtifact,
      presence::FieldPresence,
      // ...
  };
  use parser_core::{ParserInput, ParserOptions, parse_replay};
  use serde_json::json;
  ```

**Path Aliases:**
- Uses full crate paths: `parser_contract::artifact::ParseArtifact`, `parser_core::ParserInput`
- No wildcard imports; explicit imports preferred
- Module re-exports in `lib.rs`: `pub use debug_artifact::DebugParseArtifact;`, `pub use input::{ParserInput, ParserOptions};`

## Error Handling

**Patterns:**
- Uses `thiserror` for custom error types in contract crate (`crates/parser-contract/`)
- Functions return structured `Result<T, ErrorType>` with explicit error variants
- Error types implement `Display` and `Error` traits via `thiserror` derive
- No bare `unwrap()` or `panic!()` in production code; tests use `expect()` with context messages
- Parse failures captured in `ParseFailure` struct with `ErrorCode`, `ParseStage`, `Retryability` fields
- Diagnostics accumulated in `DiagnosticAccumulator` with impact tracking (`DiagnosticImpact::Info`, `NonLossWarning`, `DataLoss`)

**Examples from codebase:**
- From `crates/parser-core/src/artifact.rs`: Error handling with pattern matching:
  ```rust
  match decode_compact_root(input.bytes) {
      Ok(root) => success_artifact(input.parser, input.source, &root, diagnostic_limit),
      Err(CompactDecodeError::RootNotObject { source_cause }) => failed_artifact(...),
      Err(CompactDecodeError::JsonDecode { source_cause }) => failed_artifact(...),
  }
  ```
- From `crates/parser-core/src/diagnostics.rs`: `DiagnosticImpact` enum for tracking damage scope

## Logging

**Framework:** Uses `tracing` crate (`crates/parser-worker/Cargo.toml: tracing`, `tracing-subscriber`)

**Patterns:**
- Worker uses `tracing::info!`, `tracing::warn!`, `tracing::error!` for structured logging
- CLI may use `eprintln!` for error output
- No stdout logging in parser core; worker configures subscribers with env-filter and JSON format
- Tracing subscriber configured with `env_filter` and `fmt` features in worker

## Comments

**When to Comment:**
- Module-level documentation: `//! ` doc comment at start of each `lib.rs` or module file
- Function-level: Rust doesn't require doc comments but public items often have them
- Defensive coverage-exclusion: Comments like `// coverage-exclusion: reviewed Phase 05 defensive ...` mark intentional uncovered branches
- Allowlist reasons: `#[allow(..., reason = "...")]` provides clear justification

**JSDoc/TSDoc:**
- Not applicable to Rust; uses standard Rust doc comments with `///` for items and `//!` for modules
- Doc comments are minimal; semantic meaning comes from function names and type signatures

## Function Design

**Size:** Functions range from 3-5 lines (helpers like `parser_info()`) to 50+ lines (normalization functions)
- Reasonable unit function size; larger functions decomposed into sub-functions
- Examples: `normalize_entities_with_connected_events()` is 76 lines with clear sub-steps

**Parameters:**
- Immutable references preferred over owned values: `&RawReplay<'_>`, `&SourceContext`, `&mut DiagnosticAccumulator`
- Lifetime parameters used for zero-copy parsing of replay JSON: `ParserInput<'a>`, `RawReplay<'_>`
- Builder-style setup in tests: separate `parser_info()`, `replay_source()`, `parser_input()` functions rather than nested calls
- Callback/closure style minimal; mostly explicit function composition

**Return Values:**
- Public functions always return `#[must_use]` values or `Result<T, E>`
- Test helper functions return constructed values or parsed results
- Option/Result used for nullable/fallible cases: `Option<Diagnostic>`, `Result<RuleId>`

## Module Design

**Exports:**
- `lib.rs` files use `pub mod` for submodule visibility and `pub use` for re-exports of primary types
- Example from `crates/parser-core/src/lib.rs`:
  ```rust
  pub mod aggregates;
  pub mod artifact;
  // ...
  pub use debug_artifact::DebugParseArtifact;
  pub use input::{ParserInput, ParserOptions};
  ```
- Worker configuration module: `crates/parser-worker/src/config.rs` exports `WorkerConfig`, `WorkerConfigOverrides`

**Barrel Files:**
- Used in `lib.rs`: `pub mod diagnostics;` makes `crate::diagnostics::DiagnosticPolicy` accessible
- Deliberate re-exports: `pub use artifact::SourceContext;` when type is used across modules
- No glob imports in barrel files

## Coverage Annotations

**Pattern:**
- Defensive branches that are reviewed but intentionally not covered by tests are marked:
  ```rust
  // coverage-exclusion: reviewed Phase 05 defensive [behavior type] branches are allowlisted by exact source line.
  ```
- Examples:
  - `crates/parser-core/src/artifact.rs`: "reviewed Phase 05 defensive artifact construction branches"
  - `crates/parser-core/src/entities.rs`: "reviewed Phase 05 defensive entity normalization branches"
  - `crates/parser-contract/src/schema.rs`: "reviewed Phase 05 schema-shape defensive branches"

## Workspace Configuration

**Cargo features:**
- Workspace members do not define separate feature sets; unified lints apply to all
- Lints enforced uniformly: `[workspace.lints]` centralized; each crate uses `[lints] workspace = true`
- No optional dependencies in core modules; determinism requires all dependencies always present

**Dev profiles:**
- `[profile.dev]` and `[profile.test]` both set `debug = "line-tables-only"` and `incremental = false`
- Keeps binary sizes manageable and CI builds predictable

---

*Convention analysis: 2026-06-13*
