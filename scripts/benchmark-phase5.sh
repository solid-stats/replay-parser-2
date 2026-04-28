#!/usr/bin/env bash
set -euo pipefail

OUTPUT_ROOT=".planning/generated/phase-05/benchmarks"
REPORT_PATH="$OUTPUT_ROOT/benchmark-report.json"
BENCH_LOG="$OUTPUT_ROOT/cargo-bench.log"
FIXTURE="crates/parser-core/tests/fixtures/aggregate-combat.ocap.json"
NEW_OUTPUT="$OUTPUT_ROOT/new-artifact.json"

mkdir -p "$OUTPUT_ROOT"

if [[ "${1:-}" != "--ci" ]]; then
  printf '%s\n' "usage: scripts/benchmark-phase5.sh --ci"
  exit 2
fi

cargo bench -p parser-harness --bench parser_pipeline -- --sample-size 10 2>&1 | tee "$BENCH_LOG"

criterion_wall_ms() {
  local bench_name="$1"
  local estimates_path="crates/parser-harness/target/criterion/$bench_name/new/estimates.json"
  if command -v jq >/dev/null 2>&1 && [[ -f "$estimates_path" ]]; then
    jq -r '(.mean.point_estimate / 1000000)' "$estimates_path"
  else
    printf '0.0'
  fi
}

files_per_sec() {
  local wall_ms="$1"
  awk -v ms="$wall_ms" 'BEGIN { if (ms > 0) { printf "%.6f", 1000 / ms } else { printf "null" } }'
}

mb_per_sec() {
  local wall_ms="$1"
  local bytes="$2"
  awk -v ms="$wall_ms" -v bytes="$bytes" 'BEGIN { if (ms > 0) { printf "%.6f", (bytes / 1048576) / (ms / 1000) } else { printf "null" } }'
}

new_start_ns=$(date +%s%N)
cargo run --release -p parser-cli --bin replay-parser-2 -- parse "$FIXTURE" --output "$NEW_OUTPUT" \
  > "$OUTPUT_ROOT/new-command.log" 2>&1
new_end_ns=$(date +%s%N)
new_command_wall_ms=$(( (new_end_ns - new_start_ns) / 1000000 ))
total_bytes=$(wc -c < "$FIXTURE")
parse_only_ms=$(criterion_wall_ms "parse_only_json_decode")
aggregate_only_ms=$(criterion_wall_ms "aggregate_only_public_projection_access")
end_to_end_ms=$(criterion_wall_ms "end_to_end_parse_replay")
parse_files_per_sec=$(files_per_sec "$parse_only_ms")
aggregate_files_per_sec=$(files_per_sec "$aggregate_only_ms")
end_to_end_files_per_sec=$(files_per_sec "$end_to_end_ms")
parse_mb_per_sec=$(mb_per_sec "$parse_only_ms" "$total_bytes")
aggregate_mb_per_sec=$(mb_per_sec "$aggregate_only_ms" "$total_bytes")
end_to_end_mb_per_sec=$(mb_per_sec "$end_to_end_ms" "$total_bytes")

if [[ "${RUN_PHASE5_OLD_BASELINE:-}" == "1" ]]; then
  printf '%s\n' "RUN_PHASE5_OLD_BASELINE=1 was requested, but full old-parser benchmark execution is intentionally manual until a curated old/new workload script is approved." \
    > "$OUTPUT_ROOT/old-command.log"
fi

cat > "$REPORT_PATH" <<JSON
{
  "report_version": "1",
  "old_baseline_profile": "not-run in --ci; deterministic baseline is WORKER_COUNT=1",
  "old_command": "RUN_PHASE5_OLD_BASELINE=1 HOME=<generated-fake-home> WORKER_COUNT=1 pnpm run parse",
  "new_command": "cargo run --release -p parser-cli --bin replay-parser-2 -- parse $FIXTURE --output $NEW_OUTPUT",
  "workload": {
    "tier": "small_ci",
    "fixtures": ["$FIXTURE"],
    "corpus_selector": null,
    "total_bytes": $total_bytes
  },
  "parse_only": {
    "wall_time_ms": $parse_only_ms,
    "files_per_sec": $parse_files_per_sec,
    "mb_per_sec": $parse_mb_per_sec,
    "events_per_sec": null,
    "rss_mb": null
  },
  "aggregate_only": {
    "wall_time_ms": $aggregate_only_ms,
    "files_per_sec": $aggregate_files_per_sec,
    "mb_per_sec": $aggregate_mb_per_sec,
    "events_per_sec": null,
    "rss_mb": null
  },
  "end_to_end": {
    "wall_time_ms": $end_to_end_ms,
    "files_per_sec": $end_to_end_files_per_sec,
    "mb_per_sec": $end_to_end_mb_per_sec,
    "events_per_sec": null,
    "rss_mb": null
  },
  "parity_status": "not_run",
  "ten_x_status": "unknown",
  "triage": "old baseline not run in --ci; parity not run; bottleneck unknown until RUN_PHASE5_OLD_BASELINE=1 benchmark evidence is approved and collected; release parse command wall time was ${new_command_wall_ms}ms",
  "rss_note": "RSS is not captured in the portable CI fallback; use an external process monitor for curated or full-corpus benchmark runs."
}
JSON

cargo run -p parser-harness --bin benchmark-report-check -- --report "$REPORT_PATH"
