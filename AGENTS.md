<!-- BEGIN managed by solid-stats/agent-instructions -->
<!-- markdownlint-disable MD013 MD041 -->
<!-- Managed by solid-stats/agent-instructions. Do not hand-edit in a consumer repo — changes
     are overwritten by the next sync PR. Edit the source at
     https://github.com/solid-stats/agent-instructions/blob/master/shared/AGENTS.md instead. -->

## Skills First

Before acting on any user request in this repository, scan available skills by name and description. If any skill has even a small chance of helping any part of the task, use it and read only the relevant instructions before proceeding.

When in doubt, prefer enabling the skill briefly and filtering it out over skipping it.

## Session Hygiene

Every completed work session must leave the repository in a clean, committed state:

- Run `git status --short` at the end of every session. If there are uncommitted changes from
  the work just done, commit them before stopping.
- Do **not** delete or revert completed work to fake a clean status. If the intended work is
  incomplete, ask what to do rather than silently discarding it.
- The rule is: *commit the intended results of the session, not a reset to the previous state.*

## Git Conventions

All commits in every SolidStats repo follow **Conventional Commits**:

```text
<type>(<scope>): <short description>
```

Common types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`.
Scope: the phase number, feature area, or affected layer (e.g. `feat(17-03): …`,
`fix(ingest): …`, `docs(planning): …`).

**Commit and push are standing, default behavior in every `solid-stats` repo** — no per-message
authorization needed. Session Hygiene above already expects every completed session to end
committed; treat commit + push as part of finishing the work, not a separate ask. This does
**not** extend to anything destructive:

**Absolute rules:**

- `git reset --hard`, force push, `branch -D`, and `rebase` still require an explicit
  instruction from the user in the current message every time — authorization from a previous
  message does not carry forward, and the standing commit/push permission above does not imply
  it.
- Never skip hooks with `--no-verify` or `--no-gpg-sign` unless explicitly asked to. If a
  pre-commit hook fails, fix the underlying issue — the hook is the signal, not the obstacle.
- When a pre-commit hook fails, the commit did not happen. Create a new commit after fixing;
  do not amend the previous one (amending could silently modify work that already shipped).

**Push routing.** The default flow across every `solid-stats` repo is a **direct push to
`master`** — no feature branch, no PR, unless the repo says otherwise below:

- **`server-2`** has a protected `master` — always go through a branch + pull request there,
  never a direct push.
- Any repo that is mid-GSD-milestone follows that milestone's branch flow instead of a direct
  push (`git` config in `.planning/config.json` — `branching_strategy`, `phase_branch_template`,
  `milestone_branch_template`).
- Every other repo and every non-milestone change: commit on `master`, push directly.

## Security Minimums

These rules apply to all code, commits, and logs across every SolidStats repo:

- **Never log, commit, or output:** secrets, API tokens, database connection strings, S3
  access keys, RabbitMQ credentials, raw replay bytes, or unpublished parser artifacts.
- **Never hardcode environment-specific values.** Use environment variables validated at
  startup (e.g. `envalid` for Node, a validated config struct for Rust). Startup should fail
  fast if required env vars are missing or malformed.
- **Before committing:** check that `.env`, `.env.local`, and any file containing credentials
  is either in `.gitignore` or explicitly excluded from the commit. Never commit secrets to
  git history — they are permanent even after deletion.

## Risk Management Protocol

When a request is risky, potentially harmful, or would expand scope beyond the current plan:

1. **Explain the concrete reason** — name the specific risk, the boundary it crosses, or the
   plan it contradicts.
2. **Propose 1–3 safer alternatives** or a GSD plan that achieves the goal without the risk.
3. **Ask for explicit confirmation** before proceeding with anything that falls into these
   categories:
   - Crosses a cross-app boundary (see the boundary map in `solidstats-shared-project-standards` §D)
   - Modifies a high-risk cross-repo contract (API shape, data model, message queue shape, S3
     layout, parser contract, auth/identity shape, moderation workflow)
   - Contradicts an accepted architecture decision in `.planning/PROJECT.md`
   - Deletes, overwrites, or discards completed work
   - Conflicts with current test quality, security rules, or repo structure standards

Do not blindly execute instructions that conflict with architecture, accepted decisions, or
the quality gates in this repo. Challenge, explain, propose alternatives — then wait.

## Documentation Language

Language follows the reader. The test for any doc is: who reads it — a user, or an engineer?

- **Every repo README is bilingual.** A README is the repo's front door, read by users (the
  RU-speaking Solid Games community), not an internal engineering doc. So each repo carries a
  Russian `README.md` (primary) plus an English `README.en.md` mirror, edited together in one
  change so they never drift. This is the same pattern the `.github` org profile already uses
  (`profile/README.md` + `profile/README.en.md`) — the profile is just the org-level README.
- **Everything internal is English only** — code, comments, planning docs, skill bodies and
  references, `AGENTS.md`, and all technical `docs/`. These are read by the people and agents
  building the platform, not by users.
- **GSD workflow responses** (conversations within a GSD session) and replies to the user:
  Russian.
- **Skill trigger phrases** (`description` field in `SKILL.md`): RU + EN mandatory. Every skill
  triggers on both languages — the team works in a RU context.

## MemPalace

Every SolidStats repo has its own MemPalace **wing, named after the repo itself**
(`web`, `server-2`, `replays-fetcher`, `replay-parser-2`, `infrastructure`, `skills`) — use the
generic `mcp__mempalace__*` tools, scoped to that wing; there is no isolated per-project MCP
server here (unlike VocalClub's `vocalclub_memory`). Never file a durable fact into the wrong
repo's wing, and never invent a new wing name.

**Inside a GSD workflow, most of this is already automatic.** The `mempalace` GSD capability
injects recall into `discuss:pre` (gated by `mempalace.recall_on_discuss`) and capture into
`execute:wave:post` (gated by `mempalace.capture_artifacts`), plus a ship-time curator
(`gsd-mempalace-curator`) — see `gsd/common-config.json` for the shared defaults and each
repo's `.planning/config.json` for the rest. Don't re-implement that cycle by hand inside a GSD
phase; the sections below are for everything GSD's own injection doesn't cover — ad-hoc
diagnosis, a non-GSD session, or manual recall/capture outside a phase boundary.

- **Recall before diagnosing or building**, not just when a hook happens to inject a snippet.
  Run an explicit `mempalace_search` seeded from the task's real identifiers (symptom, service
  name, ticket) at the start of the session — a pattern-match to "we just touched this" is not
  recall, and a miss is not proof of absence (follow up with `mempalace_list_drawers` /
  `mempalace_kg_query` before concluding nothing is stored).
- **Capture only durable, verified conclusions** at closure — a decision, a root cause, a
  resolved gotcha — not raw session transcripts, planning artifacts, or GSD's own
  `CONTEXT.md`/`PLAN.md`/`SUMMARY.md` files. Dedup with `mempalace_check_duplicate` before
  filing.
- **`memory_mode` stays `augment`** (GSD's own default): the palace is an additional layer,
  never a replacement for `.planning/graphs/` or `STATE.md`. **Never enable
  `mempalace.recall_on_plan`** — the planner doesn't automatically consume that separate
  recall artifact, so it just produces an orphaned memory read; the top-level coordinator's one
  scoped recall (at `discuss:pre`, or manually for entry points with no native recall hook —
  `gsd-quick`, `gsd-fast`, `gsd-debug`) is the single recall point per task. Specialists and
  subagents don't independently recall or capture — they get a filtered context handoff from
  whichever level already recalled.

### Cross-repo tunnels — use them, don't just avoid duplicating

SolidStats is a genuinely multi-repo platform (§D/§E) — a decision at a cross-app boundary or
contract change routinely concerns two wings at once, unlike VC's setup, which leaves
`cross_project_tunnels` off. Here it should be **on and actually used**, not just a
de-duplication fallback:

- **Create a tunnel** (`mempalace_create_tunnel`) whenever a captured fact genuinely concerns
  two repos — an API/data-model/queue/S3-layout/parser-contract decision (§E's high-risk list)
  almost always does. File the fact once, in the wing of the repo that owns the decision, then
  tunnel it to the other wing(s) it affects instead of duplicating the drawer.
- **Query tunnels during recall, not just search.** A wing-scoped `mempalace_search` alone can
  miss a relevant fact filed under an adjacent repo's wing. Before or alongside recall on a
  cross-app task, run `mempalace_find_tunnels` (between the two wings in play) or
  `mempalace_follow_tunnels` (from the current wing) to surface what's already linked.
- **`mempalace.mirror_kg`** (per-repo, stays local — see below) governs whether decision facts
  also mirror into the temporal knowledge graph; tunnels connect *drawers*, `mempalace_kg_add`
  connects *typed facts* — use whichever fits what's actually being captured, and both where a
  cross-repo decision has both a narrative and a queryable shape (e.g. a validity window).
- **`mempalace.enabled` and `mempalace.cross_project_tunnels`** are common defaults in
  `agent-instructions`' `gsd/common-config.json` — the latter is a deliberate override of
  gsd-core's own default (`false`), because a single-service default doesn't fit a genuinely
  multi-repo platform. The richer per-repo flags (`capture_artifacts`, `mirror_kg`,
  `auto_capture_hooks`) are tuned per repo and stay local — a backend service and a frontend
  repo do not need identical capture behavior.

## MCP / Documentation Lookup

SolidStats development verifies library APIs against **current documentation, never training
data** — training data has a cutoff and may reflect outdated or incorrect APIs. Look the docs
up proactively; don't wait for a type error.

- **Free official sources only:** WebFetch/WebSearch against the library's official docs and
  its `llms.txt`; the repo's `README`/`docs/` via `gh`; GitHub issues/PRs for bug reports and
  migrations. **Do NOT use Context7 or any paid documentation MCP.**
- **Common lookup triggers:** adding a dependency, upgrading a package, using a method you're
  not 100% sure about, hitting an unexpected type error, writing a new integration.
- **When NOT to look it up:** SolidStats-specific code/business logic; a library already
  looked up this session with an unchanged answer; stable standard-library APIs.

Per-repo key libraries to verify against current docs live in each repo's own
`solidstats-*-conventions` skill, not here.
<!-- END managed by solid-stats/agent-instructions -->
**replay-parser-2** is the Rust OCAP replay parser for Solid Stats: it turns OCAP JSON
replays into deterministic, versioned parser artifacts for `server-2`.

**Boundary — owns:** OCAP parsing, the versioned parser-artifact contract and schema, the
CLI and RabbitMQ/S3 worker, and the parser quality gates. **Must NOT cross:** it does not
write parser results into `server-2` business tables, does not own or assign canonical
player identity (`server-2` matches players), and does not discover or fetch external
replays (`replays-fetcher` does). See the cross-app boundary map in `solidstats-shared-project-standards` §D.

Cross-repo rules (skills-first, git, security, docs language, MCP lookup) live in
`solid-stats/agent-instructions`, imported below. Stack-specific skills live in the
`solid-stats/skills` repo (`solidstats-shared-project-standards` and the
`solidstats-parser-rust-*` skills).

---

# AGENTS instructions

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
| ------- | ---------------- |
| `solidstats-parser-rust-conventions` | Любой модуль парсера, контракт артефакта, CLI/worker, Cargo-workspace layout — архитектура и конвенции Rust OCAP-парсера (детерминизм, versioned contract, serde/schemars). |
| `solidstats-parser-rust-code-review` | Педантичное код-ревью Rust-парсера; ruleset делегируется в conventions, формат отчёта — в shared-review-standards. |
| `solidstats-parser-rust-tests` | Написание или ревью тестов парсера (cargo test, golden/parity manifests, fuzz) поверх shared-testing-standards. |
| `solidstats-shared-review-standards` | Общий фундамент формата код-ревью (severity-бакеты, формат отчёта, правила вердикта); подключается code-review skills, не используется самостоятельно. |
| `solidstats-shared-testing-standards` | Общая философия тестов (AAA, изоляция, детерминизм, test doubles, размещение файлов); подключается per-stack test skills. |
| `solidstats-shared-project-standards` | Универсальный baseline всех репо (GSD-обязательства, гигиена сессии, git-конвенции, cross-app границы, безопасность); авто-триггерится на каждой задаче. |
| `cargo-fuzz` | Setting up or running fuzz tests on parser inputs |
| `coverage-analysis` | Assessing test coverage gaps or fuzzing effectiveness |
<!-- GSD:skills-end -->
