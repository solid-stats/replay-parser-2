use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    identity::EntitySide,
    presence::FieldPresence,
    source_ref::{RuleId, SourceRefs},
};

/// Normalized replay event category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedEventKind {
    /// Player or entity connected.
    Connected,
    /// Player or entity disconnected.
    Disconnected,
    /// Kill event.
    Kill,
    /// Death event.
    Death,
    /// Teamkill event.
    Teamkill,
    /// Suicide event.
    Suicide,
    /// Player-killed event.
    PlayerKilled,
    /// Vehicle-killed event.
    VehicleKilled,
    /// Event shape was preserved but could not be classified.
    Unknown,
}

/// Dominant combat semantic represented by a source-backed combat event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CombatSemantic {
    /// Enemy player or entity kill.
    EnemyKill,
    /// Same-side non-suicide kill.
    Teamkill,
    /// Actor killed itself.
    Suicide,
    /// Victim death with an explicit null killer.
    NullKillerDeath,
    /// Vehicle or static weapon was destroyed.
    VehicleDestroyed,
    /// Combat tuple was preserved but could not be classified.
    Unknown,
}

/// Victim category used by combat classification and aggregate derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CombatVictimKind {
    /// Player or infantry victim.
    Player,
    /// Vehicle victim.
    Vehicle,
    /// Static weapon victim.
    StaticWeapon,
    /// Victim kind could not be determined.
    Unknown,
}

/// Bounty eligibility state for combat events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BountyEligibilityState {
    /// Event can be consumed as a bounty-awarding input.
    Eligible,
    /// Event is auditable but excluded from bounty-awarding inputs.
    Excluded,
}

/// Reason a combat event is excluded from bounty-awarding inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BountyExclusionReason {
    /// Same-side teamkill.
    Teamkill,
    /// Suicide event.
    Suicide,
    /// Explicit null killer.
    NullKiller,
    /// Victim is a vehicle or static weapon.
    VehicleVictim,
    /// Actor lookup is incomplete.
    UnknownActor,
    /// Side evidence is incomplete.
    UnknownSide,
    /// Event is not an enemy kill.
    NotEnemyKill,
}

/// Vehicle-related evidence attached to a combat event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VehicleContext {
    /// True when the kill was attributed to a vehicle weapon/name.
    pub is_kill_from_vehicle: bool,
    /// Raw weapon string from the source tuple.
    pub raw_weapon: FieldPresence<String>,
    /// Attacker vehicle source entity identifier when matched.
    pub attacker_vehicle_entity_id: FieldPresence<i64>,
    /// Raw attacker vehicle observed name.
    pub attacker_vehicle_name: FieldPresence<String>,
    /// Raw attacker vehicle observed class.
    pub attacker_vehicle_class: FieldPresence<String>,
}

/// Bounty eligibility payload attached to a combat event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BountyEligibility {
    /// Eligibility state.
    pub state: BountyEligibilityState,
    /// Stable exclusion reasons when excluded.
    pub exclusion_reasons: Vec<BountyExclusionReason>,
}

/// Legacy counter delta produced by a combat event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LegacyCounterEffect {
    /// Player source entity identifier affected by the delta.
    pub player_entity_id: i64,
    /// Legacy field name affected by the delta.
    pub field: String,
    /// Counter delta.
    pub delta: i64,
    /// Relationship target source entity identifier when applicable.
    pub relationship_target_entity_id: Option<i64>,
}

/// Typed combat payload for normalized combat events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CombatEventAttributes {
    /// Dominant combat semantic.
    pub semantic: CombatSemantic,
    /// Killer actor when available.
    pub killer: FieldPresence<EventActorRef>,
    /// Victim actor when available.
    pub victim: FieldPresence<EventActorRef>,
    /// Victim category.
    pub victim_kind: CombatVictimKind,
    /// Weapon string when available.
    pub weapon: FieldPresence<String>,
    /// Distance in meters when available.
    pub distance_meters: FieldPresence<f64>,
    /// Vehicle evidence and taxonomy context.
    pub vehicle_context: VehicleContext,
    /// Bounty eligibility and exclusion reasons.
    pub bounty: BountyEligibility,
    /// Legacy counter deltas derived from this event.
    pub legacy_counter_effects: Vec<LegacyCounterEffect>,
}

/// Actor identity fragment referenced by a normalized event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EventActorRef {
    /// Source entity identifier for the actor.
    pub source_entity_id: FieldPresence<i64>,
    /// Observed actor name.
    pub observed_name: FieldPresence<String>,
    /// Observed actor side.
    pub side: FieldPresence<EntitySide>,
}

/// Normalized event with source references and rule provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NormalizedEvent {
    /// Stable event identifier within the artifact.
    pub event_id: String,
    /// Normalized event kind.
    pub kind: NormalizedEventKind,
    /// Event frame when available.
    pub frame: FieldPresence<u64>,
    /// Source event index when available.
    pub event_index: FieldPresence<u64>,
    /// Actors associated with this event.
    pub actors: Vec<EventActorRef>,
    /// Non-empty source references backing this event.
    pub source_refs: SourceRefs,
    /// Rule that normalized this event.
    pub rule_id: RuleId,
    /// Typed combat payload for combat events.
    pub combat: Option<CombatEventAttributes>,
    /// Rule-specific event attributes.
    pub attributes: BTreeMap<String, Value>,
}
