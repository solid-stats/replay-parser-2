use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::presence::FieldPresence;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySource {
    pub replay_id: Option<String>,
    pub source_file: String,
    pub checksum: FieldPresence<SourceChecksum>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumAlgorithm {
    Sha256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct ChecksumValue(#[schemars(pattern(r"^[0-9a-f]{64}$"))] String);

impl ChecksumValue {
    pub fn new(value: impl Into<String>) -> Result<Self, ChecksumValueError> {
        let value = value.into();
        if !is_valid_sha256_hex(&value) {
            return Err(ChecksumValueError);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("checksum value must be exactly 64 lowercase hexadecimal characters")]
pub struct ChecksumValueError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceChecksum {
    pub algorithm: ChecksumAlgorithm,
    pub value: ChecksumValue,
}

impl SourceChecksum {
    pub fn sha256(value: impl Into<String>) -> Result<Self, ChecksumValueError> {
        Ok(Self {
            algorithm: ChecksumAlgorithm::Sha256,
            value: ChecksumValue::new(value)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct SourceRef {
    pub replay_id: Option<String>,
    pub source_file: Option<String>,
    pub checksum: Option<SourceChecksum>,
    pub frame: Option<u64>,
    pub event_index: Option<u64>,
    pub entity_id: Option<i64>,
    pub json_path: Option<String>,
    pub rule_id: Option<RuleId>,
}

impl SourceRef {
    pub fn has_evidence(&self) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct SourceRefs(#[schemars(length(min = 1))] Vec<SourceRef>);

impl SourceRefs {
    pub fn new(refs: Vec<SourceRef>) -> Result<Self, SourceRefsError> {
        if refs.is_empty() {
            return Err(SourceRefsError);
        }

        Ok(Self(refs))
    }

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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("source references must include at least one source reference")]
pub struct SourceRefsError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct RuleId(pub String);

impl RuleId {
    pub fn new(value: impl Into<String>) -> Result<Self, RuleIdError> {
        let value = value.into();
        if !is_valid_rule_id(&value) {
            return Err(RuleIdError);
        }

        Ok(Self(value))
    }

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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "rule ID must be a non-empty lowercase namespaced ID containing only ASCII letters, digits, dots, hyphens, and underscores"
)]
pub struct RuleIdError;
