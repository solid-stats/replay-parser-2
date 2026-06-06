# Changelog — solidstats-parser-rust-conventions

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
