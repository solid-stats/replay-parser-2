# Determinism & the parser-contract

The two non-negotiable rules. Everything else in the conventions serves these: a parser whose output
drifts run-to-run, or whose artifact shape changes silently, breaks every downstream consumer.

## §C. Determinism — the headline rule

The same input must produce a **byte-identical** artifact on every run, machine, and process. The
`deterministic_output` test enforces this; every new derived field must preserve it.

- **Ordered output only.** Serialize from `BTreeMap` / sorted `Vec`, never from `HashMap`/`HashSet`
  iteration — their order is randomized per process and will desync artifacts.
- **`IndexMap` default serde gives NO ordering guarantee.** The official docs are explicit: the
  default serde impl serializes `IndexMap` as a normal map with no guarantee the serialization
  format preserves key-value order. Canonical output uses `BTreeMap` (or `serde_seq`, which emits
  the entries as a tuple sequence — deterministic only insofar as the insertion order is). Never
  rely on `IndexMap`'s default serde for bit-for-bit determinism.
- **No nondeterministic sources in derived data.** No `SystemTime`/`Instant`, no `rand`, no env
  reads, no thread-scheduling-dependent ordering feeding anything that lands in the artifact.
- **Floats compare with an epsilon** (`float_cmp` is denied); define a tolerance, don't `==`. Prefer
  integers/fixed-point for derived quantities where feasible.
- **Float output is already deterministic** — `serde_json` serializes `f64` via ryu (shortest
  round-trip) **unconditionally**, so emission is byte-identical across architectures with no feature
  flag. Enable the **`float_roundtrip`** feature only for *parse-side* canonicalization (a value
  parsed then re-serialized stays stable); do **not** use `arbitrary_precision` for determinism — it
  preserves the input *string*, not the value.
- **Integers are overflow-checked** — set `overflow-checks = true` in `[profile.release]`. Release
  mode otherwise wraps silently (two's-complement: deterministic but **wrong**), turning an overflow
  on an untrusted count/offset into silent corruption; use `checked_*`/`saturating_*` where graceful
  handling is wanted.
- **Canonical form for hashing** *(only if artifact content is ever hashed/signed)* — canonicalize
  per RFC 8785 (JCS), not `BTreeMap` byte order: JCS sorts object keys by UTF-16 code units, which
  diverges from Rust's UTF-8 byte order for non-ASCII keys (they agree while keys stay ASCII).
- **Total, stable ordering.** When you sort, sort by a total key (e.g. a stable id), so equal-looking
  records can't reorder between runs.
- Any new aggregation/normalization step is reviewed against this section and gets a
  `deterministic_output` fixture.

## §G. The parser-contract — versioned & stable

`parser-contract` is the published interface: the `ParseArtifact` envelope, `ParseStatus`, and the
JSON Schemas in `schemas/` that downstream consumers (server-2 ingest) validate against. Treat it
exactly like a public API.

- **Any change to artifact shape is a contract change.** Bump the contract version (`ParserInfo` /
  semver), regenerate the JSON Schema (`Schema` subcommand / schemars), and update the golden
  manifest so `schema_drift_status` stays green.
- **Additive/backward-compatible vs breaking.** Adding an optional field is additive; removing,
  renaming, retyping, or changing the meaning of a field is breaking and requires a version bump
  **and** coordination with the consumer (server-2) — never land a breaking artifact change silently
  (the code-review gate checks this).
- Contract types are documented (`missing_docs` denied) and `#[non_exhaustive]` where future growth
  is expected, so consumers don't break on additive change.
- **Semver is mechanized, not left to review** — `cargo-semver-checks` runs in CI against the last
  published `parser-contract` and fails a PR on a breaking change. (It doesn't yet catch field *type*
  changes like `i64 → String`, so those stay a manual review point.)
- **The JSON Schema is diffed at PR time** — the generated schema is committed and diffed in CI
  (catching a schema break on the PR, not only at publish), and the consumer (server-2) validates
  golden payloads against it.
- **Hide representation behind newtypes** where a contract struct exposes a collection — a
  `Players(Vec<Player>)` newtype lets the storage change without a semver break (C-NEWTYPE-HIDE).
  *(Optional: gate serde/schemars derives behind a `serde` Cargo feature per C-SERDE — usually
  unnecessary here, since the contract crate exists to serialize.)*
