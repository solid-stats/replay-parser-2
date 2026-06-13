# Codebase Concerns

**Analysis Date:** 2026-06-13

## Tech Debt

**Coverage Boundary Challenges in Stable Rust:**
- Issue: Stable Rust (1.95.0) does not provide `#[coverage(off)]` for fine-grained production code exclusions; `cargo llvm-cov` supports only `--ignore-filename-regex` which excludes whole files.
- Files: `coverage/allowlist.toml`, `crates/parser-quality/src/coverage.rs`
- Impact: Coverage enforcement requires custom post-processor over `cargo llvm-cov --json` rather than native Rust attributes; 244 allowlisted locations across 41 production files, including substantial line ranges for live RabbitMQ/S3 boundary code.
- Fix approach: Monitor Rust nightly for stabilization of `#[coverage(off)]` or `#[coverage(on)]` attributes; when stable, refactor allowlist into inline markers and simplify the coverage-check binary. Current post-processor is sound but adds tooling complexity.

**Heavy Resource Requirements for Strict Coverage:**
- Issue: Strict local coverage analysis using `cargo llvm-cov --workspace` with full instrumentation causes multi-GB `target/` directory growth and extended build times on workstations.
- Files: `.planning/codebase/` local gates, scripts, `Cargo.toml` profile settings
- Impact: Strict coverage gate is wrapped with `COVERAGE_ALLOW_HEAVY=1` opt-in; developers must explicitly opt into heavy workstation impact; CI can afford the cost, but local development is protected.
- Fix approach: Current mitigation (opt-in guard) is proportional and safe. Consider nightly incremental coverage or cached instrumentation in future v2+ work if coverage gates need to run frequently in developer workflows.

**AMQP/S3 Boundary Coverage Gaps:**
- Issue: Live RabbitMQ connection, channel, publisher-confirm, and S3 stream paths cannot be covered without live infrastructure; 79 lines in `crates/parser-worker/src/amqp.rs` and 18 lines in `crates/parser-worker/src/processor.rs` are excluded.
- Files: `crates/parser-worker/src/amqp.rs` (lines 160–279), `crates/parser-worker/src/processor.rs` (lines 45–57, 203–333), `crates/parser-worker/src/runner.rs` (lines 67–281)
- Impact: Serialization, confirmation policy, ack behavior, and processor logic are covered via no-network unit tests, but end-to-end RabbitMQ/S3 runtime is only proven through Docker smoke tests (`scripts/worker-smoke.sh`), not reachable-code coverage.
- Fix approach: No change needed. This is an architectural boundary; live infrastructure coverage is provided by Docker smoke tests and deployer integration validation. The allowlist is explicit and time-bounded (`expires: 2026-05-28` on all Phase 05 exclusions).

## Known Bugs

**Old-Parser `teamkillers` Merge Bug Preservation:**
- Symptoms: New parser emits an extra `teamkillers` relationship row when old parser's buggy merge behavior would have lost it.
- Files: `.planning/quick/260502-year-edge-parity-five-sample-rollup/KNOWN-DIFFERENCES.md`, `crates/parser-core/src/aggregates.rs`
- Trigger: Five-sample year-edge parity audit found 2 rows across 364 replays where old parser merged `killers` instead of `teamkillers` in edge cases with both normal and team kills.
- Workaround: Not a bug in new parser. Documented as an intentional difference in `/KNOWN-DIFFERENCES.md`. Old-parser behavior is preserved as a known historical artifact but not reproduced.
- Status: Resolved via documentation and accepted parity baseline.

**Duplicate-Slot `isDeadByTeamkill` State Differences:**
- Symptoms: New parser follows the current product rule (latest death state), while old parser merged duplicate slots with boolean OR over all death states.
- Files: `.planning/quick/260502-year-edge-parity-five-sample-rollup/KNOWN-DIFFERENCES.md`
- Trigger: Multiple entity slots for the same player where one dies by teamkill and a later slot dies by enemy or other cause.
- Workaround: Not a bug; new behavior is intentional and models respawn play more usefully. Documented as accepted difference in parity baseline.
- Status: Resolved via documented difference class.

