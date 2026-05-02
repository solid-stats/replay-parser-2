//! Worker runtime error types.

use thiserror::Error;

/// Errors produced by the worker runtime adapter.
#[derive(Debug, Error)]
pub enum WorkerError {
    /// Worker configuration failed validation.
    #[error("invalid worker configuration: {0}")]
    ConfigValidation(String),
    /// RabbitMQ/AMQP operation failed.
    #[error("AMQP operation failed: {0}")]
    Amqp(#[from] lapin::Error),
    /// S3 operation failed.
    #[error("S3 operation failed during {operation}: {message}")]
    S3 {
        /// Operation being attempted.
        operation: &'static str,
        /// Error details without secret-bearing configuration.
        message: String,
    },
    /// Worker message or artifact serialization failed.
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Raw or artifact checksum validation failed.
    #[error("checksum validation failed: {0}")]
    ChecksumValidation(String),
    /// Parser metadata could not be constructed or validated.
    #[error("parser metadata error: {0}")]
    ParserMetadata(String),
    /// RabbitMQ publisher confirmation was negative or missing.
    #[error("publish confirmation failed: {0}")]
    PublishConfirm(String),
}
