use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::source_ref::{RuleId, SourceRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AggregateContributionKind {
    LegacyCounter,
    BountyInput,
    VehicleScoreInput,
    Relationship,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AggregateContributionRef {
    pub contribution_id: String,
    pub kind: AggregateContributionKind,
    pub event_id: Option<String>,
    pub source_refs: Vec<SourceRef>,
    pub rule_id: RuleId,
    pub value: Value,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AggregateSection {
    pub contributions: Vec<AggregateContributionRef>,
    pub projections: BTreeMap<String, Value>,
}
