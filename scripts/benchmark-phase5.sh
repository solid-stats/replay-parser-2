#!/usr/bin/env bash
set -euo pipefail

PHASE5_GENERATED_ROOT=".planning/generated/phase-05"
OUTPUT_ROOT="$PHASE5_GENERATED_ROOT/benchmarks"
COMPARISON_ROOT="$PHASE5_GENERATED_ROOT/comparison"
REPORT_PATH="$OUTPUT_ROOT/benchmark-report.json"
BENCH_LOG="$OUTPUT_ROOT/cargo-bench.log"
FIXTURE="crates/parser-core/tests/fixtures/aggregate-combat.ocap.json"
SMOKE_OUTPUT="$OUTPUT_ROOT/smoke-artifact.json"
SELECTED_INFO="$OUTPUT_ROOT/selected-large-replay.json"
SELECTED_METADATA="$OUTPUT_ROOT/selected-large-metadata.json"
SELECTED_ARTIFACT="$OUTPUT_ROOT/selected-large-artifact.json"
SELECTED_PARSE_LOG="$OUTPUT_ROOT/selected-large-command.log"
ALL_RAW_SUMMARY="$OUTPUT_ROOT/all-raw-summary.json"
ALL_RAW_FAILURES="$OUTPUT_ROOT/all-raw-failures.json"
ALL_RAW_OVERSIZED="$OUTPUT_ROOT/all-raw-oversized-artifacts.json"
ALL_RAW_OUTPUT="$OUTPUT_ROOT/all-raw-artifacts"
OLD_SELECTED_RESPONSE="$COMPARISON_ROOT/old-selected-response.json"
OLD_SELECTED_ARTIFACT="$COMPARISON_ROOT/old-selected-artifact.json"
COMPARISON_REPORT="$COMPARISON_ROOT/comparison-report.json"
COMPARISON_MARKDOWN="$COMPARISON_ROOT/comparison-report.md"
OLD_SELECTED_LOG="$COMPARISON_ROOT/old-selected-command.log"
OLD_FULL_LOG="$COMPARISON_ROOT/old-all-raw-command.log"
OLD_FULL_SUMMARY="$COMPARISON_ROOT/old-all-raw-summary.json"
MAX_DEFAULT_ARTIFACT_BYTES=100000
SELECTION_POLICY="largest .json by byte size under ~/sg_stats/raw_replays; tie-break lexicographic path"
ALL_RAW_SELECTOR="~/sg_stats/raw_replays/**/*.json sorted lexicographically"
OLD_REPO="${PHASE5_OLD_REPO:-/home/afgan0r/Projects/SolidGames/replays-parser}"
REAL_HOME="${PHASE5_REAL_HOME:-$HOME}"
REPLAY_LIST="$REAL_HOME/sg_stats/lists/replaysList.json"
RAW_REPLAYS_DIR="$REAL_HOME/sg_stats/raw_replays"

if [[ "${1:-}" != "--ci" ]]; then
  printf '%s\n' "usage: scripts/benchmark-phase5.sh --ci"
  exit 2
fi

rm -rf \
  "$PHASE5_GENERATED_ROOT/benchmarks" \
  "$PHASE5_GENERATED_ROOT/comparison" \
  "$PHASE5_GENERATED_ROOT/coverage" \
  "$PHASE5_GENERATED_ROOT/fault-report"
mkdir -p "$OUTPUT_ROOT" "$COMPARISON_ROOT" "$ALL_RAW_OUTPUT"

printf '%s\n' \
  "Criterion parser_pipeline benchmarks are validated by the dedicated cargo bench gate." \
  > "$BENCH_LOG"

cargo build --release -q -p parser-cli --bin replay-parser-2
target/release/replay-parser-2 parse "$FIXTURE" --output "$SMOKE_OUTPUT" \
  > "$OUTPUT_ROOT/smoke-command.log" 2>&1

python3 - "$RAW_REPLAYS_DIR" "$FIXTURE" "$SELECTED_INFO" "$SELECTION_POLICY" <<'PY'
import hashlib
import json
import pathlib
import sys

raw_replays_dir, fixture, output_path, selection_policy = sys.argv[1:]
raw_dir = pathlib.Path(raw_replays_dir).expanduser()
fixture_path = pathlib.Path(fixture)
selected_path = fixture_path
selected_source = "smoke_fixture"

if raw_dir.is_dir():
    candidates = []
    for path in raw_dir.rglob("*.json"):
        try:
            candidates.append((-path.stat().st_size, str(path), path))
        except OSError:
            continue
    if candidates:
        candidates.sort()
        selected_path = candidates[0][2]
        selected_source = "raw_replays"

raw_bytes = selected_path.stat().st_size
sha256 = hashlib.sha256(selected_path.read_bytes()).hexdigest()

with open(output_path, "w", encoding="utf-8") as handle:
    json.dump(
        {
            "selection_policy": selection_policy,
            "path": str(selected_path),
            "raw_bytes": raw_bytes,
            "sha256": sha256,
            "source": selected_source,
        },
        handle,
        indent=2,
    )
    handle.write("\n")
PY

