# Phase 3: Deterministic Parser Core - Context

**Gathered:** 2026-04-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3 builds the pure Rust parser core foundation. It reads OCAP JSON
bytes/files matching the historical corpus and fills deterministic Phase 2
contract sections for replay metadata and observed entity facts. It also
establishes tolerant schema-drift handling, deterministic output ordering,
connected-player backfill compatibility, and duplicate-slot same-name
compatibility hooks.

This phase does not implement combat event semantics, kill/death/teamkill
classification, aggregate formulas, vehicle score, CLI commands, RabbitMQ/S3
worker behavior, old-vs-new comparison commands, benchmarks, or adjacent
`server-2`/`web` changes.

</domain>

<decisions>
## Implementation Decisions

### Parser Core Boundary
- **D-01:** Create a pure parser-core crate/module that accepts replay bytes
  plus explicit caller-provided source/job metadata and returns Phase 2
  `parser-contract` types.
- **D-02:** Keep CLI, worker, S3, RabbitMQ, benchmark, and comparison harness
  concerns out of parser-core. Later adapters call the same core API.
- **D-03:** Parser-core may produce structured diagnostics and failures, but it
  must not write files, publish messages, mutate databases, or perform
  canonical identity matching.

### OCAP Decode and Schema Drift
- **D-04:** Use correctness-first `serde_json` decoding with tolerant raw
  adapters. Do not optimize with alternate JSON engines in Phase 3.
- **D-05:** Use a strict-root, tolerant-fields policy. Invalid JSON, EOF-truncated
  JSON, or a root shape that cannot be treated as OCAP must produce a structured
  `ParseFailure`; localized field/section drift should produce diagnostics,
  explicit unknowns, or partial artifacts.
- **D-06:** The minimum useful Phase 3 success/partial artifact is replay
  metadata plus best-effort observed entities. Entity drift should not hard-fail
  if metadata/root evidence remains usable.
- **D-07:** Schema-drift diagnostics should be path-based and capped: include
  `json_path`, expected shape, observed shape, parser action, and source refs for
  specific problems, but collapse repeated/mass issues into summaries to avoid
  oversized artifacts.
- **D-08:** `ParseStatus::Partial` is required when diagnostics indicate data
  loss, dropped entity facts, unknowns caused by drift, or conflicting source
  evidence. Informational or non-loss diagnostics can remain `success`.
- **D-09:** Keep raw OCAP quirks behind adapter/helper code so contract types are
  populated from normalized observations, not scattered tuple/index logic.

### Determinism
- **D-10:** Output ordering must be stable across repeated parses of the same
  input and contract version.
- **D-11:** Normalize entity ordering by `source_entity_id` ascending, with stable
  secondary tie-breakers when needed. Use stable ordering for any dynamic maps;
  prefer sorted vectors or `BTreeMap` where serialized order can be observed.
- **D-12:** Avoid wall-clock timestamps inside deterministic parser-core output.
  If `produced_at` is populated later, it should be an adapter/caller concern or
  explicitly injectable.

### Observed Entity Normalization
- **D-13:** Phase 3 must extend the typed contract as needed so observed
  vehicle/static weapon names and classes are represented explicitly, not hidden
  in `extensions`. This closes the current gap between PARS-05 and the existing
  `ObservedEntity` shape.
- **D-14:** Normalize units/players, vehicles, and static weapons as observed
  facts with source IDs, names/classes, side/group/role fields where present, and
  source references.
- **D-15:** Classify only broad entity kind in Phase 3: unit/player,
  vehicle/static weapon, or unknown, while preserving raw observed `class`/name.
  Vehicle score taxonomy such as car/truck/APC/tank/heli/plane remains Phase 4.
- **D-16:** Every normalized entity should include best-known source evidence
  such as entity ID, JSON path, replay/source file, and checksum when available.
  Missing source refs should become diagnostics and may affect `partial` status
  if auditability is materially degraded.
- **D-17:** Preserve observed identifiers only. Do not infer canonical players,
  real accounts, or cross-replay identity matches in parser-core.

### Legacy Compatibility Hooks
- **D-18:** Preserve old connected-player backfill behavior as inferred observed
  entity facts, not only diagnostics. The old code adds a player from a
  `connected` event when an entity with the same ID exists, is not a vehicle, and
  the connected name is present.
