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

/// Typed value for a legacy-compatible counter contribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LegacyCounterContributionValue {
    /// Projection key this contribution feeds.
    pub projection_key: String,
    /// Source entity identifier for the affected player.
    pub player_entity_id: i64,
    /// Legacy compatibility key used by old aggregate comparison.
    pub compatibility_key: String,
    /// Legacy field name affected by the delta.
    pub field: String,
    /// Counter delta.
    pub delta: i64,
    /// Source normalized event identifier.
    pub event_id: String,
}

/// Typed value for a killed/killer or teamkilled/teamkiller relationship contribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipContributionValue {
    /// Projection key this contribution feeds.
    pub projection_key: String,
    /// Source player entity identifier.
    pub source_player_entity_id: i64,
    /// Target player entity identifier.
    pub target_player_entity_id: i64,
    /// Relationship name.
    pub relationship: String,
    /// Legacy compatibility key for the source player.
    pub compatibility_source_key: String,
    /// Legacy compatibility key for the target player.
    pub compatibility_target_key: String,
    /// Relationship count delta.
    pub count_delta: i64,
}

/// Typed value for bounty input aggregate contributions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BountyInputContributionValue {
    /// Killer source entity identifier.
    pub killer_entity_id: i64,
    /// Victim source entity identifier.
    pub victim_entity_id: i64,
    /// Killer observed side.
    pub killer_side: String,
    /// Victim observed side.
    pub victim_side: String,
    /// Source frame when available.
    pub frame: Option<u64>,
    /// Source normalized event identifier.
    pub event_id: String,
    /// Whether this input awards bounty.
    pub eligible: bool,
    /// Exclusion reasons when not eligible.
    pub exclusion_reasons: Vec<String>,
}

/// Vehicle score contribution sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VehicleScoreSign {
    /// Add weighted score.
    Award,
    /// Subtract weighted penalty.
    Penalty,
}

/// Typed value for issue #13 vehicle score input aggregate contributions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VehicleScoreInputValue {
    /// Player source entity identifier receiving the score input.
    pub player_entity_id: i64,
    /// Source normalized event identifier.
    pub event_id: String,
    /// Contribution sign.
    pub sign: VehicleScoreSign,
    /// Issue #13 attacker category.
    pub attacker_category: crate::events::VehicleScoreCategory,
    /// Issue #13 target category.
    pub target_category: crate::events::VehicleScoreCategory,
    /// Raw attacker vehicle name evidence.
    pub raw_attacker_vehicle_name: Option<String>,
    /// Raw attacker vehicle class evidence.
    pub raw_attacker_vehicle_class: Option<String>,
    /// Raw target class evidence.
    pub raw_target_class: Option<String>,
    /// Raw matrix weight from the issue #13 table.
    pub matrix_weight: f64,
    /// Applied weight after sign-specific rules such as teamkill clamp.
    pub applied_weight: f64,
    /// True when a teamkill penalty was clamped up to at least 1.
    pub teamkill_penalty_clamped: bool,
    /// True when this contribution makes the replay count in the denominator.
    pub denominator_eligible: bool,
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
