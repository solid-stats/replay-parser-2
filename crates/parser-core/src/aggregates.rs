//! Minimal artifact row derivation from normalized combat events.
// coverage-exclusion: reviewed Phase 05 defensive aggregate projection branches are allowlisted by exact source line.

use std::collections::{BTreeMap, BTreeSet};

use parser_contract::{
    events::{
        CombatEventAttributes, CombatSemantic, CombatVictimKind, EventActorRef, NormalizedEvent,
        VehicleContext,
    },
    identity::{EntityCompatibilityHintKind, EntityKind, EntitySide, ObservedEntity},
    minimal::{
        DestroyedVehicleClassification, KillClassification, MinimalDestroyedVehicleRow,
        MinimalKillRow, MinimalPlayerRow, MinimalWeaponRow,
    },
    presence::FieldPresence,
};

use crate::legacy_player::is_legacy_player_entity;

/// Minimal v3 table rows emitted by the default parser artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimalTables {
    /// Observed legacy-compatible players.
    pub players: Vec<MinimalPlayerRow>,
    /// Deterministic weapon dictionary.
    pub weapons: Vec<MinimalWeaponRow>,
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
    let mut players = minimal_players(entities);
    let weapon_ids = weapon_dictionary(events, &entity_index);
    let mut kills = Vec::new();
    let mut destroyed_vehicles = Vec::new();

    for event in events {
        let Some(combat) = &event.combat else {
            continue;
        };

        match combat.semantic {
            CombatSemantic::EnemyKill => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(
                        combat,
                        KillClassification::EnemyKill,
                        &weapon_ids,
                    ));
                    increment_killer(&mut players, combat, |player| player.kills += 1);
                    increment_victim(&mut players, combat, |player| player.deaths += 1);

                    if combat.vehicle_context.is_kill_from_vehicle {
                        increment_killer(&mut players, combat, |player| {
                            player.kills_from_vehicle += 1;
                        });
                    }
                }
            }
            CombatSemantic::Teamkill => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(combat, KillClassification::Teamkill, &weapon_ids));
                    increment_killer(&mut players, combat, |player| player.teamkills += 1);
                    increment_victim(&mut players, combat, |player| player.deaths += 1);
                }
            }
            CombatSemantic::Suicide => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(combat, KillClassification::Suicide, &weapon_ids));
                    increment_victim(&mut players, combat, |player| {
                        player.deaths += 1;
                        player.suicides += 1;
                    });
                }
            }
            CombatSemantic::NullKillerDeath => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(
                        combat,
                        KillClassification::NullKiller,
                        &weapon_ids,
                    ));
                    increment_victim(&mut players, combat, |player| {
                        player.deaths += 1;
                        player.null_killer_deaths += 1;
                    });
                }
            }
            CombatSemantic::Unknown => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    kills.push(minimal_kill_row(combat, KillClassification::Unknown, &weapon_ids));
                    increment_victim(&mut players, combat, |player| {
                        player.deaths += 1;
                        player.unknown_deaths += 1;
                    });
                }
            }
            CombatSemantic::VehicleDestroyed => {
                if vehicle_or_static_victim(combat, &entity_index) {
                    destroyed_vehicles.push(minimal_destroyed_vehicle_row(
                        combat,
                        &entity_index,
                        &weapon_ids,
                    ));
                    increment_killer(&mut players, combat, |player| player.vehicle_kills += 1);
                }
            }
        }
    }

    MinimalTables {
        players: players.into_values().collect(),
        weapons: weapon_ids.into_iter().map(|(name, id)| MinimalWeaponRow { id, name }).collect(),
        kills,
        destroyed_vehicles,
    }
}

fn minimal_players(entities: &[ObservedEntity]) -> BTreeMap<i64, MinimalPlayerRow> {
    entities
        .iter()
        .filter(|entity| is_legacy_player_entity(entity))
        .map(|entity| {
            (
                entity.source_entity_id,
                MinimalPlayerRow {
                    source_entity_id: entity.source_entity_id,
                    observed_name: player_name(entity),
                    side: present_side(&entity.identity.side),
                    group: player_group(entity),
                    role: player_role(entity),
                    steam_id: observed_string(&entity.identity.steam_id).map(ToOwned::to_owned),
                    compatibility_key: compatibility_key_override(entity),
                    kills: 0,
                    deaths: 0,
                    teamkills: 0,
                    suicides: 0,
                    null_killer_deaths: 0,
                    unknown_deaths: 0,
                    vehicle_kills: 0,
                    kills_from_vehicle: 0,
                },
            )
        })
        .collect()
}

