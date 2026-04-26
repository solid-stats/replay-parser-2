# Phase 2: Versioned Output Contract - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-04-26
**Phase:** 02-versioned-output-contract
**Areas discussed:** Artifact Envelope, Unknown/Null States, Source References, Failures and Schema Compatibility

---

## Artifact Envelope

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Top-level result shape | Unified envelope | Every parse returns one `ParseArtifact` envelope with status, versions, source IDs, warnings, and either parsed data or failure details. | yes |
| Top-level result shape | Split success/failure | Success uses `ParseArtifact`; failures use a separate `ParseFailure` payload. | |
| Top-level result shape | You decide | Let the planning agent choose the shape. | |

**User's choice:** Unified envelope.
**Notes:** The parser should produce one stable top-level shape for storage and audit.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Contract versioning | Separate versions | Use explicit `contract_version` for schema compatibility and separate `parser_version`/build metadata for implementation provenance. | yes |
| Contract versioning | Parser version only | Treat parser release version as the contract version. | |
| Contract versioning | You decide | Let the planner choose the versioning scheme. | |

**User's choice:** Separate versions.
**Notes:** Contract compatibility and parser implementation provenance are different concerns.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Artifact skeleton | Full skeleton | Define stable sections for metadata, entities, events, aggregate contributions, diagnostics, and failures now. | yes |
| Artifact skeleton | Phase 2 only | Define only metadata/failure/source-reference contract now. | |
| Artifact skeleton | You decide | Let the planner balance stability against upfront schema detail. | |

**User's choice:** Full skeleton.
**Notes:** Later phases fill behavior; Phase 2 locks schema shape.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| JSON naming/style | snake_case stable | Use Rust-friendly `snake_case`, deterministic ordering where observable, and compatibility mappings for legacy field names outside the core contract. | yes |
| JSON naming/style | Legacy-like names | Prefer old result field names where possible. | |
| JSON naming/style | You decide | Let the planner choose naming and ordering conventions. | |

**User's choice:** snake_case stable.
**Notes:** Legacy compatibility should not force mixed naming in the core contract.

---

## Unknown/Null States

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Unknown/null representation | Tagged states | Use structured states for important fields, e.g. present/unknown/missing/not_applicable/inferred with reason and source context. | yes |
| Unknown/null representation | Null plus warnings | Use normal JSON nulls and explain missing data in diagnostics. | |
| Unknown/null representation | You decide | Let the planner choose the representation. | |

**User's choice:** Tagged states.
**Notes:** The user then chose explicit semantics for every optional field.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Unknown-state scope | Domain-critical only | Apply tagged states only to winner/outcome, SteamID, killer, commander, source refs, side/entity identity, and parse status fields. | |
| Unknown-state scope | Every optional field | Use tagged states for all nullable fields. | yes |
| Unknown-state scope | You decide | Let the planner decide field-by-field. | |

**User's choice:** Every optional field.
**Notes:** This favors audit clarity over smaller JSON.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Optional representation compactness | Compact tagged union | Use a consistent small shape like `{state, value?, reason?}`. | yes |
| Optional representation compactness | Rich per domain | Allow each domain field to define its own richer missing-data object. | |
| Optional representation compactness | You decide | Let the planner choose compactness. | |

**User's choice:** Compact tagged union.
**Notes:** Explicit semantics should stay manageable in schema and artifact size.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Inferred values | Separate inferred state | Represent inferred values distinctly from observed values and include confidence/source reason where applicable. | yes |
| Inferred values | Treat as present | Emit inferred values as normal present data. | |
| Inferred values | Avoid inference | Only allow observed or unknown values for now. | |

**User's choice:** Separate inferred state.
**Notes:** Inference must remain auditable.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Non-fatal schema drift | Warnings with paths | Add diagnostics with source path, expected/observed shape, parser action, and source reference while affected fields use explicit states. | yes |
| Non-fatal schema drift | Field states only | Encode drift only on affected field values. | |
| Non-fatal schema drift | You decide | Let the planner decide drift detail. | |

**User's choice:** Warnings with paths.
**Notes:** Diagnostics should explain why a value became unknown or inferred.

---

## Source References

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Source-reference granularity | Events + contributions | Require refs on normalized events and aggregate contributions, plus replay/entity-level refs where useful. | yes |
| Source-reference granularity | Every field | Attach source refs to nearly every field. | |
| Source-reference granularity | Events only | Keep refs mainly on events and failures. | |

