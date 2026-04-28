# Phase 5: CLI, Golden Parity, Benchmarks, and Coverage Gates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-04-28T11:17:43+07:00
**Phase:** 05-cli-golden-parity-benchmarks-and-coverage-gates
**Areas discussed:** CLI command contract, Golden parity policy, Coverage and fault gates, Benchmark evidence

---

## CLI Command Contract

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Public CLI contract | `sg-replay-parser` | Keep README's initially planned form and legacy-style naming. | |
| Public CLI contract | `replay-parser-2` | Match the new Rust repository identity and avoid confusion with the old parser repo. | yes |
| Public CLI contract | Planner decides | Leave binary naming and exact subcommands to the planner. | |

**User's choice:** Use `replay-parser-2` as the public binary name.
**Notes:** The tradeoff was analyzed on request. `replay-parser-2` better separates the new Rust project from the legacy TypeScript baseline, while README examples need updating.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Structured failure output | Artifact always | `--output` receives success or failed `ParseArtifact`; failures exit non-zero and stderr is only a concise summary. | yes |
| Structured failure output | Failure to stderr | `--output` is success-only; structured failure goes to stderr. | |
| Structured failure output | Separate failure path | Failure uses `--failure-output`. | |

**User's choice:** Artifact always.
**Notes:** This keeps local reproduction and audit artifacts deterministic.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Source metadata | Auto checksum | CLI computes SHA-256, sets source_file from input path, and accepts optional replay_id. | yes |
| Source metadata | Explicit metadata | User must pass checksum and replay identity flags. | |
| Source metadata | Metadata file | User supplies a separate metadata JSON. | |

**User's choice:** Auto checksum.
**Notes:** This reduces manual local-use errors.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Compare scope | CLI focused | `compare` covers selected replay/saved artifacts; full-corpus reports live in dev harness or CI scripts around the CLI. | yes |
| Compare scope | All in CLI | Put all single-file and full-corpus workflows in one binary. | |
| Compare scope | Harness only | CLI only has `parse` and `schema`. | |

**User's choice:** CLI focused.
**Notes:** Heavy full-corpus work remains outside the public command surface while still built around the same parser core.

---

## Golden Parity Policy

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Parity truth source | Dual evidence | Compare against current `~/sg_stats/results` and regenerated old outputs; baseline drift remains `human review`. | yes |
| Parity truth source | Regenerated old only | Treat rerun old-parser output as canonical. | |
| Parity truth source | Current results only | Treat historical published result tree as canonical. | |

**User's choice:** Dual evidence.
**Notes:** This preserves both historical and regenerated evidence without hiding known drift.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Legacy worker profile | WC1 primary | `WORKER_COUNT=1` is the primary semantic baseline; default-worker output is diagnostic. | yes |
| Legacy worker profile | Default primary | Treat production-like default worker output as primary. | |
| Legacy worker profile | Both equal | Require both worker profiles as equal baselines. | |

**User's choice:** WC1 primary.
**Notes:** The default-worker profile remains useful for drift and performance investigation.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Report shape | Categorized fields | Per-field/per-surface mismatches with Phase 1 taxonomy and impact dimensions. | yes |
| Report shape | Compact pass/fail | Summary counts and overall status only. | |
| Report shape | Raw diff dump | Store detailed raw JSON diffs. | |

**User's choice:** Categorized fields.
**Notes:** Required for human-review diffs and cross-application impact assessment.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Golden fixtures | Curated compact | Commit small focused fixtures plus selected real-corpus seeds; keep full corpus generated/ignored. | yes |
| Golden fixtures | Broad committed set | Commit many real replay samples. | |
| Golden fixtures | Generated only | Generate fixtures only from local `~/sg_stats`. | |

**User's choice:** Curated compact.
**Notes:** Carries forward the Phase 1 compact-dossier approach.

---

## Coverage and Fault Gates

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Coverage tool | llvm-cov strict | Use `cargo llvm-cov` with zero uncovered lines/functions/regions and branch where supported. | yes |
| Coverage tool | Any tool ok | Planner can choose among llvm-cov, tarpaulin, grcov, or equivalent. | |
| Coverage tool | Custom script | Build custom reporting around Cargo/LLVM output. | |

**User's choice:** llvm-cov strict.
**Notes:** Existing stable Rust quality gates remain in force.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Coverage scope | Production Rust | All reachable production Rust in contract/core/CLI/harness crates; tests excluded. | yes |
| Coverage scope | Everything Rust | Include tests, examples, and benches too. | |
| Coverage scope | Core only | Gate parser-core and contract only. | |

**User's choice:** Production Rust.
**Notes:** Generated/impossible glue may only be excluded through allowlist.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Exclusions | Reviewable allowlist | Committed allowlist plus inline rationale. | yes |
| Exclusions | Inline only | Comments and attributes near code only. | |
| Exclusions | No exclusions | No exclusions under any circumstances. | |

**User's choice:** Reviewable allowlist.
**Notes:** Blanket module exclusions are not acceptable.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Mutation/fault report | Targeted required | Required for high-risk parser-core and CLI failure paths only. | |
| Mutation/fault report | Full workspace | Blocking report across all production workspace crates. | yes |
| Mutation/fault report | Advisory only | Report but do not block completion. | |

**User's choice:** Full workspace.
**Notes:** A tradeoff warning was presented; the user confirmed the stronger blocking gate.

---

## Benchmark Evidence

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Benchmark model | Two-tier | Criterion/core-stage benchmarks plus command-level old-vs-new runs on equivalent replay sets. | yes |
| Benchmark model | E2E only | Only compare full CLI/harness against old parser. | |
| Benchmark model | Parse-only only | Apply 10x target only to pure parser core. | |

**User's choice:** Two-tier.
**Notes:** Reports must state which tier meets or misses the 10x target.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Workloads | Tiered samples | Small CI sample, curated representative sample, and optional/manual full-corpus run. | yes |
| Workloads | Full corpus only | Use only full-corpus evidence. | |
| Workloads | Small sample only | Use only a fast sample. | |

**User's choice:** Tiered samples.
**Notes:** Each report records fixture list and parity status.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Metrics | Full metrics | files/sec, MB/sec or events/sec, wall time, memory/RSS, old baseline profile, parity status, and 10x status. | yes |
| Metrics | Speed only | Wall time and relative speed only. | |
| Metrics | Perf plus parity | Speed metrics plus parity status, memory optional. | |

**User's choice:** Full metrics.
**Notes:** Memory/RSS is required where practical.

| Question | Option | Description | Selected |
|----------|--------|-------------|----------|
| Under 10x policy | Block and triage | Phase 5 blocks until bottleneck is explained and fixed or accepted explicitly. | yes |
| Under 10x policy | Report only | Complete Phase 5 with honest benchmark report. | |
| Under 10x policy | Optimize now | Expand Phase 5 until 10x is met. | |

**User's choice:** Block and triage.
**Notes:** Correctness and parity must remain verified during performance triage.

## the agent's Discretion

- Exact CLI crate/package split.
- Exact flag spelling beyond `parse`, `schema`, and `compare` responsibilities.
- Exact deterministic report formats.
- Exact curated fixture list, provided it traces to Phase 1 and Phase 4 evidence.

## Deferred Ideas

None.
