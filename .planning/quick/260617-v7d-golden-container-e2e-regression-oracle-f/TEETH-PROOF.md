# Teeth Proof — Golden Container E2E Regression Oracle (260617-v7d)

**Date:** 2026-06-17
**Goal:** prove the oracle is NOT a no-op smoke test (BRAINSTORM risk: "Oracle degrades
into a no-op smoke test", HIGH). Two temporary mutations each turn the oracle RED; both
reverted; working tree clean. No mutated source/baseline committed — only this proof doc.

Docker WAS available on the authoring machine, so the container e2e red-run under
Mutation A is also recorded (not just the fast consumer).

---

## Mutation A — file-byte mutation of the committed baseline

Proves the committed `valid-minimal.expected.json` is genuinely byte-pinned, on BOTH
consumers (fast in-process + container e2e), since they share one baseline.

**Edit (temporary):**
```
crates/parser-core/tests/fixtures/golden/expected/valid-minimal.expected.json
  "value":"Altis"   ->   "value":"Altiz"     (world_name value, one byte: 's' 0x73 -> 'z' 0x7A)
```

**Fast consumer command + result (RED):**
```
$ cargo test -p parser-core --test golden_artifact_bytes
test golden_artifact_bytes_should_match_committed_baseline_byte_for_byte ... FAILED
assertion `left == right` failed: parse_replay output bytes drifted from the committed
golden baseline; if this is an intentional contract change, regenerate
tests/fixtures/golden/expected/valid-minimal.expected.json
  left:  [.. 65,108,116,105,115..]   # 'A','l','t','i','s'  (parser output)
  right: [.. 65,108,116,105,122..]   # 'A','l','t','i','z'  (mutated baseline)
test result: FAILED. 0 passed; 1 failed; ...
```

**Container e2e command + result (RED), Docker present:**
```
$ AWS_ACCESS_KEY_ID=minioadmin AWS_SECRET_ACCESS_KEY=minioadmin \
    cargo test -p parser-worker --test golden_container_e2e -- --ignored
test golden_container_e2e_should_pin_full_worker_contract_byte_for_byte ... FAILED
assertion `left == right` failed: S3 artifact bytes must equal the committed baseline
test result: FAILED. 0 passed; 1 failed; ... finished in 4.06s
```
The worker wrote the real (unmutated) bytes to MinIO; the byte-exact compare against the
mutated baseline failed — exactly as the fast consumer did, off the SAME baseline.

**Revert + confirm green:**
```
$ git checkout -- crates/parser-core/tests/fixtures/golden/expected/valid-minimal.expected.json
$ grep -o '"value":"Alti."' .../valid-minimal.expected.json   ->   "value":"Altis"
$ cargo test -p parser-core --test golden_artifact_bytes
test result: ok. 1 passed; 0 failed; ...
```

---

## Mutation B — behavioral parse-path mutation in parser-core

Proves the PARSE PATH is pinned, not just the committed file: a change in how the parser
builds the artifact turns the byte-exact compare RED even with the baseline untouched.

**Edit (temporary):**
```
crates/parser-core/src/metadata.rs:36
  raw.string_field("worldName")   ->   raw.string_field("worldNameX")
```
(the source key the parser reads world_name from is broken, so world_name no longer
resolves from the fixture and the re-serialized artifact no longer matches the baseline)

**Fast consumer command + result (RED):**
```
$ cargo test -p parser-core --test golden_artifact_bytes
test golden_artifact_bytes_should_match_committed_baseline_byte_for_byte ... FAILED
assertion `left == right` failed: parse_replay output bytes drifted from the committed
golden baseline; ...
test result: FAILED. 0 passed; 1 failed; ...
```

**Revert + confirm green:**
```
$ git checkout -- crates/parser-core/src/metadata.rs
$ sed -n '36p' crates/parser-core/src/metadata.rs   ->   raw.string_field("worldName"),
$ cargo test -p parser-core --test golden_artifact_bytes
test result: ok. 1 passed; 0 failed; ...
```

Mutation B was validated via the fast consumer. The container e2e shares the same baseline
and the same pinned identity constants (`tests/common/golden_identity.rs`), and the worker
parses through the same `parser_core` path, so a behavioral parse-path drift fails the e2e
identically; the master-only CI job runs the e2e under Docker on every pre-deploy.

---

## Bonus: the oracle caught a real bug during construction

While first running the container e2e green, the S3-bytes compare FAILED because the
baseline had been generated through `parser_core::parse_replay` (which retains per-field
`source` provenance) while the worker writes `public_parse_replay` output (provenance
stripped — processor.rs:136). The oracle flagged the mismatch immediately; the baseline
was regenerated through `public_parse_replay` and the fast consumer switched to it. This
is independent evidence the oracle is not a no-op.

---

---

## Real-corpus teeth re-confirmation (2026-06-18)

After extending the oracle with 4 byte-exact REAL OCAP fixtures pulled from Timeweb S3
(`sg-replays raw/sha256/<hash>.ocap`, sha256-verified) — small/mid/large success + one
partial (real schema-drift) — the mutation-turns-red proof was re-run on a REAL fixture,
on BOTH consumers, with Docker present.

Real fixtures (committed, gzip-at-rest under `crates/parser-core/tests/fixtures/golden/real/`):

| label | sha256 (file stem) | raw | gzip | status | baseline |
|-------|--------------------|-----|------|--------|----------|
| real-small-success | 00118b23… | 10 KB | 1776 B | success | real-small-success.expected.json |
| real-mid-success | 0006b10d… | 167 KB | 28117 B | success | real-mid-success.expected.json |
| real-large-success | 0053d62b… | 1.07 MB | 114759 B | success | real-large-success.expected.json |
| real-partial-schema-drift | 00085e03… | 273 KB | 39517 B | partial | real-partial-schema-drift.expected.json |

Total committed gzip ≈ 184 KB (≤ 500 KB target). Every committed artifact baseline ≤ 100 KB
(largest = real-large-success.expected.json at 8847 B).

### Mutation A-real (file-byte) — REAL fixture baseline

**Edit (temporary):** flip one byte in the committed real-large-success baseline —
`"status":"success"` → `"status":"xuccess"` (one byte: 's' 0x73 → 'x' 0x78), at offset 416.

**Fast consumer (RED):**
```
$ cargo test -p parser-core --test golden_artifact_bytes \
    golden_artifact_bytes_should_match_real_corpus_baselines_byte_for_byte
test ... FAILED
assertion `left == right` failed: real fixture `real-large-success`
(0053d62b…): artifact bytes drifted from its committed baseline real-large-success.expected.json
test result: FAILED. 0 passed; 1 failed; ...
```

**Container e2e (RED), Docker present:**
```
$ AWS_ACCESS_KEY_ID=minioadmin AWS_SECRET_ACCESS_KEY=minioadmin \
    cargo test -p parser-worker --test golden_container_e2e -- --ignored
thread '…' panicked at crates/parser-worker/tests/golden_container_e2e.rs:295:9:
assertion `left == right` failed: real fixture `real-large-success`
(0053d62b…): S3 artifact bytes must equal the committed baseline real-large-success.expected.json
test result: FAILED. 0 passed; 1 failed; …
```
The worker wrote the real (unmutated) bytes to MinIO; the byte-exact compare against the
mutated baseline failed — the SAME baseline both consumers share. Reverted by regenerating
the baseline (the file is new/untracked, so `git checkout` cannot restore it):
`cargo test -p parser-core --test golden_artifact_bytes -- --ignored regenerate` → status
`success`, no `xuccess` residue.

### Mutation B-real (behavioral parse-path) — REAL fixtures

**Edit (temporary):** `crates/parser-core/src/metadata.rs:36`
`raw.string_field("worldName")` → `raw.string_field("worldNameX")` (world_name no longer
resolves from the real replays, so the re-serialized artifact drifts).

**Fast consumer (RED):**
```
$ cargo test -p parser-core --test golden_artifact_bytes \
    golden_artifact_bytes_should_match_real_corpus_baselines_byte_for_byte
test ... FAILED
assertion `left == right` failed: real fixture `real-small-success`
(00118b23…): artifact bytes drifted from its committed baseline real-small-success.expected.json
test result: FAILED. 0 passed; 1 failed; ...
```
Proves the parse PATH (not just the committed file) is pinned on real data. Reverted with
`git checkout -- crates/parser-core/src/metadata.rs`; fast golden suite green again.

### Full container e2e green on real data

Before the mutations, the full container e2e ran GREEN with Docker present (booted
ephemeral RabbitMQ + MinIO, drove the real worker via `run_until_cancelled`, and for EVERY
fixture — valid-minimal + all 4 real fixtures — asserted byte-exact S3 artifact == its
committed baseline, plus key/checksum/size, idempotency, checksum-mismatch, and
artifact-conflict):
```
$ AWS_ACCESS_KEY_ID=minioadmin AWS_SECRET_ACCESS_KEY=minioadmin \
    cargo test -p parser-worker --test golden_container_e2e -- --ignored
test golden_container_e2e_should_pin_full_worker_contract_byte_for_byte ... ok
test result: ok. 1 passed; 0 failed; … finished in 13.14s
```

---

## Working tree clean

Both original mutations AND the real-corpus mutations were reverted; no mutated source or
baseline committed (only the intended task artifacts: gzipped real fixtures, generated
baselines, manifest entries, test wiring, the capture-script provenance update, and this
proof doc).

```
$ git status --short
(empty — only the intended task artifacts staged/committed; no mutation residue)
```

Final fast golden suite: **green**. Final container e2e (Docker present): **green**.
