---
quick_id: 260509-rff
date: 2026-05-09
---

# Research

Fresh raw replay samples under `~/sg_stats/raw_replays` do not have top-level
`winner`, `winningSide`, or `outcome` fields for this feature. The commander
victory data appears in event tuples:

- `[902, "mission_message", "Победа КС: [SHK]Sota. Поражение КС: [JTF2]Bas"]`
- `[54, "mission_message", "Победа КС: [RAF]Valar, Rollan. Поражение КС: [RT]Raiden, [RAF]baptized"]`
- `[87, "mission_message", "Победа КС: [31st]Flori. Поражение КС: "]`

Implementation boundary:

- Extend raw event compaction/accessors to expose `mission_message` events.
- Parse the Russian labels `Победа КС:` and `Поражение КС:` in parser-core.
- Match listed names against observed replay player names only.
- Emit `CommanderFactKind::Observed` for matched entities.
- Infer winner side only when all matched winning KS names resolve to exactly
  one non-unknown side.
- Keep outcome unknown and emit a conflict diagnostic if matched winning KS
  names resolve to multiple sides.
- Do not fabricate commander facts for unmatched names.

