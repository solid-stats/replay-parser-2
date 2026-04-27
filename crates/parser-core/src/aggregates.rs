//! Aggregate contribution derivation from normalized combat events.

#![allow(
    clippy::missing_const_for_fn,
    reason = "private aggregate builders favor uniform helper signatures over const qualification"
)]

use std::collections::{BTreeMap, BTreeSet};

use parser_contract::{
    aggregates::{
        AggregateContributionKind, AggregateContributionRef, AggregateSection,
        BountyInputContributionValue, LegacyCounterContributionValue,
        RelationshipContributionValue, VehicleScoreInputValue, VehicleScoreSign,
    },
    events::{
        BountyEligibilityState, CombatEventAttributes, CombatSemantic, EventActorRef,
        LegacyCounterEffect, NormalizedEvent, VehicleScoreCategory,
    },
    identity::{EntityCompatibilityHintKind, EntityKind, EntitySide, ObservedEntity},
    metadata::ReplayMetadata,
    presence::FieldPresence,
    source_ref::{RuleId, SourceRef, SourceRefs},
};
use serde_json::{Value, json};

use crate::{
    artifact::SourceContext,
    vehicle_score::{teamkill_penalty_weight, vehicle_score_weight},
};

const LEGACY_PLAYER_RESULTS_KEY: &str = "legacy.player_game_results";
const LEGACY_RELATIONSHIPS_KEY: &str = "legacy.relationships";
const BOUNTY_INPUTS_KEY: &str = "bounty.inputs";
const VEHICLE_SCORE_INPUTS_KEY: &str = "vehicle_score.inputs";
const VEHICLE_SCORE_DENOMINATOR_INPUTS_KEY: &str = "vehicle_score.denominator_inputs";

const AGGREGATE_LEGACY_RULE_ID: &str = "aggregate.legacy.counter";
const AGGREGATE_RELATIONSHIP_RULE_ID: &str = "aggregate.relationship.counter";
const AGGREGATE_BOUNTY_RULE_ID: &str = "aggregate.bounty.input";
const AGGREGATE_VEHICLE_SCORE_AWARD_RULE_ID: &str = "aggregate.vehicle_score.award";
const AGGREGATE_VEHICLE_SCORE_PENALTY_RULE_ID: &str = "aggregate.vehicle_score.penalty";

const RELATIONSHIP_FIELDS: &[&str] = &["killed", "killers", "teamkilled", "teamkillers"];

/// Derives per-replay aggregate contributions and projections from normalized events.
#[must_use]
pub fn derive_aggregate_section(
    replay: &ReplayMetadata,
    entities: &[ObservedEntity],
    events: &[NormalizedEvent],
    _context: &SourceContext,
) -> AggregateSection {
    let players = player_projection_identities(entities);
    let mut contributions = aggregate_contributions(events, &players, entities);
    contributions.sort_by(|left, right| left.contribution_id.cmp(&right.contribution_id));
    let projections = aggregate_projections(replay, &players, &contributions);

    AggregateSection { contributions, projections }
}

fn aggregate_projections(
    replay: &ReplayMetadata,
    players: &BTreeMap<i64, PlayerProjectionIdentity>,
    contributions: &[AggregateContributionRef],
) -> BTreeMap<String, Value> {
    let groups = player_projection_groups(players);
    let mut projections = BTreeMap::new();

    drop(projections.insert(
        LEGACY_PLAYER_RESULTS_KEY.to_string(),
        legacy_player_game_results(&groups, contributions),
    ));
    drop(projections.insert(
        LEGACY_RELATIONSHIPS_KEY.to_string(),
        legacy_relationships(&groups, contributions),
    ));
    drop(projections.insert(
        "legacy.game_type_compatibility".to_string(),
        legacy_game_type_compatibility(replay),
    ));
    drop(projections.insert("legacy.squad_inputs".to_string(), legacy_squad_inputs(&groups)));
    drop(projections.insert(
        "legacy.rotation_inputs".to_string(),
        json!({
            "requires_downstream_replay_date": true,
            "parser_action": "server_or_parity_harness_groups_by_replay_date",
        }),
    ));
    drop(projections.insert(BOUNTY_INPUTS_KEY.to_string(), bounty_inputs(&groups, contributions)));
    drop(projections.insert(
        VEHICLE_SCORE_INPUTS_KEY.to_string(),
        vehicle_score_inputs(&groups, contributions),
    ));
    drop(projections.insert(
        VEHICLE_SCORE_DENOMINATOR_INPUTS_KEY.to_string(),
        vehicle_score_denominator_inputs(&groups, contributions),
    ));

    projections
}

