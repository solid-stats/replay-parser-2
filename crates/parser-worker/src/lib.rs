//! RabbitMQ/S3 worker runtime adapter for `replay-parser-2`.
//!
//! Runtime concerns live here so `parser-core` and `parser-contract` remain
//! transport-free and deterministic.

/// Worker configuration and redacted display helpers.
pub mod config;
/// Worker runtime error types.
pub mod error;
/// Worker runtime entrypoint.
pub mod runner;
