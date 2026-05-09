---
quick_id: 260509-rff
mode: quick-full
date: 2026-05-09
---

# Plan

1. Add compact raw extraction for `[frame, "mission_message", message]` tuples.
2. Re-export a parser-core raw accessor for mission-message events.
3. Update side-fact normalization to consume mission-message KS outcome data
   before legacy debug-only top-level outcome fallbacks.
4. Wire default success artifacts to emit mission-message side facts.
5. Add tests for raw accessor behavior, inferred default artifact output,
   duplicate messages, conflicting winner sides, and unmatched names.
6. Verify with fmt, clippy, workspace tests, docs, and a real raw replay parse.

