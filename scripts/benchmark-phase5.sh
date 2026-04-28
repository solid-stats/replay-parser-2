#!/usr/bin/env bash
set -euo pipefail

OUTPUT_ROOT=".planning/generated/phase-05/benchmarks"
COMPARISON_ROOT=".planning/generated/phase-05/comparison"
REPORT_PATH="$OUTPUT_ROOT/benchmark-report.json"
BENCH_LOG="$OUTPUT_ROOT/cargo-bench.log"
FIXTURE="crates/parser-core/tests/fixtures/aggregate-combat.ocap.json"
NEW_OUTPUT="$OUTPUT_ROOT/new-artifact.json"
OLD_REPO="${PHASE5_OLD_REPO:-/home/afgan0r/Projects/SolidGames/replays-parser}"
REAL_HOME="${PHASE5_REAL_HOME:-$HOME}"
CURATED_FILENAME="2026_04_19__21_50_28__1_ocap"
CURATED_DATE="2026-04-19T18:05:45.000Z"
CURATED_MISSION="sm@133_desert_armor_part_2_v3"
CURATED_GAME_TYPE="sm"
CURATED_REPLAY_PATH="$REAL_HOME/sg_stats/raw_replays/$CURATED_FILENAME.json"
OLD_SELECTED_RESPONSE="$COMPARISON_ROOT/old-selected-response.json"
OLD_SELECTED_ARTIFACT="$COMPARISON_ROOT/old-selected-artifact.json"
NEW_SELECTED_ARTIFACT="$COMPARISON_ROOT/new-selected-artifact.json"
COMPARISON_REPORT="$COMPARISON_ROOT/comparison-report.json"
OLD_SELECTED_LOG="$COMPARISON_ROOT/old-selected-command.log"
NEW_SELECTED_LOG="$COMPARISON_ROOT/new-selected-command.log"

mkdir -p "$OUTPUT_ROOT"
mkdir -p "$COMPARISON_ROOT"

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
cargo build --release -q -p parser-cli --bin replay-parser-2
target/release/replay-parser-2 parse "$FIXTURE" --output "$NEW_OUTPUT" \
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

old_baseline_available=false
old_wall_time_ms=""
new_selected_wall_time_ms=""
old_new_speedup=""
parity_status="not_run"
ten_x_status="unknown"
triage="old baseline not run in --ci; parity not run; bottleneck unknown until curated old-baseline benchmark evidence is available; release parser-stage command wall time was ${new_command_wall_ms}ms"

if [[ "${RUN_PHASE5_OLD_BASELINE:-auto}" != "0" \
  && -d "$OLD_REPO" \
  && -f "$CURATED_REPLAY_PATH" \
  && -d "$REAL_HOME/sg_stats/config" \
  && -d "$REAL_HOME/sg_stats/lists" \
  && -d "$REAL_HOME/sg_stats/raw_replays" \
  && -x "target/release/replay-parser-2" \
  && "$(command -v pnpm || true)" != "" ]]; then
  old_baseline_available=true
fi

if [[ "$old_baseline_available" == "true" ]]; then
  OLD_RUN_ROOT="$COMPARISON_ROOT/old-home"
  OLD_RUN_HOME="$OLD_RUN_ROOT/home"
  OLD_RUNNER="$COMPARISON_ROOT/run-old-selected.ts"
  OLD_RUNNER_ABS="$(cd "$(dirname "$OLD_RUNNER")" && pwd)/$(basename "$OLD_RUNNER")"
  OLD_SELECTED_RESPONSE_ABS="$(cd "$(dirname "$OLD_SELECTED_RESPONSE")" && pwd)/$(basename "$OLD_SELECTED_RESPONSE")"
  OLD_SELECTED_ARTIFACT_ABS="$(cd "$(dirname "$OLD_SELECTED_ARTIFACT")" && pwd)/$(basename "$OLD_SELECTED_ARTIFACT")"
  rm -rf "$OLD_RUN_ROOT"
  mkdir -p "$OLD_RUN_HOME/sg_stats"
  OLD_RUN_HOME_ABS="$(cd "$OLD_RUN_HOME" && pwd)"
  ln -s "$REAL_HOME/sg_stats/raw_replays" "$OLD_RUN_HOME/sg_stats/raw_replays"
  ln -s "$REAL_HOME/sg_stats/lists" "$OLD_RUN_HOME/sg_stats/lists"
  cp -a "$REAL_HOME/sg_stats/config" "$OLD_RUN_HOME/sg_stats/config"

  cat > "$OLD_RUNNER" <<'TS'
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

