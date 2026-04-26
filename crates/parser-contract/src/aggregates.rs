use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::source_ref::{RuleId, SourceRefs};

/// Kind of aggregate contribution represented in the artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AggregateContributionKind {
    /// Contribution to a legacy-compatible counter projection.
    LegacyCounter,
    /// Contribution that can be consumed as a bounty calculation input.
    BountyInput,
    /// Contribution that can be consumed as a vehicle score input.
    VehicleScoreInput,
    /// Contribution to killed/killer or teamkilled/teamkiller relationship summaries.
    Relationship,
    /// Contribution kind could not be classified while preserving source evidence.
    Unknown,
}

/// Auditable link between a derived aggregate value and its normalized source evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AggregateContributionRef {
    /// Stable contribution identifier within the artifact.
    pub contribution_id: String,
    /// Contribution category.
    pub kind: AggregateContributionKind,
    /// Normalized event identifier when this contribution derives from a single event.
    pub event_id: Option<String>,
    /// Non-empty set of source references backing the contribution.
    pub source_refs: SourceRefs,
    /// Rule that produced this contribution.
    pub rule_id: RuleId,
    /// Contribution payload, shaped by the rule and projection.
    pub value: Value,
}

/// Aggregate projection section of a successful or partial parse artifact.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AggregateSection {
    /// Auditable contribution list used to derive projections.
    pub contributions: Vec<AggregateContributionRef>,
    /// Stable key-value projections for legacy and parser-owned aggregate surfaces.
    pub projections: BTreeMap<String, Value>,
}
