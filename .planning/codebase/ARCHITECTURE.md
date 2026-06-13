<!-- refreshed: 2026-06-13 -->
# Architecture

**Analysis Date:** 2026-06-13

## System Overview

`replay-parser-2` is a deterministic Rust parser workspace designed to convert OCAP JSON replay files into compact, versioned parser artifacts. The architecture separates concerns into a pure parser core, versioned contract types, and thin adapter layers for CLI and RabbitMQ/S3 worker modes.

```text
┌──────────────────────────────────────────────────────────────────┐
│                    Runtime Adapters (Non-Deterministic)          │
├─────────────────────────┬──────────────────────┬─────────────────┤
│  CLI Adapter            │  Worker Adapter      │  Quality Gates  │
│ `crates/parser-cli`     │ `crates/parser-     │ `crates/parser- │
│                         │  worker`            │  quality`       │
└────────────┬────────────┴──────────┬───────────┴────────────┬────┘
             │                       │                        │
             └───────────────────────┼────────────────────────┘
                                     │
┌────────────────────────────────────▼────────────────────────────┐
│               Pure Parser Core (Deterministic)                   │
│                    `crates/parser-core`                          │
│  • Input normalization  • Entity resolution  • Event semantics   │
│  • Artifact assembly    • Minimal table derivation               │
└────────────────────────────────────┬────────────────────────────┘
                                     │
┌────────────────────────────────────▼────────────────────────────┐
│            Contract & Type Definitions (Shared)                  │
│                  `crates/parser-contract`                        │
│  • ParseArtifact envelope  • Events & Entities  • Schema gen     │
│  • Worker job/result msgs  • Field presence    • Diagnostics    │
└──────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| Parser Core | Pure deterministic OCAP JSON parsing, normalization, and artifact assembly | `crates/parser-core/src/artifact.rs` |
| Input Normalization | Tolerant raw field accessors, JSON decoding, defensive shape handling | `crates/parser-core/src/raw.rs`, `crates/parser-core/src/raw_compact.rs` |
| Metadata Normalization | Replay-level field extraction (mission, world, time bounds) | `crates/parser-core/src/metadata.rs` |
| Entity Resolution | Observed player/vehicle identification, legacy compatibility, side assignment | `crates/parser-core/src/entities.rs` |
| Event Semantics | Combat event classification, bounty eligibility, kill categorization | `crates/parser-core/src/events.rs` |
| Minimal Aggregation | Derivation of player, weapon, and vehicle rows from killed events | `crates/parser-core/src/aggregates.rs` |
| Side Facts | Commander and round outcome determination from mission messages | `crates/parser-core/src/side_facts.rs` |
| Contract Types | Serializable data structures for all parser output and messages | `crates/parser-contract/src/artifact.rs`, `crates/parser-contract/src/minimal.rs` |
| CLI Adapter | File I/O, argument parsing, output formatting, schema export | `crates/parser-cli/src/main.rs` |
| Worker Runtime | RabbitMQ consumer, S3 storage, job processing, result publishing | `crates/parser-worker/src/runner.rs`, `crates/parser-worker/src/processor.rs` |
| Quality Gates | Coverage verification, fault-report validation | `crates/parser-quality/src/coverage.rs`, `crates/parser-quality/src/fault_report.rs` |

## Pattern Overview

**Overall:** Layered deterministic parser with isolated contract definitions and thin adapters.

**Key Characteristics:**
- **Pure Core:** `parser-core` accepts byte slices and caller-supplied metadata, returns deterministic artifacts. All I/O (files, network, timestamps) lives in adapters.
- **Versioned Contract:** `parser-contract` defines all types with `#[derive(Serialize, Deserialize, JsonSchema)]` and explicit version constants. Contract upgrades are additive.
- **Failure as Data:** Parse failures are structured in `ParseFailure` with error codes, stages, and retryability flags; no panics in core parsing.
- **Defensive Input:** Raw JSON accessors tolerate missing, null, or mistyped fields; diagnostics track all deviations.
- **Minimal Default Output:** The v3 artifact contains only `players[]`, `weapons[]`, `destroyed_vehicles[]`, and diagnostics; full debug sidecars are opt-in.
- **Deterministic Aggregation:** Player kill/death counts, team kills, and weapon stats are derived directly from raw `killed` event tuples without intermediate normalized event structures in the minimal path.