const relationshipRows = (players: any[], field: 'killed' | 'teamkilled') => players.flatMap((player) => (
  (player[field] ?? []).map((target: any) => ({
    actor: player.name,
    target: target.name,
    count: target.count,
    kind: field,
  }))
));

async function main() {
  const [
    oldRepo,
    filename,
    date,
    missionName,
    gameType,
    responsePath,
    artifactPath,
  ] = process.argv.slice(2);
  const workerPath = path.join(oldRepo, 'src/1 - replays/workers/parseReplayWorker.ts');
  const { runParseTask } = await import(pathToFileURL(workerPath).href);

  const start = process.hrtime.bigint();
  const response = await runParseTask({
    taskId: 'phase-05-curated',
    filename,
    date,
    missionName,
    gameType: gameType as any,
  });
  const end = process.hrtime.bigint();
  const wallTimeMs = Number(end - start) / 1_000_000;

  fs.writeFileSync(responsePath, JSON.stringify({ response, wall_time_ms: wallTimeMs }, null, 2));

  const players = response.status === 'success' ? response.data.result : [];
  const artifact = {
    status: response.status === 'success' ? 'success' : response.status,
    replay: {
      filename,
      date,
      mission_name: missionName,
      game_type: gameType,
    },
    events: [],
    aggregates: {
      projections: {
        'legacy.player_game_results': players,
        'legacy.relationships': [
          ...relationshipRows(players, 'killed'),
          ...relationshipRows(players, 'teamkilled'),
        ],
        'bounty.inputs': null,
        'vehicle_score.inputs': null,
      },
    },
  };

  fs.writeFileSync(artifactPath, JSON.stringify(artifact, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
TS
fi

if [[ "$old_baseline_available" == "true" ]]; then
  (
    cd "$OLD_REPO"
    HOME="$OLD_RUN_HOME_ABS" WORKER_COUNT=1 pnpm exec tsx "$OLD_RUNNER_ABS" \
      "$OLD_REPO" "$CURATED_FILENAME" "$CURATED_DATE" "$CURATED_MISSION" "$CURATED_GAME_TYPE" \
      "$OLD_SELECTED_RESPONSE_ABS" \
      "$OLD_SELECTED_ARTIFACT_ABS"
  ) >"$OLD_SELECTED_LOG" 2>&1
fi

if [[ "$old_baseline_available" == "true" && -f "$OLD_SELECTED_ARTIFACT" ]]; then
  selected_start_ns=$(date +%s%N)
  target/release/replay-parser-2 parse "$CURATED_REPLAY_PATH" --output "$NEW_SELECTED_ARTIFACT" \
    >"$NEW_SELECTED_LOG" 2>&1
  selected_end_ns=$(date +%s%N)
  new_selected_wall_time_ms=$(awk -v ns="$((selected_end_ns - selected_start_ns))" 'BEGIN { printf "%.6f", ns / 1000000 }')

  target/release/replay-parser-2 compare \
    --new-artifact "$NEW_SELECTED_ARTIFACT" \
    --old-artifact "$OLD_SELECTED_ARTIFACT" \
    --output "$COMPARISON_REPORT"

  read -r old_wall_time_ms old_new_speedup parity_status ten_x_status triage < <(
    python3 - "$OLD_SELECTED_RESPONSE" "$COMPARISON_REPORT" "$new_selected_wall_time_ms" <<'PY'
import json
import sys

old_response_path, comparison_report_path, new_wall_ms_raw = sys.argv[1:]
old_wall_ms = float(json.load(open(old_response_path, encoding='utf-8'))['wall_time_ms'])
new_wall_ms = float(new_wall_ms_raw)
speedup = old_wall_ms / new_wall_ms if new_wall_ms > 0 else 0.0
report = json.load(open(comparison_report_path, encoding='utf-8'))
categories = report.get('summary', {}).get('by_category', {})

if set(categories) == {'compatible'}:
    parity = 'passed'
elif categories.get('human_review', 0) > 0:
    parity = 'human_review'
else:
    parity = 'failed'

ten_x = 'pass' if speedup >= 10.0 and parity == 'passed' else 'fail'
triage = (
    f"bottleneck: curated old runParseTask vs new release CLI speedup is {speedup:.2f}x "
    f"({old_wall_ms:.3f}ms old parse task vs {new_wall_ms:.3f}ms new command), below the 10x target; "
    f"parity status is {parity}; comparison report: {comparison_report_path}"
)
print(f"{old_wall_ms:.6f} {speedup:.6f} {parity} {ten_x} {triage!r}")
PY
  )
  triage=${triage#\'}
  triage=${triage%\'}
fi

python3 - "$REPORT_PATH" \
  "$parse_only_ms" "$parse_files_per_sec" "$parse_mb_per_sec" \
  "$aggregate_only_ms" "$aggregate_files_per_sec" "$aggregate_mb_per_sec" \
  "$end_to_end_ms" "$end_to_end_files_per_sec" "$end_to_end_mb_per_sec" \
  "$total_bytes" "$FIXTURE" "$new_command_wall_ms" "$old_baseline_available" \
  "$old_wall_time_ms" "$new_selected_wall_time_ms" "$old_new_speedup" \
  "$parity_status" "$ten_x_status" "$triage" "$COMPARISON_REPORT" \
  "$CURATED_REPLAY_PATH" "$CURATED_FILENAME" "$CURATED_MISSION" "$CURATED_GAME_TYPE" <<'PY'
import json
import sys

(
    report_path,
    parse_only_ms,
    parse_files_per_sec,
    parse_mb_per_sec,
    aggregate_only_ms,
    aggregate_files_per_sec,
    aggregate_mb_per_sec,
    end_to_end_ms,
    end_to_end_files_per_sec,
    end_to_end_mb_per_sec,
    total_bytes,
    fixture,
    new_command_wall_ms,
    old_baseline_available,
    old_wall_time_ms,
    new_selected_wall_time_ms,
    old_new_speedup,
    parity_status,
    ten_x_status,
    triage,
    comparison_report,
    curated_replay_path,
    curated_filename,
    curated_mission,
    curated_game_type,
) = sys.argv[1:]

def nullable_float(raw):
    if raw in {'', 'null'}:
        return None
    return float(raw)

def metric(wall_ms, files_per_sec, mb_per_sec):
    return {
        'wall_time_ms': nullable_float(wall_ms),
        'files_per_sec': nullable_float(files_per_sec),
        'mb_per_sec': nullable_float(mb_per_sec),
        'events_per_sec': None,
        'rss_mb': None,
    }

report = {
    'report_version': '1',
    'old_baseline_profile': (
        'deterministic WORKER_COUNT=1-equivalent selected run via old parser runParseTask'
        if old_baseline_available == 'true'
        else 'not-run in --ci; deterministic baseline is WORKER_COUNT=1'
    ),
    'old_command': (
        'WORKER_COUNT=1 HOME=<generated-fake-home> pnpm exec tsx .planning/generated/phase-05/comparison/run-old-selected.ts'
        if old_baseline_available == 'true'
        else 'HOME=<generated-fake-home> WORKER_COUNT=1 pnpm run parse'
    ),
    'new_command': f'target/release/replay-parser-2 parse {fixture} --output .planning/generated/phase-05/benchmarks/new-artifact.json',
    'workload': {
        'tier': 'small_ci',
        'fixtures': [fixture],
        'corpus_selector': None,
        'total_bytes': int(total_bytes),
    },
    'parse_only': metric(parse_only_ms, parse_files_per_sec, parse_mb_per_sec),
    'aggregate_only': metric(aggregate_only_ms, aggregate_files_per_sec, aggregate_mb_per_sec),
    'end_to_end': metric(end_to_end_ms, end_to_end_files_per_sec, end_to_end_mb_per_sec),
    'parity_status': parity_status,
    'ten_x_status': ten_x_status,
    'triage': triage,
    'rss_note': 'RSS is not captured in the portable CI fallback; use an external process monitor for curated or full-corpus benchmark runs.',
    'new_command_wall_time_ms': nullable_float(new_command_wall_ms),
}

if old_baseline_available == 'true':
    report['old_new_command'] = {
        'old_wall_time_ms': nullable_float(old_wall_time_ms),
        'new_wall_time_ms': nullable_float(new_selected_wall_time_ms),
        'speedup': nullable_float(old_new_speedup),
        'comparison_report': comparison_report,
        'old_command': 'WORKER_COUNT=1 HOME=<generated-fake-home> pnpm exec tsx .planning/generated/phase-05/comparison/run-old-selected.ts',
        'new_command': f'target/release/replay-parser-2 parse {curated_replay_path} --output .planning/generated/phase-05/comparison/new-selected-artifact.json',
        'workload': {
            'filename': curated_filename,
            'replay_path': curated_replay_path,
            'mission_name': curated_mission,
            'game_type': curated_game_type,
        },
    }

with open(report_path, 'w', encoding='utf-8') as handle:
    json.dump(report, handle, indent=2)
    handle.write('\n')
PY

cargo run -p parser-harness --bin benchmark-report-check -- --report "$REPORT_PATH"
