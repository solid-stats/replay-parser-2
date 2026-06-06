# Parsing, types & error handling

How the parser consumes untrusted input, models its domain, and reports failure. Determinism and the
contract these feed live in `determinism-and-contract.md`.

## §D. Error handling

- **`thiserror` everywhere** — typed error enums per crate (libraries *and* binaries; the repo
  deliberately avoids `anyhow`). One error enum per crate/module surface, variants named for the
  failure.
- Return `Result<T, E>`; propagate with `?`; no match-chain boilerplate.
- **No `panic`/`unwrap`/`expect`** (lint-denied) on any path — malformed input is a typed error
  variant, never a crash (§F).
- Preserve the source: `#[from]` / `#[source]` so the cause chain survives; carry an identifying
  detail (the offending field/offset) in the variant where it helps diagnosis.
- Public error types are **well-behaved (C-GOOD-ERR)**: they `impl std::error::Error`, are
  `Send + Sync + 'static` (the worker moves errors across tasks), and have a lowercase,
  no-trailing-punctuation `Display`.

## §E. Types & domain modeling

- **Newtypes over primitive obsession** — `ReplayId(String)`, `SteamId64(String)`, a typed event
  kind, etc. A bare `String`/`u64` for a domain id is a finding.
- **Exhaustive matches** — match every enum variant explicitly; avoid a `_` wildcard that would
  silently absorb a new variant (the parser must consciously handle each OCAP shape).
- **Make invalid states unrepresentable** — encode invariants in the type (type-state / enums) so a
  bad combination doesn't compile, rather than guarding it at runtime.
- Prefer borrows (`&str`, `&[T]`) in signatures; `Cow` when ownership is conditional; pass small
  `Copy` types by value.
- **Conversions** are `From` / `TryFrom` (raw parser output → contract types), never a hand-written
  `Into` / `TryInto` — those come free from the blanket impl (Rust API Guidelines C-CONV).

## §F. Parsing & malformed input

The parser consumes **untrusted** OCAP JSON. It must be **total** over arbitrary bytes — every input
either parses to an artifact or returns a typed error; nothing panics, hangs, or silently drops data.

- Use `serde` derives; reach for zero-copy (`&str`, `Cow`, `serde_json::value::RawValue`) on hot
  paths where it avoids allocation.
- **Validate at the boundary** and convert violations into typed errors (§D) — never `unwrap` a
  parse, never index without a bounds-aware accessor.
- **Bound the input size.** `serde_json` has no internal size guard, so untrusted bytes from S3 /
  RabbitMQ are read through a cap — `serde_json::from_reader(reader.take(MAX_BYTES))` — and an
  oversized payload is rejected rather than allowed to exhaust worker memory.
- **Respect the recursion limit.** Never enable `serde_json`'s `unbounded_depth` /
  `disable_recursion_limit` on untrusted input — serde_json's built-in 128-deep guard prevents
  stack-overflow DoS on adversarial nesting. The parse guard does **not** cover recursive `Drop` of a
  deeply-nested `Value` tree (serde-rs/json#440), so cap nesting at the boundary if arbitrarily deep
  input is ever legitimate.
- **Reject non-finite floats.** `serde_json` serializes `NaN`/`±Inf` as `null` — a silent, lossy
  corruption of the artifact. Reject or clamp non-finite `f64` before it enters a derived field.
- **Default to `#[serde(deny_unknown_fields)]`** on artifact-bound types so unexpected/attacker-shaped
  keys are a typed error, not silently ignored; note that serde keeps the *last* of duplicate JSON
  keys. Read untrusted bodies with a size cap (above), and when pulling from S3 stream the object and
  reject on `content_length > MAX` before downloading.
- Unknown/extra fields and unexpected shapes are handled deliberately (rejected with a typed error or
  explicitly ignored per the contract) — not a silent default that hides corruption.
- Because totality can't be proven by example tests alone, fuzzing the decode path is **required**
  (see the tests skill) — the convention here is that the decode path is written to *be* fuzzable
  (no panics, bounded work).
