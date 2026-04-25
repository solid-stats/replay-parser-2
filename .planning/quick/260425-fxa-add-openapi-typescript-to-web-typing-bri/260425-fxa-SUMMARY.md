---
quick_id: 260425-fxa
status: complete
completed: 2026-04-25
---

# Quick Summary: Add openapi-typescript to Project Briefs

## Completed

- Updated `web` brief to require `openapi-typescript` generated API types from the `server-2` OpenAPI schema.
- Updated `server-2` brief to own and publish an OpenAPI 3.x schema compatible with `openapi-typescript`.
- Updated `replays-parser-2` brief to call out parser-contract changes that affect downstream API schema and generated frontend types.
- Synchronized the three updated briefs across `sg-replay-parser-2`, `server-2`, and `web`.
- Updated `.planning/STATE.md` with the quick task record.

## Verification

- Documentation-only change; no code tests were required.
- Verified brief content and markdown whitespace before commit.
