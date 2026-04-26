---
status: complete
mode: quick
created: 2026-04-25
---

# Quick Task 260425-fro: Product-Wide GSD Rules and Risk-Based Checks

## Goal

Clarify unclear GSD description points by documenting that the workflow rules are product-wide standards across `replay-parser-2`, `server-2`, and `web`, and that cross-application compatibility checks are risk-based.

## User Decisions

- Rule scope: all projects. The same GSD workflow standards should apply across `replay-parser-2`, `server-2`, and `web`.
- Compatibility depth: risk-based. Local-only changes can use local docs/briefs; contract, API/data, queue/storage, identity/auth, moderation, or UI-visible changes need adjacent app evidence or a user question.
- Extra boundary rule: no extra quick-vs-phase/artifact/escalation rule now.

## Tasks

1. Add clarifying integration requirements.
   - Files: `.planning/REQUIREMENTS.md`
   - Action: Add `INT-*` requirements for product-wide GSD rules and risk-based compatibility depth.
   - Verify: New requirements are mapped and totals are consistent.
   - Done: `INT-03` and `INT-04` added and mapped to Phase 1.

2. Align current project docs.
   - Files: `.planning/ROADMAP.md`, `.planning/PROJECT.md`, `.planning/STATE.md`, `README.md`, `AGENTS.md`
   - Action: Clarify product-wide workflow scope and risk-based compatibility checks.
   - Verify: Requirement count is updated and the risk-based escalation rule is explicit.
   - Done: Planning docs, README, AGENTS, and state updated.

3. Update cross-project briefs.
   - Files: `gsd-briefs/replay-parser-2.md`, `gsd-briefs/server-2.md`, `gsd-briefs/web.md`
   - Action: Add product-wide GSD workflow standards so future project initialization inherits the clarified rules.
   - Verify: Each brief names all three apps and includes risk-based compatibility checks.
   - Done: All three briefs updated.

4. Record quick-task completion.
   - Files: `.planning/quick/260425-fro-clarify-product-wide-gsd-rules-and-risk-/260425-fro-SUMMARY.md`
   - Action: Summarize decisions, changes, and verification.
   - Verify: Summary exists and final git status is clean after commit.
   - Done: Quick-task summary present.
