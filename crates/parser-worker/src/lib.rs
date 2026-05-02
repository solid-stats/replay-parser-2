//! RabbitMQ/S3 worker runtime adapter for `replay-parser-2`.
//!
//! Runtime concerns live here so `parser-core` and `parser-contract` remain
//! transport-free and deterministic.

/// RabbitMQ consumer, result publishing, and acknowledgement helpers.
pub mod amqp;
/// Deterministic parser artifact object keys.
pub mod artifact_key;
/// Local checksum helpers for raw replay and artifact bytes.
pub mod checksum;
/// Worker configuration and redacted display helpers.
pub mod config;
/// Worker runtime error types.
pub mod error;
/// Cached HTTP liveness and readiness probe state.
pub mod health;
/// Stable worker operations log event taxonomy.
pub mod logging;
/// End-to-end parse job processing.
pub mod processor;
/// Worker runtime entrypoint.
pub mod runner;
/// Graceful shutdown drain helpers.
pub mod shutdown;
/// S3-compatible raw replay and artifact object storage.
pub mod storage;
