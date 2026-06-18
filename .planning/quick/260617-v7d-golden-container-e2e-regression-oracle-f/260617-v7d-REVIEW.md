# Code Review — Golden Container E2E Regression Oracle (260617-v7d)

**Reviewed:** 2026-06-17
**Depth:** quick (high-signal; clippy `-D warnings` nitpicks skipped)
**Scope:** the 10 listed files (regression-oracle e2e + fast consumer + capture script + CI job)

## Contract & determinism gate

✅ Contract: **N/A** for source-shape — this change is test/infra only and introduces no
artifact-shape change. The committed baseline pins `contract_version:"3.0.0"` and the existing
artifact bytes as-is; pinned-as-is is intentional.
✅ Determinism teeth: one shared baseline (`valid-minimal.expected.json`) and one shared identity
file (`golden_identity.rs`) are `include!`d by **both** the fast in-process consumer
(`golden_artifact_bytes.rs`) and the container e2e — no duplication. Both reproduce the worker's
exact serialization (`public_parse_replay` → `serde_json::to_vec` + trailing `\n`,
matching `processor.rs:149-150`). A byte drift fails via `assert_eq!` in both layers (not swallowed).
✅ Skip-guard / no false-green: `cargo test --workspace` (verify job) runs **without** `--ignored`;
the e2e is `#[ignore]` and only the master-only `golden-container-e2e` job passes `-- --ignored`.
Missing Docker, missing seed fixture, and missing AWS creds each return `Ok(())` with a visible
`eprintln!("SKIP …")`. No silent green.
✅ Leak/flake safety: worker stopped via the `CancellationToken` seam (`run_until_cancelled`,
no real signal handler), no fixed host ports (`get_host_port_ipv4`), fresh bucket/queues + purge
per run, container handles held in `Infra` (drop = stop). The only `tokio::time::sleep` is test-side
result polling, explicitly annotated as not a worker timer.

No 🔴. Verdict: **APPROVE with warnings** — the oracle is sound; the findings below are a real
capture-script bug and a latent baseline-coupling fragility.

---

## 🟠 Warnings

### 1. `capture-golden-replays.sh` — duplicate-index bug when the store has ≤3 replays

**File:** `scripts/capture-golden-replays.sh:56`
`for IDX in 1 2 3 "$TOTAL"` produces **duplicate indices** whenever `TOTAL ≤ 3`:
- `TOTAL=3` → `1 2 3 3` (index 3 captured twice)
- `TOTAL=2` → `1 2 3 2` (index 3 skipped via the `-n` guard, index 2 captured twice)

The duplicate re-runs `gzip -c -n "$SRC" > "$GZ"` for the same `BASE` (harmless overwrite) **but emits
two manifest entries with the identical `fixture` name**, producing a malformed/duplicated
`manifest.json`. "Deterministic selection" is violated for small stores. The size-guard `continue`
also interacts badly: a duplicated index that passes the guard double-counts.

**Fix:** de-duplicate and bound the indices before the loop, e.g.
```sh
INDICES="$(printf '1\n2\n3\n%s\n' "$TOTAL" | sort -n -u)"
for IDX in $INDICES; do
  [ "$IDX" -le "$TOTAL" ] || continue
  ...
done
```
This is gated on `~/sg_stats` presence (absent on the authoring machine, so it has never run), which
is why it is 🟠 not 🔴 — but it is a genuine correctness defect in the one job the script exists to do.

### 2. Shared baseline couples two *different* crate versions — latent spurious-failure trap

**Files:** `crates/parser-core/tests/golden_artifact_bytes.rs:54` and
`crates/parser-worker/src/runner.rs:286` (exercised by `golden_container_e2e.rs`)

The single committed baseline embeds `"parser":{… "version":"0.1.0"}`. But the two consumers derive
that version from **two different crates**:
- fast consumer: `golden_parser_info()` uses `env!("CARGO_PKG_VERSION")` resolved in **parser-core**;
- worker (what the e2e actually asserts byte-for-byte): `runner::parser_info()` uses
  `env!("CARGO_PKG_VERSION")` resolved in **parser-worker**.

Today both crates are pinned to `0.1.0`, so the shared baseline is valid for both. The moment the two
crate versions diverge (an independent `parser-worker` bump, which nothing forbids), **exactly one of
the two golden layers will fail with no behavior change** — and the failure will read as "parser
drift", sending a maintainer hunting in the wrong place. The "single source of truth" comment in
`golden_identity.rs` does not cover the version, which silently comes from `CARGO_PKG_VERSION` of
whichever crate compiles the test.

**Fix:** pin the embedded version in `golden_identity.rs` as a `GOLDEN_PARSER_VERSION` constant used by
both tests (regenerate the baseline when it changes), OR assert in a small test that
`parser-core` and `parser-worker` versions are equal so the coupling is enforced loudly rather than
discovered via a confusing golden-diff.

---

## 🔵 Info

### 3. RabbitMQ boot is not skip-guarded symmetrically with MinIO

**File:** `crates/parser-worker/tests/golden_container_e2e.rs:64-71`
`MinIO::start()` failure → clean skip (`Ok(None)`), but `RabbitMq::default().start().await?` propagates
as a hard error. In practice MinIO starts first, so a *Docker-absent* environment skips before
reaching RabbitMQ — the stated "Docker absent → skip" contract holds. A genuine RabbitMQ-only failure
(image pull, port exhaustion) becoming a hard error is arguably correct (real infra fault, not "no
Docker"). Left as Info: if you want strict symmetry, mirror the `match … Ok(None)` arm for RabbitMQ;
otherwise document that only the *first* container probes for Docker presence.

---

## Non-findings checked (ruled out)

- **False-green path:** confirmed — `verify` runs `cargo test --workspace` with no `--ignored`; e2e is
  `#[ignore]`; coverage gate does not reference it. ✅
- **CI job placement:** `golden-container-e2e` is `needs: verify`, `if` master + non-PR, exports MinIO
  creds, runs `-- --ignored`, separate job — does not perturb `verify` or `image`. ✅
- **Determinism of baseline equality:** both layers use `assert_eq!` on full byte vectors incl. trailing
  `\n`; drift is not swallowed. ✅
- **Assertion completeness:** byte-exact artifact, message contract (job_id/replay_id/checksum/key/
  bucket/size), S3 key + checksum, idempotency (single artifact at key), checksum-mismatch failure,
  artifact-conflict failure (distinct replay_id so it can't poison the success assert) — all present. ✅
- **`Duration::from_mins`** (`common/mod.rs:237`): stabilized Rust 1.82, fine on MSRV 1.95. ✅
- **`expect`/`panic` lint allows** in test files: scoped `#![allow(…, reason=…)]`, infra-diagnostic
  only — within the test-file rule, not flagged. ✅
- **`set -eu`, quoting, size guard, no-op on absent `~/sg_stats`:** present and correct. ✅
- **Unwired baseline regen / `"expected_status":"unverified"`** in the capture script: deliberately
  pinned tech-debt per the header comment — not flagged. ✅
- **Cargo.lock testcontainers-modules 0.15.0** dev-dep: matches `Cargo.toml`; sanity only. ✅

---

_Reviewer: Claude (solidstats-parser-rust-code-review)_
