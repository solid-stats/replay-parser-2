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
        MinimalPlayerKillRow, MinimalPlayerRow, MinimalWeaponRow,
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
    let mut destroyed_vehicles = Vec::new();

    for event in events {
        let Some(combat) = &event.combat else {
            continue;
        };

        match combat.semantic {
            CombatSemantic::EnemyKill => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    add_player_kill(
                        &mut players,
                        combat,
                        KillClassification::EnemyKill,
                        &weapon_ids,
                    );
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
                    add_player_kill(
                        &mut players,
                        combat,
                        KillClassification::Teamkill,
                        &weapon_ids,
                    );
                    increment_killer(&mut players, combat, |player| player.teamkills += 1);
                    increment_victim(&mut players, combat, |player| player.deaths += 1);
                }
            }
            CombatSemantic::Suicide => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    increment_victim(&mut players, combat, |player| {
                        player.deaths += 1;
                        player.suicides += 1;
                    });
                }
            }
            CombatSemantic::NullKillerDeath => {
                if victim_is_or_may_be_player(combat, &entity_index) {
                    increment_victim(&mut players, combat, |player| {
                        player.deaths += 1;
                        player.null_killer_deaths += 1;
                    });
                }
            }
            CombatSemantic::Unknown => {
                if victim_is_or_may_be_player(combat, &entity_index) {
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
        players: players.rows.into_values().collect(),
        weapons: weapon_ids.into_iter().map(|(name, id)| MinimalWeaponRow { id, name }).collect(),
        destroyed_vehicles,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerRows {
    rows: BTreeMap<i64, MinimalPlayerRow>,
    entity_to_player_id: BTreeMap<i64, i64>,
}

fn minimal_players(entities: &[ObservedEntity]) -> PlayerRows {
    let mut grouped = BTreeMap::<PlayerMergeKey, Vec<&ObservedEntity>>::new();

    for entity in entities.iter().filter(|entity| is_legacy_player_entity(entity)) {
        grouped.entry(player_merge_key(entity)).or_default().push(entity);
    }

    let mut rows = BTreeMap::<i64, MinimalPlayerRow>::new();
    let mut entity_to_player_id = BTreeMap::<i64, i64>::new();

    for mut group in grouped.into_values() {
        group.sort_by_key(|entity| entity.source_entity_id);

        let Some(representative) = group.last().copied() else {
            continue;
        };
        let entity_ids = group.iter().map(|entity| entity.source_entity_id).collect::<Vec<_>>();
        let source_entity_ids = if entity_ids.len() > 1 { entity_ids.clone() } else { Vec::new() };
        let representative_id = representative.source_entity_id;
        let raw_name = player_name(representative);
        let name_parts = raw_name.as_deref().map(split_legacy_player_name);

        for entity_id in entity_ids {
            let _ = entity_to_player_id.insert(entity_id, representative_id);
        }

        drop(rows.insert(
            representative_id,
            MinimalPlayerRow {
                source_entity_id: representative_id,
                source_entity_ids,
                observed_name: name_parts.as_ref().map(|parts| parts.name.clone()),
                observed_tag: name_parts.as_ref().and_then(|parts| parts.tag.clone()),
                raw_observed_name: raw_name.filter(|raw| {
                    name_parts.as_ref().is_some_and(|parts| parts.recombined_name() != raw.as_str())
                }),
                side: present_side(&representative.identity.side),
                group: player_group(representative),
                role: player_role(representative),
                steam_id: observed_string(&representative.identity.steam_id).map(ToOwned::to_owned),
                compatibility_key: compatibility_key_override(representative),
                kills: 0,
                deaths: 0,
                teamkills: 0,
                suicides: 0,
                null_killer_deaths: 0,
                unknown_deaths: 0,
                vehicle_kills: 0,
                kills_from_vehicle: 0,
                kill_rows: Vec::new(),
            },
        ));
    }

    PlayerRows { rows, entity_to_player_id }
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
            CombatSemantic::EnemyKill | CombatSemantic::Teamkill => {
                victim_is_or_may_be_player(combat, entity_index)
            }
            CombatSemantic::Suicide | CombatSemantic::NullKillerDeath | CombatSemantic::Unknown => {
                false
            }
            CombatSemantic::VehicleDestroyed => vehicle_or_static_victim(combat, entity_index),
        };
        if emits_row && let Some(weapon_name) = observed_string(&combat.weapon) {
            let _inserted = weapon_names.insert(weapon_name.to_owned());
        }
    }

    weapon_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| (name, u32::try_from(index + 1).unwrap_or(u32::MAX)))
        .collect()
}

