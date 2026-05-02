use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::identity::EntitySide;

/// Minimal observed player row emitted by the default v3 artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinimalPlayerRow {
    /// Source entity identifier from OCAP data.
    #[serde(rename = "eid")]
    pub source_entity_id: i64,
    /// Observed participant nickname or display name.
    #[serde(default, rename = "n", skip_serializing_if = "Option::is_none")]
    pub observed_name: Option<String>,
    /// Observed side alignment.
    #[serde(default, rename = "s", skip_serializing_if = "Option::is_none")]
    pub side: Option<EntitySide>,
    /// Observed group or squad label where available.
    #[serde(default, rename = "g", skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Observed role or description where available.
    #[serde(default, rename = "r", skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Observed `SteamID`, when present.
    #[serde(default, rename = "sid", skip_serializing_if = "Option::is_none")]
    pub steam_id: Option<String>,
    /// Legacy compatibility key when it is not derivable from `eid`.
    #[serde(default, rename = "ck", skip_serializing_if = "Option::is_none")]
    pub compatibility_key: Option<String>,
    /// Enemy-kill counter.
    #[serde(default, rename = "k", skip_serializing_if = "is_zero_u64")]
    pub kills: u64,
    /// Death counter.
    #[serde(default, rename = "d", skip_serializing_if = "is_zero_u64")]
    pub deaths: u64,
    /// Teamkill counter.
    #[serde(default, rename = "tk", skip_serializing_if = "is_zero_u64")]
    pub teamkills: u64,
    /// Suicide counter.
    #[serde(default, rename = "su", skip_serializing_if = "is_zero_u64")]
    pub suicides: u64,
    /// Deaths with an explicit null killer.
    #[serde(default, rename = "nkd", skip_serializing_if = "is_zero_u64")]
    pub null_killer_deaths: u64,
    /// Deaths where player identity or actor evidence is incomplete.
    #[serde(default, rename = "ud", skip_serializing_if = "is_zero_u64")]
    pub unknown_deaths: u64,
    /// Vehicle/static destruction counter.
    #[serde(default, rename = "vk", skip_serializing_if = "is_zero_u64")]
    pub vehicle_kills: u64,
    /// Enemy-kill counter attributed to an attacker vehicle.
    #[serde(default, rename = "kfv", skip_serializing_if = "is_zero_u64")]
    pub kills_from_vehicle: u64,
}

/// Minimal player death classification emitted by `kills[]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KillClassification {
    /// Enemy player kill.
    EnemyKill,
    /// Same-side non-suicide kill.
    Teamkill,
    /// Actor killed itself.
    Suicide,
    /// Victim death with an explicit null killer.
    NullKiller,
    /// Player death with incomplete actor evidence.
    Unknown,
}

/// Minimal vehicle/static destruction classification emitted by `destroyed_vehicles[]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DestroyedVehicleClassification {
    /// Enemy-side vehicle or static weapon destruction.
    Enemy,
    /// Friendly-side vehicle or static weapon destruction.
    Friendly,
    /// Destroyed side could not be determined.
    UnknownSide,
}

/// Minimal player death row emitted by the default v3 artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinimalKillRow {
    /// Killer source entity identifier, when known.
    #[serde(default, rename = "k", skip_serializing_if = "Option::is_none")]
    pub killer_source_entity_id: Option<i64>,
    /// Victim source entity identifier, when known.
    #[serde(default, rename = "v", skip_serializing_if = "Option::is_none")]
    pub victim_source_entity_id: Option<i64>,
    /// Replay-local death classification.
    #[serde(rename = "c")]
    pub classification: KillClassification,
    /// Compact weapon dictionary identifier, when known.
    #[serde(default, rename = "w", skip_serializing_if = "Option::is_none")]
    pub weapon_id: Option<u32>,
    /// Attacker vehicle source entity identifier, when matched.
    #[serde(default, rename = "av", skip_serializing_if = "Option::is_none")]
    pub attacker_vehicle_entity_id: Option<i64>,
    /// Raw attacker vehicle observed class.
    #[serde(default, rename = "avc", skip_serializing_if = "Option::is_none")]
    pub attacker_vehicle_class: Option<String>,
}

/// Minimal vehicle/static destruction row emitted by the default v3 artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinimalDestroyedVehicleRow {
    /// Attacker source entity identifier, when known.
    #[serde(default, rename = "a", skip_serializing_if = "Option::is_none")]
    pub attacker_source_entity_id: Option<i64>,
    /// Replay-local destruction classification.
    #[serde(rename = "c")]
    pub classification: DestroyedVehicleClassification,
    /// Compact weapon dictionary identifier, when known.
    #[serde(default, rename = "w", skip_serializing_if = "Option::is_none")]
    pub weapon_id: Option<u32>,
    /// Attacker vehicle source entity identifier, when matched.
    #[serde(default, rename = "av", skip_serializing_if = "Option::is_none")]
    pub attacker_vehicle_entity_id: Option<i64>,
    /// Raw attacker vehicle observed class.
    #[serde(default, rename = "avc", skip_serializing_if = "Option::is_none")]
    pub attacker_vehicle_class: Option<String>,
    /// Destroyed source entity identifier, when known.
    #[serde(default, rename = "de", skip_serializing_if = "Option::is_none")]
    pub destroyed_entity_id: Option<i64>,
    /// Destroyed entity kind label, when known.
    #[serde(default, rename = "dt", skip_serializing_if = "Option::is_none")]
    pub destroyed_entity_type: Option<String>,
    /// Destroyed observed class, when known.
    #[serde(default, rename = "dc", skip_serializing_if = "Option::is_none")]
    pub destroyed_class: Option<String>,
}

/// Minimal deterministic weapon dictionary row emitted by the default v3 artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinimalWeaponRow {
    /// Compact replay-local weapon identifier.
    pub id: u32,
    /// Raw weapon class/name referenced by compact event rows.
    #[serde(rename = "n")]
    pub name: String,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if predicates receive borrowed field values"
)]
const fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}
