# External Integrations

**Analysis Date:** 2026-06-13

## APIs & External Services

**Message Broker:**
- RabbitMQ - Asynchronous parse job distribution and result publication
  - SDK/Client: `lapin` 4.x (AMQP 0.9.1 protocol)
  - Auth: Environment variables only; credentials passed to AMQP URL
  - Config: `REPLAY_PARSER_AMQP_URL`

**Cloud Services:**
- AWS S3 - Raw replay object download and parser artifact upload
  - SDK/Client: `aws-sdk-s3` 1.x (AWS SDK v0.x)
  - Auth: AWS credential chain (env vars: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`)
  - Config: `AWS_REGION`, `REPLAY_PARSER_S3_ENDPOINT` (optional), `REPLAY_PARSER_S3_FORCE_PATH_STYLE`

## Data Storage

**Databases:**
- None directly. Parser does not persist to PostgreSQL; output is handed to `server-2`.

**File Storage:**
- S3 (AWS or S3-compatible)
  - Raw replay input: read from `s3://{REPLAY_PARSER_S3_BUCKET}/raw/{object_key}`
  - Parser artifacts output: written to `s3://{REPLAY_PARSER_S3_BUCKET}/{REPLAY_PARSER_ARTIFACT_PREFIX}/{encoded_replay_id}/{source_sha256}.json`
  - Deterministic key construction in `crates/parser-worker/src/storage.rs`
  - Supports Timeweb S3-compatible endpoint with path-style addressing

**Caching:**
- None - Parser is stateless; no caching layer

## Authentication & Identity

**Auth Provider:**
- Custom: No OAuth/OIDC integration
  - AWS credentials: Passed through AWS SDK credential chain (env vars or IAM roles in container)
  - RabbitMQ: Credentials embedded in AMQP URL or passed through AMQP auth

**Observed Replay Identity:**
- Parser preserves only observed identity fields from raw OCAP JSON:
  - Player nicknames (slot-merged, squad-tag stripped)
  - Entity IDs and SteamID when available
  - Game-side/team information
  - Weapon and vehicle identifiers
- Canonical player identity matching is owned by `server-2`, not the parser

## Monitoring & Observability

**Structured Logging:**
- Framework: `tracing` 0.1 + `tracing-subscriber` 0.3
- Output: JSON format with optional env-filter log-level control
- Implementation: `crates/parser-worker/src/logging.rs`
- Environment: `RUST_LOG` for filtering (standard tracing convention)

**Health Check Probes:**
- HTTP server: Axum 0.8
- Endpoints:
  - `GET /readyz` - Ready only when RabbitMQ and S3 dependencies pass; flips unavailable on shutdown
  - `GET /livez` - Process liveness; remains available during dependency degradation; fails for fatal worker state
- Config: `REPLAY_PARSER_PROBES_ENABLED`, `REPLAY_PARSER_PROBE_BIND`, `REPLAY_PARSER_PROBE_PORT`
- Docker healthcheck: Built-in using `replay-parser-2 healthcheck --url http://127.0.0.1:8080/readyz`

**Metrics/Error Tracking:**
- None - Error details are published to RabbitMQ as structured `parse.failed` messages

## CI/CD & Deployment

**Hosting:**
- Docker container (Linux x86_64)
- Deployable to any container orchestration (Kubernetes, Docker Compose, etc.)
- Base image: `rust:1.95.0-bookworm` (builder), `debian:bookworm-slim` (runtime)
- User: `65532:65532` (unprivileged)
- Entrypoint: `replay-parser-2 worker` (default mode)

**CI Pipeline:**
- GitHub Actions (assumed from GitHub repository reference)
- No CI config in this repository; CI ownership belongs to upstream SolidGames infrastructure

**Container Registry:**
- Not specified; registry/push strategy is outside parser scope

## Environment Configuration

**Required env vars (Worker Mode):**
- `REPLAY_PARSER_AMQP_URL` or defaults to `amqp://127.0.0.1:5672/%2f`
- `REPLAY_PARSER_S3_BUCKET` - No default; must be supplied
- `AWS_REGION` or defaults to `us-east-1`
- `AWS_ACCESS_KEY_ID` - AWS credential chain
- `AWS_SECRET_ACCESS_KEY` - AWS credential chain

**Optional env vars:**
- `AWS_SESSION_TOKEN` - For temporary AWS credentials
- `REPLAY_PARSER_JOB_QUEUE` - defaults to `parse.jobs`
- `REPLAY_PARSER_RESULT_EXCHANGE` - defaults to `parse.results`
- `REPLAY_PARSER_COMPLETED_ROUTING_KEY` - defaults to `parse.completed`
- `REPLAY_PARSER_FAILED_ROUTING_KEY` - defaults to `parse.failed`
- `REPLAY_PARSER_S3_ENDPOINT` - Custom S3-compatible endpoint (e.g., Timeweb, MinIO)
- `REPLAY_PARSER_S3_FORCE_PATH_STYLE` - Force path-style addressing
- `REPLAY_PARSER_ARTIFACT_PREFIX` - defaults to `artifacts/v3`
- `REPLAY_PARSER_PREFETCH` - defaults to `1`
- `REPLAY_PARSER_PROBES_ENABLED` - defaults to `true`
- `REPLAY_PARSER_PROBE_BIND` - defaults to `0.0.0.0`
- `REPLAY_PARSER_PROBE_PORT` - defaults to `8080`
- `REPLAY_PARSER_WORKER_ID` - Operator-visible identifier; defaults to hostname or `replay-parser-worker`
- `RUST_LOG` - Tracing log-level filter (e.g., `RUST_LOG=info`)

**Secrets location:**
- Secrets are NOT stored in repository
- Deployment-time injection required:
  - AWS credentials via environment or IAM role (container orchestration)
  - RabbitMQ credentials in `REPLAY_PARSER_AMQP_URL` or separate auth mechanism
  - Do not commit `.env` files or hardcoded credentials

## Webhooks & Callbacks

**Incoming:**
- RabbitMQ AMQP messages:
  - Queue: `{REPLAY_PARSER_JOB_QUEUE}` (defaults to `parse.jobs`)
  - Message format: JSON `ParseJobMessage` validated by `schemas/parse-job-v1.schema.json`
  - Schema location: `crates/parser-contract/examples/parse_job.v1.json`
  - Implementation: `crates/parser-worker/src/amqp.rs`

**Outgoing:**
- RabbitMQ AMQP result messages:
  - Exchange: `{REPLAY_PARSER_RESULT_EXCHANGE}` (defaults to `parse.results`)
  - Success routing key: `{REPLAY_PARSER_COMPLETED_ROUTING_KEY}` (defaults to `parse.completed`)
    - Message format: JSON `ParseCompletedMessage` with artifact S3 reference
    - Schema: `schemas/parse-result-v1.schema.json`
    - Example: `crates/parser-contract/examples/parse_completed.v1.json`
  - Failure routing key: `{REPLAY_PARSER_FAILED_ROUTING_KEY}` (defaults to `parse.failed`)
    - Message format: JSON `ParseFailedMessage` with structured error details
    - Schema: `schemas/parse-result-v1.schema.json`
    - Example: `crates/parser-contract/examples/parse_failed.v1.json`
  - Delivery semantics: Manual acknowledgement after successful result publication; nack on publish failure triggers broker-side retry

## Cross-Application Contracts

**Upstream (Provides to):**
- `server-2`: Parse artifact JSON via S3 object reference; minimal v3 contract at `schemas/parse-artifact-v3.schema.json`
- `replays-fetcher`: No direct contract; fetcher writes raw replays, parser reads from S3

**Downstream (Consumes from):**
- `server-2`: Parse job messages on RabbitMQ; job schema at `schemas/parse-job-v1.schema.json`
- `replays-fetcher`: Raw replay objects on S3; parser expects valid OCAP JSON files

**Schema Exports:**
- Parse artifact: `schemas/parse-artifact-v3.schema.json` (generated via `cargo run -p parser-contract --example export_schema`)
- Parse job: `schemas/parse-job-v1.schema.json` (generated via `cargo run -p parser-contract --example export_worker_schemas`)
- Parse result: `schemas/parse-result-v1.schema.json` (generated via same command)
- Schemas checked into `.planning/codebase/` or exported on-demand

---

*Integration audit: 2026-06-13*
