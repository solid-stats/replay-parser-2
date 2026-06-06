---
name: solidstats-parser-rust-code-review
description: >
  Pedantic code review for the SolidStats Rust OCAP parser (replay-parser-2). Builds on
  solidstats-process-review-standards (severity buckets, output format, verdict, scope, noise
  filter) and enforces solidstats-parser-rust-conventions as its rule library. Runs a
  contract-&-determinism gate, then a convention and correctness sweep with a parser-specific
  severity table. Use when reviewing parser code, verifying a finished parser task, or checking a
  parser PR.
  Use this proactively — apply it when reviewing, verifying, or checking ANY parser Rust change, even
  casually.
  Triggers: "review parser", "review rust", "code review", "check the parser", "review the decoder",
  "ревью парсера", "посмотри rust", "проверь парсер", "проверь декодер", "проверь реализацию парсера".
---

# Parser Code Review — Rust / OCAP

**This skill builds on [`solidstats-process-review-standards`](../solidstats-process-review-standards/SKILL.md) — read it first.**
That skill owns the review philosophy, scope resolution (read every changed file in full), the
severity buckets (🔴🟠🟡🔵), continuous-numbering output, the verdict rules, the test-file rule, and
the noise filter. It must be installed alongside this skill.

**The rule library is [`solidstats-parser-rust-conventions`](../solidstats-parser-rust-conventions/SKILL.md)** —
this skill enforces it, it does not restate it. Every finding cites the convention it breaks
(`[conv: §C]` determinism, `[conv: §F]` parsing, `[conv: §G]` contract, … — the §-letters live in the
conventions' `references/` files; see its reference map for which file holds each) and takes its severity
from the Severity reference table below — the `[conv: …]` citation identifies *which* rule, not its
severity (the conventions skill states rules, not severity tags).

Review happens in two phases, in order.

---

## Phase 1 — Contract & determinism gate (blocking)

This is the parser analog of an API-contract gate. The parser's two non-negotiables (conventions §C
and §G) are gated before anything else, because a drifting output or a silent contract change breaks
every downstream consumer (server-2 ingests these artifacts).

**Determinism:**
- The `deterministic_output` test passes, and any new derived field uses ordered output
  (`BTreeMap`/sorted), `float_roundtrip` serialization, and no nondeterministic source
  (`SystemTime`/`rand`/`HashMap` iteration into the artifact). `overflow-checks` stays on.
- A change that can make the same input produce a different artifact run-to-run or arch-to-arch is a
  **BLOCK**.

**Contract:**
- Any change to the artifact shape is **versioned**: contract semver bumped (`ParserInfo`), JSON
  Schema regenerated, golden manifest updated, `schema_drift_status` green.
