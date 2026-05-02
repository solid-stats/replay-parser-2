#!/usr/bin/env python3
"""Compare old and new parser statistics on year-edge replay samples."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import random
import shutil
import subprocess
import sys
import textwrap
from collections import Counter, defaultdict
from dataclasses import dataclass
from typing import Any


GAME_TYPES = ("sg", "mace", "sm")
DEFAULT_REAL_HOME = pathlib.Path("/home/afgan0r")
DEFAULT_OLD_REPO = pathlib.Path("/home/afgan0r/Projects/SolidGames/replays-parser")
DEFAULT_OUTPUT = pathlib.Path(
    ".planning/generated/quick/260502-k2u-old-new-year-edge-parity"
)


@dataclass(frozen=True)
class ReplaySample:
    sample_id: str
    game_type: str
    year: str
    bucket: str
    filename: str
    date: str
    mission_name: str
    raw_path: pathlib.Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Compare old/new parser statistics for year-edge replay samples."
    )
    parser.add_argument("--output-root", type=pathlib.Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--real-home", type=pathlib.Path, default=DEFAULT_REAL_HOME)
    parser.add_argument("--old-repo", type=pathlib.Path, default=DEFAULT_OLD_REPO)
    parser.add_argument("--new-bin", type=pathlib.Path, default=pathlib.Path("target/release/replay-parser-2"))
    parser.add_argument("--edge-window", type=int, default=20)
    parser.add_argument("--per-edge", type=int, default=2)
    args = parser.parse_args()

    output_root = args.output_root
    output_root.mkdir(parents=True, exist_ok=True)

    replay_list = args.real_home / "sg_stats/lists/replaysList.json"
    raw_dir = args.real_home / "sg_stats/raw_replays"
    config_dir = args.real_home / "sg_stats/config"
    validate_inputs(replay_list, raw_dir, config_dir, args.old_repo)

    subprocess.run(
        ["cargo", "build", "--release", "-q", "-p", "parser-cli", "--bin", "replay-parser-2"],
        check=True,
    )

    samples = select_samples(replay_list, raw_dir, args.edge_window, args.per_edge)
    write_json(output_root / "selected-replays.json", [sample.__dict__ | {"raw_path": str(sample.raw_path)} for sample in samples])

    new_dir = output_root / "new-artifacts"
    old_dir = output_root / "old-artifacts"
    compare_dir = output_root / "comparison-reports"
    new_dir.mkdir(exist_ok=True)
    old_dir.mkdir(exist_ok=True)
    compare_dir.mkdir(exist_ok=True)

    new_results = run_new_parser(samples, args.new_bin, new_dir)
    old_results = run_old_parser(samples, args.old_repo, args.real_home, old_dir, output_root)
    comparison_results = run_comparisons(samples, args.new_bin, new_results, old_results, compare_dir)
    stats_results = compare_stats_only(samples, new_results, old_results, output_root / "stats-only-reports")
    summary = build_summary(
        samples,
        new_results,
        old_results,
        comparison_results,
        stats_results,
        args.edge_window,
        args.per_edge,
    )
    write_json(output_root / "summary.json", summary)
    write_markdown(output_root / "summary.md", summary)

    print(json.dumps(summary["result"], indent=2, ensure_ascii=False))
    result = summary["result"]
    return 0 if result["stats_only_mismatches"] == 0 and result["stats_only_new_failed"] == 0 else 1


def validate_inputs(replay_list: pathlib.Path, raw_dir: pathlib.Path, config_dir: pathlib.Path, old_repo: pathlib.Path) -> None:
    missing = [path for path in (replay_list, raw_dir, config_dir, old_repo) if not path.exists()]
    if missing:
        raise SystemExit("missing required input(s): " + ", ".join(str(path) for path in missing))
    if shutil.which("pnpm") is None:
        raise SystemExit("pnpm is required to run the old parser baseline")


def select_samples(replay_list: pathlib.Path, raw_dir: pathlib.Path, edge_window: int, per_edge: int) -> list[ReplaySample]:
    payload = json.loads(replay_list.read_text(encoding="utf-8"))
    rows = payload.get("replays", [])
    grouped: dict[tuple[str, str], list[dict[str, Any]]] = defaultdict(list)

    for row in rows:
        mission_name = row.get("mission_name") or row.get("missionName")
        filename = row.get("filename")
        date = row.get("date")
        if not (isinstance(mission_name, str) and isinstance(filename, str) and isinstance(date, str)):
            continue
        game_type = mission_name.split("@", 1)[0] if "@" in mission_name else None
        if game_type not in GAME_TYPES:
            continue
        raw_path = raw_dir / f"{filename}.json"
        if not raw_path.is_file():
            continue
        grouped[(game_type, date[:4])].append(row)

    selected: dict[str, ReplaySample] = {}
    for (game_type, year), rows_for_group in sorted(grouped.items()):
        rows_for_group.sort(key=lambda row: (row["date"], row["filename"]))
        for bucket, candidates in (
            ("start", rows_for_group[:edge_window]),
            ("end", rows_for_group[-edge_window:]),
        ):
            rng = random.Random(f"{game_type}:{year}:{bucket}:260502-k2u")
            picked = sorted(
                rng.sample(candidates, min(per_edge, len(candidates))),
                key=lambda row: (row["date"], row["filename"]),
            )
            for row in picked:
                sample_id = sample_id_for(game_type, year, bucket, row["filename"])
                selected[row["filename"]] = ReplaySample(
                    sample_id=sample_id,
                    game_type=game_type,
                    year=year,
                    bucket=bucket,
                    filename=row["filename"],
                    date=row["date"],
                    mission_name=row["mission_name"],
                    raw_path=raw_dir / f"{row['filename']}.json",
                )

    return sorted(selected.values(), key=lambda sample: (sample.game_type, sample.year, sample.bucket, sample.date, sample.filename))


def sample_id_for(game_type: str, year: str, bucket: str, filename: str) -> str:
    digest = hashlib.sha1(filename.encode("utf-8")).hexdigest()[:8]
    return f"{game_type}-{year}-{bucket}-{digest}"


def run_new_parser(samples: list[ReplaySample], new_bin: pathlib.Path, output_dir: pathlib.Path) -> dict[str, dict[str, Any]]:
    results: dict[str, dict[str, Any]] = {}
    for sample in samples:
        output = output_dir / f"{sample.sample_id}.json"
        completed = subprocess.run(
            [str(new_bin), "parse", str(sample.raw_path), "--output", str(output), "--replay-id", sample.filename],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        results[sample.sample_id] = {
            "ok": completed.returncode == 0 and output.exists(),
            "returncode": completed.returncode,
            "artifact": str(output),
            "stderr_tail": completed.stderr[-2000:],
        }
    return results


def run_old_parser(
    samples: list[ReplaySample],
    old_repo: pathlib.Path,
    real_home: pathlib.Path,
    output_dir: pathlib.Path,
    work_root: pathlib.Path,
) -> dict[str, dict[str, Any]]:
    runner = work_root / "run-old-year-edge-sample.ts"
    input_path = work_root / "old-runner-input.json"
    response_path = work_root / "old-runner-response.json"
    old_home = work_root / "old-parser-home/home"
    (old_home / "sg_stats").mkdir(parents=True, exist_ok=True)
    link_or_copy(real_home / "sg_stats/raw_replays", old_home / "sg_stats/raw_replays")
    link_or_copy(real_home / "sg_stats/lists", old_home / "sg_stats/lists")
    copy_tree(real_home / "sg_stats/config", old_home / "sg_stats/config")

    write_json(
        input_path,
        [
            {
                "sampleId": sample.sample_id,
                "filename": sample.filename,
                "date": sample.date,
                "missionName": sample.mission_name,
                "gameType": sample.game_type,
                "artifactPath": str((output_dir / f"{sample.sample_id}.json").resolve()),
            }
            for sample in samples
        ],
    )
    runner.write_text(old_runner_source(), encoding="utf-8")

    completed = subprocess.run(
        [
            "pnpm",
            "exec",
            "tsx",
            str(runner.resolve()),
            str(old_repo.resolve()),
            str(input_path.resolve()),
            str(response_path.resolve()),
        ],
        cwd=old_repo,
        env={**os.environ, "HOME": str(old_home.resolve()), "WORKER_COUNT": "1"},
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        (work_root / "old-runner.stderr.log").write_text(completed.stderr, encoding="utf-8")
        raise SystemExit(f"old parser runner failed with code {completed.returncode}; see {work_root / 'old-runner.stderr.log'}")

    payload = json.loads(response_path.read_text(encoding="utf-8"))
    return {item["sampleId"]: item for item in payload["results"]}


def old_runner_source() -> str:
    return textwrap.dedent(
        r"""
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
          const name = trimmed.replace(/\[.*?\]/g, '').replace('[', '').replace(']', '').trim();
          return { name: name || null, tag: firstTag && firstTag !== '[]' ? firstTag : null };
        };

        const compatibilityKey = (player: any) => {
          const { name } = splitPlayerName(player?.name);
          return name ? `legacy_name:${name}` : null;
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
          const [oldRepo, inputPath, responsePath] = process.argv.slice(2);
          const workerPath = path.join(oldRepo, 'src/1 - replays/workers/parseReplayWorker.ts');
          const { runParseTask } = await import(pathToFileURL(workerPath).href);
          const samples = JSON.parse(fs.readFileSync(inputPath, 'utf8'));
          const results = [];

          for (const sample of samples) {
            const start = process.hrtime.bigint();
            const response = await runParseTask({
              taskId: `year-edge-${sample.sampleId}`,
              filename: sample.filename,
              date: sample.date,
              missionName: sample.missionName,
              gameType: sample.gameType,
            });
            const end = process.hrtime.bigint();
            const wallTimeMs = Number(end - start) / 1_000_000;
            const players = response.status === 'success' ? response.data.result : [];
            const identitiesByName = identitiesByObservedName(players);
            const artifact = {
              status: response.status === 'success' ? 'success' : response.status,
              replay: {
                filename: sample.filename,
                date: sample.date,
                mission_name: sample.missionName,
                game_type: sample.gameType,
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
              bounty: { inputs: [] },
            };
            fs.writeFileSync(sample.artifactPath, JSON.stringify(artifact, null, 2));
            results.push({
              sampleId: sample.sampleId,
              ok: response.status === 'success',
              status: response.status,
              wall_time_ms: wallTimeMs,
              artifact: sample.artifactPath,
            });
          }

          fs.writeFileSync(responsePath, JSON.stringify({ results }, null, 2));
        }

        main().catch((error) => {
          console.error(error);
          process.exit(1);
        });
        """
    ).strip() + "\n"


def run_comparisons(
    samples: list[ReplaySample],
    new_bin: pathlib.Path,
    new_results: dict[str, dict[str, Any]],
    old_results: dict[str, dict[str, Any]],
    output_dir: pathlib.Path,
) -> dict[str, dict[str, Any]]:
    results: dict[str, dict[str, Any]] = {}
    for sample in samples:
        new_result = new_results[sample.sample_id]
        old_result = old_results.get(sample.sample_id, {"ok": False})
        output = output_dir / f"{sample.sample_id}.json"
        if not new_result["ok"] or not old_result.get("ok"):
            results[sample.sample_id] = {
                "ok": False,
                "skipped": True,
                "reason": "new or old parser failed",
                "report": str(output),
            }
            continue
        completed = subprocess.run(
            [
                str(new_bin),
                "compare",
                "--new-artifact",
                new_result["artifact"],
                "--old-artifact",
                old_result["artifact"],
                "--output",
                str(output),
                "--format",
                "json",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        ok = completed.returncode == 0 and output.exists()
        summary = {}
        if ok:
            report = json.loads(output.read_text(encoding="utf-8"))
            summary = report.get("summary", {})
        results[sample.sample_id] = {
            "ok": ok,
            "returncode": completed.returncode,
            "report": str(output),
            "summary": summary,
            "stderr_tail": completed.stderr[-2000:],
        }
    return results


def compare_stats_only(
    samples: list[ReplaySample],
    new_results: dict[str, dict[str, Any]],
    old_results: dict[str, dict[str, Any]],
    output_dir: pathlib.Path,
) -> dict[str, dict[str, Any]]:
    output_dir.mkdir(exist_ok=True)
    results: dict[str, dict[str, Any]] = {}

    for sample in samples:
        new_result = new_results.get(sample.sample_id, {})
        old_result = old_results.get(sample.sample_id, {})
        report_path = output_dir / f"{sample.sample_id}.json"

        if not new_result.get("ok"):
            result = stats_result("new_failed", report_path, ["new parser did not produce an artifact"])
        elif not old_result.get("ok"):
            result = stats_result("old_skipped", report_path, ["old parser returned skipped/no statistics"])
        else:
            old_view = old_stats_view(json.loads(pathlib.Path(old_result["artifact"]).read_text(encoding="utf-8")))
            new_view = new_stats_view(json.loads(pathlib.Path(new_result["artifact"]).read_text(encoding="utf-8")))
            diffs = stats_diffs(old_view, new_view)
            result = {
                "status": "match" if not diffs else "mismatch",
                "report": str(report_path),
                "diff_count": len(diffs),
                "diffs": diffs[:200],
                "truncated": len(diffs) > 200,
            }
            write_json(
                report_path,
                {
                    "sample_id": sample.sample_id,
                    "filename": sample.filename,
                    "game_type": sample.game_type,
                    "year": sample.year,
                    "bucket": sample.bucket,
                    "status": result["status"],
                    "diff_count": len(diffs),
                    "diffs": diffs,
                    "old_stats": old_view,
                    "new_stats": new_view,
                },
            )

        results[sample.sample_id] = result

    return results


def stats_result(status: str, report_path: pathlib.Path, diffs: list[str]) -> dict[str, Any]:
    write_json(report_path, {"status": status, "diff_count": len(diffs), "diffs": diffs})
    return {
        "status": status,
        "report": str(report_path),
        "diff_count": len(diffs),
        "diffs": diffs,
        "truncated": False,
    }


def old_stats_view(root: dict[str, Any]) -> dict[str, Any]:
    players = root.get("legacy", {}).get("player_game_results", [])
    player_stats: dict[str, dict[str, Any]] = {}
    weapon_stats: list[dict[str, Any]] = []
    relationships: list[dict[str, Any]] = []

    for player in players:
        key = compatibility_key_from_name(player.get("name")) or f"old_id:{player.get('id')}"
        player_stats[key] = {
            "kills": int(player.get("kills") or 0),
            "killsFromVehicle": int(player.get("killsFromVehicle") or 0),
            "vehicleKills": int(player.get("vehicleKills") or 0),
            "teamkills": int(player.get("teamkills") or 0),
            "isDead": bool(player.get("isDead")),
            "isDeadByTeamkill": bool(player.get("isDeadByTeamkill")),
        }

        for weapon in player.get("weapons") or []:
            weapon_stats.append(
                {
                    "player": key,
                    "weapon": weapon.get("name"),
                    "kills": int(weapon.get("kills") or 0),
                }
            )

        for relation in ("killed", "killers", "teamkilled", "teamkillers"):
            for target in player.get(relation) or []:
                relationships.append(
                    {
                        "relationship": relation,
                        "source": key,
                        "target": compatibility_key_from_name(target.get("name")) or str(target.get("id")),
                        "count": int(target.get("count") or 0),
                    }
                )

    return {
        "players": sort_mapping(player_stats),
        "relationships": sorted(relationships, key=stable_json_key),
        "weapons": sorted(weapon_stats, key=stable_json_key),
    }


def new_stats_view(root: dict[str, Any]) -> dict[str, Any]:
    players = root.get("players") or []
    weapons = {weapon.get("id"): weapon.get("n") for weapon in root.get("weapons") or []}
    refs = new_player_refs(players)
    teamkill_deaths = Counter()
    relationships: list[dict[str, Any]] = []
    weapon_stats = Counter()

    for player in players:
        source_key = new_player_key(player)
        for kill in player.get("kills") or []:
            victim = refs.get(kill.get("v"))
            if not victim:
                continue
            classification = kill.get("c")
            weapon_name = weapons.get(kill.get("w"))
            if classification == "enemy_kill":
                relationships.append(
                    {
                        "relationship": "killed",
                        "source": source_key,
                        "target": victim,
                        "count": 1,
                    }
                )
                relationships.append(
                    {
                        "relationship": "killers",
                        "source": victim,
                        "target": source_key,
                        "count": 1,
                    }
                )
                if weapon_name:
                    weapon_stats[(source_key, weapon_name)] += 1
            elif classification == "teamkill":
                relationships.append(
                    {
                        "relationship": "teamkilled",
                        "source": source_key,
                        "target": victim,
                        "count": 1,
                    }
                )
                relationships.append(
                    {
                        "relationship": "teamkillers",
                        "source": victim,
                        "target": source_key,
                        "count": 1,
                    }
                )
                teamkill_deaths[victim] += 1

    player_stats = {}
    for player in players:
        key = new_player_key(player)
        player_stats[key] = {
            "kills": int(player.get("k") or 0),
            "killsFromVehicle": int(player.get("kfv") or 0),
            "vehicleKills": int(player.get("vk") or 0),
            "teamkills": int(player.get("tk") or 0),
            "isDead": int(player.get("d") or 0) > 0,
            "isDeadByTeamkill": teamkill_deaths[key] > 0,
        }

    return {
        "players": sort_mapping(player_stats),
        "relationships": sorted(merge_relationship_counts(relationships), key=stable_json_key),
        "weapons": sorted(
            [
                {"player": player, "weapon": weapon, "kills": kills}
                for (player, weapon), kills in weapon_stats.items()
            ],
            key=stable_json_key,
        ),
    }


def new_player_refs(players: list[dict[str, Any]]) -> dict[int, str]:
    refs: dict[int, str] = {}
    for player in players:
        key = new_player_key(player)
        entity_ids = list(player.get("eids") or [])
        if player.get("eid") is not None:
            entity_ids.append(player["eid"])
        for entity_id in entity_ids:
            refs[int(entity_id)] = key
    return refs


def new_player_key(player: dict[str, Any]) -> str:
    if player.get("ck"):
        return str(player["ck"])
    if player.get("n"):
        return f"legacy_name:{player['n']}"
    return f"entity:{player.get('eid')}"


def merge_relationship_counts(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    counts = Counter()
    for row in rows:
        counts[(row["relationship"], row["source"], row["target"])] += row["count"]
    return [
        {"relationship": relationship, "source": source, "target": target, "count": count}
        for (relationship, source, target), count in counts.items()
    ]


def stats_diffs(old_view: dict[str, Any], new_view: dict[str, Any]) -> list[str]:
    diffs: list[str] = []
    for section in ("players", "relationships", "weapons"):
        old_value = old_view[section]
        new_value = new_view[section]
        if old_value != new_value:
            diffs.extend(section_diffs(section, old_value, new_value))
    return diffs


def section_diffs(section: str, old_value: Any, new_value: Any) -> list[str]:
    if isinstance(old_value, dict) and isinstance(new_value, dict):
        diffs = []
        for key in sorted(set(old_value) | set(new_value)):
            if old_value.get(key) != new_value.get(key):
                diffs.append(f"{section}.{key}: old={old_value.get(key)!r} new={new_value.get(key)!r}")
        return diffs

    old_rows = {stable_json_key(row): row for row in old_value}
    new_rows = {stable_json_key(row): row for row in new_value}
    diffs = []
    for key in sorted(set(old_rows) - set(new_rows)):
        diffs.append(f"{section}: missing in new {old_rows[key]!r}")
    for key in sorted(set(new_rows) - set(old_rows)):
        diffs.append(f"{section}: extra in new {new_rows[key]!r}")
    return diffs


def compatibility_key_from_name(raw_name: Any) -> str | None:
    if not isinstance(raw_name, str):
        return None
    name = strip_legacy_tag(raw_name)
    return f"legacy_name:{name}" if name else None


def strip_legacy_tag(raw_name: str) -> str:
    result = []
    in_tag = False
    for char in raw_name.strip():
        if char == "[":
            in_tag = True
            continue
        if char == "]" and in_tag:
            in_tag = False
            continue
        if not in_tag:
            result.append(char)
    return "".join(result).strip()


def sort_mapping(mapping: dict[str, Any]) -> dict[str, Any]:
    return {key: mapping[key] for key in sorted(mapping)}


def stable_json_key(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def build_summary(
    samples: list[ReplaySample],
    new_results: dict[str, dict[str, Any]],
    old_results: dict[str, dict[str, Any]],
    comparison_results: dict[str, dict[str, Any]],
    stats_results: dict[str, dict[str, Any]],
    edge_window: int,
    per_edge: int,
) -> dict[str, Any]:
    by_category = Counter()
    by_stats_status = Counter()
    by_game_type = Counter(sample.game_type for sample in samples)
    by_year = Counter(f"{sample.game_type}:{sample.year}" for sample in samples)
    failing_samples = []
    stats_mismatch_samples = []

    for sample in samples:
        comparison = comparison_results.get(sample.sample_id, {})
        categories = comparison.get("summary", {}).get("by_category", {})
        by_category.update(categories)
        non_compatible = {key: value for key, value in categories.items() if key != "compatible" and value}
        if not comparison.get("ok") or non_compatible:
            failing_samples.append(
                {
                    "sample_id": sample.sample_id,
                    "game_type": sample.game_type,
                    "year": sample.year,
                    "bucket": sample.bucket,
                    "filename": sample.filename,
                    "date": sample.date,
                    "new_ok": new_results.get(sample.sample_id, {}).get("ok", False),
                    "old_ok": old_results.get(sample.sample_id, {}).get("ok", False),
                    "comparison_ok": comparison.get("ok", False),
                    "categories": categories,
                    "report": comparison.get("report"),
                }
            )

        stats_result_for_sample = stats_results.get(sample.sample_id, {})
        stats_status = stats_result_for_sample.get("status", "missing")
        by_stats_status[stats_status] += 1
        if stats_status != "match":
            stats_mismatch_samples.append(
                {
                    "sample_id": sample.sample_id,
                    "game_type": sample.game_type,
                    "year": sample.year,
                    "bucket": sample.bucket,
                    "filename": sample.filename,
                    "date": sample.date,
                    "status": stats_status,
                    "diff_count": stats_result_for_sample.get("diff_count", 0),
                    "diffs": stats_result_for_sample.get("diffs", [])[:10],
                    "report": stats_result_for_sample.get("report"),
                }
            )

    result = {
        "selected_replays": len(samples),
        "new_successes": sum(1 for item in new_results.values() if item.get("ok")),
        "old_successes": sum(1 for item in old_results.values() if item.get("ok")),
        "comparisons_run": sum(1 for item in comparison_results.values() if item.get("ok")),
        "json_surface_failed_comparisons": len(failing_samples),
        "stats_only_matches": by_stats_status.get("match", 0),
        "stats_only_mismatches": by_stats_status.get("mismatch", 0),
        "stats_only_old_skipped": by_stats_status.get("old_skipped", 0),
        "stats_only_new_failed": by_stats_status.get("new_failed", 0),
        "all_comparable_statistics_compatible": by_stats_status.get("mismatch", 0) == 0
        and by_stats_status.get("new_failed", 0) == 0,
    }

    return {
        "selection_policy": {
            "game_types": list(GAME_TYPES),
            "edge_window": edge_window,
            "per_edge": per_edge,
            "random_seed": "game_type:year:bucket:260502-k2u",
            "dedupe": "filename",
        },
        "result": result,
        "counts": {
            "by_game_type": dict(sorted(by_game_type.items())),
            "by_game_type_year": dict(sorted(by_year.items())),
            "comparison_findings_by_category": dict(sorted(by_category.items())),
            "stats_only_by_status": dict(sorted(by_stats_status.items())),
        },
        "failing_samples": failing_samples,
        "stats_mismatch_samples": stats_mismatch_samples,
    }


def write_markdown(path: pathlib.Path, summary: dict[str, Any]) -> None:
    result = summary["result"]
    lines = [
        "# Old/New Year-Edge Replay Parity",
        "",
        "## Result",
        "",
        f"- Selected replays: {result['selected_replays']}",
        f"- New parser successes: {result['new_successes']}",
        f"- Old parser successes: {result['old_successes']}",
        f"- Comparisons run: {result['comparisons_run']}",
        f"- JSON-surface failed/non-compatible comparisons: {result['json_surface_failed_comparisons']}",
        f"- Stats-only matches: {result['stats_only_matches']}",
        f"- Stats-only mismatches: {result['stats_only_mismatches']}",
        f"- Stats-only old skipped: {result['stats_only_old_skipped']}",
        f"- Stats-only new failed: {result['stats_only_new_failed']}",
        f"- All comparable statistics compatible: {str(result['all_comparable_statistics_compatible']).lower()}",
        "",
        "## Findings By Category",
        "",
    ]
    for category, count in summary["counts"]["comparison_findings_by_category"].items():
        lines.append(f"- {category}: {count}")
    if not summary["counts"]["comparison_findings_by_category"]:
        lines.append("- none")
    lines.extend(["", "## Failing Samples", ""])
    if summary["failing_samples"]:
        for sample in summary["failing_samples"]:
            lines.append(
                f"- {sample['sample_id']} {sample['filename']} "
                f"({sample['game_type']} {sample['year']} {sample['bucket']}): {sample['categories']}"
            )
    else:
        lines.append("- none")
    lines.extend(["", "## Stats-Only Mismatches", ""])
    if summary["stats_mismatch_samples"]:
        for sample in summary["stats_mismatch_samples"]:
            lines.append(
                f"- {sample['sample_id']} {sample['filename']} "
                f"({sample['game_type']} {sample['year']} {sample['bucket']}): "
                f"{sample['status']} diffs={sample['diff_count']}"
            )
    else:
        lines.append("- none")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def link_or_copy(source: pathlib.Path, target: pathlib.Path) -> None:
    if target.exists() or target.is_symlink():
        return
    try:
        target.symlink_to(source, target_is_directory=source.is_dir())
    except OSError:
        copy_tree(source, target)


def copy_tree(source: pathlib.Path, target: pathlib.Path) -> None:
    if target.exists():
        shutil.rmtree(target)
    shutil.copytree(source, target)


def write_json(path: pathlib.Path, payload: Any) -> None:
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


if __name__ == "__main__":
    sys.exit(main())
