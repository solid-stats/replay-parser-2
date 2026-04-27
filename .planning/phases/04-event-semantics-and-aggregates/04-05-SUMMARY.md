---
phase: 04-event-semantics-and-aggregates
plan: 05
subsystem: parser-core
tags: [rust, parser-core, side-facts, commander, outcome, source-refs]

requires:
  - phase: 04-event-semantics-and-aggregates
    provides: Phase 04 Plan 00 replay-side facts contract and Phase 3 observed entities
provides:
  - Conservative top-level string evidence helpers for outcome fields
  - Replay-side outcome normalization with explicit known and unknown states
  - Commander candidate facts with confidence, rule IDs, and source references
  - Successful parser-core artifacts populated with typed side_facts
  - Behavior tests for winner, missing winner, commander candidate, canonical boundary, and warning cases
affects: [parser-core, parser-contract-consumers, server-2-commander-stats, phase-04-plan-06]

tech-stack:
  added: []
  patterns:
    - Top-level outcome evidence stays raw until side_facts normalization accepts a known side alias
    - Commander heuristics emit candidate facts only, never canonical commander truth

key-files:
  created:
    - crates/parser-core/src/side_facts.rs
    - crates/parser-core/tests/side_facts.rs
    - crates/parser-core/tests/fixtures/side-facts.ocap.json
  modified:
    - crates/parser-core/src/raw.rs
    - crates/parser-core/src/lib.rs
    - crates/parser-core/src/artifact.rs

key-decisions:
  - "Winner/outcome is known only from accepted top-level side aliases; absent or unrecognized values remain explicit unknown."
  - "Commander detection emits source-backed candidate facts with 0.6 confidence and does not introduce canonical player identity."
  - "Missing commander or winner data remains a successful artifact condition unless other data-loss diagnostics exist."

patterns-established:
  - "Side-fact behavior tests parse through the public parser-core API and assert observable artifact output."
  - "Outcome warnings use DiagnosticImpact::NonLossWarning so unrecognized values do not force partial status."

requirements-completed: [PARS-10, PARS-11]

duration: 8m30s
completed: 2026-04-27
---

# Phase 04 Plan 05: Commander and Outcome Side Facts Summary

**Typed replay-side commander candidates and winner/outcome facts with conservative known/unknown semantics and source-backed confidence metadata.**

## Performance

- **Duration:** 8m30s
- **Started:** 2026-04-27T11:56:07Z
- **Completed:** 2026-04-27T12:04:37Z
- **Tasks:** 4
- **Files modified:** 6

## Accomplishments

- Added `RawStringCandidate` and `string_candidates` for top-level outcome evidence without assigning semantics in the raw adapter.
- Added parser-core `normalize_side_facts` for explicit winner aliases, unknown outcome fallback, non-loss warnings, and commander keyword candidates.
- Wired `normalize_side_facts` into successful artifact assembly before diagnostics are finalized.
- Added focused side-fact fixture and behavior tests covering known winner, missing winner, commander candidate metadata, canonical identity boundary, and unrecognized outcome warnings.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add raw outcome field helpers** - `66efd97` (feat)
2. **Task 2: Implement side facts normalization** - `52981c0` (feat)
3. **Task 3: Wire side facts into artifact assembly** - `d3b3585` (feat)
4. **Task 4: Add side facts fixture and behavior tests** - `9acc0e8` (test)

## Files Created/Modified

- `crates/parser-core/src/raw.rs` - Added top-level raw string candidate evidence helper.
- `crates/parser-core/src/side_facts.rs` - Normalizes explicit outcome facts and commander candidates.
- `crates/parser-core/src/lib.rs` - Exports the side facts module.
- `crates/parser-core/src/artifact.rs` - Populates `ParseArtifact.side_facts` during successful parse assembly.
- `crates/parser-core/tests/side_facts.rs` - Behavior tests for outcome and commander side facts.
- `crates/parser-core/tests/fixtures/side-facts.ocap.json` - Focused OCAP fixture with explicit WEST winner and commander candidate evidence.

## Decisions Made

- Accepted outcome aliases are intentionally narrow and source-field based: unknown or unrecognized values do not become winners.
- Commander keyword matches produce `CommanderFactKind::Candidate` with confidence `0.6`, rule ID `side_facts.commander.keyword_candidate`, and non-empty entity source refs.
- Parser-core keeps manual winner correction, canonical identity, persistence, APIs, and UI presentation outside parser ownership.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Applied rustfmt to plan-owned side facts changes**
- **Found during:** Task 4 (Add side facts fixture and behavior tests)
- **Issue:** `cargo fmt --all -- --check` reported formatting changes in files touched by this plan.
- **Fix:** Ran `cargo fmt --all` and re-ran the targeted task verification.
- **Files modified:** `crates/parser-core/src/raw.rs`, `crates/parser-core/src/side_facts.rs`, `crates/parser-core/tests/side_facts.rs`
- **Verification:** `cargo test -p parser-core side_facts`, `cargo fmt --all -- --check`, `git diff --check`
- **Committed in:** `9acc0e8`

---

**Total deviations:** 1 auto-fixed (1 blocking).
**Impact on plan:** Formatting only; no behavior or ownership scope changed.

## Issues Encountered

- Local `gsd-sdk query` is unsupported in this checkout, so execution state and metadata updates were handled manually as allowed by the prompt.
- One verification attempt started two Cargo commands in parallel and briefly waited on Cargo locks; both completed successfully, and final verification was run sequentially where relevant.

## Authentication Gates

None.

## Known Stubs

None found in created or modified files.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p parser-core side_facts` - passed
- `cargo test -p parser-core deterministic_output` - passed
- `cargo check -p parser-core --all-targets` - passed
- `cargo fmt --all -- --check` - passed
- `git diff --check` - passed

## Next Phase Readiness

Plan 04-06 can refresh schema/examples and run final quality gates against artifacts that now include combat events, aggregates, vehicle score inputs, and typed side facts. No canonical identity, server persistence, API, UI, queue, or storage responsibility was introduced.

## Self-Check: PASSED

- Summary file exists.
- Created side facts module, behavior test file, and side-facts fixture exist.
- Task commits `66efd97`, `52981c0`, `d3b3585`, and `9acc0e8` exist in git history.
- No unexpected file deletions were present in task commits.

---
*Phase: 04-event-semantics-and-aggregates*
*Completed: 2026-04-27*
