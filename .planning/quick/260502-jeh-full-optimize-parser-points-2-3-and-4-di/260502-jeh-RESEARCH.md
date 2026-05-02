# Quick Task 260502-jeh Research

## Findings

The current Phase 5.2 parser already uses a borrowed compact OCAP root, but the default success path still constructed full normalized combat events before deriving minimal default rows. That kept debug-only event IDs, source refs, rule IDs, actor refs, bounty metadata, and field-presence wrappers on the ordinary ingestion path.

The event accessors also scanned `$.events` separately for connected-player backfill and killed-event semantics. For default parsing, these can be collected in one pass because both surfaces read the same top-level event tuple stream.

Vehicle context lookup was replay-local and deterministic, but the normalized path found attacker vehicles by scanning entity values for every weapon-bearing killed event. A per-replay `vehicle/static observed_name -> entity` index preserves deterministic behavior while avoiding repeated linear lookup.

The user explicitly asked not to rerun the old all-raw baseline. The latest reliable old all-raw evidence is:

- `old_wall_time_ms=501274.528655`
- `attempted_count=23473`
- `success_count=23469`
- `error_count=4`
- `skipped_count=0`

## Approach

1. Add a one-pass relevant-event collector that returns connected and killed observations together.
2. Keep `parse_replay_debug` on the existing full normalized path.
3. Change default `parse_replay` to normalize metadata/entities, then derive minimal tables directly from killed observations.
4. Build the weapon dictionary and vehicle/static lookup from direct minimal classification, preserving sorted dictionary IDs and merged player rows.
5. Cache the prior old all-raw baseline in committed planning evidence and teach the benchmark script to reuse it by default.

## Risks

- Direct minimal classification must preserve the old normalized-event semantics for enemy kills, teamkills, suicides, null-killer deaths, unknown player deaths, and destroyed vehicles.
- Diagnostics still need to mark malformed or unclassifiable killed tuples as partial where existing tests require it.
- Reusing the old all-raw baseline is valid only when the raw corpus and old-run policy remain the same as `260502-i8w`.
