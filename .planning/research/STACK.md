# Stack Research

**Domain:** Rust OCAP JSON replay parser with CLI mode, RabbitMQ worker mode, S3-compatible object input, deterministic JSON output, golden tests, legacy parser comparison, and benchmarks
**Researched:** 2026-04-24
**Confidence:** HIGH for core Rust/JSON/CLI/testing stack; MEDIUM for RabbitMQ/S3 operational details until `server-2` finalizes message schema, exchange names, retry policy, and storage endpoint conventions.

## Recommendation

Build this as a Rust 2024 workspace with a parser library, a CLI binary, and worker integration modules around the same parser entry point. Use `serde_json` as the correctness-first parser, deterministic structs/`BTreeMap` output, `lapin` for RabbitMQ AMQP 0-9-1, the official AWS SDK for S3-compatible object access, and a two-level validation harness:

1. Golden JSON tests derived from `~/sg_stats/raw_replays` and existing `~/sg_stats/results`.
2. Legacy comparison tests that run the old `/home/afgan0r/Projects/SolidGames/replays-parser` TypeScript parser as the behavioral/statistics baseline.

Do not port the old TypeScript runtime stack into the new service. Keep Node/pnpm dev-only for baseline generation and command-level benchmarks.

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust stable | 1.95.0 toolchain; `edition = "2024"`; initial `rust-version = "1.91.1"` | Core language/toolchain | Rust 2024 is stable and current crates in this stack require modern stable Rust. `aws-config`/`aws-sdk-s3` latest releases require Rust 1.91.1; using 1.95.0 in `rust-toolchain.toml` avoids MSRV friction while keeping a clear minimum. |
| Cargo workspace | Current with Rust 1.95.0 | Project organization | Use one workspace so `parser-core`, CLI, worker, tests, and benches share lockfile and types. Start simple: library + binaries is enough; split crates only when boundaries become real. |
| `serde` | 1.0.228 | Strongly typed replay and output contracts | Standard Rust serialization framework; derive typed OCAP input models where known and typed normalized output models everywhere. |
| `serde_json` | 1.0.149 | JSON parse/write engine | Correctness-first default. It integrates directly with Serde, supports streaming from `Read`, and its JSON map type is `BTreeMap` by default unless `preserve_order` is enabled. |
| `tokio` | 1.52.1 | Async runtime for worker, S3, RabbitMQ, signals | Standard runtime for Rust network services; required by the AWS SDK and supported by `lapin` defaults. Use targeted features, not `full`. |
| `clap` | 4.6.1 | CLI and env-backed worker config | Mature derive-based CLI parser. Use subcommands for `parse`, `worker`, `schema`, and `compare-legacy`; use `env` feature for worker deployment config. |
| `lapin` | 4.5.0 | RabbitMQ client | Current Rust AMQP 0-9-1 client with Tokio default runtime and Rustls default TLS. RabbitMQ docs list Lapin as a Rust client port; AMQP 0-9-1 fits a parse-job work queue. |
| `aws-config` + `aws-sdk-s3` | 1.8.16 + 1.131.0 | S3-compatible object input | Official AWS SDK for Rust, GA and async. Supports custom endpoint configuration and S3 endpoint parameters such as path-style addressing, which is required for many MinIO/S3-compatible deployments. |
| `tracing` + `tracing-subscriber` | 0.1.44 + 0.3.23 | Structured logs and spans | Standard Rust service diagnostics. Use JSON logs in worker mode and human logs in CLI mode. |
| `thiserror` + `anyhow` | 2.0.18 + 1.0.102 | Error model | Use `thiserror` for typed parser/worker failures that become structured `parse.failed`; use `anyhow` only at binary boundaries and test helpers for context. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde_path_to_error` | 0.1.20 | JSON decode diagnostics | Wrap replay deserialization so malformed OCAP files report the failing path in structured parse errors. |
| `schemars` | 1.2.1 | JSON Schema generation | Generate and check the parser output contract schema from Rust types; store schema artifacts with parser contract versions. |
| `semver` | 1.0.28 | Contract version parsing | Use for `parser_contract_version` compatibility checks; reject unsupported major versions explicitly. |
| `sha2` + `hex` | 0.11.0 + 0.4.3 | Replay checksum verification | Use SHA-256 by default when validating `checksum` from jobs. Do not rely on S3 ETag as checksum unless `server-2` explicitly defines that contract. |
| `time` | 0.3.47 | UTC timestamp/date handling | Use with `serde` feature for output timestamps. Keep timezone handling UTC-only unless old-parser comparison proves a legacy Moscow-time conversion is part of the contract. |
| `tokio-util` | 0.7.18 | Cancellation and IO adapters | Use `CancellationToken` for worker shutdown. Use IO adapters only if parsing directly from async streams; default worker path should stream S3 to a tempfile while hashing, then parse via the same `Read` path as CLI. |
| `futures-util` | 0.3.32 | Stream utilities | Needed around `lapin` consumers and async stream handling. |
| `tempfile` | 3.27.0 | Worker downloads and tests | Stream S3 bodies into temporary files while verifying checksum, then parse with the same core file parser as CLI. |
| `uuid` | 1.23.1 | Optional typed job/replay IDs | Use only if `server-2` declares job IDs or replay IDs as UUIDs. Otherwise keep observed IDs as strings. |
| `bytes` | 1.11.1 | Message payload/body buffers | Use at RabbitMQ/S3 boundaries when direct bytes are cleaner than `Vec<u8>`. Do not expose it in parser-core public types. |
| `simd-json` | 0.17.0, optional feature only | Performance accelerator candidate | Do not make this the default parser until golden tests are stable. Add behind a `simd-json` feature and adopt only if Criterion corpus benchmarks show material speedup without output drift. |

### Testing and Benchmarking Libraries

| Library/Tool | Version | Purpose | When to Use |
|--------------|---------|---------|-------------|
| `insta` | 1.47.2 with `json`, `redactions` | Focused JSON snapshots | Use for representative small/medium fixtures and structured error snapshots. Do not snapshot all 3,938 historical replays through `insta`. |
| Plain golden files + `serde_json::Value` | `serde_json` 1.0.149 | Full replay contract regression | Use for corpus-derived expected outputs. Compare parsed `Value` plus deterministic writer output; keep fixtures reviewed and versioned. |
| `similar-asserts` | 2.0.0 | Readable diffs | Use for failing golden/legacy comparisons after normalizing known unstable fields. |
| `assert_cmd` + `predicates` | 2.2.1 + 3.1.4 | CLI behavior tests | Test `parse`, `schema`, `compare-legacy`, and bad-input exit behavior. |
| `proptest` | 1.11.0 | Invariant/property tests | Use for small normalized event/aggregate invariants, not for randomly generating full OCAP files. |
| `criterion` | 0.8.2 | In-process parser benchmarks | Benchmark `parse_file`, `parse_reader`, aggregation, and hot event normalization with `Throughput::Bytes`. |
| `hyperfine` | 1.20.0 CLI | Command-level old/new benchmark | Compare old parser commands against new CLI on fixed corpora. This is the right tool for "old TypeScript command vs Rust binary" wall-clock measurements. |
| `testcontainers` | 0.27.3 | RabbitMQ/S3 integration tests | Start disposable RabbitMQ and MinIO/LocalStack-style containers. Prefer `GenericImage` first; add modules only if they cover exactly the needed service behavior. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `rustfmt` + `clippy` | Formatting and lint gates | Run on stable toolchain in CI. Treat clippy warnings as failures once initial migration skeleton is stable. |
| `cargo-nextest` | Fast test runner | Use for local and CI test execution, especially corpus partitions. Current crates.io version checked: 0.9.133. |
| `cargo-llvm-cov` | Coverage | Use after core parsing stabilizes; avoid coverage targets for the full corpus benchmark path. Current version checked: 0.8.5. |
| `cargo-deny` | License/advisory/duplicate dependency policy | Important because the binary will ship as service infrastructure. Current version checked: 0.19.4. |
| `cargo-audit` | RustSec advisory scan | Keep in CI or covered by `cargo-deny advisories`. Current version checked: 0.22.1. |
| Docker | Integration tests | Required for RabbitMQ and S3-compatible storage tests through `testcontainers`. |
| Node + pnpm | Legacy parser baseline only | Old parser uses `pnpm@10.33.0` and scripts including `parse`, `parse:dist`, and `build`. Keep this out of the Rust runtime image. |

## Installation

Recommended initial `Cargo.toml` dependency set:

```bash
# Core runtime
cargo add tokio@1.52.1 --features rt-multi-thread,macros,signal,time,fs,io-util,sync
cargo add clap@4.6.1 --features derive,env
cargo add serde@1.0.228 --features derive
cargo add serde_json@1.0.149 serde_path_to_error@0.1.20
cargo add thiserror@2.0.18 anyhow@1.0.102
cargo add tracing@0.1.44
cargo add tracing-subscriber@0.3.23 --features env-filter,json