**User's choice:** Events + contributions.
**Notes:** Good audit coverage without field-level bloat.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Source-reference shape | Structured tuple | Use replay/source file, frame, event index, entity ID, JSON path, and rule ID when available. | yes |
| Source-reference shape | JSON path only | Use source JSON path as the main reference. | |
| Source-reference shape | Domain-specific refs | Define separate ref shapes per domain. | |

**User's choice:** Structured tuple.
**Notes:** Contract should support both OCAP coordinates and JSON paths.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Rule IDs | Stable rule IDs | Derived/inferred values and aggregate contributions cite stable rule IDs. | yes |
| Rule IDs | Later phases only | Leave rule IDs optional until behavior phases. | |
| Rule IDs | No rule IDs | Use source coordinates only. | |

**User's choice:** Stable rule IDs.
**Notes:** Rule IDs are needed for auditing behavior changes across versions.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Aggregate contribution refs | Define now | Include an aggregate contribution/source-ref shape now so Phase 4 and `server-2` plan against a stable audit model. | yes |
| Aggregate contribution refs | Events now, aggregates later | Add aggregate refs during aggregate semantics. | |
| Aggregate contribution refs | You decide | Let planner decide whether aggregate refs are premature. | |

**User's choice:** Define now.
**Notes:** This is schema skeleton only; aggregate formulas remain later-phase work.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Raw source snippets | Refs only by default | Store coordinates and rule IDs in artifacts; keep raw replay data external. | yes |
| Raw source snippets | Small snippets allowed | Allow bounded raw snippets for hard-to-debug fields. | |
| Raw source snippets | You decide | Let planner set raw evidence policy. | |

**User's choice:** Refs only by default.
**Notes:** Avoid huge artifacts and raw-data duplication.

---

## Failures and Schema Compatibility

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Parse status model | success/partial/skipped/failed | Use explicit statuses with structured diagnostics and stage metadata. | yes |
| Parse status model | success/failed only | Simpler status model. | |
| Parse status model | You decide | Let planner define the status enum. | |

**User's choice:** success/partial/skipped/failed.
**Notes:** Partial historical data and skips need first-class representation.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Legacy aggregate skips | Distinct skipped status | Represent parser-level skipped outcomes separately from hard failures; legacy aggregate skip reasons remain compatibility diagnostics. | yes |
| Legacy aggregate skips | Treat as failures | Any replay that does not produce normal output is a failure. | |
| Legacy aggregate skips | Harness only | Let Phase 5 compatibility harness classify skips. | |

**User's choice:** Distinct skipped status.
**Notes:** Parser outcomes and legacy aggregate compatibility remain separate concepts.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Retryability | Enum plus reason | Use retryability enum plus stage, code, message, and source cause. | yes |
| Retryability | Boolean only | Use simple true/false. | |
| Retryability | No retry field | Leave retry policy entirely to `server-2`. | |

**User's choice:** Enum plus reason.
**Notes:** `server-2` can still own policy while parser provides structured evidence.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Schema generation | Generated from Rust | Rust contract types are source of truth; generate JSON Schema and validate examples in tests. | yes |
| Schema generation | Handwritten schema | Write schema first and make Rust conform. | |
| Schema generation | Schema later | Sketch schema in Phase 2 and generate later. | |

**User's choice:** Generated from Rust.
**Notes:** Avoid duplicate handwritten schema drift.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Cross-app contract ownership | Parser owns artifact | Parser owns `ParseArtifact`; `server-2` validates/stores/maps it and API/OpenAPI changes are coordinated separately. | yes |
| Cross-app contract ownership | Joint schema ownership | Parser and server share one schema contract directly. | |
| Cross-app contract ownership | Internal only | Parser artifact is internal. | |

**User's choice:** Parser owns artifact.
**Notes:** Preserves application ownership boundaries.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Error-code stability | Stable namespaced codes | Define stable families like `io.*`, `json.*`, `schema.*`, `unsupported.*`, and `internal.*`. | yes |
| Error-code stability | Messages first | Stabilize codes later. | |
| Error-code stability | You decide | Let planner define taxonomy depth. | |

**User's choice:** Stable namespaced codes.
**Notes:** Error codes should be durable enough for storage, retry, dashboards, and audits.

---

## the agent's Discretion

- Exact Rust type/module names, schema file paths, enum spelling, and deterministic serialization mechanism.

## Deferred Ideas

None.
