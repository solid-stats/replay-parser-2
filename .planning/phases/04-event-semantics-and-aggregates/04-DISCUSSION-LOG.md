# Phase 4: Event Semantics and Aggregates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-04-27
**Phase:** 04-event-semantics-and-aggregates
**Areas discussed:** Combat event classification, Aggregate artifact shape, Vehicle score evidence, Commander/outcome confidence

---

## Combat Event Classification

### Source `killed` Tuple Modeling

| Option | Description | Selected |
|--------|-------------|----------|
| Primary event | One source-backed event with primary classification and attributes; aggregate contributions reveal counter effects without event duplication. | yes |
| Multiple events | One tuple can emit kill/death/teamkill/relationship events separately; more explicit but harder to audit/order. | |
| Agent decides | Planner chooses while preserving legacy parity and source refs. | |

**User's choice:** Primary event.
**Notes:** Current event contract has a generic event skeleton but no combat payload yet.

### Legacy Edge Cases

| Option | Description | Selected |
|--------|-------------|----------|
| Legacy counters | Preserve old parser behavior for null killer, suicide, and same-side teamkill counters. | yes |
| Strict semantics | Split suicide/teamkill/death more aggressively, with separate compatibility projection for legacy counters. | |
| Agent decides | Planner chooses but must prove legacy counter parity. | |

**User's choice:** Legacy counters.
**Notes:** Null-killer player deaths, suicide, and same-side kills follow old `getKillsAndDeaths.ts` counter behavior.

### Unknown Actor Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Unknown plus diagnostic | Preserve source-backed unknown/partial event and diagnostics; emit aggregate contribution only with enough evidence. | yes |
| Legacy drop | Ignore cases where old parser did not mutate counters. | |
| Agent decides | Planner chooses while preserving source refs and mismatch taxonomy. | |

**User's choice:** Unknown plus diagnostic.
**Notes:** Auditability wins over silent legacy drops when the parser has source evidence.

### Bounty Eligibility

| Option | Description | Selected |
|--------|-------------|----------|
| Eligible plus reasons | Enemy kills emit `BountyInput`; excluded cases remain auditable with exclusion reason and no awarding input. | yes |
| Only eligible | Emit only valid bounty inputs; excluded cases are visible only as events/diagnostics. | |
| Agent decides | Planner chooses, but teamkills must never award bounty points. | |

**User's choice:** Eligible plus reasons.
**Notes:** This keeps teamkills/suicides/null-killer/unknown cases auditable without awarding bounty points.

---

## Aggregate Artifact Shape

### Aggregate Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Per-replay plus refs | Atomic contributions plus per-replay player/relationship projections; cross-replay outputs are derived later. | yes |
| Full legacy shapes | Parser tries to emit legacy global/squad/week/rotation shapes directly. | |
| Agent decides | Planner chooses while preserving `server-2` ownership. | |

**User's choice:** Per-replay plus refs.
**Notes:** Full old output surfaces are multi-replay and belong to `server-2` or Phase 5 comparison.

### Projection Names

| Option | Description | Selected |
|--------|-------------|----------|
| Namespaced | Use keys such as `legacy.player_game_results`, `legacy.relationships`, `bounty.inputs`, and `vehicle_score.inputs`. | yes |
| Mirror old JSON | Repeat old JSON field names and structure without namespacing. | |
| Agent decides | Planner chooses while keeping schema/order/source refs stable. | |

**User's choice:** Namespaced.
**Notes:** Legacy field names remain inside compatibility projections.

### Counter Traceability

| Option | Description | Selected |
|--------|-------------|----------|
| No silent counters | Every counter is derived from traceable contributions; otherwise diagnostic/partial and no contribution. | yes |
| Best-effort counters | Allow counters with warnings even when audit trail is incomplete. | |
| Agent decides | Planner chooses with required mismatch impact notes. | |

**User's choice:** No silent counters.
**Notes:** Aggregate outputs must be recalculable from events/contributions/source refs.

### Aggregate Identity

