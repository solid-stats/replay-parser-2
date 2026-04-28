//! Normalization for source-backed combat events.

#![allow(
    clippy::missing_const_for_fn,
    reason = "private combat builders keep signatures consistent with non-const contract values"
)]

use std::collections::BTreeMap;

use parser_contract::{
    diagnostic::{Diagnostic, DiagnosticSeverity},
    events::{
        BountyEligibility, BountyEligibilityState, BountyExclusionReason, CombatEventAttributes,
        CombatSemantic, CombatVictimKind, EventActorRef, LegacyCounterEffect, NormalizedEvent,
        NormalizedEventKind, VehicleContext, VehicleScoreCategory,
    },
    identity::{EntityKind, EntitySide, ObservedEntity},
    presence::{FieldPresence, NullReason, UnknownReason},
    source_ref::{RuleId, SourceRef, SourceRefs},
};

use crate::{
    artifact::SourceContext,
    diagnostics::{DiagnosticAccumulator, DiagnosticImpact},
    legacy_player::is_legacy_player_entity,
    raw::{KilledEventKillInfo, KilledEventObservation, RawReplay, killed_events},
    vehicle_score::category_from_vehicle_class,
};

const ENEMY_KILL_RULE_ID: &str = "event.killed.enemy";
const TEAMKILL_RULE_ID: &str = "event.killed.teamkill";
const SUICIDE_RULE_ID: &str = "event.killed.suicide";
const NULL_KILLER_RULE_ID: &str = "event.killed.null_killer";
const VEHICLE_DESTROYED_RULE_ID: &str = "event.killed.vehicle_destroyed";
const UNKNOWN_RULE_ID: &str = "event.killed.unknown";

