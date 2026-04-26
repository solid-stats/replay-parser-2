use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::source_ref::{SourceChecksum, SourceRef};

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
pub struct ErrorCode(String);

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
    const FAMILIES: [&str; 5] = ["io.", "json.", "schema.", "unsupported.", "internal."];

    let Some(prefix) = FAMILIES.iter().find(|prefix| value.starts_with(*prefix)) else {
        return false;
    };
    let suffix = &value[prefix.len()..];

    !suffix.is_empty()
        && !value.ends_with('.')
        && suffix.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
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
    "error code must start with io., json., schema., unsupported., or internal. and contain only lowercase ASCII letters, digits, dots, hyphens, and underscores after the family prefix"
)]
pub struct ErrorCodeError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParseFailure {
    pub job_id: Option<String>,
    pub replay_id: Option<String>,
    pub source_file: Option<String>,
    pub checksum: Option<SourceChecksum>,
    pub stage: ParseStage,
    pub error_code: ErrorCode,
    pub message: String,
    pub retryability: Retryability,
    pub source_cause: Option<String>,
    pub source_refs: Vec<SourceRef>,
}
