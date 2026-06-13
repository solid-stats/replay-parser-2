# Changelog — solidstats-parser-rust-tests

## 2026-06-13 — coverage-suppression discipline (shared-testing-standards §H)
- Coverage gate section rewritten: no blanket file excludes, every suppressed line in
  `coverage/allowlist.toml` with exact line numbers / owner / EXPIRY / co-located marker.
  Legitimate categories enumerated; ordinary logic branches excluded from suppression.
  Two hard requirements added: no stale entries (expired EXPIRY is a finding) and the gate
  must run in CI — an unchecked or expired allowlist is theatre.

## 2026-06-13 — insta sorted redactions (taxonomy V5, research-parser.md confirmed)
- Snapshot section: stabilize snapshots over `HashSet` / non-deterministic iteration order with
  sorted redaction selectors (`insta::sorted_redaction()`) instead of restructuring the type —
  test-only/non-artifact values only; artifact data stays ordered per conventions §C.

## 2026-06-06 — Analysis fixes (see .planning/SKILLS-ANALYSIS.md)
- Fixed the CI fuzz command: `-max_total_time` is a libFuzzer flag and must follow `--`
  (`cargo fuzz run <target> -- -max_total_time=300`).

## 2026-06-06 — Initial
- The thin per-stack Rust parser test skill on top of `solidstats-shared-testing-standards` (which
  owns the philosophy). Adds the cargo runner, the golden harness, and the property/snapshot/fuzz
  layers; assumes `solidstats-parser-rust-conventions` (determinism §C, contract §G).
- **Required tooling (ratified):** proptest, insta, cargo-fuzz — added where missing.
- **Determinism test** as the first-class guard (byte-identical artifact across runs/threads).
- **Golden/parity harness** (manifest-driven, fixtures → requirement/decision IDs): a fixture +
  manifest entry per new behavior; `schema_drift_status` + `fault_injection_regressions`.
- **Property (proptest):** roundtrip, invariants, "malformed input never panics"; **commit
  `proptest-regressions/`**; model-based for worker orchestration (MED).
- **Snapshot (insta):** artifact snapshots with minimal redactions; CI enforcement (`CI=true`
  auto-fail, `--unreferenced=auto`); `cargo insta review`.
- **Fuzzing (cargo-fuzz):** two targets (raw `&[u8]` decode + structure-aware via `arbitrary`);
  committed seed corpus from golden fixtures; `cargo fuzz cmin`; CI smoke (`-max_total_time=300`);
  `cargo fuzz coverage` to close the loop; OSS-Fuzz/ClusterFuzzLite if public (MED).
- **Contract tests:** JSON Schema validation (`jsonschema`); C-DEBUG-NONEMPTY test.
- **Coverage gate:** cargo-llvm-cov via `parser-quality` + `scripts/coverage-gate.sh`; high target,
  coverage is a floor not proof.
- References the external `cargo-fuzz` and `coverage-analysis` tool skills for mechanics.
- Incorporates the test-level findings from `.planning/RESEARCH-parser-cluster.md`. Closes the parser
  cluster.
