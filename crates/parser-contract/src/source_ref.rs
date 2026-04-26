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
        if value.trim().is_empty() {
            return Err(RuleIdError);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
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
#[error("rule ID cannot be empty")]
pub struct RuleIdError;
