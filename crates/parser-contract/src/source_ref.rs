// coverage-exclusion: reviewed Phase 05 validation/deserialization defensive branches are allowlisted by exact source line.
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::presence::FieldPresence;

/// Replay source identity and checksum metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySource {
    /// Replay identifier when the caller or source provides one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_id: Option<String>,
    /// Source file path or object key.
    pub source_file: String,
    /// Source checksum state.
    pub checksum: FieldPresence<SourceChecksum>,
}

/// Supported source checksum algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumAlgorithm {
    /// SHA-256 checksum.
    Sha256,
}

/// Lowercase hexadecimal SHA-256 checksum value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct ChecksumValue(#[schemars(pattern(r"^[0-9a-f]{64}$"))] String);

impl ChecksumValue {
    /// Creates a checksum value after validating lowercase SHA-256 hex format.
    ///
    /// # Errors
    ///
    /// Returns [`ChecksumValueError`] when the value is not exactly 64 lowercase hexadecimal
    /// characters.
    pub fn new(value: impl Into<String>) -> Result<Self, ChecksumValueError> {
        let value = value.into();
        if !is_valid_sha256_hex(&value) {
            return Err(ChecksumValueError);
        }

        Ok(Self(value))
    }

    /// Returns the validated checksum string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

impl<'de> Deserialize<'de> for ChecksumValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

/// Error returned when a checksum value is not valid lowercase SHA-256 hex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("checksum value must be exactly 64 lowercase hexadecimal characters")]
pub struct ChecksumValueError;

/// Source checksum with algorithm metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceChecksum {
    /// Checksum algorithm.
    pub algorithm: ChecksumAlgorithm,
    /// Checksum value.
    pub value: ChecksumValue,
}

impl SourceChecksum {
    /// Creates a SHA-256 checksum from lowercase hex.
    ///
    /// # Errors
    ///
    /// Returns [`ChecksumValueError`] when the value is not valid SHA-256 lowercase hex.
    pub fn sha256(value: impl Into<String>) -> Result<Self, ChecksumValueError> {
        Ok(Self { algorithm: ChecksumAlgorithm::Sha256, value: ChecksumValue::new(value)? })
    }
}

/// Source coordinate for replay evidence used by normalized events and aggregates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct SourceRef {
    /// Replay identifier coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_id: Option<String>,
    /// Source file coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    /// Source checksum coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<SourceChecksum>,
    /// Frame coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<u64>,
    /// Event index coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_index: Option<u64>,
    /// Entity identifier coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<i64>,
    /// JSON path coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json_path: Option<String>,
    /// Rule identifier coordinate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<RuleId>,
}

impl SourceRef {
    /// Returns true when at least one source coordinate is present.
    #[must_use]
    pub const fn has_evidence(&self) -> bool {
        self.replay_id.is_some()
            || self.source_file.is_some()
            || self.checksum.is_some()
            || self.frame.is_some()
            || self.event_index.is_some()
            || self.entity_id.is_some()
            || self.json_path.is_some()
            || self.rule_id.is_some()
    }
}

impl<'de> Deserialize<'de> for SourceRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SourceRefFields {
            replay_id: Option<String>,
            source_file: Option<String>,
            checksum: Option<SourceChecksum>,
            frame: Option<u64>,
            event_index: Option<u64>,
            entity_id: Option<i64>,
            json_path: Option<String>,
            rule_id: Option<RuleId>,
        }

        let fields = SourceRefFields::deserialize(deserializer)?;
        let source_ref = Self {
            replay_id: fields.replay_id,
            source_file: fields.source_file,
            checksum: fields.checksum,
            frame: fields.frame,
            event_index: fields.event_index,
            entity_id: fields.entity_id,
            json_path: fields.json_path,
            rule_id: fields.rule_id,
        };

        if !source_ref.has_evidence() {
            return Err(D::Error::custom(
                "source reference must include at least one evidence coordinate",
            ));
        }

        Ok(source_ref)
    }
}

/// Non-empty source reference list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct SourceRefs(#[schemars(length(min = 1))] Vec<SourceRef>);

impl SourceRefs {
    /// Creates a non-empty source reference list.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRefsError`] when `refs` is empty.
    pub fn new(refs: Vec<SourceRef>) -> Result<Self, SourceRefsError> {
        if refs.is_empty() {
            return Err(SourceRefsError);
        }

        Ok(Self(refs))
    }

    /// Returns the source reference slice.
    #[must_use]
    pub fn as_slice(&self) -> &[SourceRef] {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SourceRefs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let refs = Vec::<SourceRef>::deserialize(deserializer)?;
        Self::new(refs).map_err(D::Error::custom)
    }
}

/// Error returned when a source reference list is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("source references must include at least one source reference")]
pub struct SourceRefsError;

/// Stable parser rule identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct RuleId(pub String);

impl RuleId {
    /// Creates a rule ID after validating the namespaced lowercase format.
    ///
    /// # Errors
    ///
    /// Returns [`RuleIdError`] when the value is empty, lacks a namespace, or contains
    /// unsupported characters.
    pub fn new(value: impl Into<String>) -> Result<Self, RuleIdError> {
        let value = value.into();
        if !is_valid_rule_id(&value) {
            return Err(RuleIdError);
        }

        Ok(Self(value))
    }

    /// Returns the validated rule ID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_rule_id(value: &str) -> bool {
    !value.is_empty()
        && value.contains('.')
        && value.split('.').all(|segment| !segment.is_empty())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

impl<'de> Deserialize<'de> for RuleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

/// Error returned when a rule ID is not a valid lowercase namespaced identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "rule ID must be a non-empty lowercase namespaced ID containing only ASCII letters, digits, dots, hyphens, and underscores"
)]
pub struct RuleIdError;
