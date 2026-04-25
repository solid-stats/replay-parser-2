# sg-replay-parser-2

Rust replacement for the legacy SolidGames OCAP replay parser.

`sg-replay-parser-2` parses OCAP JSON replay files into deterministic normalized events and aggregate outputs that `server-2` can persist, audit, compare against golden data, and use for public Solid Stats.

## Product Context

Solid Stats is a multi-project product made of:

- `sg-replay-parser-2`: Rust parser, deterministic parse artifacts, parser contract schema, CLI/worker modes, and old-parser parity.
- `server-2`: backend source of truth, PostgreSQL persistence, APIs, canonical identity, Steam OAuth, roles, moderation, parse job orchestration, aggregate/bounty calculation, and operations visibility.
- `web`: browser UI, public stats pages, authenticated request UX, moderator/admin screens, and API consumption from `server-2`.

Before executing project-changing work, agents must verify that the change does not contradict the other applications and remains compatible with their contracts, data ownership, and user-facing expectations.

GSD workflow rules are product-wide standards for all three applications. Compatibility checks are risk-based:

- Local-only parser changes can rely on this repo's planning docs, README, AGENTS rules, and `gsd-briefs`.
- Parser contract, RabbitMQ/S3 message, artifact shape, API/data model, canonical identity, auth, moderation, or UI-visible behavior changes require checking adjacent app docs/repos when available.
- If compatibility evidence is missing or contradictory, ask before proceeding and propose a smaller GSD path or cross-project plan.

## Development Mode

This project is developed only by AI agents using the GSD workflow.

Direct non-GSD development is out of process. Project-changing work must be captured through GSD planning, phase execution, or quick-task artifacts under `.planning/`.

Completed work must leave the git working tree clean. Intended results are committed; they are not deleted or reverted just to make `git status` clean. If it is unclear whether changes should be committed, preserved uncommitted, or excluded from the task, ask before acting.

AI agents must not blindly execute instructions that conflict with current logic, architecture, accepted planning decisions, quality standards, maintainability, or proportional scope. The expected response is to explain why the request is risky, propose safer alternatives, and ask for explicit confirmation before any risky override.

## Current Status

- Current focus: Phase 1, `Legacy Baseline and Corpus`.
- Roadmap: 7 phases.
- v1 requirements: 71 mapped requirements.
- Next command: `$gsd-discuss-phase 1 --auto` or `$gsd-plan-phase 1`.

## Scope

In scope:

- Rust parser for historical OCAP JSON replay files.
- Deterministic normalized event output and aggregate projections.
- Versioned parser output contract with source references.
- CLI parsing and schema/export tooling.
- Golden corpus comparisons against `~/sg_stats`.
- RabbitMQ/S3 worker mode for `server-2` integration.
- Benchmarks against the pinned legacy parser baseline.

Out of scope:

- Public website and UI.
- Steam OAuth.
- Canonical player identity matching.
- PostgreSQL business-table persistence.
- User moderation and correction workflows.
- Final financial reward or payout rules.

## Required References

- Planning source of truth: `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md`.
- Legacy parser reference: `/home/afgan0r/Projects/SolidGames/replays-parser`.
- Historical corpus and golden data: `~/sg_stats`.
- Project agent rules: `AGENTS.md`.

## README Maintenance

Keep this README current whenever project scope, current GSD phase, architecture direction, commands, validation data, benchmark expectations, integration workflow, or development workflow changes.

The README must always state that development is performed only by AI agents using GSD.
