---
phase: 01
artifact: corpus-manifest
status: ready
---

# Corpus Manifest

## Summary

Phase 1 profiled the current full-history `~/sg_stats` reference data from local files. The committed manifest summarizes the evidence; the full generated profile remains ignored at `.planning/generated/phase-01/corpus-profiles/20260425T081026Z-corpus-profile/corpus-profile.json`.

Key facts:

- 23,473 raw replay JSON files
- 23,456 replay-list rows
- preparedAt 2026-04-25T04:42:54.889Z
- 88,485 result files
- 14 year_results files
- 17 raw basenames not present in replaysList.json
- 0 listed filenames missing from raw_replays

## Raw Replay Corpus

`~/sg_stats/raw_replays` currently contains 23,473 raw replay JSON files. The profiler parsed 23,469 files successfully and recorded 4 malformed files instead of aborting the run.

## Replay List Metadata

`~/sg_stats/lists/replaysList.json` contains 23,456 entries in both `replays` and `parsedReplays`. The replay list was prepared at 2026-04-25T04:42:54.889Z.

## Raw/List Discrepancies

The full profile found 17 raw basenames not present in replaysList.json and 0 listed filenames missing from raw_replays.

This discrepancy matters because the old parser selects from replay-list metadata; raw-not-listed files are corpus evidence but are not selected by the normal legacy baseline run.

## Existing Results

`~/sg_stats/results` currently contains 88,485 result files. These are historical golden/parity evidence, not production import data.

Plan 01-01 compared this current result tree against regenerated old-parser outputs and classified the current-vs-regenerated differences as `human review`.

## Yearly Reference Outputs

`~/sg_stats/year_results` currently contains 14 year_results files. These outputs are annual nomination references only and remain v2-deferred; they are not ordinary v1 player, squad, rotation, weekly, bounty, or parser contract support.

## Schema And Shape Profile

Observed top-level keys across valid parsed replay JSON files:

| Key | Count |
|-----|-------|
| `captureDelay` | 23,469 |
| `endFrame` | 23,469 |
| `entities` | 23,469 |
| `events` | 23,469 |
| `Markers` | 23,469 |
| `missionAuthor` | 23,469 |
| `missionName` | 23,469 |
| `playersCount` | 23,469 |
| `worldName` | 23,469 |
| `EditorMarkers` | 23,427 |

Dominant event shape samples:

| Shape | Count |
|-------|-------|
| `array(5):number,string,number,array:2,number` | 492,922 |
| `array(4):number,string,string,number` | 337,292 |
| `array(4):number,string,number,array:2` | 46,603 |
| `array(3):number,string,string` | 36,393 |
| `array(3):number,string,array:2` | 18,615 |
| `array(5):number,string,number,array:1,number` | 16,182 |

Dominant entity shape samples:

| Shape | Count |
|-------|-------|
| `object:description,framesFired,group,id,isPlayer,name,positions,side,startFrameNum,type` | 1,011,874 |
| `object:class,framesFired,id,name,positions,startFrameNum,type` | 120,585 |
| `object:class,description,framesFired,group,id,isPlayer,name,positions,side,startFrameNum,type` | 14,587 |
| `object:_class,description,framesFired,group,id,isPlayer,name,positions,side,startFrameNum,type` | 200 |

## Largest Files

| Source file | Size bytes |
|-------------|------------|
| `2021_10_31__00_13_51_ocap.json` | 19,706,937 |
| `2025_04_05__23_27_21__1_ocap.json` | 18,902,626 |
| `2025_04_05__23_20_01__2_ocap.json` | 18,615,410 |
| `2021_06_18__23_22_43_ocap.json` | 17,658,906 |
| `2020_12_13__21_37_41_ocap.json` | 17,654,469 |
| `2021_09_18__23_22_16_ocap.json` | 17,505,284 |
| `2025_05_03__23_19_32__1_ocap.json` | 17,398,621 |
| `2021_10_29__23_56_54_ocap.json` | 17,240,091 |
| `2024_02_24__23_17_55__1_ocap.json` | 17,231,039 |
| `2025_04_26__23_10_46__1_ocap.json` | 17,216,704 |

## Malformed Files

The profiler recorded 4 malformed raw files:

| Source file | Error |
|-------------|-------|
| `.json` | Unexpected HTML document token |
| `2020_12_25__20_08_44_ocap.json` | Unexpected Cyrillic token inside JSON text |
| `2024_07_16__19_13_00__1_ocap.json` | Unexpected HTML document token |
| `2025_03_04__23_06_34__1_ocap.json` | Unexpected end of JSON input |

The old parser full-run logs in Plan 01-01 observed the same three named malformed replay files that are selected by replay-list metadata. The raw file named `.json` is malformed corpus evidence but is not selected by the replay-list baseline run.

## Game-Type Distribution

Replay-list mission prefix distribution:

| Prefix bucket | Count |
|---------------|-------|
| `sg` | 2,052 |
| `mace` | 20,702 |
| `sm` | 243 |
| `sgs` | 1 |
| `other` | 458 |

## Generated Artifacts

Full generated profile:

```text
.planning/generated/phase-01/corpus-profiles/20260425T081026Z-corpus-profile/corpus-profile.json
```

The generated profile includes the full `raw_not_listed` list, top 10 largest files, top-level key counts, event shape summaries, entity shape summaries, malformed files, and game-type distribution.

## Fixture Selection Rationale

`fixture-index.json` is a compact fixture seed list derived from the generated profile. It intentionally covers:

- largest-file behavior
- `sg`, `mace`, `sm`, `sgs`, and `other` mission-prefix buckets
- raw-not-listed corpus discrepancy behavior
- malformed replay behavior

The fixture index is not the final golden fixture set; it is Phase 1 evidence for Phase 5 fixture curation.