# Worker integration
cargo add lapin@4.5.0 futures-util@0.3.32
cargo add aws-config@1.8.16 aws-sdk-s3@1.131.0
cargo add tokio-util@0.7.18 --features rt,io-util
cargo add tempfile@3.27.0

# Contract and checksums
cargo add schemars@1.2.1 semver@1.0.28
cargo add sha2@0.11.0 hex@0.4.3
cargo add time@0.3.47 --features serde,macros

# Optional performance feature, not default
cargo add simd-json@0.17.0 --optional

# Tests and benchmarks
cargo add --dev insta@1.47.2 --features json,redactions
cargo add --dev assert_cmd@2.2.1 predicates@3.1.4
cargo add --dev similar-asserts@2.0.0 proptest@1.11.0
cargo add --dev criterion@0.8.2 --features html_reports
cargo add --dev testcontainers@0.27.3
```

Recommended tool installs:

```bash
rustup toolchain install 1.95.0
cargo install cargo-nextest --version 0.9.133
cargo install cargo-llvm-cov --version 0.8.5
cargo install cargo-deny --version 0.19.4
cargo install cargo-audit --version 0.22.1
cargo install hyperfine --version 1.20.0

# Legacy baseline only, run inside /home/afgan0r/Projects/SolidGames/replays-parser
corepack enable
pnpm install --frozen-lockfile
pnpm run build
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `serde_json` first | `simd-json` | Use `simd-json` only after golden tests are locked and benchmarks prove speedup on the actual OCAP corpus. It requires mutable byte buffers and has higher integration complexity. |
| `lapin` | `amqprs` | Use `amqprs` only if `server-2` or ops already standardizes on it. `lapin` is the safer default because it is widely used, current, and listed by RabbitMQ as a Rust client port. |
| RabbitMQ AMQP 0-9-1 queue | RabbitMQ Streams client | Use streams only if the product changes from "parse jobs" to append-only event stream replay. For a work queue, streams add unnecessary offset/retention semantics. |
| Official `aws-sdk-s3` | `object_store` | Use `object_store` if storage becomes an abstraction across local filesystem/S3/GCS/Azure. For S3-compatible object input from `server-2`, the official SDK gives better endpoint, credential, retry, and S3 API coverage. |
| Official `aws-sdk-s3` | `rust-s3` | Use only for a deliberately minimal S3 client after proving the official SDK is too heavy. It is not the default for service integration. |
| Official `aws-sdk-s3` | `rusoto_s3` | Do not choose for new work. `rusoto` belongs to the pre-GA AWS Rust SDK era and does not match the current AWS Rust ecosystem. |
| `clap` | `argh`, `pico-args` | Use smaller parsers only for tiny tools. This service needs subcommands, env config, help text, and stable operator ergonomics. |
| `criterion` | Nightly `test::Bencher` | Use nightly benches only for experiments. Criterion works on stable and reports statistically useful throughput. |
| `testcontainers` | Hand-written docker compose scripts | Use compose only for local manual smoke tests. Automated integration tests should own service lifecycle and ports. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Hand-rolled JSON parser | OCAP JSON is large and messy; custom parsing risks correctness drift and hidden edge cases. | `serde_json` with typed structs and `serde_path_to_error`; optional `simd-json` after benchmarks. |
| `HashMap` in output contract types | Iteration order is randomized and will make deterministic JSON/golden tests fragile. | Structs for known fields and `BTreeMap` for dynamic maps. |
| `serde_json` `preserve_order` feature in core output | It switches `serde_json::Map` away from default sorted `BTreeMap` behavior and can preserve arbitrary source/insertion order. | Default `serde_json` plus explicit structs/`BTreeMap`. |
| Snapshotting every historical replay through `insta` | Produces noisy, hard-to-review snapshot churn and huge test artifacts. | Curated `insta` snapshots plus corpus golden files and summarized diff reports. |
| Treating the old parser as "just inspiration" | The old parser is now a behavioral reference, schema/statistics baseline, and benchmark command source. Ignoring it would make the rewrite unverifiable. | Add a legacy runner that shells out to old parser commands and stores normalized comparison artifacts. |
| Automatic TypeScript-to-Rust code generation from old `.d.ts` files | The old types are useful documentation, but generated Rust models will preserve legacy ambiguity and poor ownership boundaries. | Manually model Rust input/output types, then verify behavior against old parser fixtures and outputs. |
| Carrying Node/pnpm into the Rust service container | It bloats deployment and confuses runtime ownership. | Use Node/pnpm only in dev/CI jobs that regenerate old-parser baselines. |
| Trusting S3 ETag as a universal checksum | Multipart uploads and S3-compatible systems can make ETag unsuitable as file content hash. | Verify the `checksum` field from `server-2` with `sha2`/`hex`, after contract specifies algorithm. |
| Auto-ack RabbitMQ consumption | A worker crash after delivery but before parse completion would lose jobs. | Manual ack after successful `parse.completed` publish/storage; `nack`/dead-letter on structured failures according to retry policy. |
| Relying on experimental automatic reconnect as the only recovery plan | AMQP clients can drop channels/connections; reconnect behavior must be explicit in service control flow. | Supervised connect/consume loop with backoff, idempotent job handling, and health reporting. |
| Direct parser writes to PostgreSQL | Parser ownership is the output contract, not business persistence. | Emit completed/failed messages and/or S3 artifacts for `server-2` to persist. |

