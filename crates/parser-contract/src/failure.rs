use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::{
    presence::FieldPresence,
    source_ref::{SourceChecksum, SourceRefs},
};

/// Parser stage where a structured failure occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParseStage {
    /// Input file or object access stage.
    Input,
    /// Source checksum validation stage.
    Checksum,
    /// JSON decoding stage.
    JsonDecode,
    /// Contract or source schema validation stage.
    Schema,
    /// Replay normalization stage.
    Normalize,
    /// Aggregate derivation stage.
    Aggregate,
    /// Output writing or publishing stage.
    Output,
    /// Internal invariant or unexpected parser stage.
    Internal,
}

/// Retryability classification for a parse failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Retryability {
    /// Retrying the same job may succeed.
    Retryable,
    /// Retrying the same job is not expected to succeed.
    NotRetryable,
    /// Retryability could not be determined.
    Unknown,
}

/// Stable namespaced parse failure code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct ErrorCode(
    #[schemars(pattern(
        r"^(io|json|schema|unsupported|internal|checksum|output)\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$"
    ))]
    String,
);

impl ErrorCode {
    /// Creates an error code after validating its namespace and character set.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCodeError`] when the value is not a known parser error-code family with
    /// at least two non-empty lowercase ASCII segments.
    pub fn new(value: impl Into<String>) -> Result<Self, ErrorCodeError> {
        let value = value.into();
        if !is_valid_error_code(&value) {
            return Err(ErrorCodeError);
        }

        Ok(Self(value))
    }

    /// Returns the validated string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_error_code(value: &str) -> bool {
    const FAMILIES: [&str; 7] =
        ["io", "json", "schema", "unsupported", "internal", "checksum", "output"];

    let segments = value.split('.').collect::<Vec<_>>();
    if segments.len() < 2 || segments.iter().any(|segment| segment.is_empty()) {
        return false;
    }

    FAMILIES.contains(&segments[0])
        && segments.iter().all(|segment| {
            segment.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
            })
        })
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

/// Error returned when an error code is not a valid parser error namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "error code must have a known family and contain at least two non-empty lowercase ASCII segments separated by dots"
)]
pub struct ErrorCodeError;

/// Machine-readable parse failure payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ParseFailure {
    /// Parse job identifier, when supplied by the adapter.
    pub job_id: FieldPresence<String>,
    /// Replay identifier, when supplied by the caller or source.
    pub replay_id: FieldPresence<String>,
    /// Source file or object path.
    pub source_file: FieldPresence<String>,
    /// Source checksum state.
    pub checksum: FieldPresence<SourceChecksum>,
    /// Parser stage where the failure occurred.
    pub stage: ParseStage,
    /// Stable failure code.
    pub error_code: ErrorCode,
    /// Human-readable failure message.
    pub message: String,
    /// Retryability classification.
    pub retryability: Retryability,
    /// Original lower-level cause, when safe to expose.
    pub source_cause: FieldPresence<String>,
    /// Source references associated with the failure.
    pub source_refs: SourceRefs,
}
