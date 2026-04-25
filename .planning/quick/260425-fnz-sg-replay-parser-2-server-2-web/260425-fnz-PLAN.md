---
status: complete
mode: quick
created: 2026-04-25
---

# Quick Task 260425-fnz: Multi-Project Product Compatibility

## Goal

Document that Solid Stats is a multi-project product made of `sg-replay-parser-2`, `server-2`, and `web`, and require agents to check compatibility with the other applications before executing project-changing work.

## Tasks

1. Add integration requirements.
   - Files: `.planning/REQUIREMENTS.md`
   - Action: Add `INT-*` requirements for product composition and pre-task cross-application compatibility checks.
   - Verify: New requirements are mapped and totals are consistent.
   - Done: `INT-01` and `INT-02` added and mapped to Phase 1.

2. Align roadmap and project context.
   - Files: `.planning/ROADMAP.md`, `.planning/PROJECT.md`
   - Action: Make Phase 1 cover multi-project compatibility and document app boundaries.
   - Verify: Phase 1 success criteria mention `sg-replay-parser-2`, `server-2`, `web`, and cross-application compatibility checks.
   - Done: Roadmap and project context updated.

3. Update repository-facing docs and agent rules.
   - Files: `README.md`, `AGENTS.md`
   - Action: Add product composition, app ownership, and pre-task compatibility checks.
   - Verify: README and AGENTS state that tasks must not contradict adjacent apps.
   - Done: README and AGENTS updated.

4. Record quick-task completion.
   - Files: `.planning/STATE.md`, `.planning/quick/260425-fnz-sg-replay-parser-2-server-2-web/260425-fnz-SUMMARY.md`
   - Action: Track the quick task and summarize the documentation changes.
   - Verify: State links to the quick-task directory.
   - Done: Quick-task artifacts are present.
