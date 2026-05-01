//! Minimal artifact row derivation from normalized combat events.
// coverage-exclusion: reviewed Phase 05 defensive aggregate projection branches are allowlisted by exact source line.

use std::collections::BTreeMap;

use parser_contract::{
    events::{
        BountyEligibilityState, BountyExclusionReason, CombatEventAttributes, CombatSemantic,
        CombatVictimKind, EventActorRef, NormalizedEvent, VehicleContext,
    },
    identity::{EntityCompatibilityHintKind, EntityKind, EntitySide, ObservedEntity},
    minimal::{
        DestroyedVehicleClassification, KillClassification, MinimalDestroyedVehicleRow,
        MinimalKillRow, MinimalPlayerRow, MinimalPlayerStatsRow,
    },
    presence::FieldPresence,
};

use crate::legacy_player::is_legacy_player_entity;

/// Minimal v3 table rows emitted by the default parser artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimalTables {
    /// Observed legacy-compatible players.
    pub players: Vec<MinimalPlayerRow>,
    /// Replay-local player counters.
    pub player_stats: Vec<MinimalPlayerStatsRow>,
    /// Player death rows.
    pub kills: Vec<MinimalKillRow>,
    /// Vehicle/static weapon destruction rows.
    pub destroyed_vehicles: Vec<MinimalDestroyedVehicleRow>,
}

/// Derives minimal default artifact rows from normalized entities and combat events.
#[must_use]
pub fn derive_minimal_tables(
    entities: &[ObservedEntity],
    events: &[NormalizedEvent],
) -> MinimalTables {
    let entity_index =
        entities.iter().map(|entity| (entity.source_entity_id, entity)).collect::<BTreeMap<_, _>>();
    let players = minimal_players(entities);
    let mut stats = players
        .iter()
        .map(|player| (player.source_entity_id, PlayerStatsAccumulator::new(player)))
        .collect::<BTreeMap<_, _>>();
    let mut kills = Vec::new();
    let mut destroyed_vehicles = Vec::new();

    for event in events {
        let Some(combat) = &event.combat else {
            continue;
        };

        match combat.semantic {
            CombatSemantic::EnemyKill => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(combat, KillClassification::EnemyKill));
                    increment_killer(&mut stats, combat, |stats| stats.kills += 1);
                    increment_victim(&mut stats, combat, |stats| stats.deaths += 1);

                    if combat.vehicle_context.is_kill_from_vehicle {
                        increment_killer(&mut stats, combat, |stats| {
                            stats.kills_from_vehicle += 1;
                        });
                    }
                }
            }
            CombatSemantic::Teamkill => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(combat, KillClassification::Teamkill));
                    increment_killer(&mut stats, combat, |stats| stats.teamkills += 1);
                    increment_victim(&mut stats, combat, |stats| stats.deaths += 1);
                }
            }
            CombatSemantic::Suicide => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(combat, KillClassification::Suicide));
                    increment_victim(&mut stats, combat, |stats| {
                        stats.deaths += 1;
                        stats.suicides += 1;
                    });
                }
            }
            CombatSemantic::NullKillerDeath => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(combat, KillClassification::NullKiller));
                    increment_victim(&mut stats, combat, |stats| {
                        stats.deaths += 1;
                        stats.null_killer_deaths += 1;
                    });
                }
            }
            CombatSemantic::Unknown => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(combat, KillClassification::Unknown));
                    increment_victim(&mut stats, combat, |stats| {
                        stats.deaths += 1;
                        stats.unknown_deaths += 1;
                    });
                }
            }
            CombatSemantic::VehicleDestroyed => {
                if vehicle_or_static_victim(combat, &entity_index) {
                    destroyed_vehicles.push(minimal_destroyed_vehicle_row(combat, &entity_index));
                    increment_killer(&mut stats, combat, |stats| stats.vehicle_kills += 1);
                }
            }
        }
    }

    MinimalTables {
        players,
        player_stats: stats.into_values().map(PlayerStatsAccumulator::into_row).collect(),
        kills,
        destroyed_vehicles,
    }
}

fn minimal_players(entities: &[ObservedEntity]) -> Vec<MinimalPlayerRow> {
    entities
        .iter()
        .filter(|entity| is_legacy_player_entity(entity))
        .map(|entity| MinimalPlayerRow {
            player_id: player_id(entity.source_entity_id),
            source_entity_id: entity.source_entity_id,
            observed_name: player_name(entity),
            side: present_side(&entity.identity.side),
            group: player_group(entity),
            role: player_role(entity),
            steam_id: observed_string(&entity.identity.steam_id).map(ToOwned::to_owned),
            compatibility_key: compatibility_key(entity),
        })
        .collect()
}

