# Changelog — solidstats-parser-rust-conventions

## 2026-06-13 — Lint-suppression policy extended in §B

- Extended the §B lint-floor `#[allow]`/`#[expect]` rule to mirror the TS suppression policy
  (`solidstats-shared-ts-standards` §C): **structural-complexity lints** (`too_many_lines`,
  `too_many_arguments`, `cognitive_complexity`, `type_complexity`) are a split signal, never
  suppressed; a lint firing across many sites for one codebase-wide reason is set **once** at the
  workspace `[lints.clippy]` level, not scattered as per-site `#[expect]`. The existing
  `#[expect(reason=…)]`-over-`#[allow]` rule stays (it self-retires when unnecessary).
- Basis: the gate-suppression triage (`plans/product/skills-taxonomy/coverage-clippy-triage.md`) —
  e.g. `trivially_copy_pass_by_ref` (8 sites, one rationale) belongs in the workspace table; the 9
  redundant test-module `#![allow(expect_used)]` are already covered by `clippy.toml`
  `allow-expect-in-tests`.

## 2026-06-13 — Observability reference (§K–§M) + research deltas (taxonomy V5)
- New `references/observability-and-lifecycle.md`: **§K log hygiene** (tracing structured fields,
  level semantics, state-transition spans, no PII/struct dumps, span hierarchy), **§L diagnosability**
  (swallowed Results, error source chains, identifying context, upstream S3/lapin detail, happy-path
  legibility), **§M resource lifecycle** (unbounded worker-state collections, bounded channels,
  tempfile RAII, S3 multipart abort). Carries the bidirectional parity header naming
  `solidstats-shared-backend-ts-standards` §Z/§AA/§AB (TS mirror): update both sides in the same
  pass or leave a `TODO(#issue)`; one-sided edits are a review finding. Added to the SKILL.md
  reference map (§-letter scheme continues §K–§M).
- §H: the "Instrument with `tracing`" bullet is now a forward-reference to §K, not stand-alone prose.
- §C (research-parser.md, confirmed): `IndexMap`'s default serde gives NO ordering guarantee —
  canonical output uses `BTreeMap` or `serde_seq`; never rely on `IndexMap` default serde for
  bit-for-bit determinism.
- §H (research-parser.md, confirmed): lapin auto-recovery —
  `ConnectionProperties::default().enable_auto_recover()` + `wait_for_recovery` on recoverable
  errors; `basic_qos` prefetch clarified as delivery backpressure, NOT task concurrency — real
  concurrency is capped with a bounded semaphore/channel.
- §D's thiserror-only stance deliberately unchanged (research's "anyhow at the edge" consensus is
  noted and rejected — our convention forbids anyhow).

## 2026-06-06 — Split into spine + references (user directive)
- Restructured the single SKILL.md into a spine (intro + §A architecture + §B lint floor + reference
  map) plus `references/`: `determinism-and-contract.md` (§C/§G), `parsing-types-errors.md` (§D/§E/§F),
  `worker-build-perf.md` (§H/§I/§J) — matching the backend/frontend modular shape.

## 2026-06-06 — Analysis fixes (see .planning/SKILLS-ANALYSIS.md)
- Corrected §C: serde_json serializes `f64` via ryu deterministically **by default**; `float_roundtrip`
  reframed as parse-side canonicalization (was wrongly stated as the source of cross-arch determinism).
- Corrected §C overflow wording (wrapping is deterministic-but-wrong, not "arch-dependent").
- §F: dropped the misattributed RUSTSEC-2024-0012 (that advisory is `serde-json-wasm`); cite
  serde_json's own 128-deep guard. Added: reject non-finite floats; default `deny_unknown_fields`;
  S3 stream + content-length cap.
- §H worker hardening: poison-message/DLX + `delivery-limit`, `basic_qos` prefetch bound, `JoinSet`
  drain on shutdown, consumer-cancel/recovery, S3 operation timeouts.
- §D: C-GOOD-ERR error bounds (`Error + Send + Sync + 'static`, lowercase Display).
- Tightened the description and added determinism/contract triggers (the two headline rules had none).

## 2026-06-06 — Initial
- Authored fresh — there is no estesis Rust doctrine; grounded on the absorbed Rust generics
  (`rust-best-practices`, `rust-async-patterns`, `rust-testing`) plus replay-parser-2's actual stack.
- **Built on the repo's enforced lint floor** (unsafe forbid, no unwrap/expect/panic, clippy
  pedantic/nursery, float_cmp/as_conversions/print_stdout denied, missing_docs denied) — the
  conventions do not restate the lints, they assume them and add the discipline beyond.
- **Two headline rules:** (C) determinism — byte-identical artifacts, ordered output
  (BTreeMap/sorted), no HashMap/SystemTime/rand in derived data, float epsilon; (G) the versioned
  `parser-contract` — semver + regenerated JSON Schema + golden manifest, breaking artifact changes
  coordinated with server-2.
- Crate architecture: pure/deterministic `parser-core`, contract crate as the published interface,
  thin `cli`/`worker` adapters, `parser-quality` gates.
- thiserror everywhere (no anyhow); typed errors; malformed-input totality (the parser never panics
  on untrusted bytes — which is why fuzzing is required); newtypes + exhaustive matches; tokio/worker
  durability + graceful shutdown; docs/perf.
- **Tooling decision (user-ratified): require** proptest / insta / cargo-fuzz — the decode path is
  written to be fuzzable; specifics live in the tests skill.
- Single SKILL.md (the strict lint floor lets it stay compact); testing detail delegated to
  `solidstats-parser-rust-tests`.
- **Deep-research-driven additions** (manual multi-agent web research, authoritative sources):
  determinism — `float_roundtrip` serde_json feature + `overflow-checks` + RFC 8785 JCS note;
  untrusted-input DoS guards — input-size cap (`Read::take`) + serde_json recursion-limit discipline
  (RUSTSEC-2024-0012); contract — `cargo-semver-checks` + PR-time JSON Schema diff + C-NEWTYPE-HIDE;
  new §J build/supply-chain gates — `cargo-deny`/`cargo-audit`/MSRV-CI/reproducible builds; C-CONV;
  optional `tracing-opentelemetry`. Test- and review-level findings recorded in
  `.planning/RESEARCH-parser-cluster.md` for the remaining two skills.
