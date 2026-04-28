---
phase: 04
slug: event-semantics-and-aggregates
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-27
verified: 2026-04-28T10:29:01+07:00
---

# Phase 04 - Validation Strategy

Per-phase validation contract for event semantics and aggregate projection.

## Test Infrastructure

| Property | Value |
|----------|-------|
| Framework | Rust integration and contract tests through Cargo |
| Config file | `Cargo.toml`, `crates/parser-contract/Cargo.toml`, `crates/parser-core/Cargo.toml`, `clippy.toml`, `rustfmt.toml` |
| Quick run command | `cargo test -p parser-core` |
| Contract run command | `cargo test -p parser-contract schema_contract` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps` |
| Estimated runtime | under 90 seconds before full corpus parity exists |

## Sampling Rate

- After every task commit: run the plan-specific cargo test command in `<verify>`.
- After every plan wave: run `cargo test --workspace`.
- Before `$gsd-verify-work`: run the full suite command.
- Max feedback latency: one plan task before a targeted automated test runs.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-00-01 | 04-00 | 1 | PARS-08/PARS-10/PARS-11/AGG-02/AGG-08 | T-04-00-01 | Contract has typed, schema-visible event, aggregate, vehicle-score, and side-fact payloads | contract/schema | `cargo test -p parser-contract` | yes | passed |
| 04-01-01 | 04-01 | 2 | PARS-08/PARS-09 | T-04-01-01 | Raw killed tuple accessors preserve source coordinates without panics | unit | `cargo test -p parser-core raw_event_accessors` | yes | passed |
| 04-02-01 | 04-02 | 3 | PARS-08/PARS-09 | T-04-02-01 | Combat normalization classifies kills, deaths, teamkills, suicides, null killers, and vehicle victims from observed facts | integration | `cargo test -p parser-core combat_event_semantics` | yes | passed |
| 04-03-01 | 04-03 | 4 | AGG-01/AGG-02/AGG-03/AGG-04/AGG-05/AGG-06/AGG-07 | T-04-03-01 | Legacy, relationship, game-type, and bounty projections are derived only from event-backed contributions | integration | `cargo test -p parser-core aggregate_projection` | yes | passed |
| 04-04-01 | 04-04 | 5 | AGG-08/AGG-09/AGG-10/AGG-11 | T-04-04-01 | Vehicle score weights and teamkill clamp are source-reference-backed and recalculable | integration | `cargo test -p parser-core vehicle_score` | yes | passed |
| 04-05-01 | 04-05 | 5 | PARS-10/PARS-11 | T-04-05-01 | Missing commander/winner remains explicit unknown; candidates carry confidence and source refs | integration | `cargo test -p parser-core side_facts` | yes | passed |
| 04-06-01 | 04-06 | 6 | PARS-08..PARS-11/AGG-01..AGG-11 | T-04-06-01 | Final artifact assembly is deterministic, schema-valid, documented, and lint-clean | full | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo doc --workspace --no-deps` | yes | passed |

## Wave 0 Requirements

- No separate Wave 0 is required. Existing Cargo, rustfmt, clippy, docs, parser-contract tests, parser-core tests, and fixture directories are already present.
- Plan 04-00 extends contract/schema test infrastructure before parser-core depends on the new types.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cross-application compatibility | INT-02/AGG-06/AGG-11 | `server-2` and `web` have only brief-level docs in their repos | Verify plans keep final persistence, canonical identity, public APIs, manual winner correction, and UI presentation outside parser. |
| Suspected legacy bug triage | AGG-01/AGG-03 | Phase 1 requires human review before preserving or fixing suspected old bugs | If execution finds legacy code inconsistent with accepted Phase 4 decisions, record the mismatch category and ask for confirmation before changing semantics. |

## Security Threat Model

| ID | Threat | Severity | Mitigation | Verification |
|----|--------|----------|------------|--------------|
| T-04-00-01 | Artifact consumers cannot validate new semantic payloads | high | Add schema-visible contract tests and regenerate committed schema | `cargo test -p parser-contract schema_contract` |
| T-04-01-01 | Malformed events panic or lose source coordinates | high | Tolerant raw accessors preserve unknown events and diagnostics | `cargo test -p parser-core raw_event_accessors` |
| T-04-02-01 | Teamkills or suicides award normal kill/bounty counters | high | Combat tests cover same-side, suicide, null-killer, and enemy cases | `cargo test -p parser-core combat_event_semantics` |
| T-04-03-01 | Aggregate values cannot be audited or corrected | high | All projections derive from `AggregateContributionRef` with event ID/source refs/rule ID | `cargo test -p parser-core aggregate_projection` |
| T-04-04-01 | Vehicle score penalties are undercounted | high | Store raw and applied weights; clamp penalty below 1 to 1 | `cargo test -p parser-core vehicle_score` |
| T-04-05-01 | Missing winner/commander is treated as false or data loss | medium | Explicit unknown states; absence does not force partial status | `cargo test -p parser-core side_facts` |

## Validation Sign-Off

- [x] All planned deliverables have automated build, unit, integration, schema, or full-suite checks.
- [x] Sampling continuity: every plan has at least one targeted automated verification command.
- [x] Existing infrastructure covers Phase 4 requirements.
- [x] No watch-mode flags are required.
- [x] `nyquist_compliant: true` is set in frontmatter.

Approval: verified 2026-04-28T10:29:01+07:00.

## Phase-Level Verification Evidence

Passed on 2026-04-28:

- `cargo test -p parser-contract`
- `cargo test -p parser-core raw_event_accessors`
- `cargo test -p parser-core combat_event_semantics`
- `cargo test -p parser-core aggregate_projection`
- `cargo test -p parser-core vehicle_score`
- `cargo test -p parser-core side_facts`
- `cargo test -p parser-core deterministic_output`
- `cargo run -p parser-contract --example export_schema > /tmp/phase4-parse-artifact-v1.schema.json`
- `cmp /tmp/phase4-parse-artifact-v1.schema.json schemas/parse-artifact-v1.schema.json`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo doc --workspace --no-deps`
- `git diff --check`
- Boundary grep for PostgreSQL, queue/S3, public API/UI, replay discovery, and canonical identity terms.
