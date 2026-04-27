use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    presence::FieldPresence,
    source_ref::{RuleId, SourceRefs},
};

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

/// Legacy compatibility hint kind attached to an observed entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityCompatibilityHintKind {
    /// Entity identity was backfilled from a connected-player event.
    ConnectedPlayerBackfill,
    /// Entity is a duplicate-slot same-name candidate for later aggregate projection.
    DuplicateSlotSameName,
}

/// Auditable legacy compatibility hint for observed entity projection behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EntityCompatibilityHint {
    /// Compatibility behavior represented by this hint.
    pub kind: EntityCompatibilityHintKind,
    /// Related source entity identifiers that participate in this hint.
    #[schemars(length(min = 1))]
    pub related_entity_ids: Vec<i64>,
    /// Observed name involved in this compatibility behavior.
    pub observed_name: FieldPresence<String>,
    /// Stable rule identifier that produced this hint.
    pub rule_id: RuleId,
    /// Source references backing this hint.
    pub source_refs: SourceRefs,
}

/// Observed replay entity with identity facts and source references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedEntity {
    /// Source entity identifier from OCAP data.
    pub source_entity_id: i64,
    /// Entity kind.
    pub kind: EntityKind,
    /// Observed entity name from replay data.
    pub observed_name: FieldPresence<String>,
    /// Observed entity class from replay data.
    pub observed_class: FieldPresence<String>,
    /// Observed player flag from replay data.
    pub is_player: FieldPresence<bool>,
    /// Observed identity fields for the entity.
    pub identity: ObservedIdentity,
    /// Legacy compatibility hints without collapsing raw observed entities.
    pub compatibility_hints: Vec<EntityCompatibilityHint>,
    /// Source references backing this entity.
    pub source_refs: SourceRefs,
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
