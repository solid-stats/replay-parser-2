# Codebase Structure

**Analysis Date:** 2026-06-13

## Directory Layout

```
replay-parser-2/
├── crates/                          # Cargo workspace members
│   ├── parser-contract/             # Versioned contract types and serialization
│   │   ├── src/
│   │   │   ├── lib.rs               # Module index
│   │   │   ├── artifact.rs          # ParseArtifact envelope type
│   │   │   ├── minimal.rs           # Flat table row types (Player, Weapon, Vehicle)
│   │   │   ├── compact.rs           # Compact legacy row types (unused in v3)
│   │   │   ├── events.rs            # NormalizedEvent semantic types
│   │   │   ├── entities.rs          # ObservedEntity identity types
│   │   │   ├── identity.rs          # EntityKind, EntitySide, side assignment
│   │   │   ├── metadata.rs          # ReplayMetadata (mission, world, bounds)
│   │   │   ├── diagnostic.rs        # Diagnostic severity, code, message
│   │   │   ├── failure.rs           # ParseFailure error codes and stages
│   │   │   ├── side_facts.rs        # Commander and round outcome facts
│   │   │   ├── source_ref.rs        # Source checksum and rule ID tracking
│   │   │   ├── presence.rs          # FieldPresence enum (Present/Null/Unknown/Inferred/NotApplicable)
│   │   │   ├── version.rs           # ContractVersion and ParserInfo
│   │   │   ├── worker.rs            # ParseJobMessage, ParseCompletedMessage, ParseFailedMessage
│   │   │   ├── aggregates.rs        # Aggregate projection contract types (legacy)
│   │   │   ├── schema.rs            # JSON Schema generation via schemars
│   │   │   └── lib.rs               # Module re-exports
│   │   ├── examples/
│   │   │   ├── export_schema.rs      # Bin: cargo run --example export_schema
│   │   │   ├── export_worker_schemas.rs  # Bin: cargo run --example export_worker_schemas
│   │   │   ├── parse_artifact_success.v3.json  # Sample successful v3 artifact
│   │   │   ├── parse_failure.v3.json  # Sample failed v3 artifact
│   │   │   ├── parse_job.v1.json     # Sample RabbitMQ job message
│   │   │   ├── parse_completed.v1.json # Sample result message (success)
│   │   │   └── parse_failed.v1.json   # Sample result message (failure)
│   │   ├── tests/
│   │   │   ├── contract_schema_drift.rs  # JSON Schema stability test
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   ├── parser-core/                 # Pure deterministic parser implementation
│   │   ├── src/
│   │   │   ├── lib.rs               # Public API: parse_replay(), public_parse_replay(), parse_replay_debug()
│   │   │   ├── artifact.rs          # parse_replay(), artifact assembly, failure handling
│   │   │   ├── input.rs             # ParserInput and ParserOptions types
│   │   │   ├── raw.rs               # RawReplay wrapper, field accessors, RawField enum
│   │   │   ├── raw_compact.rs       # JSON decoding, RawOcapRoot, compact field parsing
│   │   │   ├── metadata.rs          # Replay metadata normalization
│   │   │   ├── entities.rs          # Entity observation, identity resolution
│   │   │   ├── events.rs            # Combat event semantics, kill categorization
│   │   │   ├── aggregates.rs        # Minimal table derivation from killed events
│   │   │   ├── side_facts.rs        # Commander and round outcome extraction
│   │   │   ├── legacy_player.rs     # Legacy player entity detection
│   │   │   ├── diagnostics.rs       # DiagnosticAccumulator and policy
│   │   │   ├── debug_artifact.rs    # Full debug artifact construction
│   │   │   └── Cargo.toml
│   │   ├── tests/
│   │   │   ├── parser_core_api.rs          # Public API contract tests
│   │   │   ├── golden_fixture_manifest.rs  # Fixture discovery
│   │   │   ├── golden_fixture_behavior.rs  # Regression tests against fixtures
│   │   │   ├── deterministic_output.rs     # Byte-for-byte stability
│   │   │   ├── metadata_normalization.rs   # Metadata extraction tests
│   │   │   ├── entity_normalization.rs     # Entity identity tests
│   │   │   ├── combat_event_semantics.rs   # Kill categorization tests
│   │   │   ├── aggregate_projection.rs     # Minimal table derivation tests
│   │   │   ├── side_facts.rs               # Commander/outcome tests
│   │   │   ├── raw_event_accessors.rs      # Raw field accessor tests
│   │   │   ├── fault_injection_regressions.rs  # Known malformed-file edge cases
│   │   │   ├── debug_artifact.rs           # Debug sidecar tests
│   │   │   └── schema_drift_status.rs      # Contract compatibility tests
│   │   └── Cargo.toml
│   │
│   ├── parser-cli/                  # CLI binary adapter for local parsing
│   │   ├── src/
│   │   │   ├── main.rs              # Single-file CLI (1 file, ~25 KB)
│   │   │   │                         # Commands: parse, schema, worker
│   │   │   │                         # Argument parsing via clap
│   │   │   │                         # File I/O, JSON serialization
│   │   │   │                         # Worker subprocess delegation
│   │   │   └── Cargo.toml
│   │   ├── tests/
│   │   │   ├── cli_parse_basic.rs    # Basic parse command tests
│   │   │   ├── cli_schema_export.rs  # Schema command tests
│   │   │   ├── cli_worker_integration.rs  # Worker mode smoke tests
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   ├── parser-worker/               # RabbitMQ/S3 worker runtime adapter
│   │   ├── src/
│   │   │   ├── lib.rs               # Module index
│   │   │   ├── runner.rs            # Main worker loop, RabbitMQ consumer
│   │   │   ├── processor.rs         # Job processing, parse, artifact write, result publish
│   │   │   ├── amqp.rs              # RabbitMQ client, consumer/publisher helpers
│   │   │   ├── storage.rs           # S3-compatible read/write, artifact key generation
│   │   │   ├── artifact_key.rs      # Deterministic object key format for artifacts
│   │   │   ├── checksum.rs          # SHA-256 verification for source replays
│   │   │   ├── config.rs            # WorkerConfig from env vars, redacted display
│   │   │   ├── error.rs             # WorkerError and WorkerFailureKind types
│   │   │   ├── logging.rs           # Structured event taxonomy (WORKER_JOB_STARTED, etc.)
│   │   │   ├── health.rs            # Liveness/readiness probe state cache
│   │   │   └── shutdown.rs          # Graceful shutdown drain helpers
│   │   ├── tests/
│   │   │   ├── worker_processor.rs   # Job processor tests (no network)
│   │   │   ├── worker_config.rs      # Config parsing tests
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   └── parser-quality/              # Quality gate helpers (post-execution checks)
│       ├── src/
│       │   ├── lib.rs               # Module index
│       │   ├── coverage.rs          # Code coverage requirement validation
│       │   └── fault_report.rs      # Fault report schema validation
│       ├── tests/
│       │   ├── coverage_gate.rs      # Strict coverage checks
│       │   └── fault_report_gate.rs  # Fault report compliance
│       └── Cargo.toml
│
├── schemas/                          # Generated JSON Schema files
│   ├── parse-artifact-v3.schema.json # Contract schema for parser artifact
│   ├── parse-job-v1.schema.json      # Worker job message schema
│   └── parse-result-v1.schema.json   # Worker result message schema
│
├── scripts/                          # Development and CI/CD scripts
│   ├── coverage-gate.sh              # Code coverage enforcement
│   └── fault-report-gate.sh          # Fault report validation
│
├── .planning/                        # GSD planning documents
│   ├── PROJECT.md                    # Product definition
│   ├── STATE.md                      # Current phase and status
│   ├── ROADMAP.md                    # 9-phase roadmap
│   ├── codebase/
│   │   ├── STACK.md                  # Tech dependencies
│   │   ├── INTEGRATIONS.md           # RabbitMQ/S3/etc
│   │   ├── ARCHITECTURE.md           # This document
│   │   └── STRUCTURE.md              # Directory layout
│   ├── phases/                       # Phase execution plans (3-7)
│   ├── research/                     # Historical baseline and contract analysis
│   └── quick/                        # Ad-hoc investigation summaries
│
├── Cargo.toml                        # Workspace definition
├── Cargo.lock                        # Lock file for reproducibility
├── README.md                         # Project overview
├── AGENTS.md                         # AI agent conventions and responsibilities
├── CLAUDE.md                         # Reference to AGENTS.md
├── rust-toolchain.toml               # Rust 1.95 or later
├── rustfmt.toml                      # Code formatting config
├── clippy.toml                       # Lint configuration
├── Dockerfile                        # Container image for worker
├── .dockerignore                     # Container build ignore patterns
├── .gitignore                        # Git ignore patterns
└── .github/
    └── workflows/                    # GitHub Actions CI/CD
```