## Layers

**Input & Decoding Layer:**
- Purpose: Tolerant JSON decoding and selective raw field access.
- Location: `crates/parser-core/src/raw.rs`, `crates/parser-core/src/raw_compact.rs`
- Contains: `RawReplay`, `RawOcapRoot`, field accessor methods, shape drift detection.
- Depends on: `serde_json`, caller-supplied replay bytes.
- Used by: Metadata, entity, event, and side-facts normalization modules.

**Normalization Layer:**
- Purpose: Convert raw observations into explicit present/absent/unknown/inferred states.
- Location: `crates/parser-core/src/metadata.rs`, `crates/parser-core/src/entities.rs`, `crates/parser-core/src/events.rs`, `crates/parser-core/src/side_facts.rs`
- Contains: Normalization functions for each semantic domain.
- Depends on: Raw input layer, `FieldPresence` and `DiagnosticAccumulator` from contract.
- Used by: Artifact assembly in `parser-core/src/artifact.rs`.

**Artifact Assembly Layer:**
- Purpose: Construct versioned `ParseArtifact` envelopes from normalized data.
- Location: `crates/parser-core/src/artifact.rs`
- Contains: `parse_replay()`, `public_parse_artifact()`, `parse_replay_debug()`, failure handling.
- Depends on: All normalization modules, `ParseArtifact` from contract.
- Used by: CLI and worker adapters.

**Aggregation Layer:**
- Purpose: Derive minimal table rows directly from raw killed events.
- Location: `crates/parser-core/src/aggregates.rs`
- Contains: Player, weapon, and destroyed-vehicle row derivation; fast kill accumulation.
- Depends on: Entity index, raw event observations, `MinimalPlayerRow` contract types.
- Used by: `artifact.rs` when assembling successful parses.

**Contract Layer:**
- Purpose: Define all serializable types and external message formats.
- Location: `crates/parser-contract/src/artifact.rs`, `crates/parser-contract/src/minimal.rs`, `crates/parser-contract/src/worker.rs`
- Contains: `ParseArtifact`, `MinimalPlayerRow`, `NormalizedEvent`, `ParseJobMessage`, JSON Schema generation.
- Depends on: `serde`, `schemars`.
- Used by: Core parser, adapters, schema export.

**Adapter Layer:**
- Purpose: File I/O, queuing, object storage, argument parsing.
- Location: `crates/parser-cli/src/main.rs`, `crates/parser-worker/src/`.
- Contains: CLI commands (parse, schema, worker), worker configuration, RabbitMQ/S3 integration.
- Depends on: `parser-core`, `parser-contract`, `clap`, `tokio`, `lapin`, `aws-sdk-s3`.
- Used by: External systems (shell, `server-2`).

## Data Flow

### Primary Request Path (Parse Command)

1. **Input:** User calls `replay-parser-2 parse --input foo.json --output artifact.json` (`crates/parser-cli/src/main.rs:40-54`)
2. **File Load:** CLI reads replay bytes from disk and supplies replay ID and source checksum.
3. **Parser Invocation:** CLI calls `parser_core::public_parse_replay(input)` with `ParserInput` containing bytes, source metadata, and options.
4. **Decode:** `crates/parser-core/src/artifact.rs:31` calls `decode_compact_root()` to tolerantly parse JSON root.
5. **Normalization:** If JSON valid, `success_artifact()` calls in sequence:
   - `normalize_metadata()` (`crates/parser-core/src/metadata.rs`)
   - `normalize_entities_with_connected_events()` (`crates/parser-core/src/entities.rs`)
   - `derive_minimal_tables_from_killed_events()` (`crates/parser-core/src/aggregates.rs`)
   - `normalize_mission_message_side_facts()` (`crates/parser-core/src/side_facts.rs`)
6. **Artifact Assembly:** `success_artifact()` assembles `ParseArtifact` with contract version, parser info, diagnostics, players, weapons, destroyed vehicles, and side facts.
7. **Public Stripping:** `public_parse_artifact()` removes source references from metadata fields for minimal output.
8. **Serialization:** CLI serializes artifact to JSON (minified or pretty) and writes to output file.
9. **Return:** Exit code 0 on success, 1 on failure.

### Worker Parse Job Flow

