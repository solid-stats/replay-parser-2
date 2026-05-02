---
status: complete
quick_id: 260502-nx9
date: 2026-05-02
---

# Quick Task: Old/New Year-Edge Replay Parity Second Sample

Run a second deterministic old-vs-new comparison sample using the same policy as
`260502-k2u`: for each supported game type and year, take up to two sampled
replays from the beginning of that year's replay list and up to two from the end
of that year's replay list.

## Constraints

- Use a different deterministic sample seed: `260502-nx9`.
- Keep the first `260502-k2u` generated evidence unchanged.
- Compare derived statistics rather than raw file shape, because the v3 minimal
  artifact intentionally differs from the old parser output shape.
- Store summary evidence under this quick-task directory and bulky generated
  old/new artifacts under ignored `.planning/generated/quick/`.

## Tasks

- [x] Parameterize the year-edge comparison runner with `--sample-seed`.
- [x] Run the second sample against `sg`, `mace`, and `sm` replay rows from
  `~/sg_stats/lists/replaysList.json`.
- [x] Produce selected replay, comparison detail, and summary files under
  `.planning/generated/quick/260502-nx9-old-new-year-edge-parity-second-sample/`.
- [x] Record the result in `SUMMARY.md` and `.planning/STATE.md`.
- [x] Commit the quick-task artifacts and runner change.
