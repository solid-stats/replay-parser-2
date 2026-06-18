---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Skill-Conformance Refactor
status: planning
last_updated: "2026-06-17T00:00:00.000Z"
last_activity: 2026-06-17
progress:
  total_phases: 5
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-16)

**Core value:** Parse OCAP JSON replays quickly and deterministically into compact server-facing statistics artifacts with enough contribution evidence for `server-2` to persist, audit, and use for public statistics.
**Current focus:** v1.1 Skill-Conformance Refactor — behavior-preserving conformance of the parser source to `solidstats-parser-rust-conventions` (§A–§M). Roadmap has 5 phases (8-12) mapping 23 requirements.

## Current Position

Phase: 8 — Quality-Gate Restoration (not started)
Plan: —
Status: Roadmap created; awaiting `/gsd-plan-phase 8`
Last activity: 2026-06-17 — v1.1 roadmap created (Phases 8-12, 23 requirements mapped, 100% coverage)

**Milestone invariant (carried into every phase):** behavior-preserving. Golden fixtures
byte-identical, published parse-artifact / worker-message bytes and JSON Schema unchanged, `server-2`
consumes the same bytes, full Rust gate (rustfmt + clippy deny-heavy + cargo test + coverage-gate +
fault-report) green at every phase boundary — never deferred to CI.

**v1.1 phase structure (continued numbering, no reset; v1.0 ended at Phase 7):**

| Phase | Goal | Requirements |
|-------|------|--------------|
| 8 — Quality-Gate Restoration | Re-arm strict coverage gate in CI + supply-chain/contract-drift/overflow gates (§C/§J/§G) | GATE-01..06 |
| 9 — Lint Floor & Structure | `#[allow]`→`#[expect(reason=)]`, config-once, structural split, dep-table reconcile (§B/§A) | LINT-01..05 |
| 10 — Core Determinism, Totality & Errors | Sort-key stability guard, non-finite-float guard, thiserror/Option, deny_unknown_fields, core newtypes (§C/§D/§E-core/§F) | DET-01, CORE-01..05 |
| 11 — Worker Resilience & Observability | lapin auto-recovery, S3 timeouts + read cap, parse_job spans, upstream error detail (§H/§K/§L) | WORK-01..05 |
| 12 — Milestone Conformance Audit | Goal-backward: conformance landed, zero behavioral drift | AUDIT-01 |

**Out of scope (deferred to coordinated contract bump with `server-2`):** §E contract-facing
newtypes (~125), §G `#[non_exhaustive]` + newtype-hide, TS toolchain. See REQUIREMENTS.md Out of Scope.

## Performance Metrics

**Velocity:**

- Total plans completed: 31
- Average duration: N/A
- Total execution time: 0.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 | 5 | - | - |
| 02 | 6 | - | - |
| 03 | 6 | 62m23s | 10m24s |
| 04 | 7/7 | 96m40s | 13m49s |
| 05 | 6/6 | 221m | 36m50s |
| 05.1 | 8/8 | executed; acceptance gap | - |
| 06 | 6/6 | executed | - |
| 07 | 5/5 | executed | - |

**Recent Trend:**

- Last 5 plans: N/A
- Trend: N/A