mapfile -d '' -t selected_fields < <(
  python3 - "$SELECTED_INFO" <<'PY'
import json
import sys
info = json.load(open(sys.argv[1], encoding="utf-8"))
for key in ("path", "raw_bytes", "sha256", "source"):
    sys.stdout.write(str(info[key]) + "\0")
PY
)
selected_path=${selected_fields[0]}
selected_raw_bytes=${selected_fields[1]}
selected_sha=${selected_fields[2]}
selected_source=${selected_fields[3]}

selected_start_ns=$(date +%s%N)
target/release/replay-parser-2 parse "$selected_path" --output "$SELECTED_ARTIFACT" \
  > "$SELECTED_PARSE_LOG" 2>&1
selected_end_ns=$(date +%s%N)
selected_new_wall_time_ms=$(awk -v ns="$((selected_end_ns - selected_start_ns))" \
  'BEGIN { printf "%.6f", ns / 1000000 }')
selected_artifact_bytes=$(wc -c < "$SELECTED_ARTIFACT")
selected_artifact_raw_ratio=$(awk -v artifact="$selected_artifact_bytes" -v raw="$selected_raw_bytes" \
  'BEGIN { if (raw > 0) { printf "%.8f", artifact / raw } else { printf "0.0" } }')

python3 - "$REPLAY_LIST" "$selected_path" "$SELECTED_METADATA" <<'PY'
import json
import pathlib
import sys

replay_list_path, selected_path, output_path = sys.argv[1:]
selected_stem = pathlib.Path(selected_path).stem
metadata = None

try:
    replay_list = json.load(open(replay_list_path, encoding="utf-8"))
except (OSError, json.JSONDecodeError):
    replay_list = []

if isinstance(replay_list, dict):
    rows = replay_list.get("replays", [])
elif isinstance(replay_list, list):
    rows = replay_list
else:
    rows = []

for row in rows:
    if not isinstance(row, dict):
        continue
    filename = row.get("filename") or row.get("name") or row.get("replay") or row.get("file")
    if filename == selected_stem:
        mission_name = row.get("missionName") or row.get("mission_name") or row.get("mission")
        game_type = row.get("gameType") or row.get("game_type") or row.get("type")
        if not game_type and isinstance(mission_name, str) and "@" in mission_name:
            inferred = mission_name.split("@", 1)[0]
            if inferred in {"sg", "mace", "sm"}:
                game_type = inferred
        metadata = {
            "filename": filename,
            "date": row.get("date") or row.get("startDate") or row.get("createdAt"),
            "mission_name": mission_name,
            "game_type": game_type,
        }
        break

with open(output_path, "w", encoding="utf-8") as handle:
    json.dump({"metadata": metadata}, handle, indent=2)
    handle.write("\n")
PY

mapfile -d '' -t metadata_fields < <(
  python3 - "$SELECTED_METADATA" <<'PY'
import json
import sys

metadata = json.load(open(sys.argv[1], encoding="utf-8")).get("metadata")
if metadata and all(metadata.get(key) for key in ("filename", "date", "mission_name", "game_type")):
    values = ("true", metadata["filename"], metadata["date"], metadata["mission_name"], metadata["game_type"])
else:
    values = ("false", "", "", "", "")

for value in values:
    sys.stdout.write(str(value) + "\0")
PY
)
metadata_available=${metadata_fields[0]}
selected_filename=${metadata_fields[1]}
selected_date=${metadata_fields[2]}
selected_mission=${metadata_fields[3]}
selected_game_type=${metadata_fields[4]}

old_selected_available=false
old_selected_attempted=false

if [[ "${RUN_PHASE5_OLD_BASELINE:-auto}" != "0" \
  && "$selected_source" == "raw_replays" \
  && "$metadata_available" == "true" \
  && -d "$OLD_REPO" \
  && -d "$REAL_HOME/sg_stats/config" \
  && -d "$RAW_REPLAYS_DIR" \
  && -f "$REPLAY_LIST" \
  && "$(command -v pnpm || true)" != "" ]]; then
  old_selected_attempted=true
  OLD_RUN_ROOT="$COMPARISON_ROOT/old-selected-home-$$"
  OLD_RUN_HOME="$OLD_RUN_ROOT/home"
  OLD_RUNNER="$COMPARISON_ROOT/run-old-selected.ts"
  OLD_RUNNER_ABS="$(cd "$(dirname "$OLD_RUNNER")" && pwd)/$(basename "$OLD_RUNNER")"
  OLD_SELECTED_RESPONSE_ABS="$(cd "$(dirname "$OLD_SELECTED_RESPONSE")" && pwd)/$(basename "$OLD_SELECTED_RESPONSE")"
  OLD_SELECTED_ARTIFACT_ABS="$(cd "$(dirname "$OLD_SELECTED_ARTIFACT")" && pwd)/$(basename "$OLD_SELECTED_ARTIFACT")"
  mkdir -p "$OLD_RUN_HOME/sg_stats"
  OLD_RUN_HOME_ABS="$(cd "$OLD_RUN_HOME" && pwd)"
  ln -s "$REAL_HOME/sg_stats/raw_replays" "$OLD_RUN_HOME/sg_stats/raw_replays"
  ln -s "$REAL_HOME/sg_stats/lists" "$OLD_RUN_HOME/sg_stats/lists"
  cp -a "$REAL_HOME/sg_stats/config" "$OLD_RUN_HOME/sg_stats/config"

  cat > "$OLD_RUNNER" <<'TS'
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