## Stack Patterns by Variant

**Parser core:**
- Expose `parse_reader<R: std::io::Read>() -> Result<ParseArtifact, ParseError>` and `parse_file(&Path)`.
- Deserialize raw OCAP into permissive input structs where fields are known; use `serde_json::Value` only at the uncertain boundary.
- Normalize into fully typed output structs with explicit `Unknown`/`None` states for missing winner, SteamID, commander, and side data.
- Serialize output with `serde_json::to_writer`/`to_writer_pretty` from structs. Add a trailing newline in CLI output for shell ergonomics.

**CLI mode:**
- Use `clap` subcommands:
  - `parse --input <file> --output <file|-> --contract-version <semver>`
  - `schema --output <file|->`
  - `compare-legacy --replay <file> --legacy-root <path>`
  - `bench-corpus --corpus <dir>` only if a thin command wrapper around Criterion/hyperfine is useful.
- CLI must be deterministic by default: no timestamps in output except source replay timestamps and parser contract metadata.

**Worker mode:**
- Use `tokio` async main and `tracing` JSON logs.
- Consume RabbitMQ with manual ack, bounded prefetch, and a worker concurrency limit.
- For each job: validate message with Serde, download S3 object to a tempfile while hashing, compare checksum, parse via parser-core, publish/store result, then ack.
- Add publisher confirms for `parse.completed`/`parse.failed` if result messages are published to RabbitMQ. RabbitMQ docs treat consumer acknowledgements and publisher confirms as separate reliability mechanisms.

