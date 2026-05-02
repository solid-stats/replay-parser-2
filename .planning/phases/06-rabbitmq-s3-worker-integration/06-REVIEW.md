---
phase: 06-rabbitmq-s3-worker-integration
reviewed: 2026-05-02T15:31:06Z
depth: deep
files_reviewed: 37
files_reviewed_list:
  - Cargo.toml
  - README.md
  - crates/parser-cli/Cargo.toml
  - crates/parser-cli/src/main.rs
  - crates/parser-cli/tests/parse_command.rs
  - crates/parser-cli/tests/worker_command.rs
  - crates/parser-contract/examples/export_worker_schemas.rs
  - crates/parser-contract/examples/parse_completed.v1.json
  - crates/parser-contract/examples/parse_failed.v1.json
  - crates/parser-contract/examples/parse_job.v1.json
  - crates/parser-contract/src/lib.rs
  - crates/parser-contract/src/schema.rs
  - crates/parser-contract/src/worker.rs
  - crates/parser-contract/tests/schema_contract.rs
  - crates/parser-contract/tests/worker_message_contract.rs
  - crates/parser-core/src/artifact.rs
  - crates/parser-core/src/lib.rs
  - crates/parser-core/tests/parser_core_api.rs
  - crates/parser-worker/Cargo.toml
  - crates/parser-worker/src/amqp.rs
  - crates/parser-worker/src/artifact_key.rs
  - crates/parser-worker/src/checksum.rs
  - crates/parser-worker/src/config.rs
  - crates/parser-worker/src/error.rs
  - crates/parser-worker/src/lib.rs
  - crates/parser-worker/src/processor.rs
  - crates/parser-worker/src/runner.rs
  - crates/parser-worker/src/shutdown.rs
  - crates/parser-worker/src/storage.rs
  - crates/parser-worker/tests/amqp.rs
  - crates/parser-worker/tests/artifact_key.rs
  - crates/parser-worker/tests/config.rs
  - crates/parser-worker/tests/processor.rs
  - crates/parser-worker/tests/shutdown.rs
  - crates/parser-worker/tests/storage.rs
  - schemas/parse-job-v1.schema.json
  - schemas/parse-result-v1.schema.json
findings:
  critical: 4
  warning: 1
  info: 0
  total: 5
status: issues_found
---

# Phase 6: Code Review Report

**Reviewed:** 2026-05-02T15:31:06Z
**Depth:** deep
**Files Reviewed:** 37
**Status:** issues_found

## Summary

Deep review traced the Phase 6 path from `ParseJobMessage` deserialization through S3 download, checksum validation, parser-core invocation, artifact write/reuse, RabbitMQ result publish, and final ack/nack. The worker correctly avoids the debug sidecar path, performs checksum validation before parsing, and leaves health/readiness plus multi-worker guarantees to Phase 7. However, several Phase 6 reliability and contract defects remain in the RabbitMQ/S3 integration path.

`Cargo.lock` was supplied in the workflow file list but filtered from review per the lock-file exclusion rule.

## Critical Issues

### CR-01: [BLOCKER] Result messages are acknowledged after transient RabbitMQ publishes

**File:** `/home/afgan0r/Projects/SolidGames/replay-parser-2/crates/parser-worker/src/amqp.rs:213`

**Issue:** `publish_prepared` enables publisher confirms, but publishes result messages with only `content_type` set. AMQP messages are transient unless `delivery_mode = 2` is set. `delivery_action_after_publish` then maps any confirmed `parse.completed` or `parse.failed` publish to `Ack`, so a broker restart after the confirm but before `server-2` consumes the result can lose the result while the input job has already been acknowledged. This violates the Phase 6 requirement that manual ack happen only after a durable completed/failed outcome.

**Fix:**
```rust
let properties = BasicProperties::default()
    .with_content_type(publish.content_type.into())
    .with_delivery_mode(2);

let confirm = self
    .publish_channel
    .basic_publish(
        publish.exchange.into(),
        publish.routing_key.into(),
        BasicPublishOptions { mandatory: true, ..Default::default() },
        &publish.body,
        properties,
    )
    .await
    .map_err(|source| rabbitmq_publish_error(source.to_string()))?;
```

Add an AMQP unit test that asserts the prepared or applied properties make completed and failed result messages persistent.

### CR-02: [BLOCKER] Result contract accepts contradictory `message_type` values