**Retained `Throw` and `Binoculars` Weapon Names:**
- Symptoms: New parser preserves non-empty weapon names (e.g., `Throw`, `Binoculars`) that old parser suppresses in a legacy forbidden-weapon set.
- Files: `.planning/quick/260502-year-edge-parity-five-sample-rollup/KNOWN-DIFFERENCES.md`, `crates/parser-core/src/entities.rs`
- Trigger: Delayed ordnance, thrown ordnance, or vehicle context where the source weapon name is in the legacy suppressed set.
- Workaround: Intentional. Raw non-empty weapon names carry evidence; public presentation can hide or group them later.
- Status: Resolved via documented difference class and acceptance in parity baseline (104 rows across five samples).

## Security Considerations

**OCAP JSON Input Validation:**
- Risk: Untrusted OCAP JSON may contain malformed event/entity shapes that could cause parser panics or incorrect statistics.
- Files: `crates/parser-core/src/entities.rs`, `crates/parser-core/src/events.rs`, `crates/parser-core/src/raw_compact.rs`
- Current mitigation: Tolerant OCAP parsing preserves malformed observations as diagnostics instead of panicking; raw event accessors are defensive; killed-event tuple parsing includes fallback branches for unknown actor cases. Phase 04 security audit closed all 21 threats with zero open risks.
- Recommendations: Parser-core is proven production-ready. Worker and CLI remain defensive for file I/O errors. No additional mitigations required for v1.

**Canonical Identity Boundary:**
- Risk: Parser could accidentally leak canonical player identity, conflate observed and canonical IDs, or assume `server-2` identity equivalence.
- Files: All crates; boundary grep passed with only README future-scope documentation and negative tests.
- Current mitigation: Parser preserves observed entity IDs and legacy keys only. Canonical player matching belongs to `server-2`. Phase 04 security audit verified this boundary.
- Recommendations: Enforce during code review that any new fields using player ID do not assume canonical equivalence. Boundary grep remains a release gate.

**Result Message Tampering:**
- Risk: RabbitMQ messages published with parse results could be intercepted/modified if transport lacks TLS or producer lacks auth.
- Files: `crates/parser-worker/src/amqp.rs`, `crates/parser-worker/src/processor.rs`
- Current mitigation: Worker integration follows `server-2` job/result messaging contract; message contents are signed via S3 artifact checksum verification; `lapin` TLS configuration is delegated to deployer credentials.
- Recommendations: Live deployment must provide TLS/auth credentials. Local smoke tests cover message structure and processor behavior.

## Performance Bottlenecks

**Full OCAP JSON Parsing vs. Selective Extraction:**
- Problem: Parser-core accepts complete OCAP JSON and normalizes all entities/events internally, then selects a minimal subset for compact output; full normalized state consumes memory and time before compact mapping.
- Files: `crates/parser-core/src/lib.rs`, `crates/parser-core/src/raw_compact.rs`
- Cause: Phase 5.1 inserted a selective parser boundary, but the current implementation still normalizes full OCAP before extracting minimal rows. Optimization was deferred after UAT accepted the current performance (2026-05-02).
- Improvement path: Phase 05.2 quick task `260502-jeh` optimized the default minimal parser hot path by deriving minimal rows directly from one-pass connected/killed observations. Further optimization could use streaming JSON parsing or SIMD to avoid full-DOM allocation for large replay files. Current measured performance is acceptable; no v1 blocker.

**Benchmark x3/x10 Target Gap (Accepted):**
- Problem: All-raw x10 speedup target (`10x` faster than old parser) is not met; current measured speedup is `1.8413x` over cached old baseline.
- Files: `.planning/quick/260502-jeh-full-optimize-parser-points-2-3-and-4-di/260502-jeh-SUMMARY.md`
- Cause: Old parser used worker threads; new parser uses selective extraction and sequential S3/RabbitMQ flow; both have different I/O and CPU profiles.
- Improvement path: Product owner accepted current performance on 2026-05-02 as non-blocking. Further optimization would require either selective parser streaming (large effort) or accepting that the sequential RabbitMQ worker model does not parallelize old-parser-equivalent work per file. Not a v1 concern.