type PlayerIdentity = {
  compatibilityKey: string | null;
  observedName: string | null;
  observedTag: string | null;
};

const splitPlayerName = (rawName: unknown): { name: string | null; tag: string | null } => {
  if (typeof rawName !== 'string') return { name: null, tag: null };
  const trimmed = rawName.trim();
  if (!trimmed.includes('[')) return { name: trimmed || null, tag: null };

  const firstTag = trimmed.match(/\[.*?\]/)?.[0]?.trim() ?? null;
  const name = trimmed
    .replace(/\[.*?\]/g, '')
    .replace('[', '')
    .replace(']', '')
    .trim();

  return {
    name: name || null,
    tag: firstTag && firstTag !== '[]' ? firstTag : null,
  };
};

const compatibilityKey = (player: any) => {
  const { name } = splitPlayerName(player?.name);
  if (name) {
    return `legacy_name:${name}`;
  }
  return null;
};

const playerIdentity = (player: any, byName?: Map<string, PlayerIdentity>): PlayerIdentity => {
  const split = splitPlayerName(player?.name);
  const fromList = split.name ? byName?.get(split.name) : undefined;

  return {
    compatibilityKey: compatibilityKey(player),
    observedName: split.name,
    observedTag: split.tag ?? fromList?.observedTag ?? null,
  };
};

const identitiesByObservedName = (players: any[]) => {
  const result = new Map<string, PlayerIdentity>();

  for (const player of players) {
    const identity = playerIdentity(player);
    if (identity.observedName && !result.has(identity.observedName)) {
      result.set(identity.observedName, identity);
    }
  }

  return result;
};

const relationshipRows = (
  players: any[],
  field: 'killed' | 'teamkilled',
  relationship: 'killed' | 'teamkilled',
  byName: Map<string, PlayerIdentity>,
) => players.flatMap((player) => {
  const source = playerIdentity(player, byName);

  return (player[field] ?? []).map((target: any) => {
    const targetIdentity = playerIdentity(target, byName);

    return {
      relationship,
      source_compatibility_key: source.compatibilityKey,
      source_observed_name: source.observedName,
      source_observed_tag: source.observedTag,
      target_compatibility_key: targetIdentity.compatibilityKey,
      target_observed_name: targetIdentity.observedName,
      target_observed_tag: targetIdentity.observedTag,
      count: target.count,
    };
  });
});

const inverseRelationshipRows = (
  players: any[],
  field: 'killed' | 'teamkilled',
  relationship: 'killers' | 'teamkillers',
  byName: Map<string, PlayerIdentity>,
) => players.flatMap((player) => {
  const target = playerIdentity(player, byName);

  return (player[field] ?? []).map((sourcePlayer: any) => {
    const source = playerIdentity(sourcePlayer, byName);

    return {
      relationship,
      source_compatibility_key: source.compatibilityKey,
      source_observed_name: source.observedName,
      source_observed_tag: source.observedTag,
      target_compatibility_key: target.compatibilityKey,
      target_observed_name: target.observedName,
      target_observed_tag: target.observedTag,
      count: sourcePlayer.count,
    };
  });
});

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
    taskId: 'phase-05-selected-large',
    filename,
    date,
    missionName,
    gameType: gameType as any,
  });
  const end = process.hrtime.bigint();
  const wallTimeMs = Number(end - start) / 1_000_000;

  fs.writeFileSync(responsePath, JSON.stringify({ response, wall_time_ms: wallTimeMs }, null, 2));

  const players = response.status === 'success' ? response.data.result : [];
  const identitiesByName = identitiesByObservedName(players);
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
        killed: relationshipRows(players, 'killed', 'killed', identitiesByName),
        killers: inverseRelationshipRows(players, 'killed', 'killers', identitiesByName),
        teamkilled: relationshipRows(players, 'teamkilled', 'teamkilled', identitiesByName),
        teamkillers: inverseRelationshipRows(players, 'teamkilled', 'teamkillers', identitiesByName),
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

  if (
    cd "$OLD_REPO"
    HOME="$OLD_RUN_HOME_ABS" WORKER_COUNT=1 pnpm exec tsx "$OLD_RUNNER_ABS" \
      "$OLD_REPO" "$selected_filename" "$selected_date" "$selected_mission" "$selected_game_type" \
      "$OLD_SELECTED_RESPONSE_ABS" \
      "$OLD_SELECTED_ARTIFACT_ABS"
  ) >"$OLD_SELECTED_LOG" 2>&1; then
    old_selected_available=true
  fi
fi

comparison_available=false
if [[ "$old_selected_available" == "true" && -f "$OLD_SELECTED_ARTIFACT" ]]; then
  if target/release/replay-parser-2 compare \
    --new-artifact "$SELECTED_ARTIFACT" \
    --old-artifact "$OLD_SELECTED_ARTIFACT" \
    --output "$COMPARISON_MARKDOWN" \
    --detail-output "$COMPARISON_REPORT"; then
    comparison_available=true
  fi
fi

all_raw_reason=""
all_raw_run=false

