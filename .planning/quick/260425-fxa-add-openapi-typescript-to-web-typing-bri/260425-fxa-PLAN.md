---
quick_id: 260425-fxa
status: complete
mode: quick
created: 2026-04-25
---

# Quick Plan: Add openapi-typescript to Project Briefs

## Goal

Make the frontend typing contract explicit across the Solid Stats project briefs: `web` uses `openapi-typescript` against the `server-2` OpenAPI schema, and adjacent apps account for that contract.

## Scope

- Update `gsd-briefs/web.md` with explicit `openapi-typescript` requirements.
- Update `gsd-briefs/server-2.md` with OpenAPI schema ownership and compatibility requirements.
- Update `gsd-briefs/replays-parser-2.md` with downstream API typing compatibility notes.
- Sync the updated briefs to the sibling `server-2` and `web` repositories.
- Record completion in `.planning/STATE.md`.

## Source Notes

- `openapi-typescript` generates TypeScript types from OpenAPI 3.0 and 3.1 schemas.
- It supports local or remote JSON/YAML schemas.
- The docs recommend `noUncheckedIndexedAccess` for stronger type safety.

## Verification

- Check all three brief copies contain the `openapi-typescript` guidance.
- Run markdown whitespace validation with `git diff --check`.
- Leave all three repositories with clean git status after commits.
