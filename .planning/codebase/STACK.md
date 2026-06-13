# Technology Stack

**Analysis Date:** 2026-06-13

## Languages

**Primary:**
- Rust 1.95.0 (2024 edition) - All parser, CLI, worker, and quality-gate crates
- Defined in `rust-toolchain.toml`

**Build/Scripting:**
- Bash - Scripts for coverage gates, fault reports, and cargo budget management (`.sh` files in `scripts/`)

## Runtime

**Environment:**
- Rust 1.95.0 - Native compilation target
- Tokio 1.x - Async runtime for worker mode with multi-threaded executor
- Linux x86_64 (Debian bookworm-slim container runtime)

**Package Manager:**
- Cargo (workspace resolver v3)
- Lockfile: `Cargo.lock` present and committed

## Frameworks

**Core:**
- Tokio 1.x (`features = ["rt-multi-thread", "macros", "net", "signal", "time"]`) - Async task scheduling and lifecycle management in worker mode
- Axum 0.8 - HTTP server for health check probes (`/readyz`, `/livez` endpoints)

**Parsing & Serialization:**
- `serde` 1 (with derive feature) - Serialization/deserialization foundation across all crates
- `serde_json` 1 (with `raw_value` feature in parser-core) - JSON parsing and minified/pretty output
- `schemars` 1 (with `semver1` feature) - Machine-readable JSON Schema generation for parse artifact contracts

**CLI:**
- `clap` 4 (with derive feature) - Command-line argument parsing for Parse/Schema/Worker/Healthcheck subcommands

**Messaging & Storage:**
- `lapin` 4 - RabbitMQ AMQP 0.9.1 client for job consumption and result publication
- `aws-sdk-s3` 1 - AWS S3 SDK for raw replay download and artifact upload
- `aws-config` 1 - AWS credential chain (supports `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`)

**Testing:**
- `assert_cmd` 2 - Subprocess assertion testing for CLI validation

**Development:**
- `rustfmt` (managed by `rust-toolchain.toml`) - Code formatting with standard Rust style
- `clippy` (managed by `rust-toolchain.toml`) - Linting with strict workspace lint rules

## Key Dependencies

**Critical:**
- `thiserror` 2 - Structured error types across all crates; required for `ParseFailure` and `WorkerError` handling
- `tokio` 1 - Async runtime with multi-threaded executor; blocking on worker mode startup and graceful shutdown
- `serde_json` 1 - JSON parsing with `raw_value` allows selective field extraction without full DOM cloning
- `lapin` 4 - AMQP message broker integration; non-optional for worker mode
- `aws-sdk-s3` 1 - S3-compatible object storage (supports AWS S3 and Timeweb S3); replaces HTTP fetch for raw replay input in worker mode

**Infrastructure:**
- `hex` 0.4 - Hexadecimal encoding for SHA-256 checksums (artifact key suffix and proof validation)
- `sha2` 0.10 - SHA-256 hashing for replay checksum validation and artifact naming
- `percent-encoding` 2 - URL encoding for replay ID path segments in S3 artifact keys
- `futures-util` 0.3 - Stream trait extensions and utilities for async stream handling
- `tokio-util` 0.7 - `CancellationToken` for graceful worker shutdown coordination
- `tracing` 0.1 - Structured logging foundation with `tracing-subscriber` for JSON output
- `tracing-subscriber` 0.3 (`features = ["env-filter", "fmt", "json"]`) - Structured JSON log formatting and log-level filtering
- `semver` 1 (with serde feature) - Semantic version parsing and comparison for parser contract versioning
- `toml` 0.8 - TOML parsing for quality-gate configuration files

**Testing only:**
- `jsonschema` 0.46.2 (without default features) - JSON Schema validation for contract test fixtures
- `tower` 0.5 (with util feature) - HTTP testing middleware for worker health probe validation

## Configuration

**Environment:**
- AWS SDK credential chain:
  - `AWS_ACCESS_KEY_ID` - AWS S3 access key (required in worker mode)
  - `AWS_SECRET_ACCESS_KEY` - AWS S3 secret key (required in worker mode)
  - `AWS_SESSION_TOKEN` - Optional session token for temporary credentials
  - `AWS_REGION` - S3 region (defaults to `us-east-1` if absent)

- RabbitMQ configuration:
  - `REPLAY_PARSER_AMQP_URL` - RabbitMQ AMQP connection string (defaults to `amqp://127.0.0.1:5672/%2f`)
  - `REPLAY_PARSER_JOB_QUEUE` - Parse job queue name (defaults to `parse.jobs`)
  - `REPLAY_PARSER_RESULT_EXCHANGE` - Result message exchange (defaults to `parse.results`)
  - `REPLAY_PARSER_COMPLETED_ROUTING_KEY` - Routing key for successful parses (defaults to `parse.completed`)
  - `REPLAY_PARSER_FAILED_ROUTING_KEY` - Routing key for failed parses (defaults to `parse.failed`)

- S3 configuration:
  - `REPLAY_PARSER_S3_BUCKET` - Raw replays/artifacts bucket name (required in worker mode)
  - `REPLAY_PARSER_S3_ENDPOINT` - Optional S3-compatible endpoint URL (Timeweb, MinIO, etc.)
  - `REPLAY_PARSER_S3_FORCE_PATH_STYLE` - Force path-style S3 addressing instead of virtual-hosted-style
  - `REPLAY_PARSER_ARTIFACT_PREFIX` - Artifact object key prefix (defaults to `artifacts/v3`)

- Worker lifecycle:
  - `REPLAY_PARSER_PREFETCH` - In-flight job count (defaults to `1` for single-worker safety)
  - `REPLAY_PARSER_PROBES_ENABLED` - Enable HTTP health check probes (defaults to `true`)
  - `REPLAY_PARSER_PROBE_BIND` - Probe server bind address (defaults to `0.0.0.0`)
  - `REPLAY_PARSER_PROBE_PORT` - Probe server port (defaults to `8080`)
  - `REPLAY_PARSER_WORKER_ID` - Operator-visible worker identifier for logs and probes

**Build:**
- `Cargo.toml` - Workspace root with lints, profiles, and member definitions
- `.cargo/config.toml` - Cargo aliases (fmt-check, lint, quality-check, quality-doc, quality-test)
- `rust-toolchain.toml` - Rust version and component pins
- `rustfmt.toml` - Code formatting (using defaults)
- `clippy.toml` - Clippy linting rules (using workspace configuration)
- `Dockerfile` - Multi-stage Docker image for deployable worker container

**Coverage & Quality Gates:**
- `coverage/allowlist.toml` - Allowlist of source lines excluded from coverage requirement
- `scripts/coverage-gate.sh` - Instrumented coverage validation with budget management
- `scripts/cargo-budget.sh` - Target directory size management for repeated builds
- `scripts/fault-report-gate.sh` - Structured fault validation

## Platform Requirements

**Development:**
- Rust 1.95.0+ (enforced by `rust-toolchain.toml`)
- Cargo with workspace resolver v3 support
- Docker (optional, for worker image builds)

**Production:**
- Linux x86_64 with GNU libc (Debian bookworm-slim base image)
- RabbitMQ 3.x+ AMQP broker
- AWS S3 or S3-compatible object storage (Timeweb, MinIO, etc.)
- Network access to RabbitMQ AMQP port (typically 5672)
- Network access to S3 API endpoint

**Container Runtime:**
- Docker 20.x+ for image building and execution
- User `65532:65532` (numeric UID, unprivileged) at runtime
- Port 8080 exposed for HTTP health probes

---

*Stack analysis: 2026-06-13*