if [[ "${RUN_PHASE5_FULL_CORPUS:-0}" != "1" ]]; then
  all_raw_reason="RUN_PHASE5_FULL_CORPUS not enabled"
elif [[ ! -d "$RAW_REPLAYS_DIR" ]]; then
  all_raw_reason="missing raw replay directory: $RAW_REPLAYS_DIR"
else
  all_raw_run=true
fi

if [[ "$all_raw_run" == "true" ]]; then
  python3 - "$RAW_REPLAYS_DIR" "$ALL_RAW_OUTPUT" "$ALL_RAW_SUMMARY" "$ALL_RAW_FAILURES" "$ALL_RAW_OVERSIZED" "$MAX_DEFAULT_ARTIFACT_BYTES" <<'PY'
import hashlib
import json
import math
import pathlib
import statistics
import subprocess
import sys
import time

raw_replays_dir, output_dir, summary_path, failures_path, oversized_path, limit = sys.argv[1:]
raw_dir = pathlib.Path(raw_replays_dir).expanduser()
output_root = pathlib.Path(output_dir)
limit = int(limit)
output_root.mkdir(parents=True, exist_ok=True)

paths = sorted(raw_dir.rglob("*.json"), key=lambda path: str(path))
failures = []
oversized = []
ratios = []
attempted = 0
success = 0
failed = 0
skipped = 0
raw_bytes_total = 0
artifact_bytes_total = 0
max_artifact_bytes = 0

start = time.perf_counter()
for replay_path in paths:
    attempted += 1
    try:
        raw_bytes = replay_path.stat().st_size
    except OSError as error:
        failed += 1
        failures.append({
            "path": str(replay_path),
            "stage": "stat",
            "error": str(error),
        })
        continue

    raw_bytes_total += raw_bytes
    output_name = hashlib.sha256(str(replay_path).encode("utf-8")).hexdigest() + ".artifact.json"
    output_path = output_root / output_name
    result = subprocess.run(
        ["target/release/replay-parser-2", "parse", str(replay_path), "--output", str(output_path)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0 or not output_path.exists():
        failed += 1
        failures.append({
            "path": str(replay_path),
            "output": str(output_path),
            "returncode": result.returncode,
            "stderr_tail": result.stderr[-4000:],
        })
        continue

    artifact_bytes = output_path.stat().st_size
    artifact_bytes_total += artifact_bytes
    max_artifact_bytes = max(max_artifact_bytes, artifact_bytes)
    success += 1
    ratio = artifact_bytes / raw_bytes if raw_bytes > 0 else 0.0
    ratios.append(ratio)
    if artifact_bytes > limit:
        oversized.append({
            "path": str(replay_path),
            "output": str(output_path),
            "raw_bytes": raw_bytes,
            "artifact_bytes": artifact_bytes,
            "artifact_raw_ratio": ratio,
        })

wall_ms = (time.perf_counter() - start) * 1000
ratios.sort()
median_ratio = statistics.median(ratios) if ratios else None
p95_ratio = ratios[math.ceil(len(ratios) * 0.95) - 1] if ratios else None

summary = {
    "ran": True,
    "selector": "~/sg_stats/raw_replays/**/*.json sorted lexicographically",
    "attempted_count": attempted,
    "success_count": success,
    "failed_count": failed,
    "skipped_count": skipped,
    "raw_bytes": raw_bytes_total,
    "artifact_bytes": artifact_bytes_total,
    "new_wall_time_ms": wall_ms,
    "median_artifact_raw_ratio": median_ratio,
    "p95_artifact_raw_ratio": p95_ratio,
    "max_artifact_bytes": max_artifact_bytes,
    "oversized_artifact_count": len(oversized),
    "reason": None,
}

for path, value in (
    (summary_path, summary),
    (failures_path, failures),
    (oversized_path, oversized),
):
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2)
        handle.write("\n")
PY
else
  python3 - "$ALL_RAW_SUMMARY" "$ALL_RAW_FAILURES" "$ALL_RAW_OVERSIZED" "$all_raw_reason" <<'PY'
import json
import sys

summary_path, failures_path, oversized_path, reason = sys.argv[1:]
summary = {
    "ran": False,
    "selector": "~/sg_stats/raw_replays/**/*.json sorted lexicographically",
    "attempted_count": 0,
    "success_count": 0,
    "failed_count": 0,
    "skipped_count": 0,
    "raw_bytes": 0,
    "artifact_bytes": 0,
    "new_wall_time_ms": None,
    "median_artifact_raw_ratio": None,
    "p95_artifact_raw_ratio": None,
    "max_artifact_bytes": 0,
    "oversized_artifact_count": 0,
    "reason": reason,
}

for path, value in ((summary_path, summary), (failures_path, []), (oversized_path, [])):
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2)
        handle.write("\n")
PY
fi

old_all_raw_available=false
old_all_raw_covers_all=false
old_all_raw_wall_time_ms=""
old_all_raw_attempted_count=0
old_all_raw_success_count=0
old_all_raw_error_count=0
old_all_raw_skipped_count=0

