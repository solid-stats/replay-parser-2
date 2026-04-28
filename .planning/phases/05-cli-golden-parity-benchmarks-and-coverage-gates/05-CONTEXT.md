# Phase 5: CLI, Golden Parity, Benchmarks, and Coverage Gates - Context

**Gathered:** 2026-04-28T11:17:43+07:00
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 5 turns the verified parser contract and pure parser core into executable
local tooling and release gates. It delivers a local CLI for parsing files,
schema export, focused old-vs-new comparison, curated golden fixtures,
determinism checks, strict reachable-code coverage enforcement, full-workspace
mutation or equivalent fault reporting, and benchmark reports that measure the
roughly 10x speed target on equivalent workloads.

This phase does not implement RabbitMQ/S3 worker behavior, S3 artifact writes,
queue acknowledgement, container readiness, PostgreSQL persistence, public APIs,
canonical identity matching, public UI behavior, replay discovery, production
raw replay fetching, or annual/yearly nomination product support.

</domain>

<decisions>
## Implementation Decisions

### CLI Command Contract
- **D-01:** The public local binary name is `replay-parser-2`, not
  `sg-replay-parser`. Phase 5 should update README planned command examples to
  use `replay-parser-2 parse`, `replay-parser-2 schema`, and
  `replay-parser-2 compare`.
- **D-02:** `replay-parser-2 parse <input> --output <path>` writes a
  `ParseArtifact` to the requested output path for both success and parser
  failure artifacts. Malformed, unreadable, or unsupported inputs still exit
  non-zero; stderr should contain only a concise human summary, not the primary
  structured artifact.
- **D-03:** The local parse command computes SHA-256 automatically, sets
  `source_file` from the input path, and accepts optional replay identity
  metadata such as `replay_id` by flag when available. Users should not be
  required to pass checksum metadata manually for local reproduction.
- **D-04:** `replay-parser-2 compare` is a focused command for selected replay
  files and saved old/new artifacts. Full-corpus batch parity reports can be
  implemented as dev harness or CI scripts around the CLI rather than forcing
  every heavy workflow into the public binary.

### Golden Parity Policy
- **D-05:** Phase 5 parity uses dual evidence: current `~/sg_stats/results` and
  regenerated old-parser outputs are both reference evidence. Drift between
  those baselines remains `human review` until a report explains the cause and a
  preserve/fix decision is approved.
- **D-06:** The primary semantic/parity baseline is the deterministic legacy
  `WORKER_COUNT=1` profile. The default-worker legacy profile remains diagnostic
  evidence for drift and performance context, not the primary semantic truth.
- **D-07:** Old-vs-new comparison reports must be per-field or per-surface and
  classify every mismatch with the Phase 1 taxonomy. Reports must include parser
  artifact impact, `server-2` persistence impact, `server-2` recalculation
  impact, and UI-visible public-stats impact.
- **D-08:** Commit compact curated golden fixtures only: small focused fixtures
  plus selected real-corpus seeds derived from `fixture-index.json` and Phase 4
  edge cases. Full corpus outputs, bulky profiles, logs, and generated reports
  remain under ignored generated paths.

### Coverage and Fault Gates
- **D-09:** The canonical coverage tool is `cargo llvm-cov`. Phase 5 should
  enforce strict coverage with zero uncovered lines, functions, and regions, and
  branch coverage where the supported Rust/cargo-llvm-cov setup can provide it.
  Existing stable Rust quality gates remain in force.
- **D-10:** The 100% reachable-code coverage scope includes all reachable
  production Rust in parser-contract, parser-core, CLI, harness, benchmark, and
  aggregate-related modules. Test code is excluded from the denominator.
  Generated code, impossible platform glue, or defensive unreachable branches
  may be excluded only through the allowlist policy below.
- **D-11:** Coverage exclusions require a committed reviewable allowlist plus
  inline rationale near the excluded code. No blanket module exclusions should
  be accepted without a specific impossible/generated/defensive reason.
- **D-12:** Full-workspace mutation testing or equivalent fault-injection
  reporting is a blocking Phase 5 gate. The report must distinguish caught,
  missed, timeout, and unviable cases, and high-risk survivors must be fixed or
  explicitly documented as accepted non-applicable cases.
- **D-13:** Coverage exists to support behavior quality, not replace it. Tests
  should follow the project RITE/AAA standard: readable names, isolated state,
  thorough success/boundary/error scenarios, explicit observable assertions, and
  no test-only production exports unless public behavior cannot otherwise be
  proven.

