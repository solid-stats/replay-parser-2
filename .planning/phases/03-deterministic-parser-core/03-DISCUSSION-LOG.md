# Phase 3: Deterministic Parser Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-26T06:39:00Z
**Phase:** 03-deterministic-parser-core
**Areas discussed:** Parser core boundary, OCAP decode and schema drift, determinism, observed entity normalization, legacy compatibility hooks, tests and fixtures

---

## Parser Core Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Pure core crate | Parser-core accepts bytes/source metadata and returns parser-contract types; adapters come later. | ✓ |
| Mixed CLI/core | Implement parser logic with CLI entrypoints in the same phase. | |
| Worker-aware core | Include queue/storage concerns now. | |

**User's choice:** Auto-selected recommended default.  
**Notes:** Keeps Phase 3 inside roadmap scope and preserves Phase 5/6 adapter boundaries.

---

## OCAP Decode and Schema Drift

| Option | Description | Selected |
|--------|-------------|----------|
| Tolerant serde_json adapters | Decode with `serde_json`, isolate raw OCAP quirks, emit unknowns/diagnostics/failures instead of panics. | ✓ |
| Strict typed DTOs only | Fail quickly on every unexpected source shape. | |
| Performance-first JSON engine | Introduce alternative parsing engine before behavior is proven. | |

**User's choice:** Auto-selected recommended default.  
**Notes:** Matches research guidance: correctness and corpus compatibility before optimization.

---

## Determinism

| Option | Description | Selected |
|--------|-------------|----------|
| Stable ordering everywhere observable | Use sorted collections/BTreeMap and avoid wall-clock output in parser-core. | ✓ |
| Stabilize only final serialization | Allow intermediate nondeterminism and sort only near output. | |
| Leave ordering for Phase 5 | Defer determinism until CLI/golden tests. | |

**User's choice:** Auto-selected recommended default.  
**Notes:** OUT-08 belongs to Phase 3, so determinism must be designed into the core API now.

---

## Observed Entity Normalization

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve raw observed facts | Normalize units/players, vehicles, and static weapons without canonical matching or premature aggregate merging. | ✓ |
| Collapse compatibility identities early | Merge duplicate names or slots directly in normalized observations. | |
| Defer entities until event phase | Parse only metadata in Phase 3. | |

**User's choice:** Auto-selected recommended default.  
**Notes:** Preserves parser/server identity boundaries and keeps aggregate compatibility as a later projection concern.

---

## Legacy Compatibility Hooks

| Option | Description | Selected |
|--------|-------------|----------|
| Capture hooks and evidence | Represent connected-player backfill and duplicate-slot compatibility as explicit metadata/diagnostics/hooks without collapsing raw facts. | ✓ |
| Implement aggregate merge behavior now | Apply legacy aggregate merge rules directly in parser-core. | |
| Ignore until Phase 4 | Leave no Phase 3 surface for known compatibility traps. | |

**User's choice:** Auto-selected recommended default.  
**Notes:** Satisfies PARS-06/PARS-07 while avoiding Phase 4 aggregate scope creep.

---

## Tests and Fixtures

| Option | Description | Selected |
|--------|-------------|----------|
| Focused behavior fixtures | Use small deterministic fixtures derived from Phase 1 corpus shapes; full corpus/golden parity waits for Phase 5. | ✓ |
| Full corpus now | Build broad old-vs-new comparison during parser-core foundation. | |
| Synthetic only | Avoid historical shape evidence until later. | |

**User's choice:** Auto-selected recommended default.  
**Notes:** Keeps tests RITE/AAA and avoids pulling Phase 5 harness work into Phase 3.

---

## the agent's Discretion

- Exact parser-core crate/module naming.
- Raw OCAP adapter type layout.
- Fixture directory structure.
- Diagnostic code names, as long as they remain stable and namespaced.

## Deferred Ideas

- CLI parse/schema/compare commands.
- Full corpus parity sweeps.
- Event semantics and aggregate formulas.
- Worker queue/storage handling.
