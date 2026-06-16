# Roadmap: replay-parser-2

## Milestones

- [x] **v1.0 Parser Worker Readiness** — Phases 1-7 shipped on 2026-05-09. Full archive: [v1.0-ROADMAP.md](./milestones/v1.0-ROADMAP.md).
- [ ] **v1.1 Skill-Conformance Refactor** — Phases 8-12 (in progress). Behavior-preserving conformance of the parser source to `solidstats-parser-rust-conventions`.

## Current Status

Milestone **v1.1 "Skill-Conformance Refactor"** is active. The goal is to bring `replay-parser-2`
into compliance with `solidstats-parser-rust-conventions` (§A–§M) **behavior-preserving** — golden
fixtures stay byte-identical, the published parse-artifact / worker-message bytes and JSON Schema do
not change, `server-2` consumes the same bytes, and the full Rust gate (rustfmt + clippy deny-heavy +
cargo test + coverage-gate + fault-report) is green at **every phase boundary** (never deferred to CI).

This is a Rust skill-conformance refactor, NOT a TS toolchain migration; it runs in parallel with
`server-2` Track C. Sizing evidence: [`milestones/v1.1-SIZING.md`](./milestones/v1.1-SIZING.md);
decision pack: [`milestones/DEEP-BRAINSTORM.md`](./milestones/DEEP-BRAINSTORM.md).

Start phase planning with:

```bash
$gsd-plan-phase 8
```

## v1.1 Skill-Conformance Refactor (In Progress)

**Milestone Goal:** Conform the parser source to `solidstats-parser-rust-conventions` (§A–§M) with
zero behavioral drift. Re-arm the quality gates everything else relies on, raise the lint floor and
structure to convention, harden core determinism/totality/errors, make the worker resilient and
observable, then audit goal-backward that conformance landed without changing a single shipped byte.

