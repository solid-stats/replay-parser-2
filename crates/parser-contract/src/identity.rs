use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{presence::FieldPresence, source_ref::SourceRef};

/// Observed replay entity category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    /// Infantry or player-like unit.
    Unit,
    /// Vehicle entity.
    Vehicle,
    /// Static weapon entity.
    StaticWeapon,
    /// Entity kind could not be classified.
    Unknown,
}

/// Observed side or faction alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntitySide {
    /// East side.
    East,
    /// West side.
    West,
    /// Guerilla or independent side.
    Guer,
    /// Civilian side.
    Civ,
    /// Side could not be determined.
    Unknown,
}

/// Observed replay entity with identity facts and source references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedEntity {
    /// Source entity identifier from OCAP data.
    pub source_entity_id: i64,
    /// Entity kind.
    pub kind: EntityKind,
    /// Observed identity fields for the entity.
    pub identity: ObservedIdentity,
    /// Source references backing this entity.
    pub source_refs: Vec<SourceRef>,
}

/// Observed identity fields preserved without canonical player matching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedIdentity {
    /// Observed nickname.
    pub nickname: FieldPresence<String>,
    /// Observed `SteamID`, when present.
    pub steam_id: FieldPresence<String>,
    /// Observed side.
    pub side: FieldPresence<EntitySide>,
    /// Observed faction.
    pub faction: FieldPresence<String>,
    /// Observed group.
    pub group: FieldPresence<String>,
    /// Observed squad.
    pub squad: FieldPresence<String>,
    /// Observed role.
    pub role: FieldPresence<String>,
    /// Observed description field.
    pub description: FieldPresence<String>,
}
