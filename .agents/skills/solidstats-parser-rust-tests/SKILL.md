---
name: solidstats-parser-rust-tests
description: >
  Rust testing for the SolidStats OCAP parser (replay-parser-2) — the per-stack layer on top of
  solidstats-process-testing-standards. Adds the cargo test runner, the manifest-driven golden/parity
  harness, determinism tests, required property testing (proptest), snapshot testing (insta), required
  fuzzing (cargo-fuzz), and the coverage gate. Use when writing or reviewing parser unit, property,
  snapshot, golden, or fuzz tests.
  Use this proactively — apply it when writing or reviewing ANY parser test, even when not explicitly
  asked.
  Triggers: "write parser tests", "rust tests", "fuzz the parser", "proptest", "snapshot test",
  "golden test", "напиши тесты парсера", "rust тесты", "фаззинг парсера", "property тест",
  "golden тест".
---

# Parser Tests — Rust / OCAP

**This skill builds on [`solidstats-process-testing-standards`](../solidstats-process-testing-standards/SKILL.md) — read it first.**
That skill owns the philosophy (RITE, AAA, the unit-vs-integration boundary, determinism, doubles,
oracle strength, the coverage mindset, TDD). This skill adds the **Rust/parser HOW**: the runner,
the golden harness, and the property/snapshot/fuzz layers. It assumes the rules in
[`solidstats-parser-rust-conventions`](../solidstats-parser-rust-conventions/SKILL.md), especially
determinism (§C) and the contract (§G).

**Required tooling (ratified):** `proptest`, `insta`, and `cargo-fuzz` are part of the standard for
this parser — add them where missing. They are not optional niceties; an untrusted-input parser
earns its trust from property + fuzz coverage, not example tests alone.

## Runner & layout

- `cargo test` (or `cargo nextest`). Unit tests in co-located `#[cfg(test)] mod tests`; integration
  tests in `crates/<crate>/tests/`; the golden/parity harness lives in `crates/parser-core/tests/`.
- Tests return `Result<_, _>` and use `?` rather than `unwrap` (the lint floor denies `unwrap` even in
  tests where practical; assert error *variants* with `matches!`).

## Determinism test (first-class)

The headline test: the same input produces a **byte-identical** artifact across runs.
`deterministic_output` already exists — every new derived field is added to it. Re-serialize twice and
assert equal bytes; run on more than one thread/seed to catch ordering nondeterminism. This is the
test that guards convention §C.

## Golden / parity harness

The manifest-driven harness (`crates/parser-core/tests/`, fixtures → requirement/decision IDs →
expected status/features) is the canonical behavioral test.

- Every new parsing behavior or decision gets a fixture **and** a manifest entry — `golden_fixture_manifest`
  and `golden_fixture_behavior` stay green.
- `schema_drift_status` guards the contract shape; `fault_injection_regressions` keeps fixed bugs fixed.
- A fixture is added for every regression (a real malformed input that once broke the parser).

## Property testing (proptest, required)

- **Roundtrip**: for any valid artifact, `parse(serialize(a)) == a` (or the documented normalized
  form) — the canonical parser property.
- **Invariants**: determinism (same input → same output), ordering stability, status monotonicity —
  whatever must always hold.
- **Malformed input never panics**: feed arbitrary bytes / arbitrary JSON and assert the decode path
  returns a typed `Result`, never panics or hangs (this is the property form of convention §F totality).
- **Commit `proptest-regressions/`** to source control — a once-failing case becomes a permanent
  replayed fixture; never gitignore it.
- *(MED)* model-based / stateful proptest for worker job orchestration where pure roundtrip misses
  ordering/race bugs.

## Snapshot testing (insta)

- Snapshot the generated artifact for representative fixtures (`assert_json_snapshot!`). Because
  artifacts are deterministic, snapshots should be stable with **minimal** redactions — redact only
  fields that legitimately vary (a build/version stamp), placing any `sorted_redaction` last.
- **CI enforcement**: insta auto-detects `CI=true` and fails on drift rather than writing snapshots;
  use `--unreferenced=auto` (delete locally / reject in CI). Local workflow: `cargo insta review`.

## Fuzzing (cargo-fuzz, required)

References the external `cargo-fuzz` tool skill for mechanics; the policy here:

- **Two targets**: a raw `&[u8]` target on the top-level decode path (catches panics/crashes/UB on
  malformed bytes), **and** a structure-aware target via `#[derive(Arbitrary)]` on the post-parse IR
  (exercises aggregation/normalization logic that random bytes rarely reach).
- **Seed corpus** committed under `fuzz/corpus/`, seeded from the golden `.ocap.json` fixtures;
  minimize with `cargo fuzz cmin` before long runs.
- **CI smoke**: run each target for a bounded time — `cargo fuzz run <target> -- -max_total_time=300`
  (the `-max_total_time` arg is a libFuzzer flag, so it must follow `--`) — on a nightly job; upload
  `fuzz/artifacts/` on a crash for reproduction.
- **Close the loop**: use `cargo fuzz coverage` to find decode branches the fuzzer didn't reach, then
  add seeds or adjust the target.
- *(MED, if the repo is public)* submit to OSS-Fuzz / ClusterFuzzLite for continuous long-run fuzzing.

## Contract tests

- `parser-contract` types validate against the published JSON Schema (`jsonschema`); schema drift is a
  failure (mirrors the review gate).
- **C-DEBUG-NONEMPTY**: a small test asserts every public contract type's `Debug` output is non-empty.

## Coverage gate

- `cargo-llvm-cov` via `parser-quality` + `scripts/coverage-gate.sh` (references the external
  `coverage-analysis` skill). Aim high — `parser-core` logic toward 100% reachable; rare exceptions
  justified. Coverage is a **floor, not proof** (testing-standards §H): pair with the property and
  fuzz layers, which are the real fault detectors.

## Not owned here

The testing philosophy lives in `solidstats-process-testing-standards`; the severity of a
test-quality problem in review lives in `solidstats-process-review-standards` §F (test quality is
never a standalone BLOCK unless a test actively masks a real bug).