- **D-19:** Backfilled connected-player facts must carry explicit provenance:
  `FieldPresence::Inferred` or equivalent inferred states where applicable,
  stable `rule_id`, source refs to the connected event and entity evidence, and
  diagnostics explaining the compatibility action.
- **D-20:** Preserve duplicate-slot same-name compatibility as a typed
  compatibility hint/diagnostic for later aggregate projection. Do not collapse
  normalized raw observed entities in Phase 3.
- **D-21:** Connected-player backfill and same-name compatibility hints do not
  automatically make the artifact `partial`. `success` is acceptable when there
  is no data loss or conflict. Use `partial` when compatibility evidence
  conflicts, source data is lost, or same-name candidates have incompatible
  side/source evidence.
- **D-22:** Legacy game-type filtering and annual/yearly nomination behavior
  remain outside parser-core Phase 3; they are parity/comparison or v2 concerns
  already documented in Phase 1.

### Tests and Fixtures
- **D-23:** Use small, focused behavior fixtures first, derived from Phase 1
  corpus shapes where possible. Full corpus/golden parity belongs to Phase 5.
- **D-24:** Add tests for normal metadata/entity extraction, schema drift,
  malformed input, explicit unknowns, deterministic ordering, source refs,
  connected-player backfill, and duplicate-slot compatibility hints.
- **D-25:** Legacy hook tests should use focused fixtures with comments or
  references to the old parser source files. Full real-corpus parity remains
  Phase 5.
- **D-26:** Follow the project’s RITE/AAA unit-test standard and avoid test-only
  production exports unless the public parser-core API cannot otherwise prove
  behavior.
- **D-27:** New parser-core code must pass the strict stable Rust gates added by
  quick task `260426-joq`: workspace lints, rustfmt, clippy `-D warnings`,
  tests, and docs.

### the agent's Discretion
- Exact parser-core crate/module naming is planner discretion, but it should
  follow the existing workspace style and keep parser-core separate from
  parser-contract.
- Exact raw DTO/helper structure is planner discretion as long as OCAP quirks
  stay isolated from contract types.
- Exact diagnostic code names are planner discretion, but they must be stable,
  namespaced, and covered by tests.
- Exact fixture file layout is planner discretion, provided tests remain
  deterministic and reviewable.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project and phase scope
- `.planning/PROJECT.md` - Current project state, parser/server/web ownership,
  constraints, and Phase 3 readiness.
- `.planning/REQUIREMENTS.md` - Phase 3 requirements `OUT-08`, `PARS-01`
  through `PARS-07`, plus test standards.
- `.planning/ROADMAP.md` - Phase 3 goal, success criteria, dependencies, and
  later phase boundaries.
- `.planning/STATE.md` - Current GSD state and accumulated decisions.
- `.planning/research/SUMMARY.md` - Research rationale for pure core,
  correctness-first Serde parsing, deterministic ordering, and phase boundaries.
- `README.md` - Human-facing status, contract crate commands, architecture
  direction, and development workflow.

### Phase 2 contract handoff
- `.planning/phases/02-versioned-output-contract/02-CONTEXT.md` - Locked
  contract decisions for artifact envelope, presence semantics, source refs,
  failures, and schema generation.
- `.planning/phases/02-versioned-output-contract/02-VERIFICATION.md` - Verified
  Phase 2 contract invariants and phase-goal evidence.
- `.planning/phases/02-versioned-output-contract/02-05-SUMMARY.md` - Gap-closure
  details for checksums, failure invariants, source refs, error-code families,
  and confidence bounds.
- `crates/parser-contract/src/artifact.rs` - `ParseArtifact`, `ParseStatus`, and
  status/failure validation.
- `crates/parser-contract/src/source_ref.rs` - `ReplaySource`, `SourceChecksum`,
  `SourceRef`, `SourceRefs`, and `RuleId`.
- `crates/parser-contract/src/presence.rs` - Explicit presence states and
  bounded confidence.
- `crates/parser-contract/src/metadata.rs` - Replay metadata contract to
  populate in Phase 3.
- `crates/parser-contract/src/identity.rs` - Observed identity/entity contract
  to populate and likely extend in Phase 3.
- `crates/parser-contract/src/diagnostic.rs` - Diagnostic contract for schema
  drift and tolerant parsing.
- `crates/parser-contract/src/failure.rs` - Structured parse failure contract.

