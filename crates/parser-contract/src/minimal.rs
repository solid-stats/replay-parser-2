use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::identity::EntitySide;

/// Minimal observed player row emitted by the default v3 artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinimalPlayerRow {
    /// Stable parser-local player row identifier.
    pub player_id: String,
    /// Source entity identifier from OCAP data.
    pub source_entity_id: i64,
    /// Observed participant nickname or display name.
    pub observed_name: Option<String>,
    /// Observed side alignment.
    pub side: Option<EntitySide>,
    /// Observed group or squad label where available.
    pub group: Option<String>,
    /// Observed role or description where available.
    pub role: Option<String>,
    /// Observed `SteamID`, when present.
    pub steam_id: Option<String>,
    /// Legacy compatibility key used by downstream comparison and import logic.
    pub compatibility_key: String,
}

/// Minimal replay-local player counter row emitted by the default v3 artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinimalPlayerStatsRow {
    /// Stable parser-local player row identifier.
    pub player_id: String,
    /// Source entity identifier from OCAP data.
    pub source_entity_id: i64,
    /// Enemy-kill counter.
    pub kills: u64,
    /// Death counter.
    pub deaths: u64,
    /// Teamkill counter.
    pub teamkills: u64,
    /// Suicide counter.
    pub suicides: u64,
    /// Deaths with an explicit null killer.
    pub null_killer_deaths: u64,
    /// Deaths where player identity or actor evidence is incomplete.
    pub unknown_deaths: u64,
    /// Vehicle/static destruction counter.
    #[serde(rename = "vehicleKills")]
    pub vehicle_kills: u64,
    /// Enemy-kill counter attributed to an attacker vehicle.
    #[serde(rename = "killsFromVehicle")]
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
    /// Stable parser-local killer player identifier, when known.
    pub killer_player_id: Option<String>,
    /// Killer source entity identifier, when known.
    pub killer_source_entity_id: Option<i64>,
    /// Killer observed name, when known.
    pub killer_name: Option<String>,
    /// Killer side, when known.
    pub killer_side: Option<EntitySide>,
    /// Stable parser-local victim player identifier, when known.
    pub victim_player_id: Option<String>,
    /// Victim source entity identifier, when known.
    pub victim_source_entity_id: Option<i64>,
    /// Victim observed name, when known.
    pub victim_name: Option<String>,
    /// Victim side, when known.
    pub victim_side: Option<EntitySide>,
    /// Replay-local death classification.
    pub classification: KillClassification,
    /// Raw weapon string, when available.
    pub weapon: Option<String>,
    /// Attacker vehicle source entity identifier, when matched.
    pub attacker_vehicle_entity_id: Option<i64>,
    /// Raw attacker vehicle observed name.
    pub attacker_vehicle_name: Option<String>,
    /// Raw attacker vehicle observed class.
    pub attacker_vehicle_class: Option<String>,
    /// Whether this row can be consumed as a bounty-awarding input.
    pub bounty_eligible: bool,
    /// Stable exclusion reasons when not bounty eligible.
    pub bounty_exclusion_reasons: Vec<String>,
}

/// Minimal vehicle/static destruction row emitted by the default v3 artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MinimalDestroyedVehicleRow {
    /// Stable parser-local attacker player identifier, when known.
    pub attacker_player_id: Option<String>,
    /// Attacker source entity identifier, when known.
    pub attacker_source_entity_id: Option<i64>,
    /// Attacker observed name, when known.
    pub attacker_name: Option<String>,
    /// Attacker side, when known.
    pub attacker_side: Option<EntitySide>,
    /// Replay-local destruction classification.
    pub classification: DestroyedVehicleClassification,
    /// Raw weapon string, when available.
    pub weapon: Option<String>,
    /// Attacker vehicle source entity identifier, when matched.
    pub attacker_vehicle_entity_id: Option<i64>,
    /// Raw attacker vehicle observed name.
    pub attacker_vehicle_name: Option<String>,
    /// Raw attacker vehicle observed class.
    pub attacker_vehicle_class: Option<String>,
    /// Destroyed source entity identifier, when known.
    pub destroyed_entity_id: Option<i64>,
    /// Destroyed entity kind label, when known.
    pub destroyed_entity_type: Option<String>,
    /// Destroyed observed name, when known.
    pub destroyed_name: Option<String>,
    /// Destroyed observed class, when known.
    pub destroyed_class: Option<String>,
    /// Destroyed side, when known.
    pub destroyed_side: Option<EntitySide>,
}
