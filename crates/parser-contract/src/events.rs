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
    /// Rule-specific event attributes.
    pub attributes: BTreeMap<String, Value>,
}