fn aggregate_contributions(
    events: &[NormalizedEvent],
    players: &BTreeMap<i64, PlayerProjectionIdentity>,
    entities: &[ObservedEntity],
) -> Vec<AggregateContributionRef> {
    let mut contributions = Vec::new();
    let raw_target_classes = entity_observed_classes(entities);

    for event in events {
        let Some(combat) = &event.combat else {
            continue;
        };

        for effect in &combat.legacy_counter_effects {
            if is_relationship_effect(effect) {
                if let Some(contribution) = relationship_contribution(event, effect, players) {
                    contributions.push(contribution);
                }
                continue;
            }

            if let Some(contribution) = legacy_counter_contribution(event, effect, players) {
                contributions.push(contribution);
            }
        }

        if let Some(contribution) = kills_from_vehicle_contribution(event, combat, players) {
            contributions.push(contribution);
        }

        if let Some(contribution) = bounty_input_contribution(event, combat) {
            contributions.push(contribution);
        }

        if let Some(contribution) =
            vehicle_score_contribution(event, combat, players, &raw_target_classes)
        {
            contributions.push(contribution);
        }
    }

    contributions
}

fn legacy_counter_contribution(
    event: &NormalizedEvent,
    effect: &LegacyCounterEffect,
    players: &BTreeMap<i64, PlayerProjectionIdentity>,
) -> Option<AggregateContributionRef> {
    let player = players.get(&effect.player_entity_id)?;
    let value = LegacyCounterContributionValue {
        projection_key: LEGACY_PLAYER_RESULTS_KEY.to_string(),
        player_entity_id: effect.player_entity_id,
        compatibility_key: player.compatibility_key.clone(),
        field: effect.field.clone(),
        delta: effect.delta,
        event_id: event.event_id.clone(),
    };

    contribution_ref(
        format!("aggregate.legacy.{}.{}.{}", event.event_id, effect.field, effect.player_entity_id),
        AggregateContributionKind::LegacyCounter,
        event,
        AGGREGATE_LEGACY_RULE_ID,
        serde_json::to_value(value).ok()?,
    )
}

fn kills_from_vehicle_contribution(
    event: &NormalizedEvent,
    combat: &CombatEventAttributes,
    players: &BTreeMap<i64, PlayerProjectionIdentity>,
) -> Option<AggregateContributionRef> {
    if combat.semantic != CombatSemantic::EnemyKill || !combat.vehicle_context.is_kill_from_vehicle
    {
        return None;
    }

    let killer_entity_id = actor_entity_id(&combat.killer)?;
    let player = players.get(&killer_entity_id)?;
    let value = LegacyCounterContributionValue {
        projection_key: LEGACY_PLAYER_RESULTS_KEY.to_string(),
        player_entity_id: killer_entity_id,
        compatibility_key: player.compatibility_key.clone(),
        field: "killsFromVehicle".to_string(),
        delta: 1,
        event_id: event.event_id.clone(),
    };

    contribution_ref(
        format!("aggregate.legacy.{}.killsFromVehicle.{}", event.event_id, killer_entity_id),
        AggregateContributionKind::LegacyCounter,
        event,
        AGGREGATE_LEGACY_RULE_ID,
        serde_json::to_value(value).ok()?,
    )
}

fn relationship_contribution(
    event: &NormalizedEvent,
    effect: &LegacyCounterEffect,
    players: &BTreeMap<i64, PlayerProjectionIdentity>,
) -> Option<AggregateContributionRef> {
    let target_entity_id = effect.relationship_target_entity_id?;
    let source_player = players.get(&effect.player_entity_id)?;
    let target_player = players.get(&target_entity_id)?;
    let value = RelationshipContributionValue {
        projection_key: LEGACY_RELATIONSHIPS_KEY.to_string(),
        source_player_entity_id: effect.player_entity_id,
        target_player_entity_id: target_entity_id,
        relationship: effect.field.clone(),
        compatibility_source_key: source_player.compatibility_key.clone(),
        compatibility_target_key: target_player.compatibility_key.clone(),
        count_delta: effect.delta,
    };

    contribution_ref(
        format!(
            "aggregate.relationship.{}.{}.{}.{}",
            event.event_id, effect.field, effect.player_entity_id, target_entity_id
        ),
        AggregateContributionKind::Relationship,
        event,
        AGGREGATE_RELATIONSHIP_RULE_ID,
        serde_json::to_value(value).ok()?,
    )
}

