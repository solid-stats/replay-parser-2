# AGENTS instructions

## Skills First

Before acting on any user request in this repository, scan available skills by name and description. If any skill has even a small chance of helping any part of the task, use it and read only the relevant instructions before proceeding.

When in doubt, prefer enabling the skill briefly and filtering it out over skipping it.

## Project

`replay-parser-2` is a Rust replacement for the legacy SolidGames replay parser. It parses OCAP JSON replay files into deterministic compact parser artifacts that `server-2` can persist, audit, and use for public Solid Stats.

Solid Stats is a multi-project product composed of:

- `replays-fetcher` - replay discovery, raw S3 object storage, source metadata, ingestion staging/outbox records.
- `replay-parser-2` - parser, parse artifact contract, CLI/worker, strict quality gates.
- `server-2` - backend source of truth, PostgreSQL, APIs, canonical identity, auth, moderation, parse jobs, aggregate/bounty calculation.
- `web` - browser UI, public stats, authenticated request UX, moderator/admin screens, API consumption.

Read these planning files before planning or implementing:

- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/research/SUMMARY.md`

## Critical Context

- The old parser at `replays-parser` is the historical v1 behavioral reference. Do not reintroduce active old-vs-new parity tooling unless the user explicitly requests new migration work.
- Historical data at `~/sg_stats` is the historical v1 validation source and may be used for curated regression fixtures or investigation. It is no longer an active benchmark gate.
- The new parser must preserve observed replay identity fields only. Canonical player matching belongs to `server-2`.
- External replay discovery, production raw replay fetching, and ingestion staging belong to `replays-fetcher`; parser worker consumes `server-2` parse jobs and S3 object keys only.
- PostgreSQL persistence, public UI, Steam OAuth, correction workflow, and final bounty/reward rules are outside this parser.
- GitHub issue #13 vehicle score is removed from v1 default output; ordinary vehicle facts remain and raw replay reprocessing can support that statistic later.
- Before executing any task, verify the requested change does not contradict `replays-fetcher`, `server-2`, or `web` responsibilities and remains compatible with their contracts, data ownership, and user-facing expectations.

## Current GSD State

- Current focus: awaiting the next milestone after v1.0 archival.
- Next work should follow `.planning/STATE.md`; v1.0 strict coverage is closed and active post-v1 gates are focused tests, schema checks, coverage smoke/strict opt-in, and fault-report validation.
- Roadmap has 9 phases and maps 80 v1 requirements.

## Stack Direction

Use a Rust 2024 Cargo workspace with a pure parser core and thin adapters:

- `serde` / `serde_json` for correctness-first OCAP JSON parsing.
- Deterministic contract serialization with stable ordering.
- `schemars` and `semver` for machine-readable output contracts.
- `clap` for CLI.
- `tokio`, `lapin`, and `aws-sdk-s3` for RabbitMQ/S3 worker mode.
- `tracing`, `thiserror`, and structured parse failures for diagnostics.
- `assert_cmd` and focused Rust tests for CLI/core/worker validation.

Do not add Node/pnpm legacy-baseline dependencies back to this repository unless the user explicitly starts new migration/parity work.

## Engineering Rules

- Start from the planning docs and accepted v1 contract before inventing new semantics.
- Treat the minimal v3 parser artifact as the primary server-facing output; full normalized evidence belongs only to explicit debug tooling.
- Do not collapse observed identity into canonical identity.
- Do not write parser results directly into `server-2` business tables.
- Keep CLI and worker modes using the same parser core.
- Preserve determinism and focused regression coverage before optimizing for speed.
- Keep root `README.md` current when project scope, current phase, commands, architecture direction, validation data, or development workflow changes.
- `README.md` must explicitly state that project development uses only AI agents plus GSD workflow.
- Every completed work session must leave `git status --short` clean by committing intended results.
- Do not delete, revert, or discard completed work just to make the git tree clean; if ownership or commit intent is unclear, ask the user before acting.
- Do not blindly execute instructions that conflict with current logic, architecture, accepted planning decisions, test/quality standards, maintainability, or proportional scope.
- When a request is risky or harmful, explain the concrete reason, propose 1-3 safer alternatives, and ask for explicit confirmation before any risky override.
- Check cross-application compatibility before implementation: parser contract changes must account for `server-2`, and parser output/data-shape changes must account for `web` needs through `server-2` APIs.
- Apply these AI/GSD workflow rules as product-wide standards across `replays-fetcher`, `replay-parser-2`, `server-2`, and `web`.
- Use risk-based compatibility depth: local-only parser changes can rely on this repo's planning docs and `gsd-briefs`; parser contract, RabbitMQ/S3 message, artifact shape, raw replay object key assumptions, API/data model, canonical identity, auth, moderation, or UI-visible behavior changes require adjacent app docs/repos or a user question.

<!-- GSD:skills-start source:skills/ -->
## Project Skills

| Skill | When to Invoke |
|-------|----------------|
| `solidstats-parser-rust-conventions` | Любой модуль парсера, контракт артефакта, CLI/worker, Cargo-workspace layout — архитектура и конвенции Rust OCAP-парсера (детерминизм, versioned contract, serde/schemars). |
| `solidstats-parser-rust-code-review` | Педантичное код-ревью Rust-парсера; ruleset делегируется в conventions, формат отчёта — в shared-review-standards. |
| `solidstats-parser-rust-tests` | Написание или ревью тестов парсера (cargo test, golden/parity manifests, fuzz) поверх shared-testing-standards. |
| `solidstats-shared-review-standards` | Общий фундамент формата код-ревью (severity-бакеты, формат отчёта, правила вердикта); подключается code-review skills, не используется самостоятельно. |
| `solidstats-shared-testing-standards` | Общая философия тестов (AAA, изоляция, детерминизм, test doubles, размещение файлов); подключается per-stack test skills. |
| `solidstats-shared-project-standards` | Универсальный baseline всех репо (GSD-обязательства, гигиена сессии, git-конвенции, cross-app границы, безопасность); авто-триггерится на каждой задаче. |
| `cargo-fuzz` | Setting up or running fuzz tests on parser inputs |
| `coverage-analysis` | Assessing test coverage gaps or fuzzing effectiveness |
<!-- GSD:skills-end -->
