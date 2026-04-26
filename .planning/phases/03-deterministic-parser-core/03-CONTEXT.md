# Phase 3: Deterministic Parser Core - Context

**Gathered:** 2026-04-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3 builds the pure Rust parser core foundation. It reads OCAP JSON bytes/files matching the historical corpus and fills deterministic Phase 2 contract sections for replay metadata and observed entity facts. It also establishes tolerant schema-drift handling, deterministic output ordering, connected-player backfill compatibility, and duplicate-slot same-name compatibility hooks.

This phase does not implement combat event semantics, kill/death/teamkill classification, aggregate formulas, vehicle score, CLI commands, RabbitMQ/S3 worker behavior, old-vs-new comparison commands, benchmarks, or adjacent `server-2`/`web` changes.

</domain>

<decisions>
## Implementation Decisions

### Parser Core Boundary
- **D-01:** Create a pure parser-core crate/module that accepts replay bytes plus explicit caller-provided source/job metadata and returns Phase 2 `parser-contract` types.
- **D-02:** Keep CLI, worker, S3, RabbitMQ, benchmark, and comparison harness concerns out of parser-core. Later adapters call the same core API.
- **D-03:** Parser-core may produce structured diagnostics and failures, but it must not write files, publish messages, mutate databases, or perform canonical identity matching.

### OCAP Decode and Schema Drift
- **D-04:** Use correctness-first `serde_json` decoding with tolerant raw adapters. Do not optimize with alternate JSON engines in Phase 3.
- **D-05:** Extract replay metadata from observed top-level OCAP keys listed in the requirements and Phase 1 corpus evidence: `missionName`, `worldName`, `missionAuthor`, `playersCount`, `captureDelay`, `endFrame`, `entities`, `events`, `EditorMarkers`, and `Markers`.
- **D-06:** Unknown or drifted source shapes must not panic. They should become explicit unknown states, structured diagnostics, or `ParseFailure` values depending on severity.
- **D-07:** Keep raw OCAP quirks behind adapter/helper code so contract types are populated from normalized observations, not scattered tuple/index logic.

### Determinism
- **D-08:** Output ordering must be stable across repeated parses of the same input and contract version.
- **D-09:** Use stable ordering for entity/event collections and any dynamic maps. Prefer sorted vectors or `BTreeMap` over `HashMap` where serialized order can be observed.
- **D-10:** Avoid wall-clock timestamps inside deterministic parser-core output. If `produced_at` is populated later, it should be an adapter/caller concern or explicitly injectable.

### Observed Entity Normalization
- **D-11:** Normalize units/players, vehicles, and static weapons as observed facts with source IDs, names/classes, side/group/role fields where present, and source references.
- **D-12:** Preserve observed identifiers only. Do not infer canonical players, real accounts, or cross-replay identity matches in parser-core.
- **D-13:** Maintain source-reference evidence for normalized metadata and entity facts when the source coordinate is known; use path/frame/entity coordinates from available OCAP structure.

### Legacy Compatibility Hooks
- **D-14:** Preserve connected-player backfill behavior as parser-core compatibility metadata or diagnostics where entity data omits participants, but keep raw observed identities intact.
- **D-15:** Preserve duplicate-slot same-name merge compatibility as later aggregate-projection guidance, not by collapsing normalized raw observations prematurely.
- **D-16:** Legacy game-type filtering and annual/yearly nomination behavior remain outside parser-core Phase 3; they are parity/comparison or v2 concerns already documented in Phase 1.

### Tests and Fixtures
- **D-17:** Use small, focused behavior fixtures first, derived from Phase 1 corpus shapes where possible. Full corpus/golden parity belongs to Phase 5.
- **D-18:** Add tests for normal metadata/entity extraction, schema drift, malformed input, explicit unknowns, deterministic ordering, connected-player backfill, and duplicate-slot compatibility hooks.
- **D-19:** Follow the project’s RITE/AAA unit-test standard and avoid test-only production exports unless the public parser-core API cannot otherwise prove behavior.

### the agent's Discretion
- Exact crate/module naming is planner discretion, but it should follow the existing workspace style and keep parser-core separate from parser-contract.
- Exact raw DTO/helper structure is planner discretion as long as OCAP quirks stay isolated from contract types.
- Exact fixture file layout is planner discretion, provided tests remain deterministic and reviewable.

</decisions>

<specifics>
## Specific Ideas

