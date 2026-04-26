# Quick Task 260426-rfs: replays-fetcher boundary sync

**Date:** 2026-04-26  
**Type:** docs-only quick task  
**Scope:** `replay-parser-2` planning/docs and cross-app briefs

## Goal

Update `replay-parser-2` docs after adding the `replays-fetcher` repository so parser ownership, integration flow, and worker result delivery match the accepted four-app Solid Stats architecture.

## Tasks

- [x] Update README, AGENTS, PROJECT, REQUIREMENTS, ROADMAP, STATE, and research summary to include `replays-fetcher`.
- [x] Record that production replay discovery/fetching and S3 `raw/` staging belong to `replays-fetcher`.
- [x] Record that `server-2` promotes staging rows and creates `parse_jobs`.
- [x] Record that successful parser-worker results are S3 artifacts reported by artifact reference, not full RabbitMQ payloads.
- [x] Update `gsd-briefs` for parser/backend/web/fetcher compatibility.

## Verification

- `rg -n "replays-fetcher|artifact reference|S3 artifact|raw/" README.md AGENTS.md .planning/PROJECT.md .planning/REQUIREMENTS.md .planning/ROADMAP.md .planning/STATE.md .planning/research/SUMMARY.md gsd-briefs`
- `git diff --check`
