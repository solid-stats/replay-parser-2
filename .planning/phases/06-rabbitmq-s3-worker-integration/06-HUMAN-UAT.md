---
status: complete
phase: 06-rabbitmq-s3-worker-integration
source: [06-VERIFICATION.md]
started: 2026-05-02T16:01:31Z
updated: 2026-05-02T16:29:10Z
---

# Phase 6 Human UAT

## Current Test

Live RabbitMQ/S3-compatible smoke verification passed against local Docker Compose infrastructure.

## Tests

### 1. Live RabbitMQ/S3 staging smoke

expected: Worker consumes a real server-2-compatible parse job, fetches the raw S3 object, verifies checksum, writes the parse artifact, publishes parse.completed or parse.failed, and only then ack/nacks the RabbitMQ delivery.
result: pass

evidence: `scripts/worker-smoke.sh` started live RabbitMQ and MinIO services, uploaded the valid OCAP fixture through the smoke harness, ran the worker, verified `parse.completed` plus the S3 artifact, then verified checksum-mismatch `parse.failed`.

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
