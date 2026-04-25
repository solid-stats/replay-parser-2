---
status: complete
mode: quick
created: 2026-04-25
---

# Quick Task 260425-fln: AI Pushback and Safer Alternatives

## Goal

Add requirements that AI agents must not blindly execute instructions that conflict with current logic, architecture, quality standards, maintainability, or proportional scope. Agents should explain why a request is risky, propose better options, and ask for explicit confirmation before any risky override.

## User Decisions

- Trigger strictness: very strict. Push back on architecture, scope, quality, maintainability, and supportability concerns.
- Response style: explain the issue and propose alternatives.
- If user insists: ask for explicit confirmation before proceeding.

## Tasks

1. Add workflow requirements.
   - Files: `.planning/REQUIREMENTS.md`
   - Action: Add `WF-*` requirements for AI pushback, explanation, alternatives, and explicit confirmation.
   - Verify: New requirements are mapped and totals are consistent.
   - Done: `WF-03` through `WF-05` added and mapped to Phase 1.

2. Align roadmap, project, README, and agent rules.
   - Files: `.planning/ROADMAP.md`, `.planning/PROJECT.md`, `README.md`, `AGENTS.md`
   - Action: Add strict pushback policy to visible project documentation.
   - Verify: Requirement count is updated and the explanation/alternatives/confirmation flow is explicit.
   - Done: Project context and agent instructions reflect the pushback policy.

3. Record quick-task completion.
   - Files: `.planning/STATE.md`, `.planning/quick/260425-fln-ai-pushback-policy/260425-fln-SUMMARY.md`
   - Action: Track the quick task and summarize the documentation changes.
   - Verify: State links to the quick-task directory.
   - Done: Quick-task artifacts are present.
