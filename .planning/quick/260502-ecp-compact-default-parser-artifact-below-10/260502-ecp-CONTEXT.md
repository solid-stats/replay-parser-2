# Quick Task 260502-ecp: Compact default parser artifact below 100 KB using merged player stats, numeric refs, weapon dictionary, omitted empty fields, and debug-only verbose evidence - Context

**Gathered:** 2026-05-02
**Status:** Ready for planning

<domain>
## Task Boundary

Reduce the default parser artifact size so the selected large replay no longer exceeds the 100,000 byte hard limit. The change should target the default server-facing artifact, not raw OCAP files or debug sidecars.

The parser must stay within `replay-parser-2` ownership: no canonical player matching, no PostgreSQL persistence, no public API/UI behavior, and no replay discovery/fetching.

</domain>

<decisions>
## Implementation Decisions

### Players and Stats

- Merge `player_stats[]` counters into `players[]` rows.
- Use numeric `source_entity_id`/`eid` as the replay-local player reference.
- Omit zero counters instead of serializing them explicitly.

### Event References

- In `kills[]` and `destroyed_vehicles[]`, replace repeated names and side values with player/entity ID references.
- Keep verbose identity/event evidence in debug sidecar output only.
- Default bounty eligibility is computed downstream from kill classification; omit `bounty_eligible` and `bounty_exclusion_reasons` from the default artifact.

### Vehicles

- Default event rows keep vehicle class plus source entity ID where relevant.
- Vehicle names are debug-only.
- Do not remove ordinary `vehicleKills` and `killsFromVehicle` counters.

### Weapons

- Add a weapon dictionary and reference weapons by compact IDs in event rows.

### Empty Fields

- Default JSON omits `null`, empty arrays, and zero counters.

### Row Encoding

- Use short schema-backed keys for compact default JSON.
- Avoid tuple-array rows unless the object form still fails the selected replay hard limit.

</decisions>

<specifics>
## Size Evidence From Discussion

- Current selected large artifact: `203683` bytes.
- Omitting only `null` and empty arrays: about `173 KB`, not sufficient.
- Merging player stats, omitting zero counters, and using ID-linked events: about `91 KB`.
- With weapon dictionary and bounty-field removal: about `86 KB` before short-key savings.

</specifics>

<canonical_refs>
## Canonical References

- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/generated/phase-05/benchmarks/selected-large-artifact.json`
- `crates/parser-contract/src/minimal.rs`
- `crates/parser-core/src/aggregates.rs`

</canonical_refs>
