---
status: complete
quick_id: 260502-v8n
date: 2026-05-02
---

# Quick Task: Old/New Year-Edge Replay Parity Fifth Sample

Run a fifth deterministic old-vs-new comparison sample using the same year-edge
policy as the prior parity samples.

## Constraints

- Use deterministic sample seed `260502-v8n`.
- Keep previous generated evidence unchanged.
- Compare derived statistics rather than raw file shape, because the v3 minimal
  artifact intentionally differs from the old parser output shape.
- Store summary evidence under this quick-task directory and bulky generated
  old/new artifacts under ignored `.planning/generated/quick/`.

## Tasks

- [x] Run the fifth sample against `sg`, `mace`, and `sm` replay rows from
  `~/sg_stats/lists/replaysList.json`.
- [x] Classify stats-only mismatch rows.
- [x] Record lightweight evidence and update project state.
- [x] Commit the quick-task artifacts.