fn weapon_dictionary(
    events: &[NormalizedEvent],
    entity_index: &BTreeMap<i64, &ObservedEntity>,
) -> BTreeMap<String, u32> {
    let mut weapon_names = BTreeSet::<String>::new();

    for event in events {
        let Some(combat) = &event.combat else {
            continue;
        };
        let emits_row = match combat.semantic {
            CombatSemantic::EnemyKill
            | CombatSemantic::Teamkill
            | CombatSemantic::Suicide
            | CombatSemantic::NullKillerDeath
            | CombatSemantic::Unknown => victim_is_or_may_be_player(combat, entity_index),
            CombatSemantic::VehicleDestroyed => vehicle_or_static_victim(combat, entity_index),
        };
        if emits_row
            && let Some(weapon_name) = observed_string(&combat.weapon)
        {
            let _inserted = weapon_names.insert(weapon_name.to_owned());
        }
    }

    weapon_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| (name, u32::try_from(index + 1).unwrap_or(u32::MAX)))
        .collect()
}

fn minimal_kill_row(
    combat: &CombatEventAttributes,
    classification: KillClassification,
    weapon_ids: &BTreeMap<String, u32>,
) -> MinimalKillRow {
    MinimalKillRow {
        killer_source_entity_id: actor_entity_id(&combat.killer),
        victim_source_entity_id: actor_entity_id(&combat.victim),
        classification,
        weapon_id: weapon_id(combat, weapon_ids),
        attacker_vehicle_entity_id: present_i64(&combat.vehicle_context.attacker_vehicle_entity_id),
        attacker_vehicle_class: observed_string(&combat.vehicle_context.attacker_vehicle_class)
            .map(ToOwned::to_owned),
    }
}

fn minimal_destroyed_vehicle_row(
    combat: &CombatEventAttributes,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    weapon_ids: &BTreeMap<String, u32>,
) -> MinimalDestroyedVehicleRow {
    let destroyed_entity =
        actor_entity_id(&combat.victim).and_then(|entity_id| entity_index.get(&entity_id).copied());

    MinimalDestroyedVehicleRow {
        attacker_source_entity_id: actor_entity_id(&combat.killer),
        classification: destroyed_vehicle_classification(combat),
        weapon_id: weapon_id(combat, weapon_ids),
        attacker_vehicle_entity_id: present_i64(&combat.vehicle_context.attacker_vehicle_entity_id),
        attacker_vehicle_class: observed_string(&combat.vehicle_context.attacker_vehicle_class)
            .map(ToOwned::to_owned),
        destroyed_entity_id: destroyed_entity.map(|entity| entity.source_entity_id),
        destroyed_entity_type: destroyed_entity.map(entity_kind_name).map(ToOwned::to_owned),
        destroyed_class: destroyed_entity
            .and_then(|entity| observed_string(&entity.observed_class))
            .map(ToOwned::to_owned),
    }
}

fn increment_killer(
    players: &mut BTreeMap<i64, MinimalPlayerRow>,
    combat: &CombatEventAttributes,
    update: impl FnOnce(&mut MinimalPlayerRow),
) {
    if let Some(killer) =
        actor_entity_id(&combat.killer).and_then(|entity_id| players.get_mut(&entity_id))
    {
        update(killer);
    }
}

fn increment_victim(
    players: &mut BTreeMap<i64, MinimalPlayerRow>,
    combat: &CombatEventAttributes,
    update: impl FnOnce(&mut MinimalPlayerRow),
) {
    if let Some(victim) =
        actor_entity_id(&combat.victim).and_then(|entity_id| players.get_mut(&entity_id))
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

fn compatibility_key_override(entity: &ObservedEntity) -> Option<String> {
    entity
        .compatibility_hints
        .iter()
        .find(|hint| hint.kind == EntityCompatibilityHintKind::DuplicateSlotSameName)
        .and_then(|hint| observed_string(&hint.observed_name))
        .map(|name| format!("legacy_name:{name}"))
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

fn actor_side(field: &FieldPresence<EventActorRef>) -> Option<EntitySide> {
    actor(field).and_then(|actor| present_side(&actor.side))
}

fn weapon_id(combat: &CombatEventAttributes, weapon_ids: &BTreeMap<String, u32>) -> Option<u32> {
    observed_string(&combat.weapon).and_then(|weapon_name| weapon_ids.get(weapon_name).copied())
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

#[allow(dead_code, reason = "keeps the vehicle context import exercised in docs and type checks")]
const fn _vehicle_context_type_marker(_: &VehicleContext) {}
