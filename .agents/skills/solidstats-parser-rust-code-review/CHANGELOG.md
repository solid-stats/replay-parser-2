# Changelog — solidstats-parser-rust-code-review

## 2026-06-16 — Review-lens mapping (BMAD Improvement 2)
- Added a **Review lenses** section mapping the three generic adversarial lenses from
  `solidstats-shared-review-standards` §J onto this reviewer's two phases: **Contract Adversary** →
  Phase 1 contract & determinism gate + the §I.2 blast radius onto `server-2` (which ingests the
  artifact); **Edge / Failure Hunter** → Phase 2 untrusted-input totality + worker/lifecycle (topics
  2, 7, 9); **Acceptance Auditor** → §F + the discovered PLAN `must_haves.truths` (§I.3), checked against the
  golden/parity manifest. Notes the no-forced-finding rule and the depth-tied fan-out. Pure addition.
- Provenance: ADR `decisions/0007-bmad-borrowed-improvements.md`.

## 2026-06-13 — Wire observability & lifecycle (§K–§M)
- Phase 2 sweep gains two risk-ordered steps after async/worker: **Observability** (`[conv: §K/§L]`
  — structured `tracing` fields, log-level semantics, state-transition instrumentation, swallowed
  `Result`s, error `source()` chain + identifying context, S3/lapin failure detail) and **Resource
  lifecycle** (`[conv: §M]` — unbounded worker-state growth, bounded channels, RAII temp files, S3
  multipart abort; a leak finding cites all three legs).
- Severity table gains the matching rows: worker-state leak 🟠 (🔴 on the hot delivery path);
  swallowed `Result` / `unbounded_channel` / temp-file / multipart 🟠; the §K/§L logging set 🟡;
  happy-path legibility 🔵.
- Closes the gap (ADR 0006) where `references/observability-and-lifecycle.md` §K–§M existed in the
  conventions but no review step enforced them.

## 2026-06-06 — Follow-up
- Conventions split into spine + `references/`; noted that the §-letters now live in those reference
  files (citations unchanged, resolved via the conventions reference map).

## 2026-06-06 — Analysis fixes (see .planning/SKILLS-ANALYSIS.md)
- Severity now comes from the in-skill Severity reference table (the conventions skill carries no
  severity tags); dropped the "use the severity the rule is tagged with" instruction.
- Reordered the severity table 🔴→🔵; removed the wrong "missing float_roundtrip" nondeterminism cause;
  added hunt rows for poison-message/no-DLX, unbounded prefetch, `deny_unknown_fields` off, unbounded
  S3 read, and shutdown-without-drain.

## 2026-06-06 — Initial
- The operational parser reviewer: hard-requires `solidstats-shared-review-standards` (format,
  severity, verdict, scope, noise filter) and enforces `solidstats-parser-rust-conventions` as its
  rule library. Cites rules, doesn't restate them.
- **Phase 1 — contract & determinism gate** (the parser analog of the backend API-contract gate):
  `deterministic_output` green + ordered/`float_roundtrip`/`overflow-checks` for new derived fields;
  artifact-shape changes versioned (semver + JSON Schema regen + golden manifest +
  `schema_drift_status`); `cargo-semver-checks` + PR-time JSON Schema diff; lints/`cargo-deny`/
  `cargo-audit` clean. Unflagged breaking change or nondeterminism → BLOCK. CLI/worker-only changes:
  contract N/A, gate on determinism + lints.
- **Phase 2 — convention/correctness sweep** in risk order (determinism → untrusted-input totality →
  errors → contract → types → architecture → async/worker → supply-chain → docs), each finding citing
  `[conv: …]` and using the tagged severity.
- Parser-specific severity table for a mechanical verdict.
- Incorporates the gate signals and hunt list from `.planning/RESEARCH-parser-cluster.md`
  (cargo-semver-checks, JSON Schema diff, input-size/recursion guards, float_roundtrip,
  overflow-checks, C-CONV).
- Output delegates to review-standards (§D–§E), opening with the gate result; test quality deferred to
  `solidstats-parser-rust-tests` + review-standards §F.
