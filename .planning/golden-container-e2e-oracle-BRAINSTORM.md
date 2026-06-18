# Golden Container E2E Oracle — Decision Pack

## Context
- Date: 2026-06-17
- Request: Build a behavioral regression oracle (golden end-to-end test) before the v1.1 behavior-preserving refactor reaches its risky phases. Based on the reusable brief at `/tmp/golden-integration-test-prompt.md`, tailored to this repo after verifying its claims against the live code.
- GSD stage: deep brainstorm → next is `/gsd-quick --full`.
- Target outcome: a slow, container-backed worker e2e that lives on `master` and turns red on any observable drift the refactor (Phases 9–12) could introduce, run as a separate pre-deploy gate.
- Artifact owner: parser maintainers (AI + GSD workflow).

## Goal
Pin the worker's current observable contract end-to-end against real ephemeral infrastructure, so a behavior-preserving refactor stays green and any drift fails before deploy. The fast suite already covers parser-core (`golden_fixture_behavior.rs`, `deterministic_output.rs`, `processor.rs` with fakes). This adds the layer none of those reach: a real S3 + RabbitMQ round-trip with full-byte artifact pinning over real replays.

## Users And Workflows
- Maintainers running the v1.1 refactor — the oracle is the safety net the milestone rides on.
- The deploy gate — runs on `master` before deploy, separate from the per-commit / per-phase gate. Minutes of runtime are acceptable.

## Scope
### Must Have
- testcontainers-backed e2e: ephemeral RabbitMQ + MinIO booted by the test, no manual env setup.
- Drive the real worker entry (`parser_worker::runner::run_until_cancelled` + the `CancellationToken` seam) with the live `S3ObjectStore` / `RabbitMqClient` impls.
- Real OCAP fixtures from `~/sg_stats`, committed gzipped, spread across success / partial / failed plus one large-entity replay.
- Full-contract assertions: the S3 artifact bytes equal a committed `*.expected.json` exactly; the `parse.completed` / `parse.failed` message contract; the S3 artifact key format + checksum; idempotency (duplicate redelivery → terminal state once, artifact write-if-absent).
- Proof of teeth: a deliberately injected behavioral change makes the oracle red, demonstrating it is not a smoke test.
- Isolation: `#[ignore]`, run via a dedicated `cargo test -- --ignored` in a `master`-only pre-deploy CI job.
- Reuse the same `*.expected.json` baselines in the fast in-process golden suite, so parser drift fails immediately during Phases 9–12 without waiting for the slow gate.

### Nice To Have
- Later, shrink the live S3/AMQP coverage allowlist if a coverage-instrumented run proves those adapter lines reachable. Out of scope now; the e2e runs outside coverage.

### Non Goals
- old-vs-new parity. Dropped — already proven, and reviving the legacy TS harness + Node/pnpm is barred by AGENTS.md absent a migration mandate. Verified absent: no `crates/parser-harness`, no CLI `compare` command, no parity references in `crates/` or `scripts/`.
- The fast per-phase v1.1 gate and its timing stay untouched.
- Production deploy and live Timeweb S3 validation stay deployer-run.

## Confirmed Decisions
| Decision | Choice | Rationale | Consequence |
|----------|--------|-----------|-------------|
| What to build | Full container e2e of the worker | User choice; closes the real-infra gap that fakes and the manual smoke leave open | New dev-dep `testcontainers`; Docker required on the runner |
| Where it lives | A branch off `master`, separate pre-deploy gate | The milestone rides on it as its baseline | Oracle pins pre-refactor behavior; the refactor must keep it green |
| Speed | Slow is fine; `#[ignore]` + `--ignored` | Container boot is inherently slow | Stays out of the fast suite and the coverage gate |
| old-vs-new parity | Drop | Proven; AGENTS.md forbids reintroducing it here | Pure Rust-output-vs-committed-Rust-baseline oracle |
| Baseline | Pin current bytes as-is | A regression oracle records today's behavior | Known tech-debt is pinned and commented, never "fixed" inside the oracle |
| Assertion strength | Full artifact bytes, exact compare | Only full-byte pinning catches arbitrary field drift | One committed expected JSON per fixture |

## Assumptions
| Assumption | Confidence | Evidence | How To Validate |
|------------|------------|----------|-----------------|
| Docker is available on the `master` pre-deploy runner | HIGH | User-confirmed | Confirm the runner has a Docker daemon |
| `live_smoke.rs` is the wiring to extend | HIGH | `crates/parser-worker/tests/live_smoke.rs` is an `#[ignore]` env-gated e2e | Read it in the research phase |
| `run_until_cancelled` is the clean entry seam | HIGH | `crates/parser-worker/src/runner.rs:46` | Confirm signature during research |
| `~/sg_stats` has small-enough real replays to commit gzipped | MEDIUM | PROJECT.md: 23,473 raw replays of varied size; every artifact ≤100 KB | Pick during fixture capture; gzip; capture-script fallback if too large |
| `testcontainers-rs` has usable RabbitMQ + MinIO modules | HIGH | research/STACK lists testcontainers | Confirm modules/versions in research |

