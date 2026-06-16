# Deep Brainstorm Brief — replay-parser-2 Skill-Conformance Refactor

## Context
- **Date:** 2026-06-14
- **Request:** Carry the *process* lessons from the `replays-fetcher` Track C pilot into the `replay-parser-2` refactor, per `plans/product/RELEASE-PLAN.md` Phase 2 / W3 ("most of the parser was written without a skill; bring it under `solidstats-parser-rust-*`").
- **GSD stage:** pre-`new-milestone` for parser v1.1 (parser is v1.0 DONE, "Awaiting next milestone").
- **Target outcome:** decision pack feeding `/gsd-new-milestone` for a parser v1.1 skill-conformance refactor, runnable **in parallel** with server-2 Track C.
- **Artifact owner:** orchestrator (this session).

## Goal
Bring `replay-parser-2` into compliance with `solidstats-parser-rust-conventions` (module layout, naming, error system, determinism discipline), **behavior-preserving** — golden-parity stays byte-identical and the deterministic-artifact invariants hold at every phase boundary — shipped as a v1.1 milestone.

**Hard distinction from server-2:** this is a **Rust skill-conformance** refactor, **NOT** a TS toolchain convergence. None of the fetcher's toolchain mechanics (Oxlint / Oxfmt / tsdown / Vitest / the `@solid-stats/ts-toolchain` preset) apply. Only the **GSD process discipline** transfers. The Rust analogues already exist and are strict.

## Users And Workflows
- **`server-2` (downstream):** consumes the deterministic parse artifacts; the refactor must not change artifact bytes (golden-parity is the contract).
- **CI / operators:** clippy / rustfmt / coverage-gate / golden-parity / fault-report stay the hard gate.

## Scope
### Must Have
- Refactor source into `solidstats-parser-rust-conventions` compliance: module/crate layout, naming, the error system, determinism discipline — findings from a `solidstats-parser-rust-code-review` pass resolved.
- **Behavior-preserving:** golden fixtures (`crates/parser-core/tests/fixtures/golden`) stay byte-identical; determinism invariants hold.
- Each phase lands green on the full Rust gate (clippy + rustfmt + coverage + golden-parity + fault-report) — never deferred to CI.

### Nice To Have
- A short RUST-equivalent of the fetcher's RULE-DELTA if any lint disposition changes.
- Tighten any module that the conventions skill flags as structurally non-conforming.

### Non Goals
- **No** Oxlint/Oxfmt/tsdown/Vitest/`@solid-stats/ts-toolchain` — those are the TS track (server-2 / web).
- No parser output-shape change, no new artifact fields, no parity-methodology change.
- No RabbitMQ/S3 worker contract change.