- **`cargo-semver-checks`** passes against the last published `parser-contract`, and the **PR-time
  JSON Schema diff** is reviewed (it doesn't catch field *type* changes — verify those by eye).
- A breaking artifact change (removed/renamed/retyped field, changed status meaning) is acceptable
  only if coordinated with the consumer (server-2). An unflagged breaking change with no
  justification is a **BLOCK**.

**Lints / supply-chain:** the workspace denies warnings, so a `clippy`/`cargo build` warning is a
build failure — treat an introduced lint break as a gate failure. `cargo-deny` / `cargo-audit` clean
is part of the gate.

Render the gate at the top of the report:

```
## Contract & determinism
✅ deterministic_output green; no artifact-shape change; clippy/deny clean.
⚠️ Artifact field `x` added — additive, schema regenerated, semver bumped 3.1→3.2, manifest updated.
❌ `HashMap` iteration feeds the artifact → output non-deterministic → BLOCK
❌ Field `playerId` retyped i64→string with no version bump / no server-2 coordination → BLOCK
```

A failing gate is a **BLOCK**, in addition to the standard "any 🔴 → BLOCK" rule. *(The CLI/worker
binaries have no published artifact contract surface of their own — for a change touching only
`parser-cli`/`parser-worker` internals, note "contract: N/A" and gate on determinism + lints only.)*

---

## Phase 2 — Convention & correctness sweep

Read every changed file in full, then sweep against `solidstats-parser-rust-conventions` in **risk
order**:

1. **Determinism** — ordered output, no `HashMap` into artifacts, no `SystemTime`/`rand`,
   `float_roundtrip`, `overflow-checks`, total stable ordering. `[conv: §C]`
2. **Untrusted-input safety / totality** — no `panic`/`unwrap`/`expect` on the input path,
   input-size cap (`Read::take`), serde_json recursion limit not disabled, malformed input → typed
   error (never a silent default). `[conv: §F]`
3. **Error handling** — `thiserror` typed enums, `?`, `#[from]`/`#[source]`, identifying detail in
   the variant. `[conv: §D]`
4. **Contract discipline** — versioning/schema/manifest, `#[non_exhaustive]`, conversions via
   `From`/`TryFrom` not `Into` (C-CONV), newtype-hidden representation. `[conv: §G/§E]`
5. **Types & domain modeling** — newtypes over primitives, exhaustive matches (no `_` hiding
   variants), make invalid states unrepresentable. `[conv: §E]`
6. **Architecture** — `parser-core` stays pure/deterministic (no I/O/clock/network), crate boundaries
   respected, no logic in the binaries. `[conv: §A]`
7. **Async / worker** — tokio discipline (no `std::thread::sleep`, no lock across `.await`, bounded
   concurrency), durable jobs / ack / idempotency, graceful shutdown, `tracing`. `[conv: §H]`
8. **Build & supply-chain** — `cargo-deny`/`cargo-audit`, MSRV, `overflow-checks`, profile policy.
   `[conv: §J]`
9. **Docs / perf / quality** — `missing_docs`, `#[must_use]`, perf (iterators, box large variants),
   the lint floor. `[conv: §I/§B]`

Each finding lands in one severity bucket, carries a `[topic]` tag, and cites `[conv: …]`. Take the
severity from the Severity reference table below.

---

## Severity reference

| Finding | Severity |
|---------|----------|
| Nondeterminism in derived output (HashMap iteration, SystemTime/rand, unsorted collection, non-finite float→`null`) | 🔴 (artifacts drift) |
| `panic`/`unwrap`/`expect` reachable on the untrusted-input path | 🔴 |
| Unflagged breaking contract / artifact-shape change (Phase 1) | 🔴 BLOCK |
| Missing input-size cap / disabled serde_json recursion limit on untrusted input | 🟠 (🔴 if a reachable DoS) |
| Poison message requeued with no DLX / `delivery-limit`; unbounded consumer prefetch | 🟠 |
| `overflow-checks` off in release; integer overflow on untrusted counts | 🟠 |
| Logic / clock / network in `parser-core` (impurity) | 🟠 |
| `deny_unknown_fields` off on an artifact-bound type; unbounded S3 read | 🟠 |
| `cargo-deny` / `cargo-audit` advisory; MSRV undeclared | 🟠 |
| Non-exhaustive `_` match hiding variants; hand-written `Into`/`TryInto`; missing newtype | 🟡 |
| Missing `#[non_exhaustive]` on a growing public enum; shutdown signals but doesn't drain | 🟡 |
| Docs / naming / style; missing `#[must_use]` | 🔵 |

---

## Output

Follow the output format, continuous numbering, severity buckets, and verdict rules from
`solidstats-process-review-standards` (§D–§E). Open the report with the **Contract & determinism**
gate result (above the buckets); there is no "Good" section. Cite the broken convention on each
finding. The test-file rule (test quality is never a standalone BLOCK unless a test actively masks a
real bug) lives in review-standards §F and applies unchanged; defer detailed test-quality judgement
to [`solidstats-parser-rust-tests`](../solidstats-parser-rust-tests/SKILL.md).