### Benchmark Evidence
- **D-14:** Benchmark evidence is two-tier. Use Rust benchmark tooling for
  parser-core stages such as parse-only, aggregate-only, and end-to-end core
  artifact construction, and use command-level benchmarking for old-vs-new CLI
  or harness runs on equivalent replay sets. The 10x report must state which
  tier does or does not meet the target.
- **D-15:** Benchmark workloads are tiered: a small CI sample, a curated
  representative sample, and an optional/manual full-corpus run. Each report
  must record the exact fixture list or corpus selector and the parity status
  for the measured sample.
- **D-16:** Benchmark summaries must include files/sec, MB/sec or events/sec,
  wall time, memory/RSS where practical, the old baseline profile used, parity
  status, and 10x status as pass, fail, or unknown.
- **D-17:** If a Phase 5 benchmark report shows less than roughly 10x faster
  performance, Phase 5 is blocked pending triage. The triage must explain the
  bottleneck, verify parity was not sacrificed, and either fix the issue or
  record an explicit accepted gap before completion.

### the agent's Discretion
- Exact crate/package split for the CLI and harness is planner discretion, as
  long as the public binary is `replay-parser-2` and parser-core remains pure
  and transport-free.
- Exact flag spelling beyond the locked command responsibilities is planner
  discretion if README, tests, and CLI help stay consistent.
- Exact report file formats are planner discretion, but they must be structured,
  deterministic, and include the required categories, impact dimensions, and
  benchmark fields.
- Exact curated fixture list is planner discretion if it is traceable to Phase 1
  corpus evidence, Phase 4 edge cases, and the v1 requirements.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project and Phase Scope
- `.planning/PROJECT.md` - Current parser scope, Phase 5 readiness, old-parser
  reference, historical corpus facts, 10x target, 100% coverage requirement,
  and cross-application boundary rules.
- `.planning/REQUIREMENTS.md` - Phase 5 requirements `CLI-01` through `CLI-04`
  and `TEST-01` through `TEST-12`.
- `.planning/ROADMAP.md` - Phase 5 goal, success criteria, dependencies, and
  Phase 6/7 boundaries.
- `.planning/STATE.md` - Current focus and accumulated decisions from completed
  phases.
- `.planning/research/SUMMARY.md` - Research rationale for local CLI, golden
  parity, benchmarks, and validation before worker integration.
- `README.md` - Current repository status, planned commands, validation data,
  architecture direction, and AI/GSD workflow expectations.

### Prior Phase Decisions
- `.planning/phases/04-event-semantics-and-aggregates/04-CONTEXT.md` - Event,
  aggregate, bounty, vehicle score, commander/outcome, and Phase 5 handoff
  decisions.
- `.planning/phases/03-deterministic-parser-core/03-CONTEXT.md` - Pure
  parser-core boundary, deterministic output policy, source metadata model,
  schema-drift diagnostics, and test fixture policy.
- `.planning/phases/02-versioned-output-contract/02-CONTEXT.md` - ParseArtifact
  envelope, explicit presence states, source refs, structured failures, schema
  generation, and Phase 5 CLI/comparison integration points.
- `.planning/phases/01-legacy-baseline-and-corpus/01-CONTEXT.md` - Legacy
  baseline, corpus, fixture, and mismatch policy decisions.

### Legacy Baseline and Parity Evidence
- `.planning/phases/01-legacy-baseline-and-corpus/baseline-command-runtime.md` -
  Canonical old parser command, deterministic and default worker profiles,
  generated output digests, current-vs-regenerated drift, and reproduction
  commands.
- `.planning/phases/01-legacy-baseline-and-corpus/corpus-manifest.md` - Corpus
  counts, malformed files, game-type distribution, largest files, and fixture
  selection rationale.
- `.planning/phases/01-legacy-baseline-and-corpus/fixture-index.json` - Seed
  list for real-corpus fixture curation.
- `.planning/phases/01-legacy-baseline-and-corpus/legacy-rules-output-surfaces.md`
  - Legacy filters, skip rules, config inputs, identity compatibility boundary,
  comparable ordinary output surfaces, and yearly nomination deferral.
- `.planning/phases/01-legacy-baseline-and-corpus/mismatch-taxonomy-interface-notes.md`
  - Required mismatch categories, impact dimensions, and human review gate.

### Current Code and Reusable APIs
- `Cargo.toml` - Workspace members, strict lint policy, Rust edition/MSRV, and
  existing quality gate expectations.
- `crates/parser-core/Cargo.toml` - Current parser-core dependencies and lint
  inheritance.
- `crates/parser-contract/Cargo.toml` - Current contract dependencies and
  schema validation dev dependency.