1. **Queue Receive:** Worker listens on RabbitMQ parse job queue; `crates/parser-worker/src/runner.rs` receives message.
2. **Job Deserialization:** Message body deserializes into `ParseJobMessage` from `crates/parser-contract/src/worker.rs`.
3. **Storage Fetch:** `crates/parser-worker/src/processor.rs` fetches raw replay from S3 by object key.
4. **Checksum Validation:** `verify_source_checksum()` confirms replay bytes match declared SHA-256.
5. **Parse:** Worker calls `parser_core::public_parse_replay()` with same deterministic input as CLI.
6. **Artifact Write:** `ArtifactWrite` stores minimal artifact to S3 (or returns all-raw fallback if parse fails).
7. **Result Publishing:** `publish_completed()` or `publish_failed()` sends `ParseCompletedMessage` or `ParseFailedMessage` to RabbitMQ result exchange.
8. **Delivery Acknowledgement:** Message is ACKed only after result is confirmed published.

### Debug Artifact Path

When CLI invoked with `--debug-artifact`, a second path is taken:
1. Before `public_parse_artifact()` stripping, full `DebugParseArtifact` is serialized from `crates/parser-core/src/debug_artifact.rs`.
2. Contains full source references, raw event tuples, entity snapshots, and normalized events.
3. Written to separate file specified by user; not sent to worker or public ingestion.

**State Management:**
- Immutable functional pipeline: each stage consumes input and produces output without side effects.
- `DiagnosticAccumulator` collects warnings/errors during normalization with configurable limits.
- `SourceContext` tracks replay source metadata through all stages for error reporting.
- No global state; all context passed as function parameters.

## Key Abstractions

**ParseArtifact:**
- Purpose: Versioned envelope for all parser outputs (success, partial, skipped, failed).
- Examples: `crates/parser-contract/src/artifact.rs:31-77`
- Pattern: Single struct with optional fields (diagnostics, replay, players, weapons, destroyed_vehicles, failure); status enum indicates interpretation.

**FieldPresence<T>:**
- Purpose: Explicit tracking of field state: present with value, explicit null, unknown with reason, inferred, or not applicable.
- Examples: `crates/parser-contract/src/presence.rs`
- Pattern: Enum allowing upstack code to distinguish between "field is missing" and "field is null" and "field could not be parsed."

**RawField<T>:**
- Purpose: Result type for raw JSON field accessors indicating present/absent/drift (shape mismatch).
- Examples: `crates/parser-core/src/raw.rs:90-120`
- Pattern: Enum with `Present`, `Absent`, and `Drift` variants carrying JSON path and observed/expected shape hints for diagnostics.

**MinimalPlayerRow:**
- Purpose: Replay-local player row derived from entities and killed events, containing kills, deaths, team kills, weapons, vehicles.
- Examples: `crates/parser-contract/src/minimal.rs:30-80`
- Pattern: Flat struct with no nested kills array; kills are nested in `kills[]` vector on the row itself.

**DiagnosticAccumulator:**
- Purpose: Collects warnings and errors during parsing with configurable limits and impact assessment.
- Examples: `crates/parser-core/src/diagnostics.rs`
- Pattern: Accumulator with `.push()` method accepting severity, code, and message; `.finish()` returns bounded diagnostic report.

## Entry Points

**CLI Parse Command:**
- Location: `crates/parser-cli/src/main.rs:129-170` (approximately)
- Triggers: User shell invocation `replay-parser-2 parse`
- Responsibilities: Argument parsing, file I/O, error reporting, artifact output formatting.

**CLI Schema Export:**
- Location: `crates/parser-cli/src/main.rs:171-190` (approximately)
- Triggers: User shell invocation `replay-parser-2 schema`
- Responsibilities: Invoke `schemars` schema generation, write JSON Schema.

**CLI Worker Mode:**
- Location: `crates/parser-cli/src/main.rs:191-250` (approximately)
- Triggers: User shell invocation `replay-parser-2 worker`
- Responsibilities: Parse AMQP/S3 config, start RabbitMQ consumer loop, delegate to `parser_worker::runner`.

**Worker Runtime:**
- Location: `crates/parser-worker/src/runner.rs`
- Triggers: Worker subprocess or container startup
- Responsibilities: RabbitMQ connection, job consumer loop, graceful shutdown.

