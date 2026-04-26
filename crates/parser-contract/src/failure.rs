use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::{
    presence::FieldPresence,
    source_ref::{SourceChecksum, SourceRefs},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParseStage {
    Input,
    Checksum,
    JsonDecode,
    Schema,
    Normalize,
    Aggregate,
    Output,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Retryability {
    Retryable,
    NotRetryable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct ErrorCode(
    #[schemars(pattern(
        r"^(io|json|schema|unsupported|internal|checksum|output)\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$"
    ))]
    String,
);

impl ErrorCode {
    pub fn new(value: impl Into<String>) -> Result<Self, ErrorCodeError> {
        let value = value.into();
        if !is_valid_error_code(&value) {
            return Err(ErrorCodeError);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_error_code(value: &str) -> bool {
    const FAMILIES: [&str; 7] = [
        "io",
        "json",
        "schema",
        "unsupported",
        "internal",
        "checksum",
        "output",
    ];

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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "error code must have a known family and contain at least two non-empty lowercase ASCII segments separated by dots"
)]
pub struct ErrorCodeError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ParseFailure {
    pub job_id: FieldPresence<String>,
    pub replay_id: FieldPresence<String>,
    pub source_file: FieldPresence<String>,
    pub checksum: FieldPresence<SourceChecksum>,
    pub stage: ParseStage,
    pub error_code: ErrorCode,
    pub message: String,
    pub retryability: Retryability,
    pub source_cause: FieldPresence<String>,
    pub source_refs: SourceRefs,
}