fn minimal_kill_row(
    combat: &CombatEventAttributes,
    classification: KillClassification,
) -> MinimalKillRow {
    MinimalKillRow {
        killer_player_id: actor_entity_id(&combat.killer).map(player_id),
        killer_source_entity_id: actor_entity_id(&combat.killer),
        killer_name: actor_name(&combat.killer),
        killer_side: actor_side(&combat.killer),
        victim_player_id: actor_entity_id(&combat.victim).map(player_id),
        victim_source_entity_id: actor_entity_id(&combat.victim),
        victim_name: actor_name(&combat.victim),
        victim_side: actor_side(&combat.victim),
        classification,
        weapon: observed_string(&combat.weapon).map(ToOwned::to_owned),
        attacker_vehicle_entity_id: present_i64(&combat.vehicle_context.attacker_vehicle_entity_id),
        attacker_vehicle_name: observed_string(&combat.vehicle_context.attacker_vehicle_name)
            .map(ToOwned::to_owned),
        attacker_vehicle_class: observed_string(&combat.vehicle_context.attacker_vehicle_class)
            .map(ToOwned::to_owned),
        bounty_eligible: combat.bounty.state == BountyEligibilityState::Eligible,
        bounty_exclusion_reasons: combat
            .bounty
            .exclusion_reasons
            .iter()
            .map(|reason| bounty_exclusion_reason_name(*reason).to_owned())
            .collect(),
    }
}

fn minimal_destroyed_vehicle_row(
    combat: &CombatEventAttributes,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
) -> MinimalDestroyedVehicleRow {
    let destroyed_entity =
        actor_entity_id(&combat.victim).and_then(|entity_id| entity_index.get(&entity_id).copied());

    MinimalDestroyedVehicleRow {
        attacker_player_id: actor_entity_id(&combat.killer).map(player_id),
        attacker_source_entity_id: actor_entity_id(&combat.killer),
        attacker_name: actor_name(&combat.killer),
        attacker_side: actor_side(&combat.killer),
        classification: destroyed_vehicle_classification(combat),
        weapon: observed_string(&combat.weapon).map(ToOwned::to_owned),
        attacker_vehicle_entity_id: present_i64(&combat.vehicle_context.attacker_vehicle_entity_id),
        attacker_vehicle_name: observed_string(&combat.vehicle_context.attacker_vehicle_name)
            .map(ToOwned::to_owned),
        attacker_vehicle_class: observed_string(&combat.vehicle_context.attacker_vehicle_class)
            .map(ToOwned::to_owned),
        destroyed_entity_id: destroyed_entity.map(|entity| entity.source_entity_id),
        destroyed_entity_type: destroyed_entity.map(entity_kind_name).map(ToOwned::to_owned),
        destroyed_name: destroyed_entity
            .and_then(|entity| observed_string(&entity.observed_name))
            .map(ToOwned::to_owned),
        destroyed_class: destroyed_entity
            .and_then(|entity| observed_string(&entity.observed_class))
            .map(ToOwned::to_owned),
        destroyed_side: destroyed_entity.and_then(|entity| present_side(&entity.identity.side)),
    }
}

#[derive(Debug, Clone)]
struct PlayerStatsAccumulator {
    player_id: String,
    source_entity_id: i64,
    kills: u64,
    deaths: u64,
    teamkills: u64,
    suicides: u64,
    null_killer_deaths: u64,
    unknown_deaths: u64,
    vehicle_kills: u64,
    kills_from_vehicle: u64,
}

impl PlayerStatsAccumulator {
    fn new(player: &MinimalPlayerRow) -> Self {
        Self {
            player_id: player.player_id.clone(),
            source_entity_id: player.source_entity_id,
            kills: 0,
            deaths: 0,
            teamkills: 0,
            suicides: 0,
            null_killer_deaths: 0,
            unknown_deaths: 0,
            vehicle_kills: 0,
            kills_from_vehicle: 0,
        }
    }

    fn into_row(self) -> MinimalPlayerStatsRow {
        MinimalPlayerStatsRow {
            player_id: self.player_id,
            source_entity_id: self.source_entity_id,
            kills: self.kills,
            deaths: self.deaths,
            teamkills: self.teamkills,
            suicides: self.suicides,
            null_killer_deaths: self.null_killer_deaths,
            unknown_deaths: self.unknown_deaths,
            vehicle_kills: self.vehicle_kills,
            kills_from_vehicle: self.kills_from_vehicle,
        }
    }
}

fn increment_killer(
    stats: &mut BTreeMap<i64, PlayerStatsAccumulator>,
    combat: &CombatEventAttributes,
    update: impl FnOnce(&mut PlayerStatsAccumulator),
) {
    if let Some(killer) =
        actor_entity_id(&combat.killer).and_then(|entity_id| stats.get_mut(&entity_id))
    {
        update(killer);
    }
}

fn increment_victim(
    stats: &mut BTreeMap<i64, PlayerStatsAccumulator>,
    combat: &CombatEventAttributes,
    update: impl FnOnce(&mut PlayerStatsAccumulator),
) {
    if let Some(victim) =
        actor_entity_id(&combat.victim).and_then(|entity_id| stats.get_mut(&entity_id))
    {
        update(victim);
    }
}

