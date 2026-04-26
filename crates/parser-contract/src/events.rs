use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    identity::EntitySide,
    presence::FieldPresence,
    source_ref::{RuleId, SourceRef},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedEventKind {
    Connected,
    Disconnected,
    Kill,
    Death,
    Teamkill,
    Suicide,
    PlayerKilled,
    VehicleKilled,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EventActorRef {
    pub source_entity_id: FieldPresence<i64>,
    pub observed_name: FieldPresence<String>,
    pub side: FieldPresence<EntitySide>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NormalizedEvent {
    pub event_id: String,
    pub kind: NormalizedEventKind,
    pub frame: FieldPresence<u64>,
    pub event_index: FieldPresence<u64>,
    pub actors: Vec<EventActorRef>,
    pub source_refs: Vec<SourceRef>,
    pub rule_id: RuleId,
    pub attributes: BTreeMap<String, Value>,
}
