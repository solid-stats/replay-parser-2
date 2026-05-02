# Quick Task 260502-i8w Context

## Request

User requested a full `$gsd-quick --full` remediation for Phase 5.2:

- make the old all-raw baseline cover every raw replay file instead of skipping by game type/empty/mace rules;
- check the x10 all-raw gate;
- merge two slots with the same nickname like the old parser;
- represent legacy tag and nickname separately in relationship/comparison data;
- move player-authored `kills` into `players[]`;
- clean generated benchmark files before reruns and remove useless versioned generated files under `.planning/generated/phase-05`.

## Scope Boundaries

- Parser output contract may change because Phase 5.2 is explicitly about the default parse artifact.
- Canonical identity, database persistence, public APIs, and bounty payout calculation remain `server-2` scope.
- Replay discovery and production raw fetching remain `replays-fetcher` scope.

## Interface Decision

Considered shapes:

1. Keep top-level `kills[]`.
   - Rejected: the user explicitly wants kill rows near the player who authored them for bounty/stat calculation.
2. Store all death rows under victims.
   - Rejected: it optimizes death lookups but makes bounty calculation start from victims instead of killers.
3. Store player-authored enemy/team kill rows under killer `players[].kills`, while suicides/null-killer/unknown deaths remain counters.
   - Chosen: matches bounty workflow, keeps malformed/non-authored deaths out of bounty rows, and keeps the artifact compact.

## Evidence Files

- Full benchmark report: `.planning/generated/phase-05/benchmarks/benchmark-report.json`
- Old all-raw summary: `.planning/generated/phase-05/comparison/old-all-raw-summary.json`
- Selected old/new comparison report: `.planning/generated/phase-05/comparison/comparison-report.json`