if [[ "$all_raw_run" == "true" \
  && "${RUN_PHASE5_FULL_OLD_BASELINE:-0}" == "1" \
  && -d "$OLD_REPO" \
  && -d "$REAL_HOME/sg_stats/config" \
  && -d "$RAW_REPLAYS_DIR" \
  && -f "$REPLAY_LIST" \
  && "$(command -v pnpm || true)" != "" ]]; then
  OLD_FULL_ROOT="$COMPARISON_ROOT/old-all-raw-home-$$"
  OLD_FULL_HOME="$OLD_FULL_ROOT/home"
  OLD_FULL_RUNNER="$COMPARISON_ROOT/run-old-all-raw.ts"
  OLD_FULL_RUNNER_ABS="$(cd "$(dirname "$OLD_FULL_RUNNER")" && pwd)/$(basename "$OLD_FULL_RUNNER")"
  OLD_FULL_SUMMARY_ABS="$(cd "$(dirname "$OLD_FULL_SUMMARY")" && pwd)/$(basename "$OLD_FULL_SUMMARY")"
  mkdir -p "$OLD_FULL_HOME/sg_stats"
  OLD_FULL_HOME_ABS="$(cd "$OLD_FULL_HOME" && pwd)"
  ln -s "$REAL_HOME/sg_stats/raw_replays" "$OLD_FULL_HOME/sg_stats/raw_replays"
  ln -s "$REAL_HOME/sg_stats/lists" "$OLD_FULL_HOME/sg_stats/lists"
  cp -a "$REAL_HOME/sg_stats/config" "$OLD_FULL_HOME/sg_stats/config"

  cat > "$OLD_FULL_RUNNER" <<'TS'
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

type GameType = 'sg' | 'mace' | 'sm';

type ReplayRow = {
  filename?: string;
  date?: string;
  mission_name?: string;
  missionName?: string;
  game_type?: string;
  gameType?: string;
};

type TaskRow = {
  filename: string;
  date: string;
  missionName: string;
  gameType: GameType;
  path: string;
};

const gameTypes: GameType[] = ['sg', 'mace', 'sm'];

const missionName = (row: ReplayRow) => row.mission_name ?? row.missionName ?? '';

const flattenReplayRows = (rows: unknown): ReplayRow[] => {
  if (Array.isArray(rows)) return rows as ReplayRow[];
  if (rows && typeof rows === 'object' && Array.isArray((rows as { replays?: unknown }).replays)) {
    return (rows as { replays: ReplayRow[] }).replays;
  }

  return [];
};

const replayRowsByFilename = (rows: ReplayRow[]) => {
  const result = new Map<string, ReplayRow>();

  for (const row of rows) {
    if (row.filename && !result.has(row.filename)) result.set(row.filename, row);
  }

  return result;
};

const listJsonFiles = (root: string): string[] => {
  const result: string[] = [];
  const stack = [root];

  while (stack.length > 0) {
    const current = stack.pop() as string;
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const next = path.join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(next);
      } else if (entry.isFile() && entry.name.endsWith('.json')) {
        result.push(next);
      }
    }
  }

  return result.sort();
};

const inferDateFromFilename = (filename: string): string => {
  const match = filename.match(/^(\d{4})_(\d{2})_(\d{2})__(\d{2})_(\d{2})_(\d{2})/);
  if (!match) return '1970-01-01T00:00:00.000Z';

  const [, year, month, day, hour, minute, second] = match;
  return `${year}-${month}-${day}T${hour}:${minute}:${second}.000Z`;
};

const inferGameType = (row: ReplayRow | undefined, mission: string): GameType => {
  const explicit = row?.game_type ?? row?.gameType;
  if (explicit && gameTypes.includes(explicit as GameType)) return explicit as GameType;

  const prefix = mission.includes('@') ? mission.split('@', 1)[0] : mission.split(/\s+/, 1)[0];
  if (gameTypes.includes(prefix as GameType)) return prefix as GameType;
  if (mission.startsWith('mace')) return 'mace';
  if (mission.startsWith('sm')) return 'sm';

  return 'sg';
};

const taskForRawPath = (rawPath: string, rowsByFilename: Map<string, ReplayRow>): TaskRow => {
  const filename = path.basename(rawPath, '.json');
  const row = rowsByFilename.get(filename);
  const mission = missionName(row ?? {});
  const date = row?.date ?? inferDateFromFilename(filename);

  return {
    filename,
    date,
    missionName: mission,
    gameType: inferGameType(row, mission),
    path: rawPath,
  };
};