fn bounty_input_contribution(
    event: &NormalizedEvent,
    combat: &CombatEventAttributes,
) -> Option<AggregateContributionRef> {
    if combat.bounty.state != BountyEligibilityState::Eligible
        || combat.semantic != CombatSemantic::EnemyKill
    {
        return None;
    }

    let killer = present_actor(&combat.killer)?;
    let victim = present_actor(&combat.victim)?;
    let value = BountyInputContributionValue {
        killer_entity_id: actor_source_entity_id(killer)?,
        victim_entity_id: actor_source_entity_id(victim)?,
        killer_side: actor_side_name(killer)?.to_string(),
        victim_side: actor_side_name(victim)?.to_string(),
        frame: present_u64(&event.frame),
        event_id: event.event_id.clone(),
        eligible: true,
        exclusion_reasons: Vec::new(),
    };

    contribution_ref(
        format!("aggregate.bounty.{}", event.event_id),
        AggregateContributionKind::BountyInput,
        event,
        AGGREGATE_BOUNTY_RULE_ID,
        serde_json::to_value(value).ok()?,
    )
}

fn vehicle_score_contribution(
    event: &NormalizedEvent,
    combat: &CombatEventAttributes,
    players: &BTreeMap<i64, PlayerProjectionIdentity>,
    raw_target_classes: &BTreeMap<i64, Option<String>>,
) -> Option<AggregateContributionRef> {
    if !combat.vehicle_context.is_kill_from_vehicle {
        return None;
    }

    let sign = match combat.semantic {
        CombatSemantic::EnemyKill | CombatSemantic::VehicleDestroyed => VehicleScoreSign::Award,
        CombatSemantic::Teamkill => VehicleScoreSign::Penalty,
        CombatSemantic::Suicide | CombatSemantic::NullKillerDeath | CombatSemantic::Unknown => {
            return None;
        }
    };

    let player_entity_id = actor_entity_id(&combat.killer)?;
    let _player = players.get(&player_entity_id)?;
    let attacker_category =
        present_vehicle_score_category(&combat.vehicle_context.attacker_vehicle_category)?;
    let target_category = present_vehicle_score_category(&combat.vehicle_context.target_category)?;
    let matrix_weight = vehicle_score_weight(attacker_category, target_category)?;
    let applied_weight = match sign {
        VehicleScoreSign::Award => matrix_weight,
        VehicleScoreSign::Penalty => teamkill_penalty_weight(matrix_weight),
    };
    let teamkill_penalty_clamped =
        sign == VehicleScoreSign::Penalty && applied_weight > matrix_weight;
    let raw_target_class = actor_entity_id(&combat.victim)
        .and_then(|entity_id| raw_target_classes.get(&entity_id))
        .cloned()
        .flatten();

    let value = VehicleScoreInputValue {
        player_entity_id,
        event_id: event.event_id.clone(),
        sign,
        attacker_category,
        target_category,
        raw_attacker_vehicle_name: observed_string(&combat.vehicle_context.attacker_vehicle_name)
            .map(ToOwned::to_owned),
        raw_attacker_vehicle_class: observed_string(&combat.vehicle_context.attacker_vehicle_class)
            .map(ToOwned::to_owned),
        raw_target_class,
        matrix_weight,
        applied_weight,
        teamkill_penalty_clamped,
        denominator_eligible: sign == VehicleScoreSign::Award,
    };
    let rule_id = match sign {
        VehicleScoreSign::Award => AGGREGATE_VEHICLE_SCORE_AWARD_RULE_ID,
        VehicleScoreSign::Penalty => AGGREGATE_VEHICLE_SCORE_PENALTY_RULE_ID,
    };

    contribution_ref(
        format!("aggregate.vehicle_score.{}", event.event_id),
        AggregateContributionKind::VehicleScoreInput,
        event,
        rule_id,
        serde_json::to_value(value).ok()?,
    )
}