## Confirmed Decisions
| Decision | Choice | Rationale | Consequence |
|----------|--------|-----------|-------------|
| **Separate track** | Parser gets its **own** decision pack + milestone, not merged with server-2 | Different language + different work (skill-conformance, not toolchain); the only overlap is GSD process | This file; planned independently |
| **What transfers** | GSD **process discipline** only: per-phase isolation, behavior-preserving + golden-parity gate at every boundary, atomic code+tests+docs commits, milestone-close **audit** | The fetcher's most valuable lesson was procedural (the audit caught a latent gap six verifications missed) — that transfers regardless of language | Parser milestone mirrors the GSD rigor, not the toolchain |
| **What does NOT transfer** | The entire TS toolchain/preset mechanics | Rust ≠ TS; the preset, oxlint/oxfmt/tsdown/vitest are out of scope | No preset dependency, no `verify`-surface swap |
| **Rust analogues already strict** | clippy (deny-heavy: rust + cargo + nursery + pedantic), rustfmt, coverage gate, golden-parity, fault-report; `#[allow]` permitted **only with a `reason =` justification** (~20 exist today, all justified) | `Cargo.toml` `[workspace.lints.rust]` is already deny-heavy; `clippy.toml` + `rustfmt.toml` present; golden fixtures + deterministic artifacts exist; CONVENTIONS permits justified allows | The "tooling migration" is effectively done; the work is **conformance to conventions**, not standing up tools |
| **Scheduling** | Run parser **v1.1 in parallel now** with server-2 Track C | Independent, no shared dependency; fewer files than server-2 (87 vs 185) though comparable scale (~25.6k LOC) | Two milestones in flight (consistent with RELEASE-PLAN D3's "two milestones via AI agents" pattern) |

## Assumptions
| Assumption | Confidence | Evidence | How To Validate |
|------------|------------|----------|-----------------|
| The refactor can stay behavior-preserving | high | golden-parity fixtures + deterministic-artifact tests already exist and gate CI | Run the golden-parity + determinism suite at every phase boundary |
| Rust gates need no new tooling | high | `Cargo.toml` lints deny-heavy; `clippy.toml`/`rustfmt.toml` present; coverage gate + fault-report shipped in v1.0 | Confirm the conventions skill maps cleanly onto the existing gate set |
| Skill-conformance is mostly structural, not behavioral | medium | W3 says "written without a skill" — likely layout/naming/error-system drift, not logic bugs | Run `solidstats-parser-rust-code-review` first to size the gap before planning |

## Backend And Infrastructure Notes
| Topic | Decision/Default | Frontend Consequence | Hidden Cost | Breaking Point |
|-------|------------------|----------------------|-------------|----------------|
| Determinism invariants | parser-core purity (no I/O in core), deterministic `BTreeMap` artifacts, byte-identical golden output are **blocker-level** invariants | None directly; server-2 depends on stable artifact bytes | A careless refactor that introduces `HashMap` iteration order into an artifact, or I/O into parser-core, silently breaks shipped determinism | Any non-deterministic artifact byte fails golden-parity |
| Lint floor | clippy deny-heavy floor; `#[allow]` allowed **only with a `reason =` justification** (~20 justified allows exist today) | None | A refactor that suppresses a structural-complexity lint with a bare/un-justified `#[allow]` instead of fixing it violates the floor | A single **un-justified** `#[allow]` regresses the conventions |

## Risks
| Risk | Severity | Why It Matters | Mitigation |
|------|----------|----------------|------------|
| Refactor perturbs deterministic artifact bytes | 🔴 high | Breaks golden-parity → breaks server-2's contract | Golden-parity + determinism gate at every phase boundary, never deferred |
| `#[allow]` used to "pass" a clippy floor | 🟠 medium | Erodes the conventions the refactor is meant to enforce | Conventions review forbids un-justified allows; fix, don't suppress |
| I/O or non-determinism leaks into parser-core | 🟠 medium | Violates core purity invariant | Architecture review per the conventions skill; keep I/O at the worker edge |
| Scope creep into output-shape changes | 🟡 low | Turns a refactor into a redesign | Non-goal explicitly: no artifact field changes |

## Acceptance Criteria
- Source passes `solidstats-parser-rust-code-review` with findings resolved; ingest/determinism invariants intact.
- Golden-parity byte-identical before/after the refactor; determinism suite green.
- clippy (deny-heavy floor; no new **un-justified** `#[allow]` — justified allows with `reason =` are permitted per CONVENTIONS) + rustfmt + coverage gate + fault-report all green at every phase boundary.
- No artifact-shape / worker-contract change; server-2 consumes the same bytes.

## Verification Plan
- Per-phase: full Rust gate (clippy + rustfmt + coverage + golden-parity + fault-report) green locally before phase close.
- Golden-parity diff = empty across the golden fixture set.
- Determinism: re-run produces byte-identical artifacts (BTreeMap ordering preserved).
- Milestone-close audit (the transferable fetcher lesson): goal-backward check that the refactor delivered conformance without behavioral drift.

## Open Questions
| Priority | Question | Why It Matters | Owner/Status |
|----------|----------|----------------|--------------|
| P1 | How large is the conformance gap? | Sizes the milestone (phase count) | Run `solidstats-parser-rust-code-review` before `/gsd-new-milestone` |
| P1 | Phase structure: convention-per-section (lint floor / determinism+contract / observability / testing / closure) vs mirror the fetcher's 6-phase tool model | Rust refactor is section-shaped, not tool-shaped | Finalize in v1.1 planning |
| P2 | Any clippy lints currently `#[allow]`'d that the conventions forbid? | Each is a conformance item | Grep at plan time |

## Question Ledger
| Priority | Question | Answer | Decision Impact |
|----------|----------|--------|-----------------|
| P0 | Parser in scope with server-2 or separate? | **Both covered, separate files** | This pack stands alone |
| P1 | Schedule parser vs server-2 | **Parallel now** (independent, no shared dep; 87 files / ~25.6k LOC) | Two milestones in flight |
| P1 | Does TS toolchain transfer? | **No** — only GSD process; Rust analogues already strict | No preset/oxlint/oxfmt/tsdown/vitest work |

## Recommended Next GSD Step
- **Primary:** run `solidstats-parser-rust-code-review` over the parser to **size the conformance gap**, then `/gsd-new-milestone` for parser **v1.1 skill-conformance**.
- **Rationale:** the gap size determines the phase structure; the tooling/gates already exist, so the milestone is purely conformance + behavior-preservation — best scoped from a concrete review, not assumed.
- **Alternatives:** (a) `/gsd-new-milestone` directly if the gap is already understood; (b) sequence after server-2 if agent bandwidth is constrained (RELEASE-PLAN allows parallel).
