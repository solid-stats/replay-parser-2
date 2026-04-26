# Phase 3: Deterministic Parser Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-04-26T14:43:32+07:00
**Phase:** 03-deterministic-parser-core
**Areas discussed:** Schema drift policy, entity normalization depth, legacy compatibility hooks

---

## Schema Drift Policy

### Severity policy

| Option | Description | Selected |
|--------|-------------|----------|
| Strict root, tolerant fields | Invalid JSON or unusable root returns `ParseFailure`; localized malformed/missing fields become diagnostics, unknowns, or partial. | yes |
| Mostly partial | Try to return a partial artifact almost always, even when root shape is heavily damaged. | |
| Strict contract | Fail on most unexpected shape mismatches to catch drift quickly. | |

**User's choice:** Strict root, tolerant fields.  
**Notes:** Use `serde_json` error categories and path-aware deserialization to separate hard decode/root failures from local drift.

### Minimum useful result

| Option | Description | Selected |
|--------|-------------|----------|
| Metadata plus best-effort entities | Success/partial is possible when metadata is extracted; entity drift becomes diagnostics. | yes |
| Require entities | If `entities` cannot be read as a usable collection, return failed or skipped even with metadata. | |
| No minimum | Return an artifact with unknowns/diagnostics almost always if JSON is readable. | |

**User's choice:** Metadata plus best-effort entities.  
**Notes:** Phase 3 success criteria require metadata and observed entity facts, but local entity drift should not erase usable metadata.

### Diagnostic granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Path-based with cap | Record `json_path`, expected/observed shape, and action for specific problems, with limits and summaries for repeated issues. | yes |
| Section summary | One diagnostic per metadata/entities/events section without path-level details. | |
| Exhaustive | Record every discovered problem without caps. | |

**User's choice:** Path-based with cap.  
**Notes:** Preserves auditability without oversized artifacts.

### Status mapping

| Option | Description | Selected |
|--------|-------------|----------|
| Partial on data-loss warnings | Success is allowed with info/non-loss diagnostics; partial when drift causes unknowns, dropped entities, or compatibility backfill. | yes |
| Any diagnostic partial | Any diagnostic marks the artifact partial. | |
| Warnings stay success | Status stays success until parser-core hard-fails. | |

**User's choice:** Partial on data-loss warnings.  
**Notes:** Later legacy discussion refined this for compatibility hooks: backfill/hints alone do not force partial unless conflict or data loss exists.

---

## Entity Normalization Depth

### Contract gap

| Option | Description | Selected |
|--------|-------------|----------|
| Extend contract now | Add typed observed entity fields in parser-contract in Phase 3 and update schema/tests. | yes |
| Use extensions | Temporarily place class/name in `extensions` or diagnostics without strong typed contract. | |
| Defer to Phase 4 | Leave Phase 3 broad identity/kind only. | |

**User's choice:** Extend contract now.  
**Notes:** Current `ObservedEntity` lacks explicit vehicle/static weapon class/name fields needed for PARS-05.

### Classification depth

| Option | Description | Selected |
|--------|-------------|----------|
| Broad kind plus raw class | Phase 3 determines unit/vehicle/static/unknown and preserves observed class/name; vehicle score taxonomy waits for Phase 4. | yes |
| Early vehicle taxonomy | Classify car/truck/APC/tank/heli/plane/static/player in Phase 3. | |
| Raw class only | Do not classify beyond source class/name except when obvious. | |

**User's choice:** Broad kind plus raw class.  
**Notes:** Avoids pulling vehicle-score semantics from Phase 4 into Phase 3.

### Source refs

| Option | Description | Selected |
|--------|-------------|----------|
| Best-known refs required | Each normalized entity should carry source refs with entity_id/json_path/source file where known; missing refs become diagnostics/partial as appropriate. | yes |
| Allow empty refs | Keep `Vec<SourceRef>` sometimes empty when source coordinates are inconvenient. | |
| Use non-empty type | Change contract to require `SourceRefs` for entity refs at type/schema level. | |

**User's choice:** Best-known refs required.  
**Notes:** Planner can decide whether this needs a type-level contract update or runtime invariant, but Phase 3 should not silently emit unauditable entities.

### Ordering

| Option | Description | Selected |
|--------|-------------|----------|
| Source order + fallback | Preserve OCAP array order; fallback to `source_entity_id` and stable secondary keys for unordered/drifted shapes. | |
| Pure source order | Always preserve source order when possible. | |
| Source ID ascending | Sort normalized entities by `source_entity_id` for maximum diff stability. | yes |

**User's choice:** Source ID ascending.  
**Notes:** The tradeoff was discussed. Source order is closer to OCAP shape, but `source_entity_id` ordering is more stable for old/new diffing and deterministic artifacts.

---

## Legacy Compatibility Hooks

### Connected-player backfill

| Option | Description | Selected |
|--------|-------------|----------|
| Inferred facts | Repeat the old behavior semantically by adding player facts, but make source/rule/provenance explicit. | yes |
| Diagnostics only | Do not add an entity fact; report only a backfill candidate. | |
| Extensions only | Place candidate facts in generic extensions without typed contract. | |

**User's choice:** Inferred facts.  
**Notes:** The old parser adds a player from a `connected` event when a same-ID entity exists, is not a vehicle, and the connected name is present. New parser-core should represent this as inferred observed facts with source refs and rule ID.

### Duplicate-slot same-name merge

| Option | Description | Selected |
|--------|-------------|----------|
| Compatibility hint only | Do not merge normalized entities; add typed hint/diagnostic that these source entity IDs share an observed name for later projection. | yes |
| Merge in Phase 3 | Immediately combine same-name entities into one observed entity like the legacy aggregate result. | |
| Ignore until Phase 4 | Add no Phase 3 surface for this compatibility behavior. | |

**User's choice:** Compatibility hint only.  
**Notes:** Old `combineSamePlayersInfo` merges aggregate `PlayerInfo` records by equal name. Phase 3 should preserve raw observations and leave merge semantics to later aggregate/parity projection.

### Status effect

| Option | Description | Selected |
|--------|-------------|----------|
| Success unless conflict | Backfill/hints do not change status by themselves; partial only on conflict or data loss. | yes |
| Success always | Compatibility facts never affect status unless hard failure occurs. | |
| Partial when applied | Any backfill or same-name hint marks the artifact partial. | |

**User's choice:** Success unless conflict.  
**Notes:** This refines the earlier drift status answer. Compatibility inference is expected legacy behavior, not necessarily data loss.

### Test depth

| Option | Description | Selected |
|--------|-------------|----------|
| Focused fixtures plus legacy references | Small Rust fixtures for backfill/same-name cases, with comments/refs to legacy files; full parity remains Phase 5. | yes |
| Golden samples now | Use real corpus files and compare legacy behavior for hooks during Phase 3. | |
| Unit-only synthetic | Use synthetic unit tests with no legacy references. | |

**User's choice:** Focused fixtures plus legacy references.  
**Notes:** Keeps Phase 3 test scope focused while preserving the old-code rationale.

---

## the agent's Discretion

- Exact parser-core crate/module naming.
- Raw OCAP adapter type layout.
- Diagnostic code names, as long as they are stable and namespaced.
- Fixture directory structure.

## Deferred Ideas

- CLI parse/schema/compare commands.
- Full corpus parity sweeps.
- Event semantics and aggregate formulas.
- Worker queue/storage handling.
