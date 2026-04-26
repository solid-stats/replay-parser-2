use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::source_ref::{RuleId, SourceRef};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FieldPresence<T> {
    Present {
        value: T,
        source: Option<SourceRef>,
    },
    ExplicitNull {
        reason: NullReason,
        source: Option<SourceRef>,
    },
    Unknown {
        reason: UnknownReason,
        source: Option<SourceRef>,
    },
    Inferred {
        value: T,
        reason: String,
        confidence: Option<Confidence>,
        source: Option<SourceRef>,
        rule_id: RuleId,
    },
    NotApplicable {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NullReason {
    NullKiller,
    SourceNull,
    EmptyValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnknownReason {
    MissingSteamId,
    MissingWinner,
    AbsentCommander,
    SourceFieldAbsent,
    SchemaDrift,
    ChecksumUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct Confidence(#[schemars(range(min = 0.0, max = 1.0))] f32);

impl Confidence {
    pub fn new(value: f32) -> Result<Self, ConfidenceError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ConfidenceError);
        }

        Ok(Self(value))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Confidence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f32::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("confidence must be finite and between 0.0 and 1.0")]
pub struct ConfidenceError;