fn contribution_ref(
    contribution_id: String,
    kind: AggregateContributionKind,
    event: &NormalizedEvent,
    rule_id: &str,
    value: Value,
) -> Option<AggregateContributionRef> {
    Some(AggregateContributionRef {
        contribution_id,
        kind,
        event_id: Some(event.event_id.clone()),
        source_refs: SourceRefs::new(event.source_refs.as_slice().to_vec()).ok()?,
        rule_id: RuleId::new(rule_id).ok()?,
        value,
    })
}

fn is_relationship_effect(effect: &LegacyCounterEffect) -> bool {
    effect.relationship_target_entity_id.is_some()
        && RELATIONSHIP_FIELDS.contains(&effect.field.as_str())
}

#[derive(Debug, Clone)]
struct PlayerProjectionIdentity {
    source_entity_id: i64,
    compatibility_key: String,
    observed_name: Option<String>,
    side: Option<&'static str>,
    source_refs: Vec<SourceRef>,
}

fn player_projection_identities(
    entities: &[ObservedEntity],
) -> BTreeMap<i64, PlayerProjectionIdentity> {
    entities
        .iter()
        .filter(|entity| matches!(entity.kind, EntityKind::Unit))
        .map(|entity| {
            (
                entity.source_entity_id,
                PlayerProjectionIdentity {
                    source_entity_id: entity.source_entity_id,
                    compatibility_key: compatibility_key(entity),
                    observed_name: observed_string(&entity.observed_name).map(ToOwned::to_owned),
                    side: present_side_name(&entity.identity.side),
                    source_refs: entity.source_refs.as_slice().to_vec(),
                },
            )
        })
        .collect()
}

#[derive(Debug, Clone)]
struct PlayerProjectionGroup {
    compatibility_key: String,
    observed_entity_ids: Vec<i64>,
    observed_name: Option<String>,
    side: Option<&'static str>,
    source_refs: Vec<SourceRef>,
}

fn player_projection_groups(
    players: &BTreeMap<i64, PlayerProjectionIdentity>,
) -> BTreeMap<String, PlayerProjectionGroup> {
    let mut groups = BTreeMap::<String, PlayerProjectionGroup>::new();

    for player in players.values() {
        let group = groups.entry(player.compatibility_key.clone()).or_insert_with(|| {
            PlayerProjectionGroup {
                compatibility_key: player.compatibility_key.clone(),
                observed_entity_ids: Vec::new(),
                observed_name: player.observed_name.clone(),
                side: player.side,
                source_refs: Vec::new(),
            }
        });
        group.observed_entity_ids.push(player.source_entity_id);
        group.source_refs.extend(player.source_refs.iter().cloned());

        if group.observed_name.is_none() {
            group.observed_name.clone_from(&player.observed_name);
        }
        if group.side.is_none() {
            group.side = player.side;
        }
    }

    for group in groups.values_mut() {
        group.observed_entity_ids.sort_unstable();
        group.observed_entity_ids.dedup();
    }

    groups
}

#[derive(Debug, Clone)]
struct PlayerResultAccumulator {
    group: PlayerProjectionGroup,
    kills: i64,
    kills_from_vehicle: i64,
    vehicle_kills: i64,
    teamkills: i64,
    is_dead: bool,
    is_dead_by_teamkill: bool,
    source_contribution_ids: BTreeSet<String>,
}

impl PlayerResultAccumulator {
    fn new(group: PlayerProjectionGroup) -> Self {
        Self {
            group,
            kills: 0,
            kills_from_vehicle: 0,
            vehicle_kills: 0,
            teamkills: 0,
            is_dead: false,
            is_dead_by_teamkill: false,
            source_contribution_ids: BTreeSet::new(),
        }
    }

    fn apply(&mut self, contribution_id: &str, value: &LegacyCounterContributionValue) {
        match value.field.as_str() {
            "kills" => self.kills += value.delta,
            "killsFromVehicle" => self.kills_from_vehicle += value.delta,
            "vehicleKills" => self.vehicle_kills += value.delta,
            "teamkills" => self.teamkills += value.delta,
            "isDead" => self.is_dead |= value.delta > 0,
            "isDeadByTeamkill" => self.is_dead_by_teamkill |= value.delta > 0,
            _ => return,
        }

        let _ = self.source_contribution_ids.insert(contribution_id.to_string());
    }

