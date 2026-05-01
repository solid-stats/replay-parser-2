use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    aggregates::AggregateContributionRef,
    events::{
        BountyEligibility, CombatSemantic, CombatVictimKind, EventActorRef, LegacyCounterEffect,
        VehicleContext,
    },
    identity::EntitySide,
    presence::FieldPresence,
    source_ref::{RuleId, SourceRefs},
};

/// Compact observed participant reference used by the default server-facing artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedParticipantRef {
    /// Source entity identifier from OCAP data.
    pub source_entity_id: i64,
    /// Observed participant nickname or display name.
    pub observed_name: FieldPresence<String>,
    /// Observed side alignment.
    pub side: FieldPresence<EntitySide>,
    /// Observed group or squad label where available.
    pub group: FieldPresence<String>,
    /// Observed role or description where available.
    pub role: FieldPresence<String>,
    /// Observed `SteamID`, when present.
    pub steam_id: FieldPresence<String>,
    /// Source references backing this compact participant reference.
    pub source_refs: SourceRefs,
}

/// Compact source-backed combat fact emitted by the default artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CombatFact {
    /// Stable fact identifier within the artifact.
    pub fact_id: String,
    /// Dominant combat semantic.
    pub semantic: CombatSemantic,
    /// Event frame when available.
    pub frame: FieldPresence<u64>,
    /// Source event index when available.
    pub event_index: FieldPresence<u64>,
    /// Killer actor when available.
    pub killer: FieldPresence<EventActorRef>,
    /// Victim actor when available.
    pub victim: FieldPresence<EventActorRef>,
    /// Victim category.
    pub victim_kind: CombatVictimKind,
    /// Vehicle evidence and taxonomy context.
    pub vehicle_context: VehicleContext,
    /// Bounty eligibility and exclusion reasons.
    pub bounty: BountyEligibility,
    /// Legacy counter deltas derived from this fact.
    pub legacy_counter_effects: Vec<LegacyCounterEffect>,
    /// Non-empty source references backing this fact.
    pub source_refs: SourceRefs,
    /// Rule that produced this fact.
    pub rule_id: RuleId,
}

/// Compact facts and contribution references needed for server ingestion and audit.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ParseFactSection {
    /// Compact combat facts.
    pub combat: Vec<CombatFact>,
    /// Auditable aggregate contribution references derived from compact facts.
    pub aggregate_contributions: Vec<AggregateContributionRef>,
}

/// Compact summary projections for human review and ingestion sanity checks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParseSummarySection {
    /// Stable key-value projections for legacy, bounty, and side surfaces.
    pub projections: BTreeMap<String, Value>,
}
