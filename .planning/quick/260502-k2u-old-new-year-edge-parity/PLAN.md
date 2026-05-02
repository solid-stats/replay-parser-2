---
status: complete
quick_id: 260502-k2u
date: 2026-05-02
---

# Quick Task: Old/New Year-Edge Replay Parity

Compare the new Rust parser against the old TypeScript parser on a deterministic
sample from `~/sg_stats`: for each supported game type and year, take up to two
sampled replays from the beginning of that year's replay list and up to two from
the end of that year's replay list.

## Constraints

- Use the old parser at `/home/afgan0r/Projects/SolidGames/replays-parser` as the
  behavioral reference.
- Keep replay discovery, server persistence, canonical identity, and web behavior
  out of scope.
- Compare derived statistics rather than raw file shape, because the v3 minimal
  artifact intentionally differs from the old parser output shape.
- Store summary evidence under this quick-task directory and bulky generated
  old/new artifacts under ignored `.planning/generated/quick/`.

## Tasks

- [x] Add a deterministic comparison runner for the requested year-edge sample.
- [x] Run the runner against `sg`, `mace`, and `sm` replay rows from
  `~/sg_stats/lists/replaysList.json`.
- [x] Produce machine-readable selected replay, comparison detail, and summary files
  under `.planning/generated/quick/260502-k2u-old-new-year-edge-parity/`.
- [x] Record the result in `SUMMARY.md` and `.planning/STATE.md`.
- [x] Commit the quick-task artifacts and runner.
