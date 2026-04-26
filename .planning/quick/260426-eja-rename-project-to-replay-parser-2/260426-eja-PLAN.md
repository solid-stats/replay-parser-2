---
quick_id: 260426-eja
status: complete
mode: quick
created: 2026-04-26
---

# Quick Plan: Rename Project to replay-parser-2

## Goal

Rename the project identity from `sg-replay-parser-2` / `replays-parser-2` to `replay-parser-2` in current repository documentation and planning context.

## Scope

- Confirm the workspace directory is already `/home/afgan0r/Projects/SolidGames/replay-parser-2`.
- Update active project docs, planning docs, and cross-project briefs to use `replay-parser-2`.
- Rename the parser brief from `gsd-briefs/replays-parser-2.md` to `gsd-briefs/replay-parser-2.md` and update references to it.
- Keep the legacy parser path `/home/afgan0r/Projects/SolidGames/replays-parser` and planned CLI command `sg-replay-parser` unchanged because they identify separate artifacts.
- Update `.planning/STATE.md` with this quick task.

## Verification

- Ensure tracked docs no longer use the old project identifiers except historical quick-task descriptions/paths and this rename task's own description.
- Run markdown whitespace validation with `git diff --check`.
- Commit intended results and leave the git tree clean.
