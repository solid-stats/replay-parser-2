# Phase 2: Versioned Output Contract - Context

**Gathered:** 2026-04-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 2 defines the stable, machine-checkable parser output contract that `server-2` and parser tooling can rely on. It covers the `ParseArtifact` envelope, parser/contract versioning, schema generation, explicit unknown/null states, source-reference model, and structured failure/diagnostic shape.

This phase may define the full v1 artifact skeleton, including sections later filled by parser core, event semantics, aggregates, CLI, and worker phases. It does not implement parsing behavior, event normalization, aggregate calculation, CLI commands, RabbitMQ/S3 worker behavior, old-vs-new comparison execution, or `server-2`/`web` changes.

</domain>

<decisions>
## Implementation Decisions

### Artifact Envelope
- **D-01:** Use one unified top-level `ParseArtifact` envelope for every parse result. The envelope carries status, source identifiers, versions, diagnostics, and either parsed data sections or structured failure details.
- **D-02:** Separate `contract_version` from `parser_version` and parser build/provenance metadata. Contract version controls schema compatibility; parser version identifies the implementation that produced the artifact.
- **D-03:** Define the full v1 artifact skeleton in Phase 2, including stable sections for replay metadata, observed entities/identity, normalized events, aggregate contribution references, diagnostics, and failures. Later phases fill semantics and behavior.
- **D-04:** Use `snake_case` JSON fields and deterministic serialization rules where output ordering is observable. Legacy field names belong in compatibility mappings or aggregate projections, not as the default core contract style.

### Unknown and Nullable Data
- **D-05:** Every optional field in the contract must carry explicit presence semantics, not a bare ambiguous `null`.
- **D-06:** Use a compact, consistent tagged-union shape for optional values, such as state plus optional value/reason/source metadata. Avoid bespoke verbose shapes for each nullable domain field unless a field truly needs extra metadata.
- **D-07:** Inferred values are distinct from observed present values. They must carry an inferred state plus source/reason/confidence metadata where applicable.
- **D-08:** Non-fatal schema drift or unexpected source shapes should produce path-based warnings/diagnostics with expected shape, observed shape, parser action, and source reference. Affected fields still use explicit presence states.

### Source References
- **D-09:** Require source references on normalized events and aggregate contributions. Replay/entity-level fields should also carry source references where useful for audit or mismatch classification.
- **D-10:** Use a structured source-reference tuple with fields such as replay/source file, frame, event index, entity ID, JSON path, and rule ID when available.
- **D-11:** Stable rule IDs are required for derived values, inferred values, and aggregate contributions so future behavior changes can be audited across contract/parser versions.
- **D-12:** Define aggregate contribution reference shape in Phase 2 even though aggregate behavior is implemented later. This lets Phase 4 and `server-2` plan against a stable audit model.
- **D-13:** Do not embed raw replay snippets or raw values in normal artifacts by default. Store source coordinates and rule IDs; raw replay files remain the external evidence source to avoid oversized artifacts.

### Failures and Schema Compatibility
- **D-14:** The artifact status model is `success`, `partial`, `skipped`, and `failed`.
- **D-15:** Legacy aggregate skip cases should be represented separately from hard failures. Parser-level skipped outcomes can use `skipped`; legacy aggregate skip reasons remain compatibility diagnostics rather than parser-core semantic filtering.
- **D-16:** `ParseFailure` retryability uses an enum such as `retryable`, `not_retryable`, and `unknown`, plus reason/stage/error-code/message/source-cause fields.
- **D-17:** JSON Schema should be generated from Rust contract types, not maintained as a separate handwritten source of truth. The schema should be exportable/committed and validated against examples in tests.
- **D-18:** `replay-parser-2` owns the parser artifact contract. `server-2` validates, stores, and maps that artifact into durable/API shapes; OpenAPI and `web` generated-type changes are coordinated downstream when needed.
- **D-19:** Error codes should be stable and namespaced, with families such as `io.*`, `json.*`, `schema.*`, `unsupported.*`, and `internal.*`.