/// Normalizes raw `killed` tuples into deterministic combat events.
#[allow(
    clippy::needless_pass_by_value,
    clippy::trivially_copy_pass_by_ref,
    reason = "the plan requires source observations to be consumed in source order"
)]
pub fn normalize_combat_events(
    raw: &RawReplay<'_>,
    entities: &[ObservedEntity],
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Vec<NormalizedEvent> {
    let entity_index =
        entities.iter().map(|entity| (entity.source_entity_id, entity)).collect::<BTreeMap<_, _>>();

    killed_events(*raw)
        .into_iter()
        .filter_map(|observation| {
            normalize_killed_event(&observation, &entity_index, context, diagnostics)
        })
        .collect()
}

fn normalize_killed_event(
    observation: &KilledEventObservation,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Option<NormalizedEvent> {
    if observation.frame.is_none() {
        push_unknown_diagnostic(
            diagnostics,
            UnknownDiagnostic {
                code: "event.killed_shape_unknown",
                message: "Killed event frame had an unexpected source shape",
                expected_shape: "numeric frame in killed tuple slot 0",
                observed_shape: "absent_or_non_numeric",
                parser_action: "emit_unknown_combat_event",
            },
            observation,
            context,
        );
        return build_unknown_event(observation, None, None, None, entity_index, context);
    }

    match &observation.kill_info {
        KilledEventKillInfo::NullKiller => {
            normalize_null_killer_event(observation, entity_index, context, diagnostics)
        }
        KilledEventKillInfo::Killer { killer_entity_id, weapon } => normalize_killer_event(
            observation,
            *killer_entity_id,
            weapon.as_deref(),
            entity_index,
            context,
            diagnostics,
        ),
        KilledEventKillInfo::Malformed { observed_shape } => {
            push_unknown_diagnostic(
                diagnostics,
                UnknownDiagnostic {
                    code: "event.killed_shape_unknown",
                    message: "Killed event kill-info tuple had an unexpected source shape",
                    expected_shape: "killed tuple with numeric victim and [killer_id, weapon]",
                    observed_shape,
                    parser_action: "emit_unknown_combat_event",
                },
                observation,
                context,
            );
            build_unknown_event(observation, None, None, None, entity_index, context)
        }
    }
}

fn normalize_null_killer_event(
    observation: &KilledEventObservation,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Option<NormalizedEvent> {
    let Some(victim) = victim_entity(observation, entity_index) else {
        push_actor_unknown_diagnostic(
            diagnostics,
            "Killed event has an explicit null killer but no known player victim",
            observation,
            context,
        );
        return build_unknown_event(observation, None, None, None, entity_index, context);
    };

    if !is_legacy_player_entity(victim) {
        push_actor_unknown_diagnostic(
            diagnostics,
            "Killed event has an explicit null killer and a non-player victim",
            observation,
            context,
        );
        return build_unknown_event(observation, None, Some(victim), None, entity_index, context);
    }

    let source_ref = event_source_ref(context, observation, NULL_KILLER_RULE_ID);
    let victim_actor = actor_ref(victim);
    build_event(CombatEventBuild {
        observation,
        source_ref: source_ref.clone(),
        rule_id: NULL_KILLER_RULE_ID,
        kind: NormalizedEventKind::Death,
        semantic: CombatSemantic::NullKillerDeath,
        killer: FieldPresence::ExplicitNull {
            reason: NullReason::NullKiller,
            source: Some(source_ref.clone()),
        },
        victim: FieldPresence::Present {
            value: victim_actor.clone(),
            source: Some(source_ref.clone()),
        },
        actors: vec![victim_actor],
        victim_kind: CombatVictimKind::Player,
        weapon: None,
        distance_meters: observation.distance_meters,
        vehicle_context: vehicle_context(None, Some(victim), entity_index, &source_ref),
        bounty: excluded_bounty(vec![BountyExclusionReason::NullKiller]),
        legacy_counter_effects: vec![legacy_effect(victim.source_entity_id, "isDead", 1, None)],
    })
}

fn normalize_killer_event(
    observation: &KilledEventObservation,
    killer_entity_id: i64,
    weapon: Option<&str>,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
    diagnostics: &mut DiagnosticAccumulator,
) -> Option<NormalizedEvent> {
    if observation.killed_entity_id.is_none() {
        push_unknown_diagnostic(
            diagnostics,
            UnknownDiagnostic {
                code: "event.killed_shape_unknown",
                message: "Killed event has no numeric victim entity identifier",
                expected_shape: "numeric killed entity identifier",
                observed_shape: "absent_or_non_numeric",
                parser_action: "emit_unknown_combat_event",
            },
            observation,
            context,
        );
        return build_unknown_event(observation, None, None, weapon, entity_index, context);
    }

    let killer = entity_index.get(&killer_entity_id).copied();
    let victim = victim_entity(observation, entity_index);

    let (Some(killer), Some(victim)) = (killer, victim) else {
        push_actor_unknown_diagnostic(
            diagnostics,
            "Killed event references an actor that is missing from normalized entities",
            observation,
            context,
        );
        return build_unknown_event(observation, killer, victim, weapon, entity_index, context);
    };

    if is_legacy_player_entity(killer) && is_vehicle_or_static(victim) {
        return match same_present_side(killer, victim) {
            Some(true) => build_friendly_vehicle_destroyed_event(
                observation,
                killer,
                victim,
                weapon,
                entity_index,
                context,
            ),
            Some(false) | None => build_vehicle_destroyed_event(
                observation,
                killer,
                victim,
                weapon,
                entity_index,
                context,
            ),
        };
    }

    if !(is_legacy_player_entity(killer) && is_legacy_player_entity(victim)) {
        push_actor_unknown_diagnostic(
            diagnostics,
            "Killed event actor or victim type is not auditable as a player combat event",
            observation,
            context,
        );
        return build_unknown_event(
            observation,
            Some(killer),
            Some(victim),
            weapon,
            entity_index,
            context,
        );
    }

    if killer.source_entity_id == victim.source_entity_id {
        return build_suicide_event(observation, victim, weapon, entity_index, context);
    }

    match same_present_side(killer, victim) {
        Some(true) => {
            build_teamkill_event(observation, killer, victim, weapon, entity_index, context)
        }
        Some(false) => {
            build_enemy_kill_event(observation, killer, victim, weapon, entity_index, context)
        }
        None => {
            push_actor_unknown_diagnostic(
                diagnostics,
                "Killed event player sides are incomplete for enemy/teamkill classification",
                observation,
                context,
            );
            build_unknown_event(
                observation,
                Some(killer),
                Some(victim),
                weapon,
                entity_index,
                context,
            )
        }
    }
}

fn build_enemy_kill_event(
    observation: &KilledEventObservation,
    killer: &ObservedEntity,
    victim: &ObservedEntity,
    weapon: Option<&str>,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
) -> Option<NormalizedEvent> {
    let source_ref = event_source_ref(context, observation, ENEMY_KILL_RULE_ID);
    let killer_actor = actor_ref(killer);
    let victim_actor = actor_ref(victim);

    build_event(CombatEventBuild {
        observation,
        source_ref: source_ref.clone(),
        rule_id: ENEMY_KILL_RULE_ID,
        kind: NormalizedEventKind::Kill,
        semantic: CombatSemantic::EnemyKill,
        killer: FieldPresence::Present {
            value: killer_actor.clone(),
            source: Some(source_ref.clone()),
        },
        victim: FieldPresence::Present {
            value: victim_actor.clone(),
            source: Some(source_ref.clone()),
        },
        actors: vec![killer_actor, victim_actor],
        victim_kind: CombatVictimKind::Player,
        weapon,
        distance_meters: observation.distance_meters,
        vehicle_context: vehicle_context(weapon, Some(victim), entity_index, &source_ref),
        bounty: BountyEligibility {
            state: BountyEligibilityState::Eligible,
            exclusion_reasons: Vec::new(),
        },
        legacy_counter_effects: vec![
            legacy_effect(killer.source_entity_id, "kills", 1, None),
            legacy_effect(victim.source_entity_id, "isDead", 1, None),
            legacy_effect(killer.source_entity_id, "killed", 1, Some(victim.source_entity_id)),
            legacy_effect(victim.source_entity_id, "killers", 1, Some(killer.source_entity_id)),
        ],
    })
}

fn build_teamkill_event(
    observation: &KilledEventObservation,
    killer: &ObservedEntity,
    victim: &ObservedEntity,
    weapon: Option<&str>,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
) -> Option<NormalizedEvent> {
    let source_ref = event_source_ref(context, observation, TEAMKILL_RULE_ID);
    let killer_actor = actor_ref(killer);
    let victim_actor = actor_ref(victim);

    build_event(CombatEventBuild {
        observation,
        source_ref: source_ref.clone(),
        rule_id: TEAMKILL_RULE_ID,
        kind: NormalizedEventKind::Teamkill,
        semantic: CombatSemantic::Teamkill,
        killer: FieldPresence::Present {
            value: killer_actor.clone(),
            source: Some(source_ref.clone()),
        },
        victim: FieldPresence::Present {
            value: victim_actor.clone(),
            source: Some(source_ref.clone()),
        },
        actors: vec![killer_actor, victim_actor],
        victim_kind: CombatVictimKind::Player,
        weapon,
        distance_meters: observation.distance_meters,
        vehicle_context: vehicle_context(weapon, Some(victim), entity_index, &source_ref),
        bounty: excluded_bounty(vec![BountyExclusionReason::Teamkill]),
        legacy_counter_effects: vec![
            legacy_effect(killer.source_entity_id, "teamkills", 1, None),
            legacy_effect(victim.source_entity_id, "isDead", 1, None),
            legacy_effect(victim.source_entity_id, "isDeadByTeamkill", 1, None),
            legacy_effect(killer.source_entity_id, "teamkilled", 1, Some(victim.source_entity_id)),
            legacy_effect(victim.source_entity_id, "teamkillers", 1, Some(killer.source_entity_id)),
        ],
    })
}

fn build_suicide_event(
    observation: &KilledEventObservation,
    victim: &ObservedEntity,
    weapon: Option<&str>,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
) -> Option<NormalizedEvent> {
    let source_ref = event_source_ref(context, observation, SUICIDE_RULE_ID);
    let victim_actor = actor_ref(victim);

    build_event(CombatEventBuild {
        observation,
        source_ref: source_ref.clone(),
        rule_id: SUICIDE_RULE_ID,
        kind: NormalizedEventKind::Suicide,
        semantic: CombatSemantic::Suicide,
        killer: FieldPresence::Present {
            value: victim_actor.clone(),
            source: Some(source_ref.clone()),
        },
        victim: FieldPresence::Present {
            value: victim_actor.clone(),
            source: Some(source_ref.clone()),
        },
        actors: vec![victim_actor],
        victim_kind: CombatVictimKind::Player,
        weapon,
        distance_meters: observation.distance_meters,
        vehicle_context: vehicle_context(weapon, Some(victim), entity_index, &source_ref),
        bounty: excluded_bounty(vec![BountyExclusionReason::Suicide]),
        legacy_counter_effects: vec![legacy_effect(victim.source_entity_id, "isDead", 1, None)],
    })
}

fn build_vehicle_destroyed_event(
    observation: &KilledEventObservation,
    killer: &ObservedEntity,
    victim: &ObservedEntity,
    weapon: Option<&str>,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
) -> Option<NormalizedEvent> {
    let source_ref = event_source_ref(context, observation, VEHICLE_DESTROYED_RULE_ID);
    let killer_actor = actor_ref(killer);
    let victim_actor = actor_ref(victim);

    build_event(CombatEventBuild {
        observation,
        source_ref: source_ref.clone(),
        rule_id: VEHICLE_DESTROYED_RULE_ID,
        kind: NormalizedEventKind::VehicleKilled,
        semantic: CombatSemantic::VehicleDestroyed,
        killer: FieldPresence::Present {
            value: killer_actor.clone(),
            source: Some(source_ref.clone()),
        },
        victim: FieldPresence::Present {
            value: victim_actor.clone(),
            source: Some(source_ref.clone()),
        },
        actors: vec![killer_actor, victim_actor],
        victim_kind: victim_kind(victim),
        weapon,
        distance_meters: observation.distance_meters,
        vehicle_context: vehicle_context(weapon, Some(victim), entity_index, &source_ref),
        bounty: excluded_bounty(vec![BountyExclusionReason::VehicleVictim]),
        legacy_counter_effects: vec![legacy_effect(
            killer.source_entity_id,
            "vehicleKills",
            1,
            None,
        )],
    })
}

fn build_friendly_vehicle_destroyed_event(
    observation: &KilledEventObservation,
    killer: &ObservedEntity,
    victim: &ObservedEntity,
    weapon: Option<&str>,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
) -> Option<NormalizedEvent> {
    let source_ref = event_source_ref(context, observation, TEAMKILL_RULE_ID);
    let killer_actor = actor_ref(killer);
    let victim_actor = actor_ref(victim);

    build_event(CombatEventBuild {
        observation,
        source_ref: source_ref.clone(),
        rule_id: TEAMKILL_RULE_ID,
        kind: NormalizedEventKind::VehicleKilled,
        semantic: CombatSemantic::Teamkill,
        killer: FieldPresence::Present {
            value: killer_actor.clone(),
            source: Some(source_ref.clone()),
        },
        victim: FieldPresence::Present {
            value: victim_actor.clone(),
            source: Some(source_ref.clone()),
        },
        actors: vec![killer_actor, victim_actor],
        victim_kind: victim_kind(victim),
        weapon,
        distance_meters: observation.distance_meters,
        vehicle_context: vehicle_context(weapon, Some(victim), entity_index, &source_ref),
        bounty: excluded_bounty(vec![
            BountyExclusionReason::Teamkill,
            BountyExclusionReason::VehicleVictim,
        ]),
        legacy_counter_effects: Vec::new(),
    })
}

fn build_unknown_event(
    observation: &KilledEventObservation,
    killer: Option<&ObservedEntity>,
    victim: Option<&ObservedEntity>,
    weapon: Option<&str>,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    context: &SourceContext,
) -> Option<NormalizedEvent> {
    let source_ref = event_source_ref(context, observation, UNKNOWN_RULE_ID);
    let mut actors = Vec::new();
    let killer_presence = killer.map_or_else(
        || FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(source_ref.clone()),
        },
        |entity| {
            let actor = actor_ref(entity);
            actors.push(actor.clone());
            FieldPresence::Present { value: actor, source: Some(source_ref.clone()) }
        },
    );
    let victim_presence = victim.map_or_else(
        || FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(source_ref.clone()),
        },
        |entity| {
            let actor = actor_ref(entity);
            actors.push(actor.clone());
            FieldPresence::Present { value: actor, source: Some(source_ref.clone()) }
        },
    );

    build_event(CombatEventBuild {
        observation,
        source_ref: source_ref.clone(),
        rule_id: UNKNOWN_RULE_ID,
        kind: NormalizedEventKind::Unknown,
        semantic: CombatSemantic::Unknown,
        killer: killer_presence,
        victim: victim_presence,
        actors,
        victim_kind: victim.map_or(CombatVictimKind::Unknown, victim_kind),
        weapon,
        distance_meters: observation.distance_meters,
        vehicle_context: vehicle_context(weapon, victim, entity_index, &source_ref),
        bounty: excluded_bounty(vec![BountyExclusionReason::UnknownActor]),
        legacy_counter_effects: Vec::new(),
    })
}

struct CombatEventBuild<'a> {
    observation: &'a KilledEventObservation,
    source_ref: SourceRef,
    rule_id: &'static str,
    kind: NormalizedEventKind,
    semantic: CombatSemantic,
    killer: FieldPresence<EventActorRef>,
    victim: FieldPresence<EventActorRef>,
    actors: Vec<EventActorRef>,
    victim_kind: CombatVictimKind,
    weapon: Option<&'a str>,
    distance_meters: Option<f64>,
    vehicle_context: VehicleContext,
    bounty: BountyEligibility,
    legacy_counter_effects: Vec<LegacyCounterEffect>,
}