- Treat Phase 2 `ParseArtifact`, `FieldPresence`, `SourceRefs`, `ParseFailure`, and generated schema as locked contract inputs.
- Favor explicit parser warnings/diagnostics over silent drops when a known OCAP field is present but malformed.
- Start with metadata and observed entity facts only; combat event semantics and aggregates should wait for Phase 4.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project and phase scope
- `.planning/PROJECT.md` - Current project state, parser/server/web ownership, constraints, and Phase 3 readiness.
- `.planning/REQUIREMENTS.md` - Phase 3 requirements `OUT-08`, `PARS-01` through `PARS-07`, plus test standards.
- `.planning/ROADMAP.md` - Phase 3 goal, success criteria, dependencies, and later phase boundaries.
- `.planning/STATE.md` - Current GSD state and accumulated decisions.
- `README.md` - Human-facing status, contract crate commands, architecture direction, and development workflow.

### Phase 2 contract handoff
- `.planning/phases/02-versioned-output-contract/02-CONTEXT.md` - Locked contract decisions for artifact envelope, presence semantics, source refs, failures, and schema generation.
- `.planning/phases/02-versioned-output-contract/02-VERIFICATION.md` - Verified Phase 2 contract invariants and phase-goal evidence.
- `.planning/phases/02-versioned-output-contract/02-05-SUMMARY.md` - Gap-closure details for checksums, failure invariants, source refs, error-code families, and confidence bounds.
- `crates/parser-contract/src/artifact.rs` - `ParseArtifact`, `ParseStatus`, and status/failure validation.
- `crates/parser-contract/src/source_ref.rs` - `ReplaySource`, `SourceChecksum`, `SourceRef`, `SourceRefs`, and `RuleId`.
- `crates/parser-contract/src/presence.rs` - Explicit presence states and bounded confidence.
- `crates/parser-contract/src/metadata.rs` - Replay metadata contract to populate in Phase 3.
- `crates/parser-contract/src/identity.rs` - Observed identity/entity contract to populate in Phase 3.
- `crates/parser-contract/src/diagnostic.rs` - Diagnostic contract for schema drift and tolerant parsing.
- `crates/parser-contract/src/failure.rs` - Structured parse failure contract.

### Phase 1 legacy/corpus evidence
- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` - Historical corpus counts, malformed files, observed OCAP top-level keys, and schema/profile evidence.
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md` - Legacy filters, connected-player and identity compatibility behaviors, old output surfaces, and v2 exclusions.
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md` - Mismatch taxonomy and interface impact categories.
- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` - Legacy parser command/runtime baseline and result drift context.

### Cross-application boundaries
- `gsd-briefs/replay-parser-2.md` - Parser-specific product brief and integration flow.
- `gsd-briefs/server-2.md` - Backend ownership of canonical identity, persistence, parse jobs, recalculation, and API/OpenAPI mapping.
- `gsd-briefs/web.md` - Frontend ownership and generated API type consumption through `server-2`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `parser-contract` already defines the output types, schema helper, examples, and behavior tests that parser-core must populate.
- Contract tests provide examples of expected JSON shapes and can guide parser-core fixture assertions.

### Established Patterns
- Public contract modules are split by concern: artifact, metadata, identity, source_ref, diagnostic, failure, events, aggregates, presence, schema, and version.
- Optional facts use `FieldPresence<T>` rather than bare nullable values.
- Auditable references use `SourceRefs` where non-empty source evidence is mandatory.
- Deterministic dynamic output uses `BTreeMap` in existing contract structures.

### Integration Points
- Phase 3 should add parser-core as a workspace member that depends on `parser-contract`.
- Parser-core should return contract-owned artifacts/failures that later CLI and worker adapters can serialize without reshaping.
- Phase 4 will build event semantics on top of the raw/observed facts established here.
- Phase 5 will turn Phase 3 fixtures and deterministic behavior into broader golden parity, CLI, coverage, and benchmark gates.

</code_context>

<deferred>
## Deferred Ideas

- Full old-vs-new comparison harness and full-corpus replay sweeps — Phase 5.
- Combat event semantics, vehicle context, commander-side outcome, and aggregate formulas — Phase 4.
- CLI command shape and user-facing parse output flags — Phase 5.
- RabbitMQ/S3 job message handling and artifact publication — Phase 6.

</deferred>

---

*Phase: 03-deterministic-parser-core*
*Context gathered: 2026-04-26*
