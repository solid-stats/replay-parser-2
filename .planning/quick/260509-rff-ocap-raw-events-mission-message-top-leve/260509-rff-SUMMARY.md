---
quick_id: 260509-rff
date: 2026-05-09
status: verified
---

# Summary

Parser-core now reads OCAP raw `mission_message` event tuples and derives KS
commander side facts from messages shaped as:

```json
[frame, "mission_message", "Победа КС: ... Поражение КС: ..."]
```

Changes:

- Added `MissionMessageEventObservation` and included mission messages in the
  single-pass compact relevant-event scan.
- Added `raw::mission_message_events`.
- Updated side-fact normalization to parse `Победа КС:` / `Поражение КС:`,
  match listed names to observed replay-local player entities, emit observed
  commander facts, infer winner side, and diagnose cross-side conflicts.
- Updated default success artifacts to include explicit mission-message
  side facts instead of always defaulting them empty.
- Left legacy top-level `winner` / `winningSide` / `outcome` handling only as a
  debug normalization fallback for older tests and possible old sources.

No `server-2` changes were made.

