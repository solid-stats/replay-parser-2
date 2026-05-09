---
quick_id: 260509-p4m
slug: post-v1-cleanup-migration-harness-benchm
status: complete
completed_at: 2026-05-09T18:20:00+07:00
---

# Summary

Post-v1 migration cleanup is complete.

## Changes

- Renamed `crates/parser-harness` to `crates/parser-quality`.
- Kept strict quality gate tooling: coverage postprocessor and fault-report validator.
- Removed active old-vs-new comparison and benchmark tooling from the CLI, scripts, tests, and crate code.
- Removed local worker-smoke Docker Compose deployment tooling.
- Removed historical v1 benchmark payloads and v2 artifact examples from the active tree.
- Updated README, AGENTS, GSD brief, coverage allowlist, worker live-smoke wording, and STATE to reflect the post-v1 workflow.

## Verification

- `cargo check --workspace` passed.
- `cargo check -p parser-quality --all-targets` passed.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test -p parser-quality` passed.
- `cargo test -p parser-cli` passed.
- `cargo test -p parser-worker` passed.
- `cargo test --workspace` passed.
- `scripts/coverage-gate.sh --check` passed.
- `scripts/fault-report-gate.sh` passed.
- `git diff --check` passed.

## Notes

- Strict coverage remains available through `COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict`.
- Debug sidecar parsing remains internal/dev tooling.
- `.planning/phases/**` and `.planning/quick/**` narrative history was preserved.