fn victim_is_or_may_be_player(
    combat: &CombatEventAttributes,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
) -> bool {
    match combat.victim_kind {
        CombatVictimKind::Player => true,
        CombatVictimKind::Unknown => actor_entity_id(&combat.victim)
            .and_then(|entity_id| entity_index.get(&entity_id))
            .is_none_or(|entity| {
                !matches!(entity.kind, EntityKind::Vehicle | EntityKind::StaticWeapon)
            }),
        CombatVictimKind::Vehicle | CombatVictimKind::StaticWeapon => false,
    }
}

fn vehicle_or_static_victim(
    combat: &CombatEventAttributes,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
) -> bool {
    matches!(combat.victim_kind, CombatVictimKind::Vehicle | CombatVictimKind::StaticWeapon)
        || actor_entity_id(&combat.victim)
            .and_then(|entity_id| entity_index.get(&entity_id))
            .is_some_and(|entity| {
                matches!(entity.kind, EntityKind::Vehicle | EntityKind::StaticWeapon)
            })
}

fn destroyed_vehicle_classification(
    combat: &CombatEventAttributes,
) -> DestroyedVehicleClassification {
    match (actor_side(&combat.killer), actor_side(&combat.victim)) {
        (Some(killer_side), Some(victim_side)) if killer_side == victim_side => {
            DestroyedVehicleClassification::Friendly
        }
        (Some(_), Some(_)) => DestroyedVehicleClassification::Enemy,
        _ => DestroyedVehicleClassification::UnknownSide,
    }
}

fn compatibility_key(entity: &ObservedEntity) -> String {
    entity
        .compatibility_hints
        .iter()
        .find(|hint| hint.kind == EntityCompatibilityHintKind::DuplicateSlotSameName)
        .and_then(|hint| observed_string(&hint.observed_name))
        .map_or_else(|| player_id(entity.source_entity_id), |name| format!("legacy_name:{name}"))
}

fn player_id(source_entity_id: i64) -> String {
    format!("entity:{source_entity_id}")
}

fn player_name(entity: &ObservedEntity) -> Option<String> {
    observed_string(&entity.identity.nickname)
        .or_else(|| observed_string(&entity.observed_name))
        .map(ToOwned::to_owned)
}

fn player_group(entity: &ObservedEntity) -> Option<String> {
    observed_string(&entity.identity.squad)
        .or_else(|| observed_string(&entity.identity.group))
        .map(ToOwned::to_owned)
}

fn player_role(entity: &ObservedEntity) -> Option<String> {
    observed_string(&entity.identity.role)
        .or_else(|| observed_string(&entity.identity.description))
        .map(ToOwned::to_owned)
}

const fn actor(field: &FieldPresence<EventActorRef>) -> Option<&EventActorRef> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(value),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn actor_entity_id(field: &FieldPresence<EventActorRef>) -> Option<i64> {
    actor(field).and_then(|actor| present_i64(&actor.source_entity_id))
}

fn actor_name(field: &FieldPresence<EventActorRef>) -> Option<String> {
    actor(field).and_then(|actor| observed_string(&actor.observed_name)).map(ToOwned::to_owned)
}

fn actor_side(field: &FieldPresence<EventActorRef>) -> Option<EntitySide> {
    actor(field).and_then(|actor| present_side(&actor.side))
}

const fn present_i64(field: &FieldPresence<i64>) -> Option<i64> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(*value),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

const fn present_side(field: &FieldPresence<EntitySide>) -> Option<EntitySide> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(*value),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

const fn observed_string(field: &FieldPresence<String>) -> Option<&str> {
    match field {
        FieldPresence::Present { value, source: _ } | FieldPresence::Inferred { value, .. } => {
            Some(value.as_str())
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

const fn entity_kind_name(entity: &ObservedEntity) -> &'static str {
    match entity.kind {
        EntityKind::Unit => "unit",
        EntityKind::Vehicle => "vehicle",
        EntityKind::StaticWeapon => "static_weapon",
        EntityKind::Unknown => "unknown",
    }
}

const fn bounty_exclusion_reason_name(reason: BountyExclusionReason) -> &'static str {
    match reason {
        BountyExclusionReason::Teamkill => "teamkill",
        BountyExclusionReason::Suicide => "suicide",
        BountyExclusionReason::NullKiller => "null_killer",
        BountyExclusionReason::VehicleVictim => "vehicle_victim",
        BountyExclusionReason::UnknownActor => "unknown_actor",
        BountyExclusionReason::UnknownSide => "unknown_side",
        BountyExclusionReason::NotEnemyKill => "not_enemy_kill",
    }
}

#[allow(dead_code, reason = "keeps the vehicle context import exercised in docs and type checks")]
const fn _vehicle_context_type_marker(_: &VehicleContext) {}
