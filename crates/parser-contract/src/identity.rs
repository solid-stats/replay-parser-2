use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{presence::FieldPresence, source_ref::SourceRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Unit,
    Vehicle,
    StaticWeapon,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntitySide {
    East,
    West,
    Guer,
    Civ,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedEntity {
    pub source_entity_id: i64,
    pub kind: EntityKind,
    pub identity: ObservedIdentity,
    pub source_refs: Vec<SourceRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedIdentity {
    pub nickname: FieldPresence<String>,
    pub steam_id: FieldPresence<String>,
    pub side: FieldPresence<EntitySide>,
    pub faction: FieldPresence<String>,
    pub group: FieldPresence<String>,
    pub squad: FieldPresence<String>,
    pub role: FieldPresence<String>,
    pub description: FieldPresence<String>,
}
