# Retrospective

## Milestone: v1.0 — Parser Worker Readiness

**Shipped:** 2026-05-09  
**Phases:** 9 roadmap phases plus inserted Phase 5.1/5.2 gap-closure phases  
**Plans:** 56 phase plans  
**Git timeline:** 2026-04-24 to 2026-05-09

### What Was Built

v1.0 replaced the legacy TypeScript replay parser with a deterministic Rust
parser application that can parse OCAP JSON locally, emit a compact v3 minimal
server-facing artifact, compare selected outputs against the old parser, and run
as a RabbitMQ/S3 worker with deterministic artifact keys and artifact-reference
result messages.

The milestone also shipped strict validation infrastructure: schema-backed
contracts, golden fixtures, deterministic behavior tests, `cargo llvm-cov`
strict coverage postprocessing, deterministic fault reporting, accepted
full-corpus benchmark evidence, duplicate-redelivery tests, local worker smoke,
structured logs, and HTTP probes.

### What Worked

- Grounding behavior in the old parser and `~/sg_stats` corpus caught semantic
  and output-shape risks before worker integration.
- The Phase 5 UAT rejection was useful: it forced the default artifact away from
  a large normalized replay dump and into a minimal server-facing statistics
  contract.
- Keeping parser-core transport-free made CLI, comparison, benchmark, and worker
  behavior share the same deterministic parse path.
- Treating `server-2`, `replays-fetcher`, and `web` as separate owners prevented
  parser work from absorbing replay discovery, persistence, UI, auth, moderation,
  or canonical identity responsibilities.

### What Was Inefficient

- The initial normalized-artifact direction created substantial rework in Phase
  5.1 and Phase 5.2.
- Strict coverage stayed fragile across worker/CLI additions until the final
  allowlist and marker pass.
- Some GSD artifact statuses remained stale even after later phases superseded
  them, so milestone close needed explicit deferred-item acknowledgement.

### Patterns Established

- Default worker artifacts should stay compact; full event/entity/source-ref
  evidence belongs behind explicit debug/parity sidecars or raw reprocessing.
- Parser contract changes require explicit downstream compatibility review or a
  recorded product-owner acceptance.
- Worker messages should carry artifact references, not inline parse artifacts.
- Strict coverage exceptions must be exact-line, reviewed, and paired with
  inline `coverage-exclusion:` markers.

### Key Lessons

- Performance targets must compare product-relevant workloads, not only a single
  replay path.
- A hard max artifact size is a clearer ingestion gate than ratio-only metrics
  when replay sizes vary heavily.
- Accepted known-bad replay failures need both path allowlists and old/new
  failure parity evidence.
- Live provider validation should be separated from parser-owned local smoke when
  credentials are not available.

### Cost Observations

- The milestone required multiple inserted phases and quick tasks after UAT
  corrected the artifact direction.
- Agent workflow overhead was justified for contract, coverage, and worker
  correctness, but generated planning state should be pruned or archived after
  milestone close to keep future context cheap.

## Cross-Milestone Trends

No prior milestones exist yet.