    fn to_value(&self) -> Value {
        let deaths_total = i64::from(self.is_dead);
        let deaths_by_teamkills = i64::from(self.is_dead_by_teamkill);
        let contribution_ids =
            self.source_contribution_ids.iter().cloned().collect::<Vec<String>>();

        json!({
            "compatibility_key": self.group.compatibility_key,
            "observed_entity_ids": self.group.observed_entity_ids,
            "observed_name": self.group.observed_name,
            "side": self.group.side,
            "kills": self.kills,
            "killsFromVehicle": self.kills_from_vehicle,
            "vehicleKills": self.vehicle_kills,
            "teamkills": self.teamkills,
            "isDead": self.is_dead,
            "isDeadByTeamkill": self.is_dead_by_teamkill,
            "deaths": {
                "total": deaths_total,
                "byTeamkills": deaths_by_teamkills,
            },
            "kdRatio": kd_ratio(self.kills, self.teamkills, deaths_total, deaths_by_teamkills),
            "killsFromVehicleCoef": kills_from_vehicle_coef(self.kills, self.kills_from_vehicle),
            "score": score(1, self.kills, self.teamkills, deaths_by_teamkills),
            "totalPlayedGames": 1,
            "source_contribution_ids": contribution_ids,
        })
    }
}

fn legacy_player_game_results(
    groups: &BTreeMap<String, PlayerProjectionGroup>,
    contributions: &[AggregateContributionRef],
) -> Value {
    let mut rows = BTreeMap::<String, PlayerResultAccumulator>::new();

    for contribution in contributions {
        if contribution.kind != AggregateContributionKind::LegacyCounter {
            continue;
        }

        let Some(value) = legacy_counter_value(contribution) else {
            continue;
        };
        let Some(group) = groups.get(&value.compatibility_key).cloned() else {
            continue;
        };

        rows.entry(value.compatibility_key.clone())
            .or_insert_with(|| PlayerResultAccumulator::new(group))
            .apply(&contribution.contribution_id, &value);
    }

    Value::Array(rows.values().map(PlayerResultAccumulator::to_value).collect())
}

#[derive(Debug, Clone)]
struct RelationshipAccumulator {
    relationship: String,
    source_group: PlayerProjectionGroup,
    target_group: PlayerProjectionGroup,
    count: i64,
    event_ids: BTreeSet<String>,
    source_contribution_ids: BTreeSet<String>,
}

impl RelationshipAccumulator {
    fn new(
        relationship: String,
        source_group: PlayerProjectionGroup,
        target_group: PlayerProjectionGroup,
    ) -> Self {
        Self {
            relationship,
            source_group,
            target_group,
            count: 0,
            event_ids: BTreeSet::new(),
            source_contribution_ids: BTreeSet::new(),
        }
    }

    fn apply(
        &mut self,
        contribution: &AggregateContributionRef,
        value: &RelationshipContributionValue,
    ) {
        self.count += value.count_delta;
        if let Some(event_id) = &contribution.event_id {
            let _ = self.event_ids.insert(event_id.clone());
        }
        let _ = self.source_contribution_ids.insert(contribution.contribution_id.clone());
    }

    fn to_value(&self) -> Value {
        json!({
            "relationship": self.relationship,
            "source_compatibility_key": self.source_group.compatibility_key,
            "source_observed_entity_ids": self.source_group.observed_entity_ids,
            "source_observed_name": self.source_group.observed_name,
            "target_compatibility_key": self.target_group.compatibility_key,
            "target_observed_entity_ids": self.target_group.observed_entity_ids,
            "target_observed_name": self.target_group.observed_name,
            "count": self.count,
            "event_ids": self.event_ids.iter().cloned().collect::<Vec<String>>(),
            "source_contribution_ids": self.source_contribution_ids.iter().cloned().collect::<Vec<String>>(),
        })
    }
}

