# Roadmap: replay-parser-2

## Milestones

- [x] **v1.0 Parser Worker Readiness** — Phases 1-7 shipped on 2026-05-09. Full archive: [v1.0-ROADMAP.md](./milestones/v1.0-ROADMAP.md).

## Current Status

v1.0 is complete and archived. The project is awaiting the next milestone.

Start the next milestone with:

```bash
$gsd-new-milestone
```

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

## Notes

- Phase directories remain in `.planning/phases/` as execution history. Use `$gsd-cleanup` later if you want to move raw phase artifacts under `.planning/milestones/`.
- Live Timeweb S3 validation remains deployer-run operational evidence requiring credentials; local MinIO/two-worker smoke covers parser-owned behavior.
