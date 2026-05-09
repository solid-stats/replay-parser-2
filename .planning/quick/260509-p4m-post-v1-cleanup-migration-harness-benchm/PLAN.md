---
quick_id: 260509-p4m
slug: post-v1-cleanup-migration-harness-benchm
status: in_progress
created_at: 2026-05-09T18:05:33+07:00
mode: full
---

# Post-v1 Cleanup: Migration Harness, Benchmarks, Compare, Worker Smoke

## Goal

Remove active v1 migration tooling from the post-v1 repository workflow while keeping strict quality gates and current runtime/parser behavior intact.

## Locked Decisions

- Keep strict coverage and fault-report gates.
- Rename `parser-harness` to `parser-quality` because the remaining crate should only own quality gate tooling.
- Remove old-vs-new compare/parity tooling from CLI, scripts, tests, and crate code.
- Remove benchmark tooling and historical benchmark payloads from active workflow.
- Remove local worker-smoke Docker Compose deployment tooling.
- Keep phase and quick narrative history; delete generated/raw evidence payloads only.
- Keep debug sidecar as internal/dev tooling.

## Tasks

1. Rename `crates/parser-harness` to `crates/parser-quality` and keep only coverage/fault modules, bins, and tests.
2. Remove CLI `compare` command and parser CLI dependency on the old harness crate.
3. Delete benchmark, comparison, generated evidence, and worker-smoke files selected for cleanup.
4. Update README, AGENTS, GSD brief, coverage allowlist, scripts, and state references to reflect the post-v1 workflow.
5. Verify with formatting, focused package tests, quality scripts, workspace check, and git diff checks.
6. Write SUMMARY.md, update STATE.md quick task table, and commit all intended changes atomically.

## Verification Plan

- `cargo fmt --all --check`
- `cargo test -p parser-quality`
- `cargo test -p parser-cli`
- `cargo test -p parser-worker`
- `scripts/coverage-gate.sh --check`
- `scripts/fault-report-gate.sh`
- `cargo check --workspace`
- `git diff --check`
