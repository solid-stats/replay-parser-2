# Requirements: replay-parser-2 — Milestone v1.1 (Skill-Conformance Refactor)

**Defined:** 2026-06-17
**Core Value:** Parse OCAP JSON replays quickly and deterministically into compact server-facing
artifacts — and keep that engine maintainable by conforming the source to
`solidstats-parser-rust-conventions`, behavior-preserving.

> **Milestone invariant (applies to every requirement):** behavior-preserving. Golden fixtures stay
> byte-identical, the published parse-artifact / worker-message bytes and JSON Schema do not change,
> `server-2` consumes the same bytes, and the full Rust gate is green at every phase boundary. Sizing
> evidence: `.planning/milestones/v1.1-SIZING.md`.

## Milestone v1.1 Requirements

### Quality Gates (§C / §J / §G)

- [ ] **GATE-01**: Every `coverage/allowlist.toml` entry is renewed (fresh owner review + live expiry) or resolved by a new test; no expired entry remains.
- [ ] **GATE-02**: CI runs `scripts/coverage-gate.sh --strict` (with `COVERAGE_ALLOW_HEAVY=1`) as a blocking verify-job step.
- [ ] **GATE-03**: CI runs `cargo-deny` (advisories, licenses, bans, sources) from a committed `deny.toml`.
- [ ] **GATE-04**: CI runs `cargo-audit` against the pinned `Cargo.lock`.
- [ ] **GATE-05**: CI runs `cargo-semver-checks` against the last published `parser-contract` and diffs the committed JSON Schema.
- [ ] **GATE-06**: `[profile.release]` sets `overflow-checks = true`.

### Lint Floor & Structure (§B / §A)

- [ ] **LINT-01**: Every production `#[allow(...)]` is converted to `#[expect(..., reason=)]`; no un-reasoned `#[allow]` remains.
- [ ] **LINT-02**: Redundant test-module suppressions already covered by `clippy.toml` (`allow-expect-in-tests`) are deleted.
- [ ] **LINT-03**: The `too_many_lines` suppression at the `raw_compact.rs` test fixture is removed by splitting the fixture (structural-gate split, never suppressed).
- [ ] **LINT-04**: Codebase-wide single-lint suppressions (e.g. `trivially_copy_pass_by_ref` ×8) are promoted once to `[workspace.lints.clippy]`.
- [ ] **LINT-05**: The `parser-cli → parser-worker` dependency edge is reconciled with the conventions §A dependency model (RULE-DELTA documented).

### Core Determinism, Totality & Errors (§C / §D / §E-core / §F)

- [ ] **DET-01**: A determinism guard keeps the 3 artifact-path sort sites stable (no `sort_unstable*`), so the non-total `compare_entities` comparator cannot desync output run-to-run.
- [ ] **CORE-01**: Non-finite `f64` is guarded (`is_finite()`) before entering a derived artifact field (`null` → typed `Unknown`); a proptest covers overflowing-magnitude floats. (Operator decision: `null`→`unknown` accepted; `server-2` heads-up.)
- [ ] **CORE-02**: The by-construction `expect()` sites on the non-test path are converted to typed/`Option` handling.
- [ ] **CORE-03**: `parser-cli` `CliError` derives `thiserror` instead of hand-rolling `Display`/`source`.
- [ ] **CORE-04**: `ParseJobMessage` and its untrusted-input nested types carry `#[serde(deny_unknown_fields)]`.
- [ ] **CORE-05**: Core-internal intermediate domain values use newtypes (behavior-neutral; never serialized into the artifact).

### Worker Resilience & Observability (§H / §K / §L)

- [ ] **WORK-01**: The lapin connection enables `enable_auto_recover()` + `wait_for_recovery()`; broker stream-end / failover is treated as reconnect, not a clean worker exit.
- [ ] **WORK-02**: The S3 client sets explicit `operation_timeout` + `operation_attempt_timeout`.
- [ ] **WORK-03**: The worker bounds the S3 object read (check `content_length` / `Read::take`) before buffering into memory.
- [ ] **WORK-04**: The job handler opens a `parse_job` span (`replay_id` / `job_id`), with child spans for the S3 download and the parse call.
- [ ] **WORK-05**: S3 / lapin error paths log the SDK status / request-id (S3) or channel error kind (lapin) before propagating.

### Milestone Audit

- [ ] **AUDIT-01**: A milestone-close goal-backward audit confirms conformance landed with zero behavioral drift — golden byte-identical end-to-end, determinism suite green, re-run byte-identical, contract / artifact / worker-message bytes and JSON Schema untouched.

## Out of Scope

Explicitly excluded from v1.1. Deferred to a coordinated contract-version bump with `server-2`
(tracked in `plans/replay-parser-2/briefs/replay-parser-2-contract-conformance-deferred.md`).

| Item | Reason |
|------|--------|
| §E contract-facing domain newtypes (~125 sites) | `#[serde(transparent)]` keeps bytes identical but schemars `$defs` indirection changes the JSON Schema → breaks behavior-preserving; needs server-2 coordination. |
| §G `#[non_exhaustive]` on growing public enums (~22) | Byte/schema-safe, but forces `_` wildcard arms that conflict with the §E exhaustive-match rule enforced this milestone; no external Rust consumer; low internal value — decide in the coordinated contract review. |
| §G newtype-hide of `ParseArtifact` `Vec` fields (3) | Transparent newtype is byte-identical but changes JSON Schema shape (schema-diff); low value. |
| TS toolchain (Oxlint/Oxfmt/tsdown/Vitest/`@solid-stats/ts-toolchain`) | server-2 / web track, not this Rust repo. |
| Parser output-shape / new artifact fields / parity-methodology change | Behavior-preserving non-goal; `server-2` must consume the same bytes. |
| RabbitMQ/S3 worker-contract change (DLX/prefetch posture) | server-2-owned coordination; §H.3 poison handling is conformant-by-design (publish-failed-then-ack). |

## Traceability

Each requirement maps to exactly one phase (continued numbering: v1.1 runs Phases 8-12).

| Requirement | Phase | Status |
|-------------|-------|--------|
| GATE-01 | Phase 8 | Pending |
| GATE-02 | Phase 8 | Pending |
| GATE-03 | Phase 8 | Pending |
| GATE-04 | Phase 8 | Pending |
| GATE-05 | Phase 8 | Pending |
| GATE-06 | Phase 8 | Pending |
| LINT-01 | Phase 9 | Pending |
| LINT-02 | Phase 9 | Pending |
| LINT-03 | Phase 9 | Pending |
| LINT-04 | Phase 9 | Pending |
| LINT-05 | Phase 9 | Pending |
| DET-01 | Phase 10 | Pending |
| CORE-01 | Phase 10 | Pending |
| CORE-02 | Phase 10 | Pending |
| CORE-03 | Phase 10 | Pending |
| CORE-04 | Phase 10 | Pending |
| CORE-05 | Phase 10 | Pending |
| WORK-01 | Phase 11 | Pending |
| WORK-02 | Phase 11 | Pending |
| WORK-03 | Phase 11 | Pending |
| WORK-04 | Phase 11 | Pending |
| WORK-05 | Phase 11 | Pending |
| AUDIT-01 | Phase 12 | Pending |

**Coverage:**
- v1.1 requirements: 23 total
- Mapped to phases: 23 (100%)
- Unmapped: 0

---
*Requirements defined: 2026-06-17 — milestone v1.1 (Skill-Conformance Refactor)*
*Traceability populated: 2026-06-17 — gsd-roadmapper (Phases 8-12)*