fn build_event(spec: CombatEventBuild<'_>) -> Option<NormalizedEvent> {
    let source_refs = SourceRefs::new(vec![spec.source_ref.clone()]).ok()?;
    let rule_id = RuleId::new(spec.rule_id).ok()?;

    Some(NormalizedEvent {
        event_id: format!("event.killed.{}", spec.observation.event_index),
        kind: spec.kind,
        frame: frame_presence(spec.observation, &spec.source_ref),
        event_index: event_index_presence(spec.observation, &spec.source_ref),
        actors: spec.actors,
        source_refs,
        rule_id,
        combat: Some(CombatEventAttributes {
            semantic: spec.semantic,
            killer: spec.killer,
            victim: spec.victim,
            victim_kind: spec.victim_kind,
            weapon: string_presence(spec.weapon, &spec.source_ref),
            distance_meters: distance_presence(spec.distance_meters, &spec.source_ref),
            vehicle_context: spec.vehicle_context,
            bounty: spec.bounty,
            legacy_counter_effects: spec.legacy_counter_effects,
        }),
        attributes: BTreeMap::new(),
    })
}

fn event_source_ref(
    context: &SourceContext,
    observation: &KilledEventObservation,
    rule_id: &str,
) -> SourceRef {
    context.event_source_ref(
        &observation.json_path,
        observation.frame,
        u64::try_from(observation.event_index).ok(),
        observation.killed_entity_id,
        RuleId::new(rule_id).ok(),
    )
}

