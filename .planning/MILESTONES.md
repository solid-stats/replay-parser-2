# Milestones

## v1.0 Parser Worker Readiness (Shipped: 2026-05-09)

**Phases completed:** 10 phases, 56 plans, 141 tasks

**Known deferred items at close:** 18 open artifact-audit items acknowledged as
non-blocking; see `.planning/STATE.md` Deferred Items.

**Key accomplishments:**

- Generated evidence hygiene plus a repaired canonical legacy source-command preflight for Phase 1 baseline work
- Reproducible legacy parser baseline dossier with two isolated full-corpus runs and D-08 comparison evidence
- Full-history corpus manifest for 23,473 raw replay JSON files with compact fixture seed coverage
- Legacy rule and output-surface inventory for parity harness ownership and identity compatibility boundaries
- Mismatch taxonomy and README baseline update tying Phase 1 evidence to parser, server, and web impact
- Rust parser contract crate foundation with pinned toolchain, semver-backed version metadata, and tests separating contract_version from parser.version
- Unified parse artifact envelope with exact status values, source identity, path-based diagnostics, and a validated success artifact example
- Explicit presence-state contract for replay metadata and observed identity without canonical matching
- Auditable event and aggregate skeletons with validated stable rule IDs
- Machine-checkable ParseArtifact success/failure contract generated from Rust types
- Machine-checkable parser contract invariants for checksums, failed artifacts, source references, error codes, and inferred confidence
- Typed observed entity name/class fields with non-empty provenance and compatibility hints for Phase 3 parser-core population
- Pure parser-core crate with deterministic artifact shells and structured JSON/root failures
- Tolerant OCAP top-level extraction with deterministic ReplayMetadata, explicit unknowns, and path-based drift diagnostics
- Sorted observed unit/player, vehicle, and static weapon facts with source refs and drift diagnostics
- Capped diagnostics with partial-status escalation and byte-identical parser-core serialization tests
- Auditable connected-player inferred identity and duplicate same-name compatibility hints without entity merging
- Schema-visible combat, aggregate contribution, vehicle score, and replay-side fact contracts for Phase 4 parser-core work.
- Tolerant parser-core killed tuple observations and event-coordinate source refs for later combat semantics and auditable aggregates.
- Source-backed killed tuple normalization with explicit combat semantics, bounty exclusions, legacy effects, and partial diagnostics for unauditable actors.
- Auditable per-replay legacy, relationship, game-type, squad, rotation, and bounty projections derived from normalized combat events.
- Issue #13 vehicle score inputs with mapped categories, auditable weights, teamkill penalty clamp evidence, and per-replay denominator rows.
- Typed replay-side commander candidates and winner/outcome facts with conservative known/unknown semantics and source-backed confidence metadata.
- Populated Phase 4 artifacts now serialize deterministically, validate against the committed schema, and are documented as complete without claiming later-phase runtime surfaces.
- Local `replay-parser-2` CLI with deterministic parse artifacts, contract-backed schema export, and command-level failure/determinism tests
- Traceable compact golden fixture manifest with parser-core behavior regression tests for malformed, partial, winner, vehicle, teamkill, commander, null-killer, duplicate-slot, and connected-player cases
- Reusable old-vs-new comparison reports plus public `replay-parser-2 compare` command for selected artifacts or selected replay files
- Strict reachable-production coverage gate using `cargo llvm-cov` JSON evidence and a reviewable exact-line allowlist
- Mutation or equivalent deterministic fault-injection gate for high-risk parser-core and aggregate behavior
- Benchmark report validation, CI benchmark entrypoint, README handoff, and final Phase 5 execution gates
- Compatibility review and user-approved gate for replacing full parser dumps with compact server-facing artifacts
- Compact parser contract envelope with participant refs, combat facts, contribution facts, summaries, and refreshed schema/examples
- Product-owner-approved gate for replacing the compact parser artifact with minimal flat v1 statistics tables
- v3 parser contract with minimal flat tables and no active issue 13 vehicle-score surfaces
- parser-core now returns v3 minimal flat artifacts by default with retired issue 13 vehicle-score code and an explicit deterministic debug sidecar
- CLI parse now emits minified v3 minimal artifacts by default, with explicit pretty formatting, opt-in debug sidecars, and v3 public schema/docs
- Old-vs-new parity now derives legacy comparison surfaces from v3 minimal tables, excludes vehicle-score output, and keeps Markdown as the default review format
- Phase 05.2 benchmark reports now enforce selected x3, all-raw x10, zero-failure, percentile, and exact 100000-byte artifact gates
- Final Phase 05.2 quality gates pass structurally, but Phase 6 remains blocked by benchmark acceptance evidence
- Schema-backed RabbitMQ worker job/result envelopes with artifact-reference success messages and structured failed-message identifiers
- RabbitMQ/S3 worker crate shell with validated env/flag configuration, redacted startup diagnostics, and a thin `replay-parser-2 worker` CLI delegate
- S3-compatible worker storage boundary with local SHA-256 verification, encoded deterministic artifact keys, and checksum/size-guarded artifact reuse
- RabbitMQ adapter with manual prefetch-1 consumption, confirmed parse result publishing, and ack-after-confirm delivery policy
- Worker job processor with checksum-before-parse, shared CLI artifact bytes, confirmed result publication, and one-job shutdown drain
- Final Phase 6 gate run with worker schema freshness, full-corpus benchmark acceptance, README worker-mode documentation, and Phase 7 handoff
- Race-safe deterministic artifact writes using S3 conditional create, exact checksum/size reuse, and processor-level duplicate redelivery proof
- Cached container probes with worker identity, `/livez`/`/readyz` JSON, S3 readiness checks, and shutdown readiness draining
- Stable worker operations logs with worker identity, decision-point fields, durations, and redaction boundaries
- Container evidence for two worker instances, probes, duplicate artifact reuse, artifact conflict, structured logs, and Timeweb S3 compatibility hooks
- Final evidence and documentation handoff for parallel worker and container readiness hardening

---