## Directory Purposes

**`crates/parser-contract/`:**
- Purpose: All serializable types shared between parser core, adapters, and external systems.
- Contains: Artifact envelope, minimal tables, events, entities, metadata, diagnostics, failure reporting, worker messages.
- Key files: `artifact.rs`, `minimal.rs`, `worker.rs`.
- Committed to git: Yes. Changes are version-gated and reviewed for schema stability.

**`crates/parser-core/`:**
- Purpose: Pure deterministic parser implementation. No I/O, no non-determinism.
- Contains: Raw JSON accessors, normalization pipelines, artifact assembly, debug sidecar generation.
- Key files: `artifact.rs` (entry point), `raw.rs` (field accessors), `metadata.rs`, `entities.rs`, `events.rs`, `aggregates.rs`.
- Committed to git: Yes. All logic must be deterministic and thoroughly tested.

**`crates/parser-cli/`:**
- Purpose: Shell-callable binary for local replay parsing and schema export.
- Contains: Argument parsing, file I/O, output formatting, worker mode delegation.
- Key files: `main.rs` (single file).
- Committed to git: Yes. Binary name is `replay-parser-2`.

**`crates/parser-worker/`:**
- Purpose: RabbitMQ/S3 runtime adapter for `server-2` integration.
- Contains: Consumer loop, job processing, S3 read/write, result publishing, graceful shutdown.
- Key files: `runner.rs`, `processor.rs`, `amqp.rs`, `storage.rs`.
- Committed to git: Yes. Async runtime lives here; parser core delegates all I/O.