fn frame_presence(
    observation: &KilledEventObservation,
    source_ref: &SourceRef,
) -> FieldPresence<u64> {
    observation.frame.map_or_else(
        || FieldPresence::Unknown {
            reason: UnknownReason::SchemaDrift,
            source: Some(source_ref.clone()),
        },
        |value| FieldPresence::Present { value, source: Some(source_ref.clone()) },
    )
}

fn event_index_presence(
    observation: &KilledEventObservation,
    source_ref: &SourceRef,
) -> FieldPresence<u64> {
    u64::try_from(observation.event_index).map_or_else(
        |_| FieldPresence::Unknown {
            reason: UnknownReason::SchemaDrift,
            source: Some(source_ref.clone()),
        },
        |value| FieldPresence::Present { value, source: Some(source_ref.clone()) },
    )
}

fn actor_ref(entity: &ObservedEntity) -> EventActorRef {
    EventActorRef {
        source_entity_id: FieldPresence::Present {
            value: entity.source_entity_id,
            source: entity.source_refs.as_slice().first().cloned(),
        },
        observed_name: entity.observed_name.clone(),
        side: entity.identity.side.clone(),
    }
}

const fn is_vehicle_or_static(entity: &ObservedEntity) -> bool {
    matches!(entity.kind, EntityKind::Vehicle | EntityKind::StaticWeapon)
}

