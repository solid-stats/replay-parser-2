---
status: complete
quick_id: 260502-gn2
verified: 2026-05-02
---

# Verification: Quick 260502-gn2

## Commands

- `bash -n scripts/benchmark-phase5.sh` - passed.
- `env RUN_PHASE5_FULL_CORPUS=0 RUN_PHASE5_FULL_OLD_BASELINE=0 scripts/benchmark-phase5.sh --ci` - completed; benchmark acceptance failed as expected because selected x3/parity and all-raw gates do not pass in smoke mode.
- `env RUN_PHASE5_FULL_CORPUS=1 RUN_PHASE5_FULL_OLD_BASELINE=1 scripts/benchmark-phase5.sh --ci` - completed; full evidence generated and acceptance failed with the expected gate statuses.
- `cargo run -q -p parser-harness --bin benchmark-report-check -- --report .planning/generated/phase-05/benchmarks/benchmark-report.json --mode structural` - passed.
- `cargo test -p parser-harness benchmark_report` - passed, 15 tests.
- `git diff --check` - passed.

## Full Benchmark Result

- Selected x3: `fail`, speedup `2.499596692060698x`.
- Selected parity: `human_review`.
- Selected artifact size: `pass`, artifact bytes `40042`.
- All-raw x10: `unknown`, because the old direct baseline attempted `22996` replay-list tasks while the new all-raw run attempted `23473` raw files.
- All-raw size gate: `fail`, p95 artifact/raw ratio `0.12199904605437491` exceeds `0.10`.
- All-raw zero-failure: `fail`, because the new parser failed 4 raw files.

## Process Fix Verification

The stale legacy `pnpm run parse` path was not parsing the corpus; it was failing old worker startup with `ERR_UNKNOWN_FILE_EXTENSION ".ts"`. The fixed benchmark runner bypasses that WorkerPool path for all-raw old evidence and calls the old parser's `runParseTask` directly through a generated `pnpm exec tsx` runner.
