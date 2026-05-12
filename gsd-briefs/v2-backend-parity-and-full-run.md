# replay-parser-2 v2 Milestone Brief: Parser Contract Support for Backend Parity

**Created:** 2026-05-12
**Intended command:** `$gsd-new-milestone --auto @gsd-briefs/v2-backend-parity-and-full-run.md` only if contract support is needed
**Application:** `replay-parser-2`
**Primary role:** conditional support dependency

## Cross-App Briefs

Read these sibling briefs before drafting any parser milestone:

- `/home/afgan0r/Projects/SolidGames/server-2/gsd-briefs/v2-backend-parity-and-full-run.md`
- `/home/afgan0r/Projects/SolidGames/replays-fetcher/gsd-briefs/v2-backend-parity-and-full-run.md`
- `/home/afgan0r/Projects/SolidGames/infrastructure/gsd-briefs/v2-backend-parity-and-full-run.md`
- `/home/afgan0r/Projects/SolidGames/web/gsd-briefs/v2-backend-parity-and-full-run.md`

## Global Sequence

1. `server-2` implements parity foundation and determines whether parser artifact evidence is sufficient.
2. `replays-fetcher` makes full-corpus ingest resumable.
3. `infrastructure` runs the controlled full corpus and captures legacy/new evidence.
4. `web` consumes stable backend contracts after parity is trustworthy.

`replay-parser-2` should not start a broad v2 milestone by default. It should be activated only for contract fixes or additional examples/tests required by `server-2` parity work.

## Goal

Preserve and, if necessary, clarify the parser artifact contract that `server-2` uses for public-stat parity.

The key parser obligation is to keep compact player counters and kill rows deterministic, documented, and safe for backend recalculation.

## Source Evidence

- `.planning/STATE.md`
- `.planning/quick/260502-k2u-old-new-year-edge-parity/SUMMARY.md`
- `.planning/quick/260502-nx9-old-new-year-edge-parity-second-sample/SUMMARY.md`
- `crates/parser-contract/src/minimal.rs`
- `crates/parser-core/tests/combat_event_semantics.rs`
- `crates/parser-core/tests/aggregate_projection.rs`
- `/home/afgan0r/Projects/SolidGames/server-2/gsd-briefs/v2-backend-parity-and-full-run.md`

## Required Decisions Already Made

- `server-2` should use parser compact counters such as `players[].d`, `players[].td`, `players[].su`, `players[].nkd`, and `players[].ud` as replay-level counter evidence.
- `players[].kills[]` remains necessary for kill relationships, bounty inputs, and weapon/vehicle context, but it should not be the only death source in `server-2`.
- Public legacy export should normalize parser-level differences that are not intended as public product differences.
- The only expected public-data difference class is documented `deaths.byTeamkills` behavior for duplicate-slot/respawn teamkill-death edge cases.

## Current Contract Notes

The compact v3 artifact already exposes:

- `players[].d`: death counter.
- `players[].td`: teamkill-death marker.
- `players[].tk`: teamkill counter.
- `players[].su`: suicide counter.
- `players[].nkd`: null-killer death counter.
- `players[].ud`: unknown death counter.
- `players[].vk`: vehicle kill counter.
- `players[].kfv`: kills from vehicle.
- `players[].kills[]`: player-authored kill rows with classification, victim, weapon, and vehicle context.

Existing parser parity evidence already documents retained weapon-name differences and duplicate-slot/respawn teamkill-death behavior. Do not erase these decisions without an explicit product reversal.

## Conditional Milestone Triggers

Create a parser milestone only if `server-2` parity work finds one of these blockers:

- A compact counter needed by `server-2` is missing, ambiguous, or not schema-documented.
- Existing examples do not cover a backend-critical death/teamkill/null-killer/unknown-death case.
- The parser schema version or worker contract must change for `server-2` recalculation.
- The diff harness needs a parser-provided diagnostic or source reference that is not currently emitted.

## Suggested Conditional Phases

### Phase 1: Contract Clarification

Goal: make the compact counter contract unambiguous for backend consumers.

Acceptance criteria:

- Parser docs and schema examples explain `d`, `td`, `su`, `nkd`, `ud`, `tk`, `vk`, `kfv`, and `kills[]`.
- Examples include teamkill death, later non-teamkill death after teamkill, suicide, null-killer death, and unknown actor death.
- `server-2` can map these fields without inference from Rust internals.

### Phase 2: Backend Compatibility Fixtures

Goal: lock the behavior that `server-2` parity depends on.

Acceptance criteria:

- Add fixtures or examples selected with `server-2` that cover the backend parity edge cases.
- Keep old-vs-new parser parity evidence attached to the fixture or planning note.
- Verify schema export and parser-core tests after changes.

## Dependencies On Other Apps

- `server-2` owns the first decision on whether parser changes are needed.
- `replays-fetcher` depends only on raw object and checksum compatibility, not parser internals.
- `infrastructure` should pin parser images but should not define parser semantics.
- `web` consumes parser-derived data only through `server-2` APIs.

## Non-Goals

- Do not rebuild the legacy TypeScript parser.
- Do not make parser output responsible for canonical player identity or rotations.
- Do not add production database writes to the parser.
- Do not implement public diff tooling here unless `server-2` explicitly requires parser-side comparison support.

## Recommended Next Command

Do not start this milestone first. Start `server-2` first, then open this parser milestone only for a concrete contract-support blocker.