**Pure Parser Core:**
- Location: `crates/parser-core/src/lib.rs:25-41`
- Public API: `parse_replay()`, `public_parse_replay()`, `parse_replay_debug()`
- Accepts: `ParserInput<'_>` (bytes, source metadata, options)
- Returns: `ParseArtifact` or `DebugParseArtifact`

## Architectural Constraints

- **Threading:** Single-threaded parser core. Worker runtime uses `tokio` for async I/O; consumer task runs on one thread, processor spawns compute tasks on a separate `rayon` pool.
- **Global state:** No module-level singletons. All state passed through function parameters. Worker configuration is immutable after startup.
- **Circular imports:** None; dependency graph is acyclic (contract ← core ← adapters).
- **JSON processing:** Tolerant selective decoding using `serde_json::RawValue` for unvalidated fields; normalization happens in safe Rust types.
- **Error handling:** Parsing never panics; all errors are structured in `ParseFailure` or diagnostics. Adapter I/O errors propagate as non-deterministic failures.
- **Determinism:** Parser core depends only on input bytes and caller-supplied metadata. No system time, environment variables, or random number generation in core logic.
- **Artifact size:** Default minimal artifact targets < 100 KB; full debug sidecar can exceed input replay size (kept opt-in).
- **Backwards compatibility:** Artifact contract version is explicit; schema changes are versioned. Old v1 parser at `replays-parser` is historical reference only.

## Anti-Patterns

### Raw Value Unpacking Without Shape Defense

**What happens:** Code tries to directly call `.as_*()` methods on `serde_json::Value` without checking the type first.
**Why it's wrong:** Panics on type mismatches; fails the determinism contract.
**Do this instead:** Use `crates/parser-core/src/raw.rs` accessor methods like `u64_field()` or `string_field()`, which return `RawField<T>` with `Absent` or `Drift` variants for shape mismatches. Diagnostics are automatically tracked.

### Storing Source References in Default Artifact

**What happens:** Code includes `source_ref` fields in all artifact fields before public stripping.
**Why it's wrong:** Bloats minimal artifacts; leaks internal parsing logic to downstream systems.
**Do this instead:** Use `FieldPresence::Present { value, source }` for internal debug tracking, then call `strip_field_presence_source()` in `public_parse_artifact()` to remove sources before returning to CLI/worker.

### Normalizing Events into Intermediate Collections

**What happens:** Creating full `NormalizedEvent` structs for all killed events even when only aggregates are needed.
**Why it's wrong:** Doubles memory usage; slows down default minimal artifact path.
**Do this instead:** Derive minimal player rows directly from raw killed-event tuples in `crates/parser-core/src/aggregates.rs:derive_minimal_tables_from_killed_events()` without intermediate normalization. Full normalized events are only for debug artifacts.

### Unvalidated Callee Assumptions

**What happens:** Functions assume their inputs are well-formed without checking boundaries or invariants.
**Why it's wrong:** Panics when invariants are violated; makes defensive replay input handling fragile.
**Do this instead:** All public parser-core functions validate inputs with diagnostics. Use `#[must_use]` on functions returning results and always check `RawField` variants.

## Error Handling

**Strategy:** Errors are data. Parser never panics on replay content. All issues are either:
1. **Deterministic parse failures** (malformed JSON, structural gaps) → `ParseStatus::Failed` with `ParseFailure` struct.
2. **Warnings/recoverable issues** → `diagnostics[]` vector with severity/code/message.
3. **Adapter failures** (file not found, network timeout) → Worker logs and retries.

**Patterns:**
- `ParseFailure` struct (`crates/parser-contract/src/failure.rs`) carries error code, stage, retryability, message, and optional source reference.
- Deterministic failures (JSON decode, root not an object) → `ParseStatus::Failed` artifact with zero tables.
- Non-deterministic failures (network, timeout) → Logged and retried by worker.
- Diagnostics never prevent successful parse if replay has valid core structure.

## Cross-Cutting Concerns

**Logging:** Worker runtime uses `tracing` crate; events log to stdout for container/orchestration consumption. Parser core has no logging (determinism constraint). CLI prints errors to stderr.

**Validation:** Raw replay bytes are validated against JSON schema via defensive `RawField` accessors. No upstream validation assumed.

**Authentication:** Worker config uses environment variables for RabbitMQ/S3 credentials; no auth logic in parser core.

---

*Architecture analysis: 2026-06-13*