**`crates/parser-quality/`:**
- Purpose: Quality gate validation (coverage, fault reports).
- Contains: Coverage percentage checkers, fault report schema validation.
- Key files: `coverage.rs`, `fault_report.rs`.
- Committed to git: Yes. Executed as part of CI gate scripts.

**`schemas/`:**
- Purpose: Generated JSON Schema files for artifact and worker messages.
- Contains: `parse-artifact-v3.schema.json`, `parse-job-v1.schema.json`, `parse-result-v1.schema.json`.
- Generated: Yes (via `cargo run --example export_schema` and `cargo run --example export_worker_schemas`).
- Committed: Yes. Schema stability is verified by contract tests.

**`scripts/`:**
- Purpose: Development and CI/CD helper scripts.
- Contains: Coverage gate enforcement, fault report validation, pre-flight checks.
- Key files: `coverage-gate.sh`, `fault-report-gate.sh`.
- Committed to git: Yes. Used by `.github/workflows` and local developer commands.

**`.planning/codebase/`:**
- Purpose: GSD codebase analysis documents.
- Contains: ARCHITECTURE.md, STRUCTURE.md (this doc), technology stack, integrations.
- Committed to git: Yes. Consumed by `/gsd-plan-phase` and `/gsd-execute-phase`.

## Key File Locations

**Entry Points:**
- `crates/parser-cli/src/main.rs`: CLI binary entry (`fn main()` around line 200).
- `crates/parser-worker/src/runner.rs`: Worker async main loop (`async fn run()`).
- `crates/parser-core/src/lib.rs:25-41`: Pure parser API (`parse_replay()`, `public_parse_replay()`, `parse_replay_debug()`).

**Configuration:**
- `Cargo.toml`: Workspace definition, lints, profiles.
- `rust-toolchain.toml`: Rust version requirement (1.95.0+).
- `rustfmt.toml`: Code formatting rules.
- `clippy.toml`: Lint rules and exceptions.
- `crates/parser-worker/src/config.rs`: Runtime configuration from environment variables.

**Core Logic:**
- `crates/parser-core/src/artifact.rs`: Main parser entry (`parse_replay()`) and artifact assembly.
- `crates/parser-core/src/raw.rs`: Defensive JSON field accessors.
- `crates/parser-core/src/metadata.rs`: Replay metadata extraction.
- `crates/parser-core/src/entities.rs`: Entity identity resolution.
- `crates/parser-core/src/events.rs`: Combat event semantics and kill categorization.
- `crates/parser-core/src/aggregates.rs`: Minimal table derivation from killed events.

**Contract:**
- `crates/parser-contract/src/artifact.rs`: `ParseArtifact` envelope and `ParseStatus` enum.
- `crates/parser-contract/src/minimal.rs`: Flat table types (`MinimalPlayerRow`, `MinimalWeaponRow`, `MinimalDestroyedVehicleRow`).
- `crates/parser-contract/src/failure.rs`: `ParseFailure` error codes and stages.
- `crates/parser-contract/src/presence.rs`: `FieldPresence<T>` enum for explicit state tracking.
- `crates/parser-contract/src/worker.rs`: RabbitMQ message types.

**Testing:**
- `crates/parser-core/tests/`: All regression and behavior tests.
  - `golden_fixture_behavior.rs`: Curated replay fixtures with expected output.
  - `deterministic_output.rs`: Byte-for-byte stability checks.
  - `fault_injection_regressions.rs`: Known malformed-file edge cases.
