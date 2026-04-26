use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySource {
    pub replay_id: Option<String>,
    pub source_file: String,
    pub checksum: SourceChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceChecksum {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceRef {
    pub replay_id: Option<String>,
    pub source_file: Option<String>,
    pub checksum: Option<String>,
    pub frame: Option<u64>,
    pub event_index: Option<u64>,
    pub entity_id: Option<i64>,
    pub json_path: Option<String>,
    pub rule_id: Option<RuleId>,
}

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
