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

## Working tree clean

Both mutations reverted; no mutated source or baseline committed.

```
$ git status --short
(empty — only the intended task artifacts staged/committed; no mutation residue)
```

Final fast golden suite: **green**. Final container e2e (Docker present): **green**.
