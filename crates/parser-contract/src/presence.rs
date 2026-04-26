use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
        confidence: Option<f32>,
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
}