fn victim_entity<'a>(
    observation: &KilledEventObservation,
    entity_index: &'a BTreeMap<i64, &ObservedEntity>,
) -> Option<&'a ObservedEntity> {
    observation.killed_entity_id.and_then(|entity_id| entity_index.get(&entity_id).copied())
}

fn same_present_side(killer: &ObservedEntity, victim: &ObservedEntity) -> Option<bool> {
    let killer_side = present_side(&killer.identity.side)?;
    let victim_side = present_side(&victim.identity.side)?;

    Some(killer_side == victim_side)
}

const fn present_side(side: &FieldPresence<EntitySide>) -> Option<EntitySide> {
    match side {
        FieldPresence::Present { value, source: _ } => match value {
            EntitySide::East | EntitySide::West | EntitySide::Guer | EntitySide::Civ => {
                Some(*value)
            }
            EntitySide::Unknown => None,
        },
        FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

const fn victim_kind(entity: &ObservedEntity) -> CombatVictimKind {
    match entity.kind {
        EntityKind::Unit => CombatVictimKind::Player,
        EntityKind::Vehicle => CombatVictimKind::Vehicle,
        EntityKind::StaticWeapon => CombatVictimKind::StaticWeapon,
        EntityKind::Unknown => CombatVictimKind::Unknown,
    }
}

fn vehicle_context(
    weapon: Option<&str>,
    victim: Option<&ObservedEntity>,
    entity_index: &BTreeMap<i64, &ObservedEntity>,
    source_ref: &SourceRef,
) -> VehicleContext {
    let attacker_vehicle = weapon.and_then(|weapon| vehicle_by_observed_name(entity_index, weapon));

    VehicleContext {
        is_kill_from_vehicle: attacker_vehicle.is_some(),
        raw_weapon: string_presence(weapon, source_ref),
        attacker_vehicle_entity_id: attacker_vehicle.map_or_else(
            || FieldPresence::NotApplicable {
                reason: "source weapon did not match a vehicle entity".to_owned(),
            },
            |entity| FieldPresence::Present {
                value: entity.source_entity_id,
                source: entity.source_refs.as_slice().first().cloned(),
            },
        ),
        attacker_vehicle_name: attacker_vehicle.map_or_else(
            || FieldPresence::NotApplicable {
                reason: "source weapon did not match a vehicle entity".to_owned(),
            },
            |entity| entity.observed_name.clone(),
        ),
        attacker_vehicle_class: attacker_vehicle.map_or_else(
            || FieldPresence::NotApplicable {
                reason: "source weapon did not match a vehicle entity".to_owned(),
            },
            |entity| entity.observed_class.clone(),
        ),
        attacker_vehicle_category: attacker_vehicle.map_or_else(
            || FieldPresence::NotApplicable {
                reason: "source weapon did not match a vehicle entity".to_owned(),
            },
            |entity| FieldPresence::Present {
                value: vehicle_score_category(entity),
                source: entity.source_refs.as_slice().first().cloned(),
            },
        ),
        target_category: victim.map_or_else(
            || FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: Some(source_ref.clone()),
            },
            |entity| FieldPresence::Present {
                value: vehicle_score_category(entity),
                source: entity.source_refs.as_slice().first().cloned(),
            },
        ),
    }
}

fn vehicle_by_observed_name<'a>(
    entity_index: &'a BTreeMap<i64, &ObservedEntity>,
    weapon: &str,
) -> Option<&'a ObservedEntity> {
    entity_index.values().copied().find(|entity| {
        is_vehicle_or_static(entity) && observed_string(&entity.observed_name) == Some(weapon)
    })
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

