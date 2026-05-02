//! Worker runtime error types.

use parser_contract::{
    failure::{ParseStage, Retryability},
    source_ref::SourceChecksum,
};
use thiserror::Error;

/// Structured worker failure categories that can be mapped into `parse.failed` payloads.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkerFailureKind {
    /// Raw replay checksum did not match the parse job checksum.
    #[error(
        "checksum.mismatch: expected SHA-256 {expected_value}, computed SHA-256 {actual_value}",
        expected_value = expected.value.as_str(),
        actual_value = actual.value.as_str()
    )]
    ChecksumMismatch {
        /// Checksum supplied by the parse job.
        expected: SourceChecksum,
        /// Locally computed checksum from downloaded bytes.
        actual: SourceChecksum,
    },
    /// Deterministic artifact key already contains different bytes.
    #[error(
        "output.artifact_conflict: existing object at {key} has SHA-256 {existing_value} and {existing_size_bytes} bytes, new artifact has SHA-256 {new_value} and {new_size_bytes} bytes",
        existing_value = existing_checksum.value.as_str(),
        new_value = new_checksum.value.as_str()
    )]
    ArtifactConflict {
        /// Artifact object key.
        key: String,
        /// Existing object checksum.
        existing_checksum: SourceChecksum,
        /// Existing object byte size.
        existing_size_bytes: u64,
        /// Newly produced artifact checksum.
        new_checksum: SourceChecksum,
        /// Newly produced artifact byte size.
        new_size_bytes: u64,
    },
    /// Internal worker failure with a stable error-code string.
    #[error("{code}: {message}")]
    Internal {
        /// Stable parser error-code string.
        code: &'static str,
        /// Safe human-readable message.
        message: String,
    },
}

impl WorkerFailureKind {
    /// Returns the stable parse failure error code.
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::ChecksumMismatch { .. } => "checksum.mismatch",
            Self::ArtifactConflict { .. } => "output.artifact_conflict",
            Self::Internal { code, .. } => code,
        }
    }

    /// Returns the parser stage where the worker failure belongs.
    #[must_use]
    pub const fn stage(&self) -> ParseStage {
        match self {
            Self::ChecksumMismatch { .. } => ParseStage::Checksum,
            Self::ArtifactConflict { .. } => ParseStage::Output,
            Self::Internal { .. } => ParseStage::Internal,
        }
    }

    /// Returns the retryability classification for this failure.
    #[must_use]
    pub const fn retryability(&self) -> Retryability {
        match self {
            Self::ChecksumMismatch { .. } => Retryability::NotRetryable,
            Self::ArtifactConflict { .. } | Self::Internal { .. } => Retryability::Unknown,
        }
    }
}

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
    /// Deterministic artifact key construction failed.
    #[error("artifact key construction failed: {0}")]
    ArtifactKey(String),
    /// Raw or artifact checksum validation failed.
    #[error("checksum validation failed: {0}")]
    ChecksumValidation(String),
    /// Structured worker failure that should become a parse.failed result.
    #[error(transparent)]
    Failure(#[from] WorkerFailureKind),
    /// Parser metadata could not be constructed or validated.
    #[error("parser metadata error: {0}")]
    ParserMetadata(String),
    /// RabbitMQ publisher confirmation was negative or missing.
    #[error("publish confirmation failed: {0}")]
    PublishConfirm(String),
}