**Artifact Size p95 Ratio Above Target (Accepted):**
- Problem: P95 artifact/raw ratio (`0.1243...`) exceeds the original target of `<= 0.10`.
- Files: `.planning/quick/260502-jeh-full-optimize-parser-points-2-3-and-4-di/260502-jeh-SUMMARY.md`
- Cause: Some replays have small raw OCAP (few events) but still require certain fixed fields (player headers, weapon dict, diagnostics), driving up the ratio.
- Improvement path: Product owner accepted p95 above target as non-blocking on 2026-05-02 because the hard max artifact size (`<= 100 KB`) passes. No further action required.

## Fragile Areas

**Coverage Allowlist Maintenance:**
- Files: `coverage/allowlist.toml`, `crates/parser-quality/src/coverage.rs`, `crates/parser-quality/src/bin/coverage-check.rs`
- Why fragile: Every phase-05 exclusion has `expires: 2026-05-28`, suggesting a manual audit requirement every ~4 weeks. Stale allowlist entries cause the coverage gate to fail (as seen in v1.0 gap-closure plan). New production code in worker/CLI/core must update both the source marker and allowlist, or coverage gate blocks commit.
- Safe modification: When adding new production code, check current coverage with `scripts/coverage-gate.sh --check` early in development. For defensive/boundary code, add exact lines to `coverage/allowlist.toml` with a review reason, expiry date, and matching `coverage-exclusion:` marker in the source. Test locally with `COVERAGE_ALLOW_HEAVY=1 scripts/coverage-gate.sh` before pushing.
- Test coverage: 41 production files; 244 allowlisted locations; 0 uncovered locations as of v1.0 close (2026-05-09). Coverage remains enforced at build and CI gates.

**Worker/CLI Signal Handling and Shutdown:**
- Files: `crates/parser-worker/src/runner.rs` (lines 67–281, excluded from coverage), `crates/parser-cli/src/main.rs` (lines 452–457, excluded)
- Why fragile: Live orchestration over S3, RabbitMQ, OS signals, and tokio streams requires clean shutdown paths. Graceful shutdown on SIGTERM (for container orchestration), channel closure on RabbitMQ disconnect, and S3 connection pooling are not fully testable without live infrastructure.
- Safe modification: Worker shutdown is tested through `scripts/worker-smoke.sh` two-worker Docker test. CLI stdio error paths are tested via integration tests. If adding new signal handlers or orchestration, ensure no panics on channel closure, verify shutdown order (RabbitMQ close before exiting), and test with Docker smoke.
- Test coverage: Docker smoke test (`scripts/worker-smoke.sh`) covers spawn, readiness probe, two-worker concurrent parsing, parse result publication, and container shutdown. This is sufficient for v1.

**RabbitMQ/S3 Connection Pooling and Error Recovery:**
- Files: `crates/parser-worker/src/amqp.rs` (79 excluded lines), `crates/parser-worker/src/processor.rs` (18 excluded lines)
- Why fragile: Connection retries, channel closure recovery, S3 transient failures, and publisher-confirm timeouts are live-infrastructure scenarios. Current code is defensive (fail-fast, nack on error, auto-retry via job redelivery), but exhaustion scenarios (e.g., RabbitMQ stops accepting publishes after many failures) are not tested locally.
- Safe modification: Worker retries are delegated to RabbitMQ redelivery (prefetch=1, manual ack). Do not add custom exponential backoff or circuit breakers without coordinating with `server-2` job lifecycle. If modifying connection setup, test with Docker compose RabbitMQ; if modifying publish flow, ensure nack/retry semantics are preserved.
- Test coverage: No-network unit tests cover processor logic; Docker smoke tests cover a two-worker parse cycle with explicit test assertions on artifact keys and result publication.

## Scaling Limits

**Max Workers per RabbitMQ Connection:**
- Current capacity: Single AMQP connection per worker instance; prefetch=1 per consumer.
- Limit: RabbitMQ channel limit per connection (typically 2048 on default config) is not approached with one consumer per worker. Horizontal scaling adds more worker instances, each with its own connection.
- Scaling path: For v1, assume one worker instance = one AMQP connection. Prod deployment can run N instances behind a load balancer or orchestrator. If future work needs connection pooling or multiplexing, implement in a second phase.