fn minimal_player_kill_row(
    combat: &CombatEventAttributes,
    classification: KillClassification,
    weapon_ids: &BTreeMap<String, u32>,
) -> MinimalPlayerKillRow {
    MinimalPlayerKillRow {
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
    players: &mut PlayerRows,
    combat: &CombatEventAttributes,
    update: impl FnOnce(&mut MinimalPlayerRow),
) {
    if let Some(killer) = actor_entity_id(&combat.killer)
        .and_then(|entity_id| players.entity_to_player_id.get(&entity_id))
        .and_then(|player_id| players.rows.get_mut(player_id))
    {
        update(killer);
    }
}

fn increment_victim(
    players: &mut PlayerRows,
    combat: &CombatEventAttributes,
    update: impl FnOnce(&mut MinimalPlayerRow),
) {
    if let Some(victim) = actor_entity_id(&combat.victim)
        .and_then(|entity_id| players.entity_to_player_id.get(&entity_id))
        .and_then(|player_id| players.rows.get_mut(player_id))
    {
        update(victim);
    }
}

fn add_player_kill(
    players: &mut PlayerRows,
    combat: &CombatEventAttributes,
    classification: KillClassification,
    weapon_ids: &BTreeMap<String, u32>,
) {
    let Some(killer_player_id) = actor_entity_id(&combat.killer)
        .and_then(|entity_id| players.entity_to_player_id.get(&entity_id))
        .copied()
    else {
        return;
    };
    let victim_player_id = actor_entity_id(&combat.victim)
        .and_then(|entity_id| players.entity_to_player_id.get(&entity_id))
        .copied();

    let mut kill = minimal_player_kill_row(combat, classification, weapon_ids);
    if victim_player_id.is_some() {
        kill.victim_source_entity_id = victim_player_id;
    }

    if let Some(killer) = players.rows.get_mut(&killer_player_id) {
        killer.kill_rows.push(kill);
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PlayerMergeKey {
    Name(String),
    EntityId(i64),
}

fn player_merge_key(entity: &ObservedEntity) -> PlayerMergeKey {
    player_name(entity)
        .map_or(PlayerMergeKey::EntityId(entity.source_entity_id), PlayerMergeKey::Name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LegacyPlayerNameParts {
    name: String,
    tag: Option<String>,
}

impl LegacyPlayerNameParts {
    fn recombined_name(&self) -> String {
        self.tag.as_ref().map_or_else(|| self.name.clone(), |tag| format!("{tag}{}", self.name))
    }
}

fn split_legacy_player_name(raw_name: &str) -> LegacyPlayerNameParts {
    let trimmed = raw_name.trim();
    let Some(open) = trimmed.find('[') else {
        return LegacyPlayerNameParts { name: trimmed.to_owned(), tag: None };
    };
    let Some(close_offset) = trimmed[open..].find(']') else {
        return LegacyPlayerNameParts { name: trimmed.to_owned(), tag: None };
    };
    let close = open + close_offset;
    let tag = &trimmed[open..=close];
    let mut name = String::new();
    let mut rest = trimmed;
    while let Some(open_index) = rest.find('[') {
        name.push_str(rest[..open_index].trim());
        let Some(close_index) = rest[open_index..].find(']') else {
            name.push_str(rest[open_index..].trim());
            rest = "";
            break;
        };
        rest = &rest[open_index + close_index + 1..];
    }
    name.push_str(rest.trim());
    let name = name.trim().to_owned();

    if tag == "[]" || name.is_empty() {
        return LegacyPlayerNameParts { name, tag: None };
    }

    LegacyPlayerNameParts { name, tag: Some(tag.to_owned()) }
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
