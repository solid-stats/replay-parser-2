---
quick_id: 260509-ocj
slug: close-test-07-strict-coverage-blocker-fo
status: planned
created: 2026-05-09T17:31:49+07:00
description: Close TEST-07 strict coverage blocker for v1.0 milestone gap closure
requirements_addressed: [TEST-07, WF-01, WF-02]
---

# Quick Task 260509-ocj: Close TEST-07 Strict Coverage Blocker

## Objective

Close the remaining v1.0 milestone blocker by producing fresh strict coverage
evidence for the current codebase, updating milestone/state tracking, and
committing the evidence/doc changes atomically.

## Scope

In scope:

- Run the intentional strict coverage gate with the heavy opt-in.
- If strict coverage fails, repair reachable gaps with behavior tests or narrow,
  reviewed coverage exclusions with matching inline `coverage-exclusion:` markers.
- Refresh `.planning/v1.0-MILESTONE-AUDIT.md` and `.planning/STATE.md` to record
  the TEST-07 outcome.
- Write this quick task summary and commit the intended results.

Out of scope:

- Parser artifact shape, worker message contracts, S3/RabbitMQ behavior, replay
  discovery, `server-2`, `replays-fetcher`, and `web` changes.
- Reopening accepted Phase 5.2 benchmark decisions or live Timeweb validation.

## Tasks

### Task 1: Produce Fresh Strict Coverage Evidence

Files:

- `.coverage/reports/strict-summary.txt`
- `.coverage/reports/strict-missing-lines.txt`
- `.coverage/reports/coverage.json`
- `coverage/allowlist.toml` if strict evidence reveals stale exclusions
- Rust test/source files only if reachable gaps require tests or markers

Action:

- Run `COVERAGE_ALLOW_HEAVY=1 COVERAGE_JOBS=1 scripts/coverage-gate.sh --strict`.
- Inspect the strict summary and missing-lines output.
- If `uncovered_locations > 0`, close gaps with behavior tests first and exact-line
  allowlist updates only for defensive/unreachable code.

Verify:

- Strict summary reports `uncovered_locations=0`.

Done:

- Fresh strict coverage evidence exists for the current codebase.

### Task 2: Refresh Milestone Tracking

Files:

- `.planning/v1.0-MILESTONE-AUDIT.md`
- `.planning/STATE.md`

Action:

- Update audit status from `gaps_found` to passed/closed for TEST-07 if strict
  coverage passes.
- Update current state so v1.0 gap closure no longer lists strict coverage as a
  blocker.
- Keep accepted benchmark and Timeweb deployer caveats intact.

Verify:

- `rg -n "TEST-07|strict coverage|gaps_found|unsatisfied|uncovered_locations" .planning/v1.0-MILESTONE-AUDIT.md .planning/STATE.md` reflects the closed status and no stale blocker wording.

Done:

- GSD state and milestone audit match the fresh strict coverage evidence.

### Task 3: Final Gates, Summary, Commit

Files:

- `.planning/quick/260509-ocj-close-test-07-strict-coverage-blocker-fo/260509-ocj-SUMMARY.md`
- Any files changed by Tasks 1-2

Action:

- Run focused final verification.
- Write the quick summary with command evidence.
- Commit intended quick-task results atomically.

Verify:

- `git diff --check`
- `git status --short` is clean after commit.

Done:

- Quick task is committed and the repository is clean.
