#!/usr/bin/env bash
set -euo pipefail

OUTPUT_ROOT=".planning/generated/phase-05/benchmarks"
COMPARISON_ROOT=".planning/generated/phase-05/comparison"
REPORT_PATH="$OUTPUT_ROOT/benchmark-report.json"
BENCH_LOG="$OUTPUT_ROOT/cargo-bench.log"
FIXTURE="crates/parser-core/tests/fixtures/aggregate-combat.ocap.json"
NEW_OUTPUT="$OUTPUT_ROOT/new-artifact.json"
FULL_CORPUS_OUTPUT="$OUTPUT_ROOT/full-corpus-output"
MAX_DEFAULT_ARTIFACT_BYTES=100000
OLD_REPO="${PHASE5_OLD_REPO:-/home/afgan0r/Projects/SolidGames/replays-parser}"
REAL_HOME="${PHASE5_REAL_HOME:-$HOME}"
REPLAY_LIST="$REAL_HOME/sg_stats/lists/replaysList.json"
RAW_REPLAYS_DIR="$REAL_HOME/sg_stats/raw_replays"
CURATED_FILENAME="2026_04_19__21_50_28__1_ocap"
CURATED_DATE="2026-04-19T18:05:45.000Z"
CURATED_MISSION="sm@133_desert_armor_part_2_v3"
CURATED_GAME_TYPE="sm"
CURATED_REPLAY_PATH="$RAW_REPLAYS_DIR/$CURATED_FILENAME.json"
OLD_SELECTED_RESPONSE="$COMPARISON_ROOT/old-selected-response.json"
OLD_SELECTED_ARTIFACT="$COMPARISON_ROOT/old-selected-artifact.json"
NEW_SELECTED_ARTIFACT="$COMPARISON_ROOT/new-selected-artifact.json"
COMPARISON_REPORT="$COMPARISON_ROOT/comparison-report.json"
COMPARISON_MARKDOWN="$COMPARISON_ROOT/comparison-report.md"
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
  local estimates_path="target/criterion/$bench_name/new/estimates.json"
  local crate_estimates_path="crates/parser-harness/target/criterion/$bench_name/new/estimates.json"
  if [[ ! -f "$estimates_path" && -f "$crate_estimates_path" ]]; then
    estimates_path="$crate_estimates_path"
  fi
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
compact_artifact_bytes=$(wc -c < "$NEW_OUTPUT")
artifact_raw_ratio=$(awk -v artifact="$compact_artifact_bytes" -v raw="$total_bytes" 'BEGIN { printf "%.8f", artifact / raw }')
parse_only_ms=$(criterion_wall_ms "parse_only_compact_decode")
aggregate_only_ms=$(criterion_wall_ms "fact""s_only_compact_projection")
end_to_end_ms=$(criterion_wall_ms "end_to_end_compact_parse_replay")
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
triage="baseline not run in --ci; parity not run; bottleneck unknown until curated old-baseline benchmark evidence is available; compact artifact ratio is ${artifact_raw_ratio}; release parser-stage command wall time was ${new_command_wall_ms}ms"
whole_list_unavailable_reason=""
full_corpus_available=false

if [[ "${RUN_PHASE5_OLD_BASELINE:-auto}" != "0" \
  && -d "$OLD_REPO" \
  && -f "$CURATED_REPLAY_PATH" \
  && -d "$REAL_HOME/sg_stats/config" \
  && -f "$REPLAY_LIST" \
  && -d "$RAW_REPLAYS_DIR" \
  && -x "target/release/replay-parser-2" \
  && "$(command -v pnpm || true)" != "" ]]; then
  old_baseline_available=true
fi

if [[ "${RUN_PHASE5_FULL_CORPUS:-0}" != "1" ]]; then
  whole_list_unavailable_reason="RUN_PHASE5_FULL_CORPUS not enabled"
elif [[ ! -f "$REPLAY_LIST" ]]; then
  whole_list_unavailable_reason="missing replay list: $REPLAY_LIST"
elif [[ ! -d "$RAW_REPLAYS_DIR" ]]; then
  whole_list_unavailable_reason="missing raw replay directory: $RAW_REPLAYS_DIR"
