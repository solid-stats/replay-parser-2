//! Aggregate contribution derivation from normalized combat events.

use std::collections::BTreeMap;

use parser_contract::{
    aggregates::{
        AggregateContributionKind, AggregateContributionRef, AggregateSection,
        BountyInputContributionValue, LegacyCounterContributionValue,
        RelationshipContributionValue,
    },
    events::{
        BountyEligibilityState, CombatEventAttributes, CombatSemantic, EventActorRef,
        LegacyCounterEffect, NormalizedEvent,
    },
    identity::{EntityCompatibilityHintKind, EntityKind, EntitySide, ObservedEntity},
    metadata::ReplayMetadata,
    presence::FieldPresence,
    source_ref::{RuleId, SourceRefs},
};

use crate::artifact::SourceContext;

const LEGACY_PLAYER_RESULTS_KEY: &str = "legacy.player_game_results";
const LEGACY_RELATIONSHIPS_KEY: &str = "legacy.relationships";
const BOUNTY_INPUTS_KEY: &str = "bounty.inputs";

const AGGREGATE_LEGACY_RULE_ID: &str = "aggregate.legacy.counter";
const AGGREGATE_RELATIONSHIP_RULE_ID: &str = "aggregate.relationship.counter";
const AGGREGATE_BOUNTY_RULE_ID: &str = "aggregate.bounty.input";

const RELATIONSHIP_FIELDS: &[&str] = &["killed", "killers", "teamkilled", "teamkillers"];

/// Derives per-replay aggregate contributions and projections from normalized events.
#[must_use]
pub fn derive_aggregate_section(
    _replay: &ReplayMetadata,
    entities: &[ObservedEntity],
    events: &[NormalizedEvent],
    _context: &SourceContext,
) -> AggregateSection {
    let players = player_projection_identities(entities);
    let mut contributions = aggregate_contributions(events, &players);
    contributions.sort_by(|left, right| left.contribution_id.cmp(&right.contribution_id));

    AggregateSection { contributions, projections: BTreeMap::new() }
}

fn aggregate_contributions(
    events: &[NormalizedEvent],
    players: &BTreeMap<i64, PlayerProjectionIdentity>,
) -> Vec<AggregateContributionRef> {
    let mut contributions = Vec::new();

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

fn contribution_ref(
    contribution_id: String,
    kind: AggregateContributionKind,
    event: &NormalizedEvent,
    rule_id: &str,
    value: serde_json::Value,
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
    compatibility_key: String,
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
                PlayerProjectionIdentity { compatibility_key: compatibility_key(entity) },
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

#[allow(dead_code, reason = "projection population is added in the next plan task")]
const fn bounty_inputs_key() -> &'static str {
    BOUNTY_INPUTS_KEY
}
