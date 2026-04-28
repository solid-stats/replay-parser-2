---
phase: 05
slug: cli-golden-parity-benchmarks-and-coverage-gates
status: executed
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-28
---

# Phase 05 - Validation Strategy

## Test Infrastructure

| Property | Value |
|----------|-------|
| Framework | Rust integration/unit tests through Cargo, plus CLI command tests through `assert_cmd` |
| Config file | `Cargo.toml`, `.cargo/config.toml`, future `coverage/allowlist.toml` |
| Quick run command | `cargo test --workspace` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps` |
| Estimated runtime | ~60-180 seconds locally before coverage/mutation/full-corpus optional runs |

## Sampling Rate

- After every task commit: run the most focused package test plus `git diff --check`.
- After every plan wave: run `cargo test --workspace`.
- Before `$gsd-verify-work`: run the full suite plus coverage, mutation/fault report validation, and benchmark report validation.
- Max feedback latency: target under 180 seconds for normal quick gates; mutation and manual full-corpus benchmarks are explicitly longer gates.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-00-01 | 00 | 1 | CLI-01 | T-05-00-01 | Local CLI only reads requested input and writes requested output. | integration | `cargo test -p parser-cli parse_command` | yes | passed |
| 05-00-02 | 00 | 1 | CLI-02 | T-05-00-02 | Schema output comes from contract source of truth. | integration | `cargo test -p parser-cli schema_command` | yes | passed |
| 05-00-03 | 00 | 1 | CLI-04 | T-05-00-03 | Failure artifacts are structured and stderr is not the primary machine output. | integration | `cargo test -p parser-cli parse_failure_command` | yes | passed |
| 05-01-01 | 01 | 2 | TEST-01 | T-05-01-01 | Fixture manifest prevents untraceable or bulky golden data. | unit | `cargo test -p parser-core golden_fixture_manifest` | yes | passed |
| 05-01-02 | 01 | 2 | TEST-03, TEST-08, TEST-10, TEST-11 | T-05-01-02 | Fixtures cover edge behavior through public parser APIs. | integration | `cargo test -p parser-core golden_fixture_behavior` | yes | passed |
| 05-02-01 | 02 | 3 | CLI-03, TEST-02 | T-05-02-01 | Comparison reports require mismatch categories and impact dimensions. | unit | `cargo test -p parser-harness comparison_report` | yes | passed |
| 05-02-02 | 02 | 3 | CLI-03, TEST-02 | T-05-02-02 | Compare CLI does not mutate old or current result trees. | integration | `cargo test -p parser-cli compare_command` | yes | passed |
| 05-03-01 | 03 | 4 | TEST-07, TEST-08, TEST-09, TEST-10, TEST-11 | T-05-03-01 | Coverage allowlist is explicit and behavior tests remain public API based. | script + tests | `scripts/coverage-gate.sh --check` | yes | passed |
| 05-04-01 | 04 | 5 | TEST-12 | T-05-04-01 | Fault report blocks high-risk missed cases. | script + tests | `scripts/fault-report-gate.sh` | yes | passed |
| 05-05-01 | 05 | 6 | TEST-04, TEST-05, TEST-06 | T-05-05-01 | Benchmark reports include parity status before speed claims. | bench/report | `cargo test -p parser-harness benchmark_report` | yes | passed; CI 10x status unknown |
| 05-05-02 | 05 | 6 | CLI-01..CLI-04, TEST-01..TEST-12 | T-05-05-02 | Final docs do not claim worker/server/UI ownership. | full suite | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo doc --workspace --no-deps` | yes | passed |

## Wave 0 Requirements

- No standalone Wave 0 plan is required.
- Plan 00 creates CLI test infrastructure.
- Plan 01 creates fixture manifest infrastructure.
- Plan 03 creates coverage allowlist and command wrappers.
- Plan 04 creates mutation/fault report command wrappers.
- Plan 05 creates benchmark report validators.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Optional manual full-corpus benchmark | TEST-04, TEST-05, TEST-06 | Full corpus timing may be too expensive for normal CI. | Run the documented manual benchmark command and confirm the report contains workload selector, parity status, old profile, throughput, memory/RSS note, and 10x status. |
| Human-review mismatch decisions | TEST-02 | Some current-vs-regenerated legacy drifts require domain approval. | Confirm report keeps unexplained drift as `human review` and does not auto-classify suspected legacy bugs as preserved or fixed. |

## Validation Sign-Off

- [x] All planned tasks have automated verification or explicit manual-only scope.
- [x] Sampling continuity: no three consecutive tasks lack an automated verify command.
- [x] Wave 0 dependencies are represented in the first relevant execution plans.
- [x] No watch-mode commands are used.
- [x] Feedback latency target is documented.
- [x] `nyquist_compliant: true` set in frontmatter.

Approval: execution complete; Phase 5 verification pending
