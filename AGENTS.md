# AGENTS instructions

## Skills First

Before acting on any user request in this repository, scan available skills by name and description. If any skill has even a small chance of helping any part of the task, use it and read only the relevant instructions before proceeding.

When in doubt, prefer enabling the skill briefly and filtering it out over skipping it.

## Project

`replays-parser-2` is a Rust replacement for the legacy SolidGames replay parser. It parses OCAP JSON replay files into deterministic normalized events and aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public Solid Stats.

Read these planning files before planning or implementing:

- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/research/SUMMARY.md`

## Critical Context

- The old parser at `/home/afgan0r/Projects/SolidGames/replays-parser` is the required v1 behavioral reference.
- Historical data at `~/sg_stats` is the golden/test and benchmark baseline.
- The new parser must preserve observed replay identity fields only. Canonical player matching belongs to `server-2`.
- PostgreSQL persistence, public UI, Steam OAuth, correction workflow, and final bounty/reward rules are outside this parser.
- GitHub issue #13 vehicle score is an explicit v1 requirement and is covered in Phase 4.

## Current GSD State

- Current focus: Phase 1, `Legacy Baseline and Corpus`.
- Next command: `$gsd-discuss-phase 1 --auto` or `$gsd-plan-phase 1`.
- Roadmap has 7 phases and maps all 54 v1 requirements.

## Stack Direction

Use a Rust 2024 Cargo workspace with a pure parser core and thin adapters:

- `serde` / `serde_json` for correctness-first OCAP JSON parsing.
- Deterministic contract serialization with stable ordering.
- `schemars` and `semver` for machine-readable output contracts.
- `clap` for CLI.
- `tokio`, `lapin`, and `aws-sdk-s3` for RabbitMQ/S3 worker mode.
- `tracing`, `thiserror`, and structured parse failures for diagnostics.
- `insta`, `assert_cmd`, `criterion`, `hyperfine`, and old-parser comparison harnesses for validation.

Keep Node/pnpm only as a development dependency for running the legacy parser baseline.

## Engineering Rules

- Start from the planning docs and old parser behavior before inventing new semantics.
- Treat normalized events and source references as the primary artifact; aggregates are derived projections.
- Do not collapse observed identity into canonical identity.
- Do not write parser results directly into `server-2` business tables.
- Keep CLI and worker modes using the same parser core.
- Prove parity and determinism before optimizing for speed.
