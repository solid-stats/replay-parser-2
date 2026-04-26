---
quick_id: 260426-eja
status: complete
completed: 2026-04-26
---

# Quick Summary: Rename Project to replay-parser-2

## Completed

- Confirmed the repository directory is already `/home/afgan0r/Projects/SolidGames/replay-parser-2`.
- Renamed current project references from `sg-replay-parser-2` and `replays-parser-2` to `replay-parser-2` across README, AGENTS, planning docs, research docs, phase docs, and cross-project briefs.
- Renamed `gsd-briefs/replays-parser-2.md` to `gsd-briefs/replay-parser-2.md` and updated references to that brief.
- Renamed older quick-task directories whose slugs embedded the old project name, then updated `STATE.md` links.
- Left the legacy parser path `/home/afgan0r/Projects/SolidGames/replays-parser` and planned CLI command `sg-replay-parser` unchanged because they identify separate artifacts.
- Updated `.planning/STATE.md` with the quick-task record.

## Verification

- Re-ran `gsd-sdk query init.quick` and confirmed `project_title` now resolves to `replay-parser-2`.
- Documentation-only change; no code tests were required.
- Markdown whitespace validation passed before commit.