**S3-compatible storage:**
- Use `aws_config::defaults(BehaviorVersion::latest())` and build an S3 client with `endpoint_url` for non-AWS storage.
- Include a config option for path-style addressing; many S3-compatible deployments need it.
- Keep bucket, endpoint URL, region, access key, secret, and force-path-style in worker config, not parser-core.

**Legacy parser migration/comparison:**
- Discover and document the exact old command before locking benchmarks. Current local old parser evidence:
  - Root: `/home/afgan0r/Projects/SolidGames/replays-parser`
  - Package manager: `pnpm@10.33.0`
  - Relevant scripts: `parse` (`tsx src/start.ts`), `build`, and `parse:dist` (`node dist/start.js`)
- Add a dev-only legacy runner that executes old commands from the old parser root with controlled env and fixed fixture directories.
- Normalize old outputs only for fields known to be outside the new contract. Do not normalize away statistic differences without explicit migration notes.

**Benchmarking:**
- Use Criterion for Rust function-level parser throughput and allocation-sensitive hot paths.
- Use hyperfine for full-command comparisons:
  - old TypeScript parser command from `replays-parser`
  - new Rust CLI parse command
  - fixed replay subset and full corpus runs
- Record bytes/sec, replays/sec, p50/p95 wall time, and output-equivalence status. The 10x goal must be stated relative to the discovered old command.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| Rust 1.95.0 | All recommended latest crates | Current stable as of 2026-04-24. Pin in `rust-toolchain.toml` for reproducible implementation. |
| Rust 1.91.1 minimum | `aws-config` 1.8.16, `aws-sdk-s3` 1.131.0 | Crates.io metadata reports Rust 1.91.1 MSRV for both. This is the effective workspace MSRV if using latest AWS SDK. |
| Rust 1.88+ | `lapin` 4.5.0, `simd-json` 0.17.0, `testcontainers` 0.27.3 | Worker/tests/perf optional pieces exceed the Rust 2024 minimum. |
| Rust 1.85+ | Rust 2024 edition, `clap` 4.6.1, `sha2` 0.11.0, `assert_cmd` 2.2.1, `proptest` 1.11.0 | Rust 1.85 stabilized Rust 2024, but this project should not target 1.85 as the whole-workspace MSRV if using latest AWS SDK. |
| `lapin` 4.5.0 | Tokio default runtime + Rustls default TLS | Default features include `default-runtime` -> `tokio` and `rustls`. Avoid `native-tls` unless deployment cert policy requires it. |
| `aws-sdk-s3` 1.131.0 | Tokio runtime + Rustls default HTTPS | Default features include `rt-tokio`, `rustls`, and default HTTPS client. Start with defaults; trim only if build size becomes a problem. |
| `serde_json` 1.0.149 | Deterministic output with default map backing | Default `serde_json::Map` uses `BTreeMap`; avoid `preserve_order` in core output. |
| Old parser | Node + pnpm 10.33.0 | Dev/CI baseline only. Keep old-parser dependencies isolated from Rust service lockfile/image. |

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Rust toolchain | HIGH | Official Rust release pages confirm Rust 1.95.0 current stable and Rust 2024 stabilization in 1.85.0. Crates.io metadata confirms latest dependency MSRVs. |
| JSON parsing/output | HIGH | Serde/serde_json are standard and official docs confirm default sorted map backing. Determinism still depends on output type discipline. |
| CLI stack | HIGH | `clap` derive/env is a stable standard for Rust CLIs and fits both local parse and worker config. |
| RabbitMQ stack | MEDIUM-HIGH | `lapin` is current and RabbitMQ lists it as a Rust client port. Final ack/retry/DLX details depend on `server-2` contract. |
| S3 stack | MEDIUM-HIGH | AWS SDK is official and supports endpoint customization. Exact force-path-style/credentials behavior must be validated against the chosen S3-compatible service. |
| Legacy comparison stack | MEDIUM | Local old parser commands were discovered from `package.json`, but exact benchmark command, inputs, env, and old output tolerances still need implementation-phase confirmation. |
| Performance stack | MEDIUM | Criterion/hyperfine are appropriate, but `serde_json` vs `simd-json` must be decided from the actual OCAP corpus. |