## Backend And Infrastructure Notes
| Topic | Decision/Default | Frontend/Caller Consequence | Hidden Cost | Breaking Point |
|-------|------------------|------------------------------|-------------|----------------|
| Ephemeral infra | testcontainers RabbitMQ + MinIO | Self-contained; no env to set | First-run image pulls; Docker daemon needed | A runner without Docker is a hard blocker |
| Worker loop seam | `run_until_cancelled` + `CancellationToken` | No signal handlers or real timers in the test, so nothing leaks across tests | Test must cancel the token after the job to end the loop | — |
| Coverage | e2e `#[ignore]`, excluded from `cargo llvm-cov` | No coverage obligation added | Live adapters stay allowlisted for now | — |

## Risks
| Risk | Severity | Why It Matters | Mitigation |
|------|----------|----------------|------------|
| Oracle degrades into a no-op smoke test | HIGH | Would give false safety straight through the refactor | Mandatory "mutation turns it red" proof; full-byte assertions, not existence checks |
| Real fixtures too big to commit | MEDIUM | Repo bloat | gzip + pick small real replays; capture-script fallback with a presence skip-guard (brief principle 8) |
| testcontainers flaky or slow in CI | MEDIUM | Could block deploy | Dedicated job, generous timeout, image caching |
| Branching entangles Phase-8 WIP untracked files | MEDIUM | Wrong files land on the oracle branch | Branch from `master` cleanly; leave `deny.toml` / `08-PATTERNS.md` on the refactor branch |
| Nondeterministic artifact across container runs | LOW | parser-core is pure, no clock | Determinism already proven; re-run-equals assertion repeated in the e2e |

## Acceptance Criteria
- The e2e boots RabbitMQ + MinIO via testcontainers, runs the real worker, and for each real fixture: the S3 artifact bytes equal the committed `*.expected.json` exactly, the published message matches the contract, and the S3 key + checksum match.
- Duplicate redelivery and a missing / checksum-mismatch object are asserted (terminal-once; `parse.failed`).
- A deliberately injected behavioral change turns the oracle red — teeth proven, on the record.
- The same expected baselines added to the fast in-process golden suite stay green and catch parser drift without containers.
- `verify` stays green without live fixtures: clippy/typecheck + the fast suite, with the e2e skipping cleanly when Docker is absent.
- The coverage gate is unaffected (e2e excluded).

## Verification Plan
- `/gsd-quick --full` research traces the real worker call path — every S3/AMQP read and write — and confirms the testcontainers module wiring before planning (brief principle 3).
- Tests authored through `solidstats-parser-rust-tests` + `solidstats-parser-rust-conventions`, never hand-rolled.
- Local run: `cargo test -p parser-worker -- --ignored` with Docker up; fast suite via normal `cargo test`.
- The mutation check is documented and run once to show the oracle goes red.

## Open Questions
| Priority | Question | Why It Matters | Owner/Status |
|----------|----------|----------------|--------------|
| P2 | Exact real-replay selection + sizes | Commit-gzipped vs capture-script | Resolve in research (fixture capture) |
| P2 | Dedicated `master` pre-deploy CI job wiring | Where the slow gate runs | Infra, during plan |
| P3 | Later: shrink live-adapter coverage allowlist using the e2e | Allowlist hygiene | Deferred |

## Question Ledger
| Priority | Question | Answer | Decision Impact |
|----------|----------|--------|-----------------|
| P0 | Build a new oracle, or does the existing suite suffice? | Build the missing layer | Proceed |
| P0 | Scope: add-layer / synthetic-only / container-e2e / skip | Full container e2e | testcontainers worker e2e |
| P0 | Home and timing | On `master`, separate pre-deploy gate | Branch from `master` |
| P0 | old-vs-new parity | Drop | Pure Rust oracle, AGENTS-compliant |
| P0 | Docker on the runner | Yes | testcontainers is viable |
| P1 | Flow count | No limit; must catch regression | Full-byte pinning + mutation proof + edge spread |

## Recommended Next GSD Step
- Primary: `/gsd-quick --full` with the updated brief, on a branch off `master` (e.g. `test/golden-container-e2e-oracle`). Research must trace the worker path and confirm testcontainers wiring; tests go through the rust-tests skill.
- Rationale: requirements and design are locked here. What's left — fixture selection, CI job wiring — is research/plan-level work that `/gsd-quick` handles with atomic commits and a verify gate.
- Alternatives: (a) if the deploy runner turns out to lack Docker, fall back to the env-gated `live_smoke` plus the fast committed-baseline golden layer only; (b) run `gsd-spec-phase` first if you want a formal SPEC before the quick task.