async function main() {
  const [oldRepo, replayListPath, rawReplaysDir, summaryPath] = process.argv.slice(2);
  const parseReplayInfoPath = path.join(oldRepo, 'src/2 - parseReplayInfo/index.ts');
  const prepareNamesListPath = path.join(oldRepo, 'src/0 - utils/namesHelper/prepareNamesList.ts');
  const { default: parseReplayInfo } = await import(pathToFileURL(parseReplayInfoPath).href);
  const { prepareNamesList } = await import(pathToFileURL(prepareNamesListPath).href);
  const replayList = JSON.parse(fs.readFileSync(replayListPath, 'utf8'));
  const rows = flattenReplayRows(replayList);
  const rowsByFilename = replayRowsByFilename(rows);
  const rawPaths = listJsonFiles(rawReplaysDir);
  const tasks = rawPaths.map((rawPath) => taskForRawPath(rawPath, rowsByFilename));
  const statusCounts: Record<string, number> = {};
  const firstErrors: unknown[] = [];
  const byGameType: Record<GameType, number> = { sg: 0, mace: 0, sm: 0 };

  prepareNamesList();

  const start = process.hrtime.bigint();

  for (let index = 0; index < tasks.length; index += 1) {
    const task = tasks[index];
    byGameType[task.gameType] += 1;

    try {
      const replayInfo = JSON.parse(fs.readFileSync(task.path, 'utf8')) as ReplayInfo;
      const parsedReplayInfo = parseReplayInfo(replayInfo, task.date);
      const result = Object.values(parsedReplayInfo);
      statusCounts.success = (statusCounts.success ?? 0) + 1;
      if (index % 1000 === 0) {
        process.stderr.write(`old baseline parsed ${index + 1}/${tasks.length}\n`);
      }
      void result;
    } catch (error) {
      statusCounts.error = (statusCounts.error ?? 0) + 1;
      if (firstErrors.length < 10) {
        firstErrors.push({
          taskId: `phase-05-all-raw-${index + 1}`,
          status: 'error',
          filename: task.filename,
          path: task.path,
          message: error instanceof Error ? error.message : String(error),
          stack: error instanceof Error ? error.stack : undefined,
        });
      }
    }
  }

  const end = process.hrtime.bigint();
  const wallTimeMs = Number(end - start) / 1_000_000;

  fs.writeFileSync(summaryPath, JSON.stringify({
    source: 'raw_replays_full_direct_parseReplayInfo_no_skip',
    attempted_count: tasks.length,
    success_count: statusCounts.success ?? 0,
    error_count: statusCounts.error ?? 0,
    skipped_count: 0,
    status_counts: statusCounts,
    by_game_type: byGameType,
    replay_list_unique_count: rows.length,
    raw_file_count: rawPaths.length,
    wall_time_ms: wallTimeMs,
    first_errors: firstErrors,
    first_skipped: [],
  }, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
TS

  old_full_start_ns=$(date +%s%N)
  if (
    cd "$OLD_REPO"
    HOME="$OLD_FULL_HOME_ABS" WORKER_COUNT=1 pnpm exec tsx "$OLD_FULL_RUNNER_ABS" \
      "$OLD_REPO" "$OLD_FULL_HOME_ABS/sg_stats/lists/replaysList.json" \
      "$OLD_FULL_HOME_ABS/sg_stats/raw_replays" "$OLD_FULL_SUMMARY_ABS"
  ) >"$OLD_FULL_LOG" 2>&1; then
    old_all_raw_available=true
  fi
  old_full_end_ns=$(date +%s%N)
  old_all_raw_wall_time_ms=$(awk -v ns="$((old_full_end_ns - old_full_start_ns))" \
    'BEGIN { printf "%.6f", ns / 1000000 }')

  read -r old_all_raw_attempted_count old_all_raw_success_count old_all_raw_error_count old_all_raw_skipped_count all_raw_attempted < <(
    python3 - "$OLD_FULL_SUMMARY" "$ALL_RAW_SUMMARY" <<'PY'
import json
import sys

try:
    old_summary = json.load(open(sys.argv[1], encoding="utf-8"))
except (OSError, json.JSONDecodeError):
    old_summary = {}
summary = json.load(open(sys.argv[2], encoding="utf-8"))
print(
    old_summary.get("attempted_count", 0),
    old_summary.get("success_count", 0),
    old_summary.get("error_count", 0),
    old_summary.get("skipped_count", 0),
    summary.get("attempted_count", 0),
)
PY
  )
  if [[ "$old_all_raw_available" == "true" \
    && "$old_all_raw_attempted_count" == "$all_raw_attempted" \
    && "$old_all_raw_skipped_count" == "0" ]]; then
    old_all_raw_covers_all=true
  fi
fi

python3 - "$REPORT_PATH" "$SELECTED_INFO" "$SELECTED_ARTIFACT" "$selected_new_wall_time_ms" \
  "$selected_artifact_bytes" "$selected_artifact_raw_ratio" "$old_selected_available" \
  "$OLD_SELECTED_RESPONSE" "$comparison_available" "$COMPARISON_REPORT" "$old_selected_attempted" \
  "$ALL_RAW_SUMMARY" "$old_all_raw_available" "$old_all_raw_covers_all" "$old_all_raw_wall_time_ms" \
  "$MAX_DEFAULT_ARTIFACT_BYTES" "$ALL_RAW_FAILURES" "$ALL_RAW_OVERSIZED" "$ALL_RAW_SELECTOR" \
  "${RUN_PHASE5_FULL_CORPUS:-0}" "${RUN_PHASE5_FULL_OLD_BASELINE:-0}" \
  "$old_all_raw_attempted_count" "$old_all_raw_success_count" "$old_all_raw_error_count" "$old_all_raw_skipped_count" <<'PY'
import json
import sys

(
    report_path,
    selected_info_path,
    selected_artifact_path,
    selected_new_wall_ms_raw,
    selected_artifact_bytes_raw,
    selected_artifact_ratio_raw,
    old_selected_available_raw,
    old_selected_response_path,
    comparison_available_raw,
    comparison_report_path,
    old_selected_attempted_raw,
    all_raw_summary_path,
    old_all_raw_available_raw,
    old_all_raw_covers_all_raw,
    old_all_raw_wall_ms_raw,
    limit_raw,
    all_raw_failures_path,
    all_raw_oversized_path,
    all_raw_selector,
    run_full_corpus,
    run_full_old_baseline,
    old_all_raw_attempted_count_raw,
    old_all_raw_success_count_raw,
    old_all_raw_error_count_raw,
    old_all_raw_skipped_count_raw,
) = sys.argv[1:]

limit = int(limit_raw)

def nullable_float(raw):
    if raw in {"", "null", "None"}:
        return None
    return float(raw)

def gate_triage(scope, reason):
    return (
        f"bottleneck: {scope} cannot pass because {reason}; "
        f"parity: selected parity must pass and all-raw old coverage must be deterministic; "
        f"artifact: default artifacts must satisfy median <= 5%, p95 <= 10%, and each artifact <= {limit} bytes; "
        f"failure: failed/skipped artifacts remain blocking unless an explicit allowlist is accepted by the user."
    )

selected_info = json.load(open(selected_info_path, encoding="utf-8"))
all_raw_summary = json.load(open(all_raw_summary_path, encoding="utf-8"))
selected_artifact_bytes = int(selected_artifact_bytes_raw)
selected_new_wall_ms = nullable_float(selected_new_wall_ms_raw)
selected_speedup = None
selected_old_wall_ms = None
selected_parity = "not_run"

if old_selected_available_raw == "true":
    selected_old_wall_ms = float(json.load(open(old_selected_response_path, encoding="utf-8"))["wall_time_ms"])
    if selected_new_wall_ms and selected_new_wall_ms > 0:
        selected_speedup = selected_old_wall_ms / selected_new_wall_ms

if comparison_available_raw == "true":
    comparison = json.load(open(comparison_report_path, encoding="utf-8"))
    legacy_surfaces = {
        "status",
        "replay",
        "legacy.player_game_results",
        "legacy.relationships",
        "bounty.inputs",
    }
    legacy_categories = {
        finding.get("category")
        for finding in comparison.get("findings", [])
        if finding.get("surface") in legacy_surfaces
    }
    legacy_categories.discard(None)
    if legacy_categories == {"compatible"}:
        selected_parity = "passed"
    elif "human_review" in legacy_categories:
        selected_parity = "human_review"
    else:
        selected_parity = "failed"

selected_artifact_status = "pass" if selected_artifact_bytes <= limit else "fail"
if selected_speedup is None or selected_parity == "not_run":
    selected_x3_status = "unknown"
elif selected_speedup >= 3.0 and selected_parity == "passed":
    selected_x3_status = "pass"
else:
    selected_x3_status = "fail"

selected_triage = None
if selected_x3_status != "pass" or selected_parity != "passed" or selected_artifact_status != "pass":
    if selected_info["source"] != "raw_replays":
        reason = "no raw replay corpus was available, so the selected replay is a smoke fixture"
    elif old_selected_available_raw != "true":
        reason = "selected old WORKER_COUNT=1 baseline or metadata was unavailable"
    elif comparison_available_raw != "true":
        reason = "selected old-vs-new comparison did not complete"
    else:
        reason = (
            f"selected speedup={selected_speedup}, parity={selected_parity}, "
            f"artifact_bytes={selected_artifact_bytes}"
        )
    selected_triage = gate_triage("selected_large_replay", reason)

all_raw_old_wall_ms = nullable_float(old_all_raw_wall_ms_raw)
all_raw_new_wall_ms = all_raw_summary.get("new_wall_time_ms")
all_raw_speedup = None
if old_all_raw_available_raw == "true" and old_all_raw_covers_all_raw == "true":
    if all_raw_new_wall_ms and all_raw_new_wall_ms > 0 and all_raw_old_wall_ms:
        all_raw_speedup = all_raw_old_wall_ms / all_raw_new_wall_ms

if all_raw_speedup is None:
    all_raw_x10_status = "unknown"
elif all_raw_speedup >= 10.0:
    all_raw_x10_status = "pass"
else:
    all_raw_x10_status = "fail"

median_ratio = all_raw_summary.get("median_artifact_raw_ratio")
p95_ratio = all_raw_summary.get("p95_artifact_raw_ratio")
if not all_raw_summary.get("ran"):
    all_raw_size_status = "unknown"
elif (
    all_raw_summary.get("success_count", 0) > 0
    and median_ratio is not None and median_ratio <= 0.05
    and p95_ratio is not None and p95_ratio <= 0.10
    and all_raw_summary.get("max_artifact_bytes", 0) <= limit
    and all_raw_summary.get("oversized_artifact_count", 0) == 0
):
    all_raw_size_status = "pass"
else:
    all_raw_size_status = "fail"

if not all_raw_summary.get("ran"):
    zero_failure_status = "unknown"
elif all_raw_summary.get("failed_count", 0) == 0 and all_raw_summary.get("skipped_count", 0) == 0:
    zero_failure_status = "pass"
else:
    zero_failure_status = "fail"

all_raw_triage = None
if all_raw_x10_status != "pass" or all_raw_size_status != "pass" or zero_failure_status != "pass":
    if not all_raw_summary.get("ran"):
        reason = all_raw_summary.get("reason") or "all-raw corpus was not run"
    elif old_all_raw_available_raw != "true":
        reason = "all-raw old baseline was not run; set RUN_PHASE5_FULL_OLD_BASELINE=1 for full evidence"
    elif old_all_raw_covers_all_raw != "true":
        reason = (
            "old baseline did not cover every raw replay file "
            f"(old_attempted={old_all_raw_attempted_count_raw}, "
            f"old_success={old_all_raw_success_count_raw}, "
            f"old_error={old_all_raw_error_count_raw}, "
            f"old_skipped={old_all_raw_skipped_count_raw}, "
            f"new_attempted={all_raw_summary.get('attempted_count', 0)})"
        )
    else:
        reason = (
            f"speedup={all_raw_speedup}, median={median_ratio}, p95={p95_ratio}, "
            f"max_artifact_bytes={all_raw_summary.get('max_artifact_bytes')}, "
            f"oversized_artifact_count={all_raw_summary.get('oversized_artifact_count')}, "
            f"failed_count={all_raw_summary.get('failed_count')}, skipped_count={all_raw_summary.get('skipped_count')}"
        )
    all_raw_triage = gate_triage("all_raw_corpus", reason)

old_profile = "deterministic old baseline uses WORKER_COUNT=1"
if old_selected_available_raw == "true":
    old_profile += "; selected_large_replay old baseline captured with HOME=<generated-fake-home> WORKER_COUNT=1 pnpm exec tsx"
elif old_selected_attempted_raw == "true":
    old_profile += "; selected_large_replay old baseline attempted but unavailable"
else:
    old_profile += "; selected_large_replay old baseline not-run in --ci"

if old_all_raw_available_raw == "true":
    old_profile += "; all_raw_corpus old baseline captured with HOME=<generated-fake-home> WORKER_COUNT=1 pnpm exec tsx run-old-all-raw.ts"
elif run_full_old_baseline == "1":
    old_profile += "; all_raw_corpus old baseline attempted but unavailable"
else:
    old_profile += "; all_raw_corpus old baseline not-run in --ci"

report = {
    "report_version": "2",
    "phase": "05.2",
    "old_baseline_profile": old_profile,
    "artifact_size_limit_bytes": limit,
    "selected_large_replay": {
        "selection_policy": selected_info["selection_policy"],
        "path": selected_info["path"],
        "raw_bytes": selected_info["raw_bytes"],
        "sha256": selected_info["sha256"],
        "old_wall_time_ms": selected_old_wall_ms,
        "new_wall_time_ms": selected_new_wall_ms,
        "speedup": selected_speedup,
        "x3_status": selected_x3_status,
        "parity_status": selected_parity,
        "artifact_bytes": selected_artifact_bytes,
        "artifact_raw_ratio": float(selected_artifact_ratio_raw),
        "artifact_size_status": selected_artifact_status,
        "triage": selected_triage,
    },
    "all_raw_corpus": {
        "selector": all_raw_selector,
        "attempted_count": all_raw_summary.get("attempted_count", 0),
        "success_count": all_raw_summary.get("success_count", 0),
        "failed_count": all_raw_summary.get("failed_count", 0),
        "skipped_count": all_raw_summary.get("skipped_count", 0),
        "raw_bytes": all_raw_summary.get("raw_bytes", 0),
        "artifact_bytes": all_raw_summary.get("artifact_bytes", 0),
        "old_wall_time_ms": all_raw_old_wall_ms,
        "new_wall_time_ms": all_raw_new_wall_ms,
        "speedup": all_raw_speedup,
        "x10_status": all_raw_x10_status,
        "median_artifact_raw_ratio": median_ratio,
        "p95_artifact_raw_ratio": p95_ratio,
        "max_artifact_bytes": all_raw_summary.get("max_artifact_bytes", 0),
        "oversized_artifact_count": all_raw_summary.get("oversized_artifact_count", 0),
        "size_gate_status": all_raw_size_status,
        "zero_failure_status": zero_failure_status,
        "triage": all_raw_triage,
    },
    "allowlist": None,
    "rss_note": "RSS is not captured in the portable CI fallback; use an external process monitor for selected or all-raw benchmark runs.",
    "evidence_files": {
        "selected_info": selected_info_path,
        "selected_artifact": selected_artifact_path,
        "all_raw_summary": all_raw_summary_path,
        "all_raw_failures": all_raw_failures_path,
        "all_raw_oversized_artifacts": all_raw_oversized_path,
        "comparison_report": comparison_report_path if comparison_available_raw == "true" else None,
    },
    "full_acceptance_requires": (
        "selected_large_replay x3_status=pass, parity_status=passed, artifact_size_status=pass; "
        "all_raw_corpus x10_status=pass, zero_failure_status=pass, size_gate_status=pass; "
        "artifact_size_limit_bytes=100000 with max_artifact_bytes <= 100000 and oversized_artifact_count == 0"
    ),
}

with open(report_path, "w", encoding="utf-8") as handle:
    json.dump(report, handle, indent=2)
    handle.write("\n")
PY

cargo run -p parser-harness --bin benchmark-report-check -- \
  --report "$REPORT_PATH" \
  --mode acceptance