## Sources

- Context7 `/websites/rs_tokio_1_49_0` and `/tokio-rs/tokio` - Tokio runtime/shutdown docs consulted.
- Context7 `/clap-rs/clap` - derive, subcommand, and env argument docs consulted.
- Context7 `/serde-rs/json` and serde_json docs.rs map docs - JSON serialization and default `BTreeMap` map backing verified: https://docs.rs/serde_json/latest/serde_json/map/index.html
- Context7 `/websites/rs_lapin` - AMQP connection, publish/consume, ack/QoS API docs consulted.
- Context7 `/bheisler/criterion.rs` - Criterion throughput benchmark docs consulted.
- Context7 `/mitsuhiko/insta` - snapshot testing docs consulted.
- Context7 `/gresau/schemars` - Serde-compatible JSON Schema generation docs consulted.
- Context7 `/testcontainers/testcontainers-rs` - async container lifecycle and wait strategies consulted.
- Context7 `/tokio-rs/tracing`, `/dtolnay/anyhow`, `/dtolnay/thiserror` - logging/error handling patterns consulted.
- Rust release announcements - Rust 1.95.0 current stable and release list: https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/ and https://blog.rust-lang.org/releases/
- Rust 1.85.0 / Rust 2024 stabilization: https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/
- AWS SDK for Rust endpoint configuration, including custom endpoint URL and S3 endpoint parameters: https://docs.aws.amazon.com/sdk-for-rust/latest/dg/endpoints.html
- AWS S3 examples using SDK for Rust: https://docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_s3_code_examples.html
- RabbitMQ tutorials/client ports and AMQP 0-9-1 context: https://www.rabbitmq.com/tutorials
- RabbitMQ acknowledgements and publisher confirms reliability guide: https://www.rabbitmq.com/docs/confirms
- hyperfine project source/version guidance: https://github.com/sharkdp/hyperfine
- Crates.io API queried 2026-04-24 for current versions/MSRV: `tokio`, `clap`, `serde`, `serde_json`, `serde_path_to_error`, `simd-json`, `lapin`, `aws-config`, `aws-sdk-s3`, `tracing`, `tracing-subscriber`, `thiserror`, `anyhow`, `criterion`, `insta`, `testcontainers`, `schemars`, `cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-llvm-cov`, `hyperfine`.
- Local legacy parser source consulted: `/home/afgan0r/Projects/SolidGames/replays-parser/package.json`, `/home/afgan0r/Projects/SolidGames/replays-parser/README.md`, `/home/afgan0r/Projects/SolidGames/replays-parser/src/start.ts`, `/home/afgan0r/Projects/SolidGames/replays-parser/src/index.ts`.

---
*Stack research for: Rust OCAP JSON replay parser/worker service*
*Researched: 2026-04-24*