fn vehicle_score_category(entity: &ObservedEntity) -> VehicleScoreCategory {
    match entity.kind {
        EntityKind::Unit => VehicleScoreCategory::Player,
        EntityKind::StaticWeapon => VehicleScoreCategory::StaticWeapon,
        EntityKind::Vehicle | EntityKind::Unknown => {
            category_from_vehicle_class(observed_string(&entity.observed_class))
        }
    }
}

fn string_presence(value: Option<&str>, source_ref: &SourceRef) -> FieldPresence<String> {
    value.map_or_else(
        || FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(source_ref.clone()),
        },
        |value| FieldPresence::Present {
            value: value.to_owned(),
            source: Some(source_ref.clone()),
        },
    )
}

fn distance_presence(value: Option<f64>, source_ref: &SourceRef) -> FieldPresence<f64> {
    value.map_or_else(
        || FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: Some(source_ref.clone()),
        },
        |value| FieldPresence::Present { value, source: Some(source_ref.clone()) },
    )
}

fn excluded_bounty(exclusion_reasons: Vec<BountyExclusionReason>) -> BountyEligibility {
    BountyEligibility { state: BountyEligibilityState::Excluded, exclusion_reasons }
}

fn legacy_effect(
    player_entity_id: i64,
    field: &str,
    delta: i64,
    relationship_target_entity_id: Option<i64>,
) -> LegacyCounterEffect {
    LegacyCounterEffect {
        player_entity_id,
        field: field.to_owned(),
        delta,
        relationship_target_entity_id,
    }
}

