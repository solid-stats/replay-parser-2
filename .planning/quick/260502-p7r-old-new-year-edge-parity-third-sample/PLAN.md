---
status: complete
quick_id: 260502-p7r
date: 2026-05-02
---

# Quick Task: Old/New Year-Edge Replay Parity Third Sample

Run a third deterministic old-vs-new comparison sample using the same year-edge
policy as `260502-k2u` and `260502-nx9`.

## Constraints

- Use a different deterministic sample seed: `260502-p7r`.
- Keep previous generated evidence unchanged.
- Compare derived statistics rather than raw file shape, because the v3 minimal
  artifact intentionally differs from the old parser output shape.
- Store summary evidence under this quick-task directory and bulky generated
  old/new artifacts under ignored `.planning/generated/quick/`.

## Tasks

- [x] Run the third sample against `sg`, `mace`, and `sm` replay rows from
  `~/sg_stats/lists/replaysList.json`.
- [x] Produce selected replay, comparison detail, and summary files under
  `.planning/generated/quick/260502-p7r-old-new-year-edge-parity-third-sample/`.
- [x] Classify stats-only mismatch rows.
- [x] Record the result in `SUMMARY.md` and `.planning/STATE.md`.
- [x] Commit the quick-task artifacts.