| Option | Description | Selected |
|--------|-------------|----------|
| Dual identity | Contributions hold observed entity IDs; legacy projections may apply same-name compatibility with provenance. | yes |
| Observed only | No same-name merge in parser projections; all compatibility identity stays for later phases. | |
| Legacy merge | Merge projection records by name as close to old parser as possible. | |

**User's choice:** Dual identity.
**Notes:** No canonical player IDs are introduced in parser output.

---

## Vehicle Score Evidence

### Vehicle Taxonomy

| Option | Description | Selected |
|--------|-------------|----------|
| Raw plus mapped | Preserve raw class/name and mapped issue #13 category with rule/confidence; unknown has diagnostic. | yes |
| Mapped only | Store only issue #13 categories. | |
| Agent decides | Planner chooses with source refs and auditability. | |

**User's choice:** Raw plus mapped.
**Notes:** Current Phase 3 entities preserve raw observed class/name but no issue #13 taxonomy yet.

### Weighted Contributions

| Option | Description | Selected |
|--------|-------------|----------|
| Per event | Each eligible vehicle kill/teamkill emits contribution with player, target category, evidence, weight, sign, and source refs. | yes |
| Totals only | Store only per-player weighted totals. | |
| Agent decides | Planner chooses if score can be recalculated from artifact. | |

**User's choice:** Per event.
**Notes:** This directly supports `server-2` audit/recalculation.

### Denominator

| Option | Description | Selected |
|--------|-------------|----------|
| Denominator input | Emit per-player per-replay denominator eligibility; server/harness computes cross-replay final score. | yes |
| Final per replay | Parser writes a final per-replay score using current replay as denominator. | |
| Agent decides | Planner chooses, but issue #13 formula must be reconstructable. | |

**User's choice:** Denominator input.
**Notes:** Parser handles single artifacts; cross-replay score ownership stays downstream.

### Penalty Clamp

| Option | Description | Selected |
|--------|-------------|----------|
| In contribution | Store raw matrix weight and applied penalty weight after `max(matrix_weight, 1)` in each penalty contribution. | yes |
| Final only | Apply clamp only at final formula level without per-event applied weight. | |
| Agent decides | Planner chooses with mandatory tests for weights below 1. | |

**User's choice:** In contribution.
**Notes:** Teamkill penalty clamp must be auditable per source event.

---

## Commander/Outcome Confidence

### Commander Extraction Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Observed plus candidates | Explicit facts are present; heuristic KS candidates are inferred/candidate facts with confidence, rule ID, and source refs. | yes |
| Observed only | Missing explicit KS stays unknown; no candidates. | |
| Aggressive infer | Pick best-guess commander per side to fill stats more often. | |

**User's choice:** Observed plus candidates.
**Notes:** Candidates are not canonical truth.

### Winner Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Known/unknown | Emit present only from reliable source evidence; otherwise explicit unknown. Manual/final winner is `server-2`. | yes |
| Infer winner | Allow inferred winner candidates from indirect evidence. | |
| Agent decides | Planner chooses if legacy missing winner remains representable as unknown. | |

**User's choice:** Known/unknown.
**Notes:** `server-2` owns manual outcome correction for old data.

### Artifact Location

| Option | Description | Selected |
|--------|-------------|----------|
| Typed section | Extend parser-contract with typed replay-side commander/outcome facts; projections may consume that section. | yes |
| Aggregates only | Store commander/outcome only as aggregate projection. | |
| Extensions first | Put the data in `extensions` first to avoid larger contract changes. | |

**User's choice:** Typed section.
**Notes:** `extensions` should not be the primary path for data expected by `server-2`.

### Status Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Unknown ok | Expected missing commander/winner data does not make artifact partial; conflict/loss diagnostics may. | yes |
| Unknown partial | Any missing commander or winner makes artifact partial. | |
| Agent decides | Planner chooses if explicit unknown/null states remain mandatory. | |

**User's choice:** Unknown ok.
**Notes:** Unknown is a valid state for legacy data, not automatically data loss.

---

## the agent's Discretion

- Exact Rust module/file names, helper structure, diagnostic code names, fixture
  layout, and typed payload details remain planner discretion within the locked
  decisions in `04-CONTEXT.md`.

## Deferred Ideas

None.