#[derive(Clone, Copy)]
struct UnknownDiagnostic<'a> {
    code: &'static str,
    message: &'static str,
    expected_shape: &'static str,
    observed_shape: &'a str,
    parser_action: &'static str,
}

fn push_actor_unknown_diagnostic(
    diagnostics: &mut DiagnosticAccumulator,
    message: &'static str,
    observation: &KilledEventObservation,
    context: &SourceContext,
) {
    push_unknown_diagnostic(
        diagnostics,
        UnknownDiagnostic {
            code: "event.killed_actor_unknown",
            message,
            expected_shape: "known player killer and known killed actor",
            observed_shape: "missing_or_unclassifiable_actor",
            parser_action: "emit_unknown_combat_event",
        },
        observation,
        context,
    );
}

fn push_unknown_diagnostic(
    diagnostics: &mut DiagnosticAccumulator,
    diagnostic: UnknownDiagnostic<'_>,
    observation: &KilledEventObservation,
    context: &SourceContext,
) {
    let source_ref = event_source_ref(context, observation, UNKNOWN_RULE_ID);
    let Some(source_refs) = SourceRefs::new(vec![source_ref]).ok() else {
        return;
    };

    diagnostics.push(
        Diagnostic {
            code: diagnostic.code.to_owned(),
            severity: DiagnosticSeverity::Warning,
            message: diagnostic.message.to_owned(),
            json_path: Some(observation.json_path.clone()),
            expected_shape: Some(diagnostic.expected_shape.to_owned()),
            observed_shape: Some(diagnostic.observed_shape.to_owned()),
            parser_action: diagnostic.parser_action.to_owned(),
            source_refs,
        },
        DiagnosticImpact::DataLoss,
    );
}