*Updated after each plan completion*
| Phase 02 P00 | 10m26s | 2 tasks | 18 files |
| Phase 02 P01 | 5m22s | 3 tasks | 10 files |
| Phase 02 P02 | 3m51s | 3 tasks | 4 files |
| Phase 02 P03 | 4m53s | 3 tasks | 5 files |
| Phase 02 P04 | 8m47s | 4 tasks | 11 files |
| Phase 02 P05 | planned | 4 tasks | 16 files |
| Phase 02 P05 | 7m24s | 4 tasks | 16 files |
| Phase 03 P00 | 11m44s | 2 tasks | 5 files |
| Phase 03 P01 | 6m39s | 2 tasks | 9 files |
| Phase 03 P02 | 14m | 2 tasks | 9 files |
| Phase 03 P03 | 11m | 2 tasks | 8 files |
| Phase 03 P04 | 7m | 3 tasks | 8 files |
| Phase 03 P05 | 12m | 4 tasks | 7 files |
| Phase 04 P00 | 14m | 4 tasks | 17 files |
| Phase 04 P01 | 5m31s | 3 tasks | 4 files |
| Phase 04 P02 | 8m27s | 3 tasks | 5 files |
| Phase 04 P03 | 11m45s | 4 tasks | 5 files |
| Phase 04 P04 | 8m27s | 3 tasks | 5 files |
| Phase 04 P05 | 8m30s | 4 tasks | 6 files |
| Phase 04 P06 | 40m | 4 tasks | 14 files |
| Phase 05 P00 | 22m | 4 tasks | 7 files |
| Phase 05 P01 | 13min | 3 tasks | 5 files |
| Phase 05 P02 | 65min | 4 tasks | 11 files |
| Phase 05 P03 | 73min | 3 tasks | 22 files |
| Phase 05 P04 | 14min | 3 tasks | 7 files |
| Phase 05 P05 | 34min | 4 tasks | 8 files |
| Phase 05.2 P00 | 1m | 2 tasks | 2 files |
| Phase 05.2 P01 | 10 min | 3 tasks | 18 files |
| Phase 05.2 P02 | 22m04s | 3 tasks | 21 files |
| Phase 05.2 P03 | 9m22s | 3 tasks | 5 files |
| Phase 05.2 P04 | 14m18s | 3 tasks | 4 files |
| Phase 05.2 P05 | 15m | 3 tasks | 6 files |
| Phase 05.2 P06 | 14m48s | 3 tasks | 16 files |
| Phase 06 P05 | 49m26s | 2 tasks | 23 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- v1.1 is a Rust skill-conformance refactor of `replay-parser-2` against `solidstats-parser-rust-conventions` (§A–§M), behavior-preserving — NOT a TS toolchain migration; runs in parallel with `server-2` Track C.
- Behavior-preserving is a blocker-level milestone invariant: golden fixtures byte-identical, published parse-artifact / worker-message bytes and JSON Schema unchanged, `server-2` consumes the same bytes, full Rust gate green at every phase boundary (never deferred to CI).
- The strict reachable-code coverage gate is currently OFF in CI, so Quality-Gate Restoration (Phase 8) runs first — the rest of the milestone relies on those gates being armed.
- §F non-finite float is fixed in-scope (operator decision 2026-06-16): `is_finite()` guard → typed `Unknown` (not raw `null`); `server-2` gets a heads-up that a previously-`null` field on a non-finite input now reads as `unknown`/absent; covered by a proptest over overflowing-magnitude floats.
- §E contract-facing domain newtypes (~125), §G `#[non_exhaustive]` on public enums, and §G newtype-hide of `ParseArtifact` `Vec` fields are OUT of scope (schema/semver-affecting or §E-wildcard-conflicting) — deferred to a coordinated contract-version bump with `server-2`.
- The §C sort-key total-ordering audit (Phase 10) is behavior-critical: a latent non-total comparator at the 3 artifact-path sort sites is the one thing that could break golden parity during the refactor.
- v1.1 numbering is continued (no reset): v1.0 ended at Phase 7, v1.1 runs Phases 8-12; v1.0 phase directories (01–07, 05.1, 05.2) stay as completed history and are not renumbered.
- V1 behavior must be grounded in the old TypeScript parser at `replays-parser`.
- Parser output preserves observed identifiers only; canonical player matching and PostgreSQL persistence belong to `server-2`.
- README.md must stay current and explicitly state that project development uses only AI agents plus GSD workflow.
- Completed work must leave the git tree clean by committing intended results; never delete completed work just to make status clean, and ask when unclear.
- AI agents must challenge requests that conflict with project logic, architecture, quality, maintainability, or proportional scope; explain the risk, offer safer alternatives, and ask for explicit confirmation before a risky override.
- Solid Stats consists of `replays-fetcher`, `replay-parser-2`, `server-2`, and `web`; tasks must be checked for compatibility with adjacent application contracts and ownership before execution.