### the agent's Discretion
- Exact Rust module names, type names, enum variant spelling, and schema file path are planner discretion as long as they preserve the decisions above.
- Exact compact tagged-union field names are planner discretion, but the shape must remain consistent across optional fields.
- Exact deterministic ordering mechanism is planner discretion.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project scope and phase requirements
- `.planning/PROJECT.md` - Project boundaries, parser/server/web ownership, identity constraints, integration flow, and architecture direction.
- `.planning/REQUIREMENTS.md` - Phase 2 `OUT-*` requirements and v1/v2 scope boundaries.
- `.planning/ROADMAP.md` - Phase 2 goal, success criteria, dependencies, and phase ordering.
- `.planning/STATE.md` - Current project state and accumulated decisions.
- `README.md` - Human-facing project status, expected architecture direction, planned command shape, and workflow expectations.

### Phase 1 contract handoff
- `.planning/phases/01-legacy-baseline-and-corpus/01-CONTEXT.md` - Locked Phase 1 decisions, especially observed identity vs compatibility identity and v2-deferred yearly nominations.
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md` - Parser-core vs parity-harness boundary, legacy skip rules, identity compatibility rules, and ordinary output surfaces.
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md` - Required mismatch categories, impact dimensions, human-review gate, and Phase 2 handoff.
- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` - Historical corpus shape, malformed files, observed OCAP top-level keys, and event/entity shape evidence.
- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` - Legacy baseline command/runtime facts and current-vs-regenerated result drift that future contracts must help audit.

### Cross-application boundaries
- `gsd-briefs/replay-parser-2.md` - Parser-specific product brief, identity constraints, bounty inputs, parser integration flow, and output contract notes.
- `gsd-briefs/server-2.md` - Backend ownership of persistence, parse jobs, canonical identity, recalculation, public APIs, and OpenAPI contract.
- `gsd-briefs/web.md` - Frontend ownership and generated API type consumption through `server-2`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- There is no Rust workspace or source code yet. The reusable inputs for Phase 2 are planning artifacts, Phase 1 dossiers, and cross-project briefs.
- Phase 1 dossiers provide the current evidence base for malformed replay cases, old parser skip behavior, legacy identity compatibility rules, and source-reference needs.

### Established Patterns
- Parser core must preserve observed identity. Canonical player matching, nickname/SteamID history, persistence, recalculation, moderation, and public APIs belong to `server-2`.
- Legacy game-type filtering, `sgs` exclusion, `sm` cutoff, player/replay exclusions, name-change compatibility, and same-name combining are parity-harness or compatibility-layer concerns, not parser-core contract filtering.
- Annual/yearly nomination outputs remain v2-deferred references and must not enter the v1 ordinary parser artifact contract.
- Expected implementation direction is Rust 2024 with `serde`/`serde_json`, deterministic serialization, `schemars`, semantic versioning, and structured diagnostics/tracing.

### Integration Points
- Phase 3 parser core consumes this contract to fill replay metadata and observed entity facts.
- Phase 4 event/aggregate semantics consume the event and aggregate contribution skeleton plus source-reference/rule-ID model.
- Phase 5 CLI/comparison/validation consumes JSON Schema export and mismatch-impact fields.
- Phase 6 worker integration consumes `ParseArtifact`, `ParseFailure`, retryability, and status semantics for RabbitMQ/S3 result handling.
- `server-2` consumes the artifact as parser-owned input, then maps it into PostgreSQL and OpenAPI-owned API shapes.

</code_context>

<specifics>
## Specific Ideas

- Treat “full v1 artifact skeleton now” as a schema/shape decision only. Do not implement event semantics or aggregate formulas in Phase 2.
- Use explicit presence semantics for every optional field despite the larger JSON shape, because historical replay gaps and future moderation/audit workflows need unambiguous data.
- Keep artifacts lean by default: source coordinates and rule IDs are stored, raw replay snippets are not embedded unless a future explicit debug artifact mode is added.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope.

</deferred>

---

*Phase: 02-versioned-output-contract*
*Context gathered: 2026-04-26*