fn legacy_relationships(
    groups: &BTreeMap<String, PlayerProjectionGroup>,
    contributions: &[AggregateContributionRef],
) -> Value {
    let mut rows_by_relationship =
        BTreeMap::<String, BTreeMap<String, RelationshipAccumulator>>::new();

    for relationship in RELATIONSHIP_FIELDS {
        drop(rows_by_relationship.insert((*relationship).to_string(), BTreeMap::new()));
    }

    for contribution in contributions {
        if contribution.kind != AggregateContributionKind::Relationship {
            continue;
        }

        let Some(value) = relationship_value(contribution) else {
            continue;
        };
        let Some(source_group) = groups.get(&value.compatibility_source_key).cloned() else {
            continue;
        };
        let Some(target_group) = groups.get(&value.compatibility_target_key).cloned() else {
            continue;
        };

        let row_key = format!(
            "{}|{}|{}",
            value.relationship, value.compatibility_source_key, value.compatibility_target_key
        );
        rows_by_relationship
            .entry(value.relationship.clone())
            .or_default()
            .entry(row_key)
            .or_insert_with(|| {
                RelationshipAccumulator::new(value.relationship.clone(), source_group, target_group)
            })
            .apply(contribution, &value);
    }

    let mut value = serde_json::Map::new();
    for relationship in RELATIONSHIP_FIELDS {
        let rows = rows_by_relationship
            .remove(*relationship)
            .unwrap_or_default()
            .into_values()
            .map(|row| row.to_value())
            .collect::<Vec<_>>();
        drop(value.insert((*relationship).to_string(), Value::Array(rows)));
    }

    Value::Object(value)
}

fn legacy_game_type_compatibility(replay: &ReplayMetadata) -> Value {
    let mission_name = observed_string(&replay.mission_name).map(ToOwned::to_owned);
    let prefix_bucket = mission_name.as_deref().map_or("other", mission_prefix_bucket);
    let source_refs = field_source_ref(&replay.mission_name).map_or_else(
        || Value::Array(Vec::new()),
        |source_ref| source_refs_value(std::slice::from_ref(source_ref)),
    );

    json!({
        "mission_name": mission_name,
        "prefix_bucket": prefix_bucket,
        "parser_action": "emit_filter_metadata_only",
        "source_refs": source_refs,
    })
}

fn mission_prefix_bucket(mission_name: &str) -> &'static str {
    let trimmed = mission_name.trim().to_ascii_lowercase();

    if trimmed.starts_with("sgs") {
        return "sgs";
    }
    if trimmed.starts_with("mace") {
        return "mace";
    }
    if trimmed.starts_with("sm") {
        return "sm";
    }
    if trimmed.starts_with("sg") {
        return "sg";
    }

    "other"
}

fn legacy_squad_inputs(groups: &BTreeMap<String, PlayerProjectionGroup>) -> Value {
    Value::Array(
        groups
            .values()
            .map(|group| {
                json!({
                    "compatibility_key": group.compatibility_key,
                    "observed_name": group.observed_name,
                    "squad_prefix": group.observed_name.as_deref().and_then(bracket_squad_prefix),
                    "source_entity_ids": group.observed_entity_ids,
                    "source_refs": source_refs_value(&group.source_refs),
                })
            })
            .collect(),
    )
}

fn bracket_squad_prefix(observed_name: &str) -> Option<String> {
    let trimmed = observed_name.trim();
    let start = trimmed.find('[')?;
    let end = trimmed[start..].find(']')? + start;

    if end <= start + 1 {
        return None;
    }

    Some(trimmed[start..=end].to_string())
}

fn bounty_inputs(
    groups: &BTreeMap<String, PlayerProjectionGroup>,
    contributions: &[AggregateContributionRef],
) -> Value {
    Value::Array(
        contributions
            .iter()
            .filter(|contribution| contribution.kind == AggregateContributionKind::BountyInput)
            .filter_map(|contribution| bounty_input_row(groups, contribution))
            .collect(),
    )
}

fn bounty_input_row(
    groups: &BTreeMap<String, PlayerProjectionGroup>,
    contribution: &AggregateContributionRef,
) -> Option<Value> {
    let value = bounty_input_value(contribution)?;
    let killer_group = group_by_entity_id(groups, value.killer_entity_id)?;
    let victim_group = group_by_entity_id(groups, value.victim_entity_id)?;

    Some(json!({
        "event_id": value.event_id,
        "source_contribution_id": contribution.contribution_id,
        "killer_entity_id": value.killer_entity_id,
        "killer_compatibility_key": killer_group.compatibility_key,
        "killer_observed_name": killer_group.observed_name,
        "killer_side": value.killer_side,
        "victim_entity_id": value.victim_entity_id,
        "victim_compatibility_key": victim_group.compatibility_key,
        "victim_observed_name": victim_group.observed_name,
        "victim_side": value.victim_side,
        "frame": value.frame,
        "eligible": value.eligible,
        "exclusion_reasons": value.exclusion_reasons,
        "source_refs": source_refs_value(contribution.source_refs.as_slice()),
    }))
}