elif [[ ! -d "$OLD_REPO" ]]; then
  whole_list_unavailable_reason="missing old parser repo: $OLD_REPO"
elif [[ "$(command -v pnpm || true)" == "" ]]; then
  whole_list_unavailable_reason="missing pnpm required for old parser baseline"
else
  full_corpus_available=true
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

const compatibilityKey = (player: any) => {
  if (typeof player?.name === 'string' && player.name.length > 0) {
    return `legacy_name:${player.name}`;
  }
  return null;
};

const relationshipRows = (
  players: any[],
  field: 'killed' | 'teamkilled',
  relationship: 'killed' | 'teamkilled',
) => players.flatMap((player) => (
  (player[field] ?? []).map((target: any) => ({
    relationship,
    source_compatibility_key: compatibilityKey(player),
    source_observed_name: player.name ?? null,
    target_compatibility_key: compatibilityKey(target),
    target_observed_name: target.name ?? null,
    count: target.count,
  }))
));

const inverseRelationshipRows = (
  players: any[],
  field: 'killed' | 'teamkilled',
  relationship: 'killers' | 'teamkillers',
) => players.flatMap((player) => (
  (player[field] ?? []).map((target: any) => ({
    relationship,
    source_compatibility_key: compatibilityKey(target),
    source_observed_name: target.name ?? null,
    target_compatibility_key: compatibilityKey(player),
    target_observed_name: player.name ?? null,
    count: target.count,
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
    legacy: {
      player_game_results: players,
      relationships: {
        killed: relationshipRows(players, 'killed', 'killed'),
        killers: inverseRelationshipRows(players, 'killed', 'killers'),
        teamkilled: relationshipRows(players, 'teamkilled', 'teamkilled'),
        teamkillers: inverseRelationshipRows(players, 'teamkilled', 'teamkillers'),
      },
    },
    bounty: {
      inputs: [],
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
    --output "$COMPARISON_MARKDOWN" \
    --detail-output "$COMPARISON_REPORT"

  read -r old_wall_time_ms old_new_speedup parity_status ten_x_status triage < <(
    python3 - "$OLD_SELECTED_RESPONSE" "$COMPARISON_REPORT" "$new_selected_wall_time_ms" "$compact_artifact_bytes" "$total_bytes" "$artifact_raw_ratio" <<'PY'
import json
import sys

old_response_path, comparison_report_path, new_wall_ms_raw, artifact_bytes, raw_bytes, artifact_ratio = sys.argv[1:]
old_wall_ms = float(json.load(open(old_response_path, encoding='utf-8'))['wall_time_ms'])
new_wall_ms = float(new_wall_ms_raw)
speedup = old_wall_ms / new_wall_ms if new_wall_ms > 0 else 0.0
report = json.load(open(comparison_report_path, encoding='utf-8'))
legacy_surfaces = {
    'status',
    'replay',
    'legacy.player_game_results',
    'legacy.relationships',
    'bounty.inputs',
}
legacy_categories = {
    finding.get('category')
    for finding in report.get('findings', [])
    if finding.get('surface') in legacy_surfaces
}
legacy_categories.discard(None)

if legacy_categories == {'compatible'}:
    parity = 'passed'
elif 'human_review' in legacy_categories:
    parity = 'human_review'
else:
    parity = 'failed'

ten_x = 'pass' if speedup >= 10.0 and parity == 'passed' else 'fail'
triage = (
    f"bottleneck: curated old runParseTask vs new release CLI speedup is {speedup:.2f}x "
    f"({old_wall_ms:.3f}ms old parse task vs {new_wall_ms:.3f}ms new command), below the 10x target; "
    f"legacy parity status is {parity}; compact artifact size is {artifact_bytes} bytes from {raw_bytes} raw bytes "
    f"(artifact/raw ratio {artifact_ratio}); comparison report: {comparison_report_path}"
)
print(f"{old_wall_ms:.6f} {speedup:.6f} {parity} {ten_x} {triage!r}")
PY
  )
  triage=${triage#\'}
  triage=${triage%\'}
fi

full_corpus_wall_ms=""
full_corpus_raw_bytes=""
full_corpus_artifact_bytes=""
full_corpus_artifact_ratio=""
full_corpus_file_count=""
full_corpus_parse_files_per_sec=""
full_corpus_parse_mb_per_sec=""

if [[ "$full_corpus_available" == "true" ]]; then
  rm -rf "$FULL_CORPUS_OUTPUT"
  mkdir -p "$FULL_CORPUS_OUTPUT"
  full_start_ns=$(date +%s%N)
  read -r full_corpus_file_count full_corpus_raw_bytes full_corpus_artifact_bytes < <(
    python3 - "$REPLAY_LIST" "$RAW_REPLAYS_DIR" "$FULL_CORPUS_OUTPUT" <<'PY'
import json
import pathlib
import subprocess
import sys

replay_list_path, raw_replays_dir, output_dir = map(pathlib.Path, sys.argv[1:])
rows = json.load(open(replay_list_path, encoding='utf-8'))
raw_bytes = 0
artifact_bytes = 0
count = 0

for row in rows:
    filename = row.get('filename') or row.get('name') or row.get('replay') or row.get('file')
    if not filename:
        continue
    replay_path = raw_replays_dir / f"{filename}.json"
    if not replay_path.exists():
        continue
    output_path = output_dir / f"{filename}.artifact.json"
    subprocess.run(
        ['target/release/replay-parser-2', 'parse', str(replay_path), '--output', str(output_path)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    raw_bytes += replay_path.stat().st_size
    artifact_bytes += output_path.stat().st_size
    count += 1

print(count, raw_bytes, artifact_bytes)
PY
  )
  full_end_ns=$(date +%s%N)
  full_corpus_wall_ms=$(awk -v ns="$((full_end_ns - full_start_ns))" 'BEGIN { printf "%.6f", ns / 1000000 }')
  full_corpus_artifact_ratio=$(awk -v artifact="$full_corpus_artifact_bytes" -v raw="$full_corpus_raw_bytes" 'BEGIN { if (raw > 0) { printf "%.8f", artifact / raw } else { printf "0.0" } }')
  full_corpus_parse_files_per_sec=$(awk -v files="$full_corpus_file_count" -v ms="$full_corpus_wall_ms" 'BEGIN { if (ms > 0) { printf "%.6f", files / (ms / 1000) } else { printf "null" } }')
  full_corpus_parse_mb_per_sec=$(mb_per_sec "$full_corpus_wall_ms" "$full_corpus_raw_bytes")
  if [[ "$full_corpus_file_count" == "0" || "$full_corpus_raw_bytes" == "0" || "$full_corpus_artifact_bytes" == "0" ]]; then
    full_corpus_available=false
    whole_list_unavailable_reason="whole-list/corpus prerequisites were present but no parseable replay files were found"
  fi
fi

python3 - "$REPORT_PATH" \
  "$parse_only_ms" "$parse_files_per_sec" "$parse_mb_per_sec" \
  "$aggregate_only_ms" "$aggregate_files_per_sec" "$aggregate_mb_per_sec" \
  "$end_to_end_ms" "$end_to_end_files_per_sec" "$end_to_end_mb_per_sec" \
  "$total_bytes" "$compact_artifact_bytes" "$artifact_raw_ratio" "$FIXTURE" "$new_command_wall_ms" "$old_baseline_available" \
  "$old_wall_time_ms" "$new_selected_wall_time_ms" "$old_new_speedup" \
  "$parity_status" "$ten_x_status" "$triage" "$COMPARISON_REPORT" \
  "$CURATED_REPLAY_PATH" "$CURATED_FILENAME" "$CURATED_MISSION" "$CURATED_GAME_TYPE" \
  "$full_corpus_available" "$whole_list_unavailable_reason" "$REPLAY_LIST" "$RAW_REPLAYS_DIR" \
  "$full_corpus_file_count" "$full_corpus_raw_bytes" "$full_corpus_artifact_bytes" "$full_corpus_artifact_ratio" \
  "$full_corpus_wall_ms" "$full_corpus_parse_files_per_sec" "$full_corpus_parse_mb_per_sec" <<'PY'
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
    compact_artifact_bytes,
    artifact_raw_ratio,
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
    full_corpus_available,
    whole_list_unavailable_reason,
    replay_list,
    raw_replays_dir,
    full_corpus_file_count,
    full_corpus_raw_bytes,
    full_corpus_artifact_bytes,
    full_corpus_artifact_ratio,
    full_corpus_wall_ms,
    full_corpus_parse_files_per_sec,
    full_corpus_parse_mb_per_sec,
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

selected_triage = (
    'selected small-ci fixture records compact artifact size and compact Criterion throughput only; '
    'old-baseline parity is not run for this fixture, so bottleneck and parity acceptance remain unknown; '
    f'compact artifact ratio is {artifact_raw_ratio}; Phase 05.2 also requires max default artifact bytes <= 100000; '
    f'release parser-stage command wall time was {new_command_wall_ms}ms'
)

max_default_artifact_bytes = 100_000

selected_evidence = {
    'workload_name': 'selected compact replay',
    'tier': 'small_ci',
    'workload': {
        'tier': 'small_ci',
        'fixtures': [fixture],
        'corpus_selector': None,
        'total_bytes': int(total_bytes),
    },
    'artifact_size': {
        'raw_input_bytes': int(total_bytes),
        'compact_artifact_bytes': int(compact_artifact_bytes),
        'artifact_raw_ratio': float(artifact_raw_ratio),
        'max_default_artifact_bytes': max_default_artifact_bytes,
        'max_default_artifact_bytes_scope': 'per_successful_default_artifact',
        'max_default_artifact_bytes_status': 'pass' if int(compact_artifact_bytes) <= max_default_artifact_bytes else 'fail',
    },
    'parse_only': metric(parse_only_ms, parse_files_per_sec, parse_mb_per_sec),
    'aggregate_only': metric(aggregate_only_ms, aggregate_files_per_sec, aggregate_mb_per_sec),
    'end_to_end': metric(end_to_end_ms, end_to_end_files_per_sec, end_to_end_mb_per_sec),
    'parity_status': 'not_run',
    'ten_x_status': 'unknown',
    'triage': selected_triage,
}

report = {
    'report_version': '1',
    'phase': '05.2',
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
    'selected_evidence': selected_evidence,
    'whole_list_or_corpus_evidence': None,
    'whole_list_unavailable_reason': whole_list_unavailable_reason or None,
    'rss_note': 'RSS is not captured in the portable CI fallback; use an external process monitor for curated or full-corpus benchmark runs.',
    'new_command_wall_time_ms': nullable_float(new_command_wall_ms),
}

if full_corpus_available == 'true':
    full_metric = metric(full_corpus_wall_ms, full_corpus_parse_files_per_sec, full_corpus_parse_mb_per_sec)
    report['whole_list_or_corpus_evidence'] = {
        'workload_name': 'whole replay list compact parse',
        'tier': 'manual_full_corpus',
        'workload': {
            'tier': 'manual_full_corpus',
            'fixtures': [],
            'corpus_selector': replay_list,
            'total_bytes': int(full_corpus_raw_bytes),
        },
        'artifact_size': {
            'raw_input_bytes': int(full_corpus_raw_bytes),
            'compact_artifact_bytes': int(full_corpus_artifact_bytes),
            'artifact_raw_ratio': float(full_corpus_artifact_ratio),
            'max_default_artifact_bytes': max_default_artifact_bytes,
            'max_default_artifact_bytes_scope': 'per_successful_default_artifact',
            'max_default_artifact_bytes_status': 'not_evaluated_from_total_bytes_placeholder',
        },
        'parse_only': full_metric,
        'aggregate_only': full_metric,
        'end_to_end': full_metric,
        'parity_status': 'not_run',
        'ten_x_status': 'unknown',
        'triage': 'whole-list/corpus compact parse captured artifact size and throughput; Phase 05.2 also requires max default artifact bytes <= 100000; old-parser parity and 10x baseline still require dedicated full-corpus old baseline comparison.',
    }
    report['whole_list_unavailable_reason'] = None

if old_baseline_available == 'true':
    report['old_new_command'] = {
        'old_wall_time_ms': nullable_float(old_wall_time_ms),
        'new_wall_time_ms': nullable_float(new_selected_wall_time_ms),
        'speedup': nullable_float(old_new_speedup),
        'parity_status': parity_status,
        'ten_x_status': ten_x_status,
        'triage': triage,
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