**File:** `/home/afgan0r/Projects/SolidGames/replay-parser-2/crates/parser-contract/src/worker.rs:50`

**Issue:** Both `ParseCompletedMessage` and `ParseFailedMessage` declare `message_type: ParseResultKind`, and `ParseResultKind` allows both `parse.completed` and `parse.failed`. The generated schema repeats that unrestricted enum for both message shapes. As a result, a completed-shaped message with `"message_type": "parse.failed"` is schema-valid and deserializable, and vice versa for failed-shaped messages. Constructors set the right value, but the wire contract itself does not enforce the discriminator, which can break `server-2` routing/validation assumptions.

**Fix:**
```rust
fn ensure_completed_kind(kind: ParseResultKind) -> Result<ParseResultKind, String> {
    if kind == ParseResultKind::Completed {
        Ok(kind)
    } else {
        Err("completed result must use message_type parse.completed".to_owned())
    }
}
```

Apply equivalent validation during deserialization for both message structs, or split the discriminator into variant-specific const schema types. Also patch `parse_result_schema()` so `ParseCompletedMessage.message_type` has `const: "parse.completed"` and `ParseFailedMessage.message_type` has `const: "parse.failed"`, then add schema tests for the swapped values.

### CR-03: [BLOCKER] Raw S3 read failures are published as output write failures

**File:** `/home/afgan0r/Projects/SolidGames/replay-parser-2/crates/parser-worker/src/storage.rs:257`

**Issue:** `s3_error()` hard-codes `stage: ParseStage::Output`. It is used by `get_object_bytes()` for raw replay downloads, so transient `get_object` failures before checksum and parse are later mapped by `storage_failed_message()` to `output.s3_write` instead of `io.s3_read`. That produces incorrect `parse.failed` stage/error data for WORK-02/WORK-06 and can send `server-2` down the wrong remediation path.

**Fix:**
```rust
fn s3_error(
    operation: &'static str,
    bucket: &str,
    key: &str,
    stage: ParseStage,
    retryability: Retryability,
    source: impl std::error::Error,
) -> WorkerError {
    WorkerError::S3 {
        operation,
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        stage,
        retryability,
        message: source.to_string(),
    }
}
```

Call it with `ParseStage::Input` for raw `get_object` / body collection and `ParseStage::Output` for artifact existence checks and `put_object`. Add a processor test that a raw S3 get failure publishes `io.s3_read`.

### CR-04: [BLOCKER] AMQP URL redaction can leak part of a password containing `@`

**File:** `/home/afgan0r/Projects/SolidGames/replay-parser-2/crates/parser-worker/src/config.rs:282`

**Issue:** `redact_userinfo()` redacts up to the first `@`. If an operator supplies an AMQP URL with an unescaped `@` in the password, for example `amqp://worker:p@ss@rabbitmq:5672/%2f`, the redacted debug output becomes `amqp://***@ss@rabbitmq:5672/%2f`, leaking `ss`. `runner::run()` logs `config.redacted()` at startup, so this is a secret disclosure edge case in the Phase 6 redaction boundary.

**Fix:**
```rust
fn redact_userinfo(value: &str) -> String {
    value.rfind('@').map_or_else(
        || value.to_owned(),
        |userinfo_end| format!("***@{}", &value[userinfo_end + 1..]),
    )
}
```

Add a regression test with a password containing `@`; over-redaction is safer than leaking userinfo in logs.

## Warnings

### WR-01: [WARNING] Empty job identifiers and object keys pass the runtime contract

**File:** `/home/afgan0r/Projects/SolidGames/replay-parser-2/crates/parser-contract/src/worker.rs:17`

**Issue:** `job_id`, `replay_id`, and `object_key` are plain `String` fields, and the committed parse-job schema only requires `"type": "string"`. The processor accepts an empty `job_id` and can publish an uncorrelatable `parse.completed`; it also attempts S3 access before rejecting an empty `replay_id` during artifact-key construction. Empty `object_key` can cause a read attempt against the empty S3 key instead of a schema failure.

**Fix:** Add non-empty validation to the job contract and runtime before storage access. For example, add `minLength: 1` to schema generation and call a `validate_job_fields(&job)` helper immediately after deserialization in `process_decoded_job`; invalid values should publish non-retryable `schema.parse_job` failures without touching S3.

---

_Reviewed: 2026-05-02T15:31:06Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