- `crates/parser-core/src/lib.rs` - Public pure `parse_replay` API used by CLI,
  harnesses, tests, and benchmarks.
- `crates/parser-core/src/input.rs` - `ParserInput` and `ParserOptions`,
  including caller-supplied source metadata and diagnostic limit.
- `crates/parser-core/src/artifact.rs` - Current artifact construction, failure
  artifact behavior, source context, and deterministic parser-core output.
- `crates/parser-contract/examples/export_schema.rs` - Existing schema export
  example that Phase 5 can promote into the public CLI.
- `crates/parser-contract/tests/schema_contract.rs` - Schema freshness and
  example validation tests.
- `crates/parser-core/tests/deterministic_output.rs` - Existing deterministic
  serialization behavior tests.
- `crates/parser-core/tests/fixtures/` - Current focused parser-core fixtures
  that Phase 5 can extend with curated golden samples.

### Cross-Application Boundaries
- `gsd-briefs/replay-parser-2.md` - Parser-owned contract, CLI, worker, parity,
  and benchmark responsibilities.
- `gsd-briefs/server-2.md` - Backend ownership of persistence, parse jobs,
  canonical identity, recalculation, and public API mapping.
- `gsd-briefs/replays-fetcher.md` - Replay discovery/raw object ownership and
  checksum/key compatibility boundary.
- `gsd-briefs/web.md` - Web ownership of public UI and generated API types
  through `server-2`.

### Test Philosophy
- `/home/afgan0r/.agents/skills/unit-tests-philosophy/SKILL.md` - RITE/AAA
  test quality standard that Phase 5 tests and coverage gates must follow.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `parser_core::parse_replay(ParserInput)` already exposes a pure, deterministic
  API that future CLI, comparison harnesses, tests, and benchmarks should call.
- `ParserInput` already separates replay bytes, source metadata, parser info,
  and deterministic parser options, which fits local CLI auto-checksum behavior.
- `ParseArtifact` already supports failed artifacts with structured
  `ParseFailure`, which supports the decision to write success or failure
  artifacts to `--output`.
- `crates/parser-contract/examples/export_schema.rs` already generates the
  current schema and is the natural implementation seed for
  `replay-parser-2 schema`.
- Existing parser-core fixtures cover metadata, entity drift, connected-player
  backfill, combat events, aggregate projection, side facts, vehicle score, and
  deterministic output.

### Established Patterns
- Parser-core is pure and transport-free. File I/O, command-line parsing,
  old-parser invocation, generated reports, benchmark timing, and non-deterministic
  timestamps belong in adapters or harness code.
- Contract/schema tests compare freshly generated schema against the committed
  schema and validate success/failure examples. Phase 5 should preserve this
  generated-from-Rust source of truth.
- Phase 1 committed compact dossiers and fixture indexes while keeping full
  corpus outputs, regenerated result trees, logs, and bulky reports under
  ignored `.planning/generated/` paths.
- Legacy game-type filters, skip rules, config inputs, and name compatibility
  belong to the parity harness or compatibility layer, not parser-core.
- Existing workspace lints are strict and should be inherited by any new CLI,
  harness, or benchmark crates unless a specific reviewed exception is needed.

### Integration Points
- Add a CLI crate or binary workspace member that depends on parser-core and
  parser-contract, uses the same parser-core API, and exposes
  `parse`, `schema`, and `compare`.
- Add comparison/harness code that applies legacy selection and compatibility
  rules outside parser-core, then emits categorized reports using the Phase 1
  mismatch taxonomy.
- Add coverage and mutation/fault commands or scripts that operate across all
  reachable production Rust crates in the workspace.
- Add benchmark harnesses that reuse curated fixture selectors and record parity
  status for each measured workload before making speed claims.

</code_context>

<specifics>
## Specific Ideas

- The public binary should intentionally differ from the legacy project name:
  use `replay-parser-2` for the Rust CLI and reserve `sg-replay-parser` for the
  old TypeScript baseline repo in docs and benchmark reports.
- The first comparison implementation should not try to solve every full-corpus
  mismatch. It should produce structured, reviewable reports that keep
  unexplained current-vs-regenerated and worker-profile drift in `human review`.
- The coverage gate should be framed as "100% reachable production behavior"
  with a reviewable allowlist, not as an incentive to write brittle tests around
  private implementation details.
- The 10x claim should be attached to explicit benchmark tiers and workloads,
  not a single unlabeled timing number.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope.

</deferred>

---

*Phase: 05-cli-golden-parity-benchmarks-and-coverage-gates*
*Context gathered: 2026-04-28T11:17:43+07:00*
