---
quick_id: 260509-rff
mode: quick-full
date: 2026-05-09
---

# Context

User reported that SolidGames commander-side victory detection was looking for
nonexistent top-level raw replay fields such as `winner`, `winningSide`, or
`outcome`, while the newly added source data exists in OCAP raw `events`.

The relevant raw OCAP event shape observed in `~/sg_stats/raw_replays` is:

```json
[902, "mission_message", "Победа КС: [SHK]Sota. Поражение КС: [JTF2]Bas"]
```

Scope is parser-only. Do not change `server-2`, canonical identity, persistence,
API, UI, or replay fetching. The parser may emit observed replay-local evidence
only.