### Phase 1 legacy/corpus evidence
- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` -
  Historical corpus counts, malformed files, observed OCAP top-level keys, and
  schema/profile evidence.
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md`
  - Legacy filters, connected-player and identity compatibility behaviors, old
  output surfaces, and v2 exclusions.
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md`
  - Mismatch taxonomy and interface impact categories.
- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` -
  Legacy parser command/runtime baseline and result drift context.

### Legacy parser source
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/getEntities.ts`
  - Old entity extraction, vehicle capture, and connected-player backfill logic.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/2 - parseReplayInfo/combineSamePlayersInfo.ts`
  - Old duplicate-slot same-name aggregate merge behavior.
- `/home/afgan0r/Projects/SolidGames/replays-parser/src/0 - types/replay.d.ts`
  - Old replay/entity/event/player type shapes.

### Strict quality quick task
- `.planning/quick/260426-joq-strict-quality-rules/260426-joq-CONTEXT.md` -
  Strict stable Rust lint/format/type-safety decisions.
- `.planning/quick/260426-joq-strict-quality-rules/260426-joq-SUMMARY.md` -
  Implemented quality gates and verification commands.
- `Cargo.toml` - Workspace lint policy inherited by future crates.
- `.cargo/config.toml` - Cargo quality aliases.
- `rustfmt.toml` - Stable rustfmt policy.

### Cross-application boundaries
- `gsd-briefs/replay-parser-2.md` - Parser-specific product brief and
  integration flow.
- `gsd-briefs/server-2.md` - Backend ownership of canonical identity,
  persistence, parse jobs, recalculation, and API/OpenAPI mapping.
- `gsd-briefs/web.md` - Frontend ownership and generated API type consumption
  through `server-2`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `parser-contract` already defines the output types, schema helper, examples,
  and behavior tests that parser-core must populate.
- Contract tests provide examples of expected JSON shapes and can guide
  parser-core fixture assertions.
- Strict workspace lints and cargo aliases are already configured and should be
  inherited by any new parser-core crate.

### Established Patterns
- Public contract modules are split by concern: artifact, metadata, identity,
  source_ref, diagnostic, failure, events, aggregates, presence, schema, and
  version.
- Optional facts use `FieldPresence<T>` rather than bare nullable values.
- Auditable references use `SourceRefs` where non-empty source evidence is
  mandatory.
- Deterministic dynamic output uses `BTreeMap` in existing contract structures.
- Existing contract code forbids unsafe code and denies warnings, missing docs,
  many Clippy lints, stdout/stderr macros, `unwrap`, and `expect` outside
  justified test allowances.

### Integration Points
- Phase 3 should add parser-core as a workspace member that depends on
  `parser-contract`.
- Parser-core should return contract-owned artifacts/failures that later CLI and
  worker adapters can serialize without reshaping.
- Phase 3 may need a focused parser-contract update for typed entity
  name/class fields before parser-core can satisfy PARS-05.
- Phase 4 will build event semantics on top of the raw/observed facts and
  compatibility hints established here.
- Phase 5 will turn Phase 3 fixtures and deterministic behavior into broader
  golden parity, CLI, coverage, and benchmark gates.

</code_context>

<specifics>
## Specific Ideas

- Treat Phase 2 `ParseArtifact`, `FieldPresence`, `SourceRefs`, `ParseFailure`,
  and generated schema as locked contract inputs unless Phase 3 must extend the
  typed entity contract to close the PARS-05 vehicle/static class/name gap.
- Favor explicit parser warnings/diagnostics over silent drops when a known OCAP
  field is present but malformed.
- Start with metadata and observed entity facts only; combat event semantics and
  aggregates should wait for Phase 4.
- Old connected-player backfill should be represented as inferred observed
  facts with source evidence, not as canonical identity matching.
- Old same-name merge should remain a later compatibility projection hint, not a
  Phase 3 raw observation collapse.

</specifics>

<deferred>
## Deferred Ideas

- Full old-vs-new comparison harness and full-corpus replay sweeps - Phase 5.
- Combat event semantics, vehicle context, commander-side outcome, and aggregate
  formulas - Phase 4.
- CLI command shape and user-facing parse output flags - Phase 5.
- RabbitMQ/S3 job message handling and artifact publication - Phase 6.

</deferred>

---

*Phase: 03-deterministic-parser-core*
*Context gathered: 2026-04-26*