### Roadmap Evolution

- v1.1 milestone "Skill-Conformance Refactor" opened 2026-06-16; sizing recorded in `.planning/milestones/v1.1-SIZING.md` (347 raw divergent sites, ≈165 byte-safe in-scope).
- v1.1 roadmap created 2026-06-17: 5 phases (8-12), 23 requirements, 100% coverage. Structure is the operator-confirmed convention-per-section / gates-first decomposition from v1.1-SIZING.md (§C-first ordering; P10 carries the §C sort-key audit, P11 carries §H worker resilience folded in from the completeness critic).
- Phase 5.1 inserted after Phase 5 (URGENT, v1.0): Compact Artifact and Selective Parser Redesign.
- Phase 05.2 inserted after Phase 5 (v1.0): Minimal Artifact and Performance Acceptance.

### Pending Todos

None yet.

### Blockers/Concerns

None blocking v1.1 planning. Sizing confirmed 0 blockers for the refactor itself: determinism is
clean, parser-core is pure, no contract break. The strict coverage gate being OFF in CI is addressed
first as Phase 8 (Quality-Gate Restoration), not a blocker. The §H requeue policy
(amqp.rs:108 `requeue=true`) is to be reconciled against the prior review's "poison→ack-after-publish
is conformant" finding during Phase 11 planning; DLX posture is `server-2`-owned coordination.

v1.0 close state (reference): `scripts/coverage-gate.sh --check` passes, focused
`cargo test -p parser-worker -p parser-cli` passes, `scripts/fault-report-gate.sh` passes,
`scripts/worker-smoke.sh` passes. Strict local coverage still requires explicit `COVERAGE_ALLOW_HEAVY=1`
opt-in to avoid accidental workstation freezes.

### Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| 260617-v7d | Golden container-e2e regression oracle (testcontainers worker e2e + fast byte-exact in-process consumer of one shared baseline; #[ignore] master-only pre-deploy gate) + real-corpus fixtures from Timeweb S3 (small/mid/large success + partial), byte-exact, Docker e2e green | 2026-06-18 | 3f90f6c | Verified | [260617-v7d-golden-container-e2e-regression-oracle-f](./quick/260617-v7d-golden-container-e2e-regression-oracle-f/) |

## Deferred Items

Items acknowledged and deferred at v1.0 milestone close on 2026-05-09:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| quick_task | 16 legacy quick-task directories reported by `audit-open` as `missing` status | acknowledged-non-blocking | v1.0 close |
| uat_gap | Phase 05 `05-UAT.md` reports `issues-found` with zero open scenarios; superseded by Phase 5.1/5.2 and v1.0 audit acceptance | acknowledged-non-blocking | v1.0 close |
| verification_gap | Phase 05.1 `05.1-VERIFICATION.md` reports `gaps_found`; superseded by Phase 5.2 acceptance and v1.0 audit acceptance | acknowledged-non-blocking | v1.0 close |

v1.1 contract-conformance deferrals (coordinated bump with `server-2`):

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| contract_conformance | §E contract-facing domain newtypes (~125 sites) | out-of-scope (schema-affecting) | v1.1 scoping |
| contract_conformance | §G `#[non_exhaustive]` on public enums + newtype-hide of `Vec` fields | out-of-scope (semver/schema-affecting) | v1.1 scoping |

## Session Continuity

Last session: 2026-06-17 — v1.1 roadmap created
Stopped at: Roadmap created for Phases 8-12; awaiting phase planning
Resume file: .planning/ROADMAP.md (v1.1 Skill-Conformance Refactor section)

**Completed Milestone:** v1.0 Parser Worker Readiness — Phases 1-7 (+ 5.1, 5.2) — shipped 2026-05-09
**Next Step:** Plan Phase 8 with `/gsd-plan-phase 8`.

## Operator Next Steps

- Branch per GSD convention for milestone code-work: `gsd/v1.1-skill-conformance` (at phase-planning time).
- Plan the first phase with `/gsd-plan-phase 8` (Quality-Gate Restoration — gates first because strict coverage is OFF in CI).
