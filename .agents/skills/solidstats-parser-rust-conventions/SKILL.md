---
name: solidstats-parser-rust-conventions
description: >
  Prescriptive conventions for the SolidStats Rust OCAP parser (replay-parser-2). Two
  non-negotiables: the parser is deterministic (byte-identical artifacts) and the versioned
  parser-contract stays stable. Also covers crate architecture, thiserror error handling, domain
  typing, malformed-input totality, the tokio worker, and build/supply-chain gates — on top of the
  repo's strict lint floor. Consult before writing or changing any parser Rust; it is the rule source
  that solidstats-parser-rust-code-review enforces and solidstats-parser-rust-tests assumes.
  Use this proactively — read it before writing or changing ANY parser Rust, even when the task
  doesn't say "conventions".
  Triggers: "parser conventions", "rust conventions", "edit the parser", "ocap parsing",
  "deterministic output", "parser contract", "artifact schema", "конвенции парсера",
  "детерминизм парсера", "контракт парсера", "поменяй парсер", "разбор ocap".
---

# SolidStats Parser Conventions — Rust / OCAP

These are the **prescriptive** conventions for `replay-parser-2`: the deterministic OCAP-JSON →
artifact parser (CLI + durable worker). They define what good parser code *should* look like; where
current code diverges, the code is brought into line — the convention wins. This skill is the rule
source that [`solidstats-parser-rust-code-review`](../solidstats-parser-rust-code-review/SKILL.md)
enforces and that [`solidstats-parser-rust-tests`](../solidstats-parser-rust-tests/SKILL.md) assumes.

Two rules are non-negotiable and everything else serves them: **(1) the parser is deterministic**
and **(2) the parser-contract is versioned and stable** — both detailed in
`references/determinism-and-contract.md` (§C, §G). A parser whose output drifts run to run, or whose
artifact shape changes silently, breaks every consumer downstream.

This SKILL.md owns the spine — the crate architecture and the lint floor everything else builds on.
Load the `references/` file for the area you're touching.

---

## A. Crate architecture

The workspace is five crates, each with one job. Keep logic in the core, keep binaries thin.

| Crate | Responsibility | Depends on |
|-------|----------------|------------|
| `parser-contract` | The versioned `ParseArtifact` envelope, `ParseStatus`, worker messages, and the JSON Schema (schemars + semver). The published contract consumers (server-2 ingest) depend on. | — |
| `parser-core` | Deterministic parsing, normalization, aggregation. Pure logic — no I/O, no clock, no network. | contract |
| `parser-cli` | `clap` binary `replay-parser-2` (Parse / Schema / Worker / Healthcheck). Thin adapter. | core, contract |
| `parser-worker` | Durable RabbitMQ/S3 worker + axum healthcheck + graceful shutdown. | core, contract |
| `parser-quality` | Coverage and fault-injection (`fault_injection_regressions`) quality gates (build-time, not runtime). | — |

- **`parser-core` is pure and deterministic**: it takes input and returns artifacts/errors with no
  side effects — no clock reads, no filesystem, no network, no randomness. I/O lives in the
  binaries (`cli`, `worker`).
- Dependencies point toward `parser-contract`; binaries depend on `core` + `contract`; nothing
  depends on a binary.
- Binaries hold **no parsing logic** — they wire arguments/transport to `parser-core` and serialize
  the result.

---

## B. The lint floor

The workspace already denies a large surface (`unsafe_code = forbid`, `warnings = deny`,
`missing_docs = deny`, clippy `all`/`cargo`/`nursery`/`pedantic = deny`, plus `unwrap_used`,
`expect_used`, `panic`, `todo`, `unimplemented`, `float_cmp`, `as_conversions`, `integer_division`,
`print_stdout`, `dbg_macro`). These conventions **build on that floor — they do not restate it.**

- A clean `cargo clippy` and `cargo build` (warnings are errors) is a hard gate, not a nicety.
- Never silence a lint with `#[allow(...)]`. Use `#[expect(lint, reason = "…")]` so the suppression
  is justified and fails the build when it becomes unnecessary.
- `unwrap`/`expect`/`panic` are denied — a parser never panics on input (see parsing); reach for
  typed errors.

---

## Reference map

Load the file that matches the code you're touching — not all at once.

| File | Covers |
|------|--------|
| `references/determinism-and-contract.md` | **§C Determinism** (byte-identical artifacts, ordered output, float/overflow rules) and **§G the parser-contract** (semver, JSON Schema, golden manifest, cargo-semver-checks) — the two headline rules. |
| `references/parsing-types-errors.md` | **§D error handling** (thiserror, C-GOOD-ERR), **§E types** (newtypes, exhaustive matches, C-CONV), **§F parsing & malformed-input totality** (size/recursion caps, deny_unknown_fields, fuzzability). |
| `references/worker-build-perf.md` | **§H async & worker** (tokio, DLX/prefetch/drain/S3 timeouts, durability), **§I docs/API hygiene/perf**, **§J build/supply-chain/CI** (cargo-deny/audit, MSRV, reproducible builds). |

---

## Testing

Test conventions — the golden/parity harness, required proptest/insta/fuzz, determinism tests, and
the coverage gate — live in
[`solidstats-parser-rust-tests`](../solidstats-parser-rust-tests/SKILL.md), on top of
[`solidstats-process-testing-standards`](../solidstats-process-testing-standards/SKILL.md).