**S3 Artifact Key Determinism and Collision Avoidance:**
- Current capacity: Artifact keys are `artifacts/v3/{encoded_replay_id}/{source_sha256}.json` (deterministic, keyed by raw source checksum).
- Limit: No collision risk for valid unique source checksums. Collision risk only if two different OCAP files hash to the same SHA-256 (cryptographically negligible).
- Scaling path: Deterministic keys allow multi-worker reuse and conflict fallback (conditional create, compare, discard duplicate). No changes needed for scaling.

**Strict Coverage Local Resource Footprint:**
- Current capacity: Strict `cargo llvm-cov` with full workspace instrumentation causes multi-GB target/ directory.
- Limit: Workstation with 8 GB RAM may exhaust swap; CI with 16+ GB can afford multiple coverage runs.
- Scaling path: Opt-in guard (`COVERAGE_ALLOW_HEAVY=1`) protects developers. Future v2 work could cache instrumented binaries or use incremental coverage, but v1 is acceptable.

## Dependencies at Risk

**Transitive Duplicate Crate Versions:**
- Risk: AWS SDK and AMQP TLS/HTTP trees introduce duplicate crate versions (e.g., multiple tokio, serde versions).
- Impact: Slightly larger binary size and potential for maintenance confusion; `Cargo.lock` ensures reproducible builds.
- Migration plan: Workspace lint `multiple_crate_versions = "allow"` is intentionally set; dependency audit remains the practical control. Monorepo consolidation of `server-2`, `replays-fetcher`, and `web` dependencies may reduce duplication in future product work.

**AWS SDK and Timeweb S3 Compatibility:**
- Risk: AWS SDK v1 defaults to AWS region/endpoint; Timeweb S3-compatible storage requires `https://s3.twcstorage.ru` and optional no-secret capability labels.
- Impact: Live Timeweb validation remains pending deployer-supplied credentials; local MinIO covers parser-owned behavior.
- Migration plan: Deployer provides Timeweb credentials and validates endpoint configuration. No parser code changes required. Phase 07 documentation records `https://s3.twcstorage.ru` and path-style mode requirements.

## Missing Critical Features

**Yearly Nomination Statistics (Deferred to v2):**
- Problem: Annual/yearly nomination statistics are out of v1 scope; `src/!yearStatistics` in old parser is historical reference only.
- Blocks: v2 product surface for year-end nomination rankings.
- Current status: `.planning/quick/260502-year-edge-parity-five-sample-rollup/` documents current-year stats parity; raw OCAP reprocessing path is available for future yearly work.

**Live Timeweb Provider Validation:**
- Problem: Timeweb S3 endpoint testing requires external credentials and live connection.
- Blocks: No parser blocker; deployer-run operational validation only.
- Current status: Phase 07 documents Timeweb settings; local MinIO smoke tests cover S3 adapter behavior.

## Test Coverage Gaps

**Live Worker Orchestration End-to-End:**
- What's not tested: Graceful shutdown on SIGTERM, long-running job processing, S3 transient failure recovery, RabbitMQ channel closure and reconnection.
- Files: `crates/parser-worker/src/runner.rs`, `crates/parser-worker/src/amqp.rs`
- Risk: Deployment may encounter unexpected shutdown behavior or connection recovery paths not covered by unit/smoke tests.
- Priority: Medium — Docker smoke tests cover the happy path. v2 work could add chaos engineering (kill RabbitMQ, fill S3) for production hardening.

**Compact JSON Visitor Error Paths:**
- What's not tested: All possible malformed compact JSON shapes; serde visitor is defensive but low-probability failure modes exist.
- Files: `crates/parser-core/src/raw_compact.rs` (lines 566–598 excluded)
- Risk: Unusual malformed input could trigger a covered but untested visitor error.
- Priority: Low — parser gracefully handles shape mismatches; end-to-end test would require synthetic malformed JSON generation.

**Malformed OCAP Diagnostics:**
- What's not tested: Every possible edge case in malformed event/entity shapes; parser emits diagnostics instead of panicking, but gap discovery requires live malformed files.
- Files: `crates/parser-core/src/entities.rs`, `crates/parser-core/src/events.rs`
- Risk: New malformed input categories might not produce expected diagnostics.
- Priority: Low — Five-sample year-edge parity confirmed 23,473 real replays parse without failures. Regression testing on known malformed files is tracked in `.planning/benchmarks/phase-05-all-raw-accepted-failures.json`.

---

*Concerns audit: 2026-06-13*