**Milestone invariant (carried into every phase's success criteria): behavior-preserving.** Golden
fixtures byte-identical, published parse-artifact / worker-message bytes and JSON Schema unchanged,
`server-2` consumes the same bytes, full Rust gate green at each phase boundary.

**Out of scope** (deferred to a coordinated contract-version bump with `server-2`): §E contract-facing
domain newtypes (~125 sites, schema-affecting), §G `#[non_exhaustive]` on public enums + §G newtype-hide
of `ParseArtifact` `Vec` fields (schema/semver-affecting or §E-wildcard-conflicting), and the TS
toolchain (server-2/web track). See [`REQUIREMENTS.md`](./REQUIREMENTS.md) Out of Scope.

### Phases

**Phase Numbering:** continued from v1.0 (no reset). v1.0 ended at Phase 7; v1.1 runs Phases 8-12.
Integer phases are planned milestone work; decimal phases (e.g. 8.1) would be urgent insertions.

- [ ] **Phase 8: Quality-Gate Restoration** - Re-arm the strict coverage gate in CI and add supply-chain / contract-drift / overflow gates.
- [ ] **Phase 9: Lint Floor & Structure** - Convert `#[allow]`→`#[expect(reason=)]`, config-once promotion, structural split, dep-table reconcile.
- [ ] **Phase 10: Core Determinism, Totality & Errors** - Sort-key stability guard, non-finite-float guard, `thiserror`/`Option` cleanups, `deny_unknown_fields`, core newtypes.
- [ ] **Phase 11: Worker Resilience & Observability** - lapin auto-recovery, S3 timeouts + read cap, `parse_job` span hierarchy, upstream error detail.
- [ ] **Phase 12: Milestone Conformance Audit** - Goal-backward audit that conformance landed with zero behavioral drift.

## Phase Details

### Phase 8: Quality-Gate Restoration
**Goal**: Re-arm the quality gates the rest of the milestone relies on — the strict reachable-code
coverage gate is currently OFF in CI, so it goes first. Renew/resolve the coverage allowlist, wire
`coverage-gate.sh --strict` into CI as a blocking step, add supply-chain and contract-drift gates, and
turn on release overflow checks. (§C / §J / §G — process & build only, no source-behavior change.)
**Depends on**: Phase 7 (v1.0 close)
**Requirements**: GATE-01, GATE-02, GATE-03, GATE-04, GATE-05, GATE-06
**Success Criteria** (what must be TRUE):
  1. Every `coverage/allowlist.toml` entry is renewed (fresh owner review + live expiry) or resolved by a new test — no expired entry remains (GATE-01).
  2. CI runs `scripts/coverage-gate.sh --strict` (with `COVERAGE_ALLOW_HEAVY=1`) as a blocking verify-job step that fails the build on any unallowlisted uncovered production region (GATE-02).
  3. CI runs `cargo-deny` (advisories/licenses/bans/sources) from a committed `deny.toml` and `cargo-audit` against the pinned `Cargo.lock`, both blocking (GATE-03, GATE-04).
  4. CI runs `cargo-semver-checks` against the last published `parser-contract` and diffs the committed JSON Schema, blocking on any contract/schema drift (GATE-05).
  5. `[profile.release]` sets `overflow-checks = true`, and the golden/determinism suite + full Rust gate stay byte-identical and green (GATE-06; invariant).
**Plans**: TBD

### Phase 9: Lint Floor & Structure
**Goal**: Raise the parser to the §B lint floor and §A structure without suppression. Convert every
production `#[allow]` to `#[expect(..., reason=)]`, delete redundant test-module suppressions already
covered by `clippy.toml`, split the oversized `raw_compact.rs` test fixture instead of suppressing
`too_many_lines`, promote codebase-wide single-lint noise to the workspace lints table once, and
reconcile the `parser-cli → parser-worker` dependency edge against the conventions §A model. (§B / §A.)
**Depends on**: Phase 8
**Requirements**: LINT-01, LINT-02, LINT-03, LINT-04, LINT-05
**Success Criteria** (what must be TRUE):
  1. No un-reasoned `#[allow]` remains on the production path — every one is a `#[expect(..., reason=)]` (LINT-01).
  2. Redundant test-module suppressions already covered by `clippy.toml` (`allow-expect-in-tests`) are deleted, and `cargo clippy --workspace --all-targets -D warnings` stays green (LINT-02).
  3. The `too_many_lines` suppression at the `raw_compact.rs` test fixture is gone because the fixture is split (structural-gate split, never suppressed) (LINT-03).
  4. Codebase-wide single-lint suppressions (e.g. `trivially_copy_pass_by_ref` ×8) are promoted once to `[workspace.lints.clippy]` rather than repeated per-site (LINT-04).
  5. The `parser-cli → parser-worker` dependency edge is reconciled with the conventions §A dependency model, with a RULE-DELTA documented; golden/determinism bytes unchanged (LINT-05; invariant).
**Plans**: TBD

### Phase 10: Core Determinism, Totality & Errors
**Goal**: Harden the parser core's determinism, malformed-input totality, and error system — the one
cluster where a careless edit could break golden parity, so it carries a behavior-critical sort-key
audit. Guard the artifact-path sort sites against a non-total comparator desyncing output, guard
non-finite floats before they reach a derived field (`null` → typed `Unknown`, with a `server-2`
heads-up), convert by-construction `expect()` to typed/`Option` handling, derive `thiserror` for the
CLI error, add `deny_unknown_fields` to the untrusted job message, and introduce core-internal
newtypes that never touch the serialized artifact. (§C / §D / §E-core / §F.)
**Depends on**: Phase 9
**Requirements**: DET-01, CORE-01, CORE-02, CORE-03, CORE-04, CORE-05
**Success Criteria** (what must be TRUE):
  1. A determinism guard keeps the 3 artifact-path sort sites stable (no `sort_unstable*`), so the non-total `compare_entities` comparator cannot desync output run-to-run; re-run produces byte-identical artifacts (DET-01; invariant).
  2. Non-finite `f64` is guarded with `is_finite()` before entering a derived artifact field (`null` → typed `Unknown`), covered by a proptest over overflowing-magnitude floats; golden fixtures (which carry no non-finite input) stay byte-identical (CORE-01).
  3. The by-construction `expect()` sites on the non-test path are converted to typed/`Option` handling, and `parser-cli` `CliError` derives `thiserror` instead of hand-rolling `Display`/`source` (CORE-02, CORE-03).
  4. `ParseJobMessage` and its untrusted-input nested types carry `#[serde(deny_unknown_fields)]` (CORE-04).
  5. Core-internal intermediate domain values use newtypes that are behavior-neutral and never serialized into the artifact; full Rust gate green (CORE-05; invariant).
**Plans**: TBD

### Phase 11: Worker Resilience & Observability
**Goal**: Make the RabbitMQ/S3 worker resilient and observable per §H/§K/§L — worker-behavior change
only, never a contract/artifact change. Enable lapin auto-recovery so a recoverable broker blip is a
reconnect rather than a clean worker exit, bound S3 with operation timeouts and a read size-cap before
buffering into memory, open a `parse_job` span hierarchy keyed by `replay_id`/`job_id` with child
spans for download and parse, and enrich S3/lapin error paths with SDK status/request-id detail before
propagating. (§H / §K / §L.)
**Depends on**: Phase 10
**Requirements**: WORK-01, WORK-02, WORK-03, WORK-04, WORK-05
**Success Criteria** (what must be TRUE):
  1. The lapin connection enables `enable_auto_recover()` + `wait_for_recovery()`; a broker stream-end / failover is treated as reconnect, not a clean worker exit (WORK-01).
  2. The S3 client sets explicit `operation_timeout` + `operation_attempt_timeout`, so a stalled read cannot hang the serial worker (WORK-02).
  3. The worker bounds the S3 object read (checks `content_length` / `Read::take`) before buffering the object into memory (WORK-03).
  4. The job handler opens a `parse_job` span (`replay_id` / `job_id`) with child spans for the S3 download and the parse call (WORK-04).
  5. S3 / lapin error paths log the SDK status / request-id (S3) or channel error kind (lapin) before propagating; published parse-result / worker-message bytes and JSON Schema are unchanged (WORK-05; invariant).
**Plans**: TBD

### Phase 12: Milestone Conformance Audit
**Goal**: Goal-backward milestone-close audit confirming conformance landed across §A–§M with zero
behavioral drift. Prove the parser is convention-conformant and that nothing a downstream consumer can
observe changed: golden byte-identical end-to-end, determinism suite green, a fresh re-run byte-identical,
and the contract / artifact / worker-message bytes and JSON Schema untouched.
**Depends on**: Phase 11
**Requirements**: AUDIT-01
**Success Criteria** (what must be TRUE):
  1. A milestone-close goal-backward audit report confirms the in-scope conformance clusters (§A/§B/§C/§D/§F/§H/§J/§K/§L + core-§E) all landed, with out-of-scope items (§E contract newtypes, §G) explicitly deferred and untouched (AUDIT-01).
  2. Golden fixtures are byte-identical end-to-end and the determinism suite is green; a fresh re-run produces byte-identical artifacts (AUDIT-01; invariant).
  3. Contract / artifact / worker-message bytes and the committed JSON Schema are confirmed untouched against the v1.0 baseline (`cargo-semver-checks` + schema-diff clean) (AUDIT-01; invariant).
  4. The full Rust gate (rustfmt + clippy deny-heavy + cargo test + `coverage-gate.sh --strict` + `fault-report-gate.sh`) is green, confirming the gates re-armed in Phase 8 hold at milestone close (AUDIT-01; invariant).
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 8 → 9 → 10 → 11 → 12.

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 8. Quality-Gate Restoration | v1.1 | 0/TBD | Not started | - |
| 9. Lint Floor & Structure | v1.1 | 0/TBD | Not started | - |
| 10. Core Determinism, Totality & Errors | v1.1 | 0/TBD | Not started | - |
| 11. Worker Resilience & Observability | v1.1 | 0/TBD | Not started | - |
| 12. Milestone Conformance Audit | v1.1 | 0/TBD | Not started | - |

## Archived Phases

<details>
<summary>v1.0 Parser Worker Readiness — shipped 2026-05-09</summary>

- [x] Phase 1: Legacy Baseline and Corpus — completed 2026-04-25.
- [x] Phase 2: Versioned Output Contract — completed 2026-04-26.
- [x] Phase 3: Deterministic Parser Core — completed 2026-04-27.
- [x] Phase 4: Event Semantics and Aggregates — completed 2026-04-28.
- [x] Phase 5: CLI, Golden Parity, Benchmarks, and Coverage Gates — execution completed 2026-04-28; accepted gaps resolved through Phase 5.1, Phase 5.2, and v1.0 gap closure.
- [x] Phase 5.1: Compact Artifact and Selective Parser Redesign — inserted after Phase 5; superseded by Phase 5.2 acceptance.
- [x] Phase 5.2: Minimal Artifact and Performance Acceptance — completed 2026-05-02 with product-owner benchmark and malformed-file acceptance.
- [x] Phase 6: RabbitMQ/S3 Worker Integration — completed 2026-05-02.
- [x] Phase 7: Parallel and Container Hardening — completed 2026-05-02.

</details>

## Archive Links

- Requirements archive: [v1.0-REQUIREMENTS.md](./milestones/v1.0-REQUIREMENTS.md)
- Milestone audit: [v1.0-MILESTONE-AUDIT.md](./milestones/v1.0-MILESTONE-AUDIT.md)
- Milestone index: [MILESTONES.md](./MILESTONES.md)
- v1.1 sizing: [v1.1-SIZING.md](./milestones/v1.1-SIZING.md)
- v1.1 decision pack: [DEEP-BRAINSTORM.md](./milestones/DEEP-BRAINSTORM.md)

## Notes

- v1.0 phase directories (01–07, 05.1, 05.2) remain in `.planning/phases/` as completed execution
  history. v1.1 numbering continues at Phase 8; the existing v1.0 directories are not renumbered.
- Live Timeweb S3 validation remains deployer-run operational evidence requiring credentials; local
  MinIO/two-worker smoke covers parser-owned behavior.
- v1.1 is behavior-preserving: every phase boundary keeps golden fixtures byte-identical and the full
  Rust gate green locally, never deferred to CI.
