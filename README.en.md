# replay-parser-2

[Русский](README.md) · **English**

Rust OCAP-replay parser for **Solid Stats** — the game statistics of the
[Solid Games](https://sg.zone) community (ArmA 3). It turns OCAP JSON replays into compact,
deterministic, versioned artifacts that `server-2` persists, audits at the
statistics-contribution level, and uses for public stats. It is the Rust replacement for
the legacy parser.

The guiding v1 principle: the default artifact must *reduce* replay data. A 10–15 MB OCAP
file must not become another 10–15 MB JSON on the ordinary ingestion path.

Part of a multi-repo platform: raw replay discovery lives in `replays-fetcher`, business
state, the API, and job orchestration in `server-2`, the web interface in `web`, and runtime
and operations in `infrastructure`. replay-parser-2 owns only parsing and the output artifact
contract; canonical player identity stays with `server-2`.

> Solid Stats is built end to end by AI agents following the
> [GSD](https://github.com/open-gsd/gsd-core) process. Development outside GSD is out of
> process.

## Status

The v1.0 milestone is complete and archived: artifact contract `3.0.0`, the CLI and the
RabbitMQ/S3 worker shipped, strict quality gates in place. Current focus: awaiting the next
milestone definition.

## Quick start

Build and parse one replay into minimal JSON:

```bash
cargo build --release
replay-parser-2 parse path/to/replay.json --output path/to/artifact.json
```

Run the worker (consumes RabbitMQ jobs, reads raw S3 objects, writes artifacts, and
publishes `parse.completed` / `parse.failed`):

```bash
replay-parser-2 worker
```

Workspace quality gate:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The full command list, worker options, deployment, and coverage gates are in
[docs/parser-reference.md](docs/parser-reference.md).

## Documentation

- [docs/parser-reference.md](docs/parser-reference.md) — artifact contract, CLI and worker,
  deployment, quality gates, validation data, the v1.0 acceptance record.
- `.planning/` — product context, requirements, roadmap, and state (GSD).
- `gsd-briefs/` — project briefs for `replays-fetcher`, `replay-parser-2`, `server-2`, `web`.

## Stack

Rust 2024 (1.95) · Cargo workspace · serde / serde_json · schemars · tokio · lapin (RabbitMQ)
· aws-sdk-s3 · tracing

## License — MIT
