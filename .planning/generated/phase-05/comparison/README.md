# Phase 5 Comparison Evidence

This directory is the generated evidence location for curated old-vs-new parser comparison runs.

`scripts/benchmark-phase5.sh --ci` writes:

- `old-selected-response.json` - old TypeScript parser `runParseTask` response for the curated replay.
- `old-selected-artifact.json` - selected old-side artifact wrapper consumed by `replay-parser-2 compare`.
- `new-selected-artifact.json` - new Rust parser artifact for the same curated replay.
- `comparison-report.json` - structured old-vs-new selected-surface comparison report.
- `old-selected-command.log` and `new-selected-command.log` - command logs for the curated run.

The selected old parser response is regenerated on every benchmark run so the reported old/new timing is not a stale cache.

Generated JSON/log files remain ignored. The committed README documents the convention only.