fn vehicle_score_inputs(
    groups: &BTreeMap<String, PlayerProjectionGroup>,
    contributions: &[AggregateContributionRef],
) -> Value {
    Value::Array(
        contributions
            .iter()
            .filter(|contribution| {
                contribution.kind == AggregateContributionKind::VehicleScoreInput
            })
            .filter_map(|contribution| vehicle_score_input_row(groups, contribution))
            .collect(),
    )
}

fn vehicle_score_input_row(
    groups: &BTreeMap<String, PlayerProjectionGroup>,
    contribution: &AggregateContributionRef,
) -> Option<Value> {
    let value = vehicle_score_value(contribution)?;
    let player_group = group_by_entity_id(groups, value.player_entity_id)?;

    Some(json!({
        "source_contribution_id": contribution.contribution_id,
        "event_id": value.event_id,
        "player_entity_id": value.player_entity_id,
        "compatibility_key": player_group.compatibility_key,
        "observed_entity_ids": player_group.observed_entity_ids,
        "sign": value.sign,
        "attacker_category": value.attacker_category,
        "target_category": value.target_category,
        "raw_attacker_vehicle_name": value.raw_attacker_vehicle_name,
        "raw_attacker_vehicle_class": value.raw_attacker_vehicle_class,
        "raw_target_class": value.raw_target_class,
        "matrix_weight": value.matrix_weight,
        "applied_weight": value.applied_weight,
        "teamkill_penalty_clamped": value.teamkill_penalty_clamped,
        "denominator_eligible": value.denominator_eligible,
        "rule_id": contribution.rule_id.as_str(),
        "source_refs": source_refs_value(contribution.source_refs.as_slice()),
    }))
}

fn vehicle_score_denominator_inputs(
    groups: &BTreeMap<String, PlayerProjectionGroup>,
    contributions: &[AggregateContributionRef],
) -> Value {
    let mut rows = BTreeMap::<String, (PlayerProjectionGroup, BTreeSet<String>)>::new();

    for contribution in contributions {
        if contribution.kind != AggregateContributionKind::VehicleScoreInput {
            continue;
        }

        let Some(value) = vehicle_score_value(contribution) else {
            continue;
        };
        if value.sign != VehicleScoreSign::Award || !value.denominator_eligible {
            continue;
        }
        let Some(group) = group_by_entity_id(groups, value.player_entity_id).cloned() else {
            continue;
        };

        let (_, contribution_ids) =
            rows.entry(group.compatibility_key.clone()).or_insert_with(|| (group, BTreeSet::new()));
        let _ = contribution_ids.insert(contribution.contribution_id.clone());
    }

    Value::Array(
        rows.into_values()
            .map(|(group, contribution_ids)| {
                json!({
                    "compatibility_key": group.compatibility_key,
                    "observed_entity_ids": group.observed_entity_ids,
                    "has_vehicle_kill": true,
                    "source_contribution_ids": contribution_ids.into_iter().collect::<Vec<_>>(),
                })
            })
            .collect(),
    )
}

fn group_by_entity_id(
    groups: &BTreeMap<String, PlayerProjectionGroup>,
    entity_id: i64,
) -> Option<&PlayerProjectionGroup> {
    groups.values().find(|group| group.observed_entity_ids.binary_search(&entity_id).is_ok())
}

fn entity_observed_classes(entities: &[ObservedEntity]) -> BTreeMap<i64, Option<String>> {
    entities
        .iter()
        .map(|entity| {
            (
                entity.source_entity_id,
                observed_string(&entity.observed_class).map(ToOwned::to_owned),
            )
        })
        .collect()
}

fn compatibility_key(entity: &ObservedEntity) -> String {
    entity
        .compatibility_hints
        .iter()
        .find(|hint| hint.kind == EntityCompatibilityHintKind::DuplicateSlotSameName)
        .and_then(|hint| observed_string(&hint.observed_name))
        .map_or_else(
            || format!("entity:{}", entity.source_entity_id),
            |name| format!("legacy_name:{name}"),
        )
}