- `crates/parser-contract/tests/`: Schema stability and type contract tests.
- `crates/parser-cli/tests/`: CLI command and I/O tests.
- `crates/parser-worker/tests/`: Job processor and config tests (mocked networking).

## Naming Conventions

**Files:**
- Source modules use lowercase with underscores: `raw.rs`, `raw_compact.rs`, `aggregates.rs`.
- Binary crate: Single `main.rs` in `crates/parser-cli/src/`.
- Test files follow pattern: `<feature>_<aspect>.rs` e.g., `combat_event_semantics.rs`, `fault_injection_regressions.rs`.

**Functions:**
- Public API: `parse_replay()`, `public_parse_artifact()`, `parse_replay_debug()`.
- Internal helpers: `snake_case` for all functions.
- Constructors: `new()`, `from_*()` patterns used sparingly.

**Variables:**
- `camelCase` is NOT used; all variables are `snake_case`.
- Acronyms are lowercase: `amqp_url`, `s3_bucket` (not `AMQPUrl`, `S3Bucket`).
- Entity IDs: `entity_id`, `killer_entity_id`, `victim_entity_id`.
- Collections: Plural noun form: `players`, `weapons`, `entities`, `diagnostics`.

**Types:**
- Contracts use `PascalCase`: `ParseArtifact`, `MinimalPlayerRow`, `NormalizedEvent`, `FieldPresence`.
- Enums use `PascalCase` variants: `ParseStatus::Success`, `EntitySide::Blue`.
- Error types: `ParseFailure`, `WorkerError`, `CompactDecodeError`.
- Result wrappers: Implicit `Result<T, E>` in function signatures; no custom `Result` type alias.

**Modules:**
- Module names match file names: `raw`, `aggregates`, `events`, `artifacts`.
- Nested modules use the pattern: `pub mod metadata { /* content */ }`.
- Re-exports from `lib.rs`: `pub use input::{ParserInput, ParserOptions};`.

## Where to Add New Code

**New Feature (e.g., additional diagnostic rules):**
- Primary code: `crates/parser-core/src/diagnostics.rs` (if diagnostic logic) or domain module (e.g., `events.rs`).
- Contract changes: `crates/parser-contract/src/diagnostic.rs` if adding new error codes.
- Tests: `crates/parser-core/tests/` with descriptive name like `<feature_name>_detection.rs`.

**New Component/Module (e.g., additional field normalization):**
- Implementation: Create `crates/parser-core/src/<new_module>.rs`.
- Export: Add `pub mod <new_module>;` to `crates/parser-core/src/lib.rs`.
- Contract types: Add to `crates/parser-contract/src/<domain>.rs` if new contract types are needed.
- Tests: Create `crates/parser-core/tests/<new_module>_tests.rs`.

**Utilities (shared helpers):**
- Shared in core? Add to relevant domain module (e.g., `entities.rs` for entity-related helpers).
- Shared across crates? Add to `parser-contract` only if types; keep logic in `parser-core`.

**CLI enhancements:**
- New commands: Edit `crates/parser-cli/src/main.rs`, add variant to `Commands` enum.
- New arguments: Use `clap` derive macros; document with `#[arg()]` attributes.
- Tests: Add to `crates/parser-cli/tests/`.

**Worker enhancements:**
- Config: Update `crates/parser-worker/src/config.rs`.
- Processing: Modify `crates/parser-worker/src/processor.rs` for job handling.
- Logging: Add new events to `crates/parser-worker/src/logging.rs`.
- Tests: Extend `crates/parser-worker/tests/`.

## Special Directories

**`.planning/`:**
- Purpose: GSD project planning, phase execution, research, and codebase analysis.
- Generated: No (human/AI-authored).
- Committed: Yes. Critical for downstream `/gsd-plan-phase` and `/gsd-execute-phase`.

**`target/`:**
- Purpose: Cargo build artifacts.
- Generated: Yes (by `cargo build` and `cargo test`).
- Committed: No (in `.gitignore`).

**`schemas/`:**
- Purpose: Exported JSON Schema files for artifact contracts.
- Generated: Yes (by `cargo run --example export_schema`).
- Committed: Yes. Schema stability is a test requirement.

**`coverage/`:**
- Purpose: LCOV coverage reports and HTML reports.
- Generated: Yes (by coverage-gate.sh script).
- Committed: No (in `.gitignore`).

**`deploy/`:**
- Purpose: Deployment manifests and container configurations.
- Generated: No (human-authored).
- Committed: Yes. Used for worker container deployment.

---

*Structure analysis: 2026-06-13*