fn present_actor(field: &FieldPresence<EventActorRef>) -> Option<&EventActorRef> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(value),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn actor_entity_id(field: &FieldPresence<EventActorRef>) -> Option<i64> {
    present_actor(field).and_then(actor_source_entity_id)
}

fn actor_source_entity_id(actor: &EventActorRef) -> Option<i64> {
    present_i64(&actor.source_entity_id)
}

fn actor_side_name(actor: &EventActorRef) -> Option<&'static str> {
    present_side_name(&actor.side)
}

fn present_i64(field: &FieldPresence<i64>) -> Option<i64> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(*value),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn present_u64(field: &FieldPresence<u64>) -> Option<u64> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(*value),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn present_side_name(field: &FieldPresence<EntitySide>) -> Option<&'static str> {
    match field {
        FieldPresence::Present { value, source: _ } => Some(match value {
            EntitySide::East => "east",
            EntitySide::West => "west",
            EntitySide::Guer => "guer",
            EntitySide::Civ => "civ",
            EntitySide::Unknown => "unknown",
        }),
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn legacy_counter_value(
    contribution: &AggregateContributionRef,
) -> Option<LegacyCounterContributionValue> {
    serde_json::from_value(contribution.value.clone()).ok()
}

fn relationship_value(
    contribution: &AggregateContributionRef,
) -> Option<RelationshipContributionValue> {
    serde_json::from_value(contribution.value.clone()).ok()
}

fn bounty_input_value(
    contribution: &AggregateContributionRef,
) -> Option<BountyInputContributionValue> {
    serde_json::from_value(contribution.value.clone()).ok()
}

fn vehicle_score_value(contribution: &AggregateContributionRef) -> Option<VehicleScoreInputValue> {
    serde_json::from_value(contribution.value.clone()).ok()
}

fn present_vehicle_score_category(
    field: &FieldPresence<VehicleScoreCategory>,
) -> Option<VehicleScoreCategory> {
    match field {
        FieldPresence::Present { value, source: _ } if *value != VehicleScoreCategory::Unknown => {
            Some(*value)
        }
        FieldPresence::Present { .. }
        | FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn kd_ratio(kills: i64, teamkills: i64, deaths_total: i64, deaths_by_teamkills: i64) -> f64 {
    let deaths_without_teamkills = (deaths_total - deaths_by_teamkills).abs();
    let total = kills - teamkills;

    if deaths_without_teamkills == 0 {
        return legacy_count_to_f64(total);
    }

    round_2(legacy_count_to_f64(total) / legacy_count_to_f64(deaths_without_teamkills))
}

fn score(total_played_games: i64, kills: i64, teamkills: i64, deaths_by_teamkills: i64) -> f64 {
    let total_score = kills - teamkills;
    let games_count = total_played_games - deaths_by_teamkills;

    if games_count <= 0 {
        return legacy_count_to_f64(total_score);
    }

    round_2(legacy_count_to_f64(total_score) / legacy_count_to_f64(games_count))
}

fn kills_from_vehicle_coef(kills: i64, kills_from_vehicle: i64) -> f64 {
    if kills == 0 || kills_from_vehicle == 0 {
        return 0.0;
    }

    round_2(legacy_count_to_f64(kills_from_vehicle) / legacy_count_to_f64(kills))
}

fn round_2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "legacy aggregate formulas are defined as f64 projections from replay-local counters"
)]
fn legacy_count_to_f64(value: i64) -> f64 {
    value as f64
}

fn source_refs_value(source_refs: &[SourceRef]) -> Value {
    serde_json::to_value(source_refs).unwrap_or_else(|_| Value::Array(Vec::new()))
}

fn field_source_ref<T>(field: &FieldPresence<T>) -> Option<&SourceRef> {
    match field {
        FieldPresence::Present { source: Some(source), .. }
        | FieldPresence::ExplicitNull { source: Some(source), .. }
        | FieldPresence::Unknown { source: Some(source), .. }
        | FieldPresence::Inferred { source: Some(source), .. } => Some(source),
        FieldPresence::Present { source: None, .. }
        | FieldPresence::ExplicitNull { source: None, .. }
        | FieldPresence::Unknown { source: None, .. }
        | FieldPresence::Inferred { source: None, .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn observed_string(field: &FieldPresence<String>) -> Option<&str> {
    match field {
        FieldPresence::Present { value, source: _ } | FieldPresence::Inferred { value, .. } => {
            Some(value.as_str())
        }
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}
