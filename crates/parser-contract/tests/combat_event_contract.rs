//! Combat event payload contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::collections::BTreeMap;

use parser_contract::{
    events::{
        BountyEligibility, BountyEligibilityState, CombatEventAttributes, CombatSemantic,
        CombatVictimKind, EventActorRef, LegacyCounterEffect, NormalizedEvent, NormalizedEventKind,
        VehicleContext, VehicleScoreCategory,
    },
    identity::EntitySide,
    presence::{FieldPresence, UnknownReason},
    source_ref::{RuleId, SourceChecksum, SourceRef, SourceRefs},
};

const fn present<T>(value: T) -> FieldPresence<T> {
    FieldPresence::Present { value, source: None }
}

const fn unknown<T>(reason: UnknownReason) -> FieldPresence<T> {
    FieldPresence::Unknown { reason, source: None }
}

fn checksum() -> SourceChecksum {
    SourceChecksum::sha256("0000000000000000000000000000000000000000000000000000000000000000")
        .expect("test checksum should be valid")
}

fn source_ref(rule_id: &str) -> SourceRef {
    SourceRef {
        replay_id: Some("replay-0001".to_string()),
        source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
        checksum: Some(checksum()),
        frame: Some(12_345),
        event_index: Some(7),
        entity_id: Some(101),
        json_path: Some("$.events[7]".to_string()),
        rule_id: Some(RuleId::new(rule_id).expect("test source rule ID should be valid")),
    }
}

fn actor(source_entity_id: i64, observed_name: &str, side: EntitySide) -> EventActorRef {
    EventActorRef {
        source_entity_id: present(source_entity_id),
        observed_name: present(observed_name.to_string()),
        side: present(side),
    }
}

fn enemy_kill_combat() -> CombatEventAttributes {
    CombatEventAttributes {
        semantic: CombatSemantic::EnemyKill,
        killer: present(actor(101, "Afganor", EntitySide::West)),
        victim: present(actor(202, "Target", EntitySide::East)),
        victim_kind: CombatVictimKind::Player,
        weapon: present("rhs_t72".to_string()),
        distance_meters: present(315.5),
        vehicle_context: VehicleContext {
            is_kill_from_vehicle: true,
            raw_weapon: present("rhs_t72".to_string()),
            attacker_vehicle_entity_id: present(501),
            attacker_vehicle_name: present("Tank 1".to_string()),
            attacker_vehicle_class: present("rhs_t72ba_tv".to_string()),
            attacker_vehicle_category: present(VehicleScoreCategory::Tank),
            target_category: present(VehicleScoreCategory::Player),
        },
        bounty: BountyEligibility {
            state: BountyEligibilityState::Eligible,
            exclusion_reasons: Vec::new(),
        },
        legacy_counter_effects: vec![
            LegacyCounterEffect {
                player_entity_id: 101,
                field: "kills".to_string(),
                delta: 1,
                relationship_target_entity_id: Some(202),
            },
            LegacyCounterEffect {
                player_entity_id: 202,
                field: "deaths.total".to_string(),
                delta: 1,
                relationship_target_entity_id: Some(101),
            },
        ],
    }
}

#[test]
fn combat_event_contract_should_serialize_enemy_kill_combat_payload_when_event_has_vehicle_context()
{
    let event = NormalizedEvent {
        event_id: "event-0007".to_string(),
        kind: NormalizedEventKind::Kill,
        frame: present(12_345),
        event_index: present(7),
        actors: vec![
            actor(101, "Afganor", EntitySide::West),
            actor(202, "Target", EntitySide::East),
        ],
        source_refs: SourceRefs::new(vec![source_ref("event.kill.source")])
            .expect("source refs should be non-empty"),
        rule_id: RuleId::new("event.kill.combat").expect("test rule ID should be valid"),
        combat: Some(enemy_kill_combat()),
        attributes: BTreeMap::new(),
    };

    let serialized = serde_json::to_value(&event).expect("combat event should serialize");
    let serialized_text =
        serde_json::to_string(&serialized).expect("serialized combat event should stringify");

    assert!(serialized.get("combat").is_some());
    assert_eq!(serialized["kind"], "kill");
    assert_eq!(serialized["combat"]["semantic"], "enemy_kill");
    assert_eq!(serialized["combat"]["bounty"]["state"], "eligible");
    assert_eq!(serialized["combat"]["vehicle_context"]["is_kill_from_vehicle"], true);
    assert_eq!(
        serialized["combat"]["vehicle_context"]["attacker_vehicle_category"]["value"],
        "tank"
    );
    assert_eq!(serialized["combat"]["legacy_counter_effects"][0]["field"], "kills");
    for expected_fragment in [
        "\"combat\"",
        "\"enemy_kill\"",
        "\"eligible\"",
        "\"is_kill_from_vehicle\"",
        "\"attacker_vehicle_category\"",
        "\"legacy_counter_effects\"",
    ] {
        assert!(
            serialized_text.contains(expected_fragment),
            "combat event JSON should contain {expected_fragment}"
        );
    }
}

#[test]
fn combat_event_contract_should_serialize_kill_kind_as_snake_case_when_kind_is_kill() {
    let serialized =
        serde_json::to_value(NormalizedEventKind::Kill).expect("event kind should serialize");

    assert_eq!(serialized, "kill");
}

#[test]
fn combat_event_contract_should_preserve_unknown_vehicle_score_category_when_vehicle_evidence_is_missing()
 {
    let combat = CombatEventAttributes {
        semantic: CombatSemantic::Unknown,
        killer: unknown(UnknownReason::SourceFieldAbsent),
        victim: unknown(UnknownReason::SourceFieldAbsent),
        victim_kind: CombatVictimKind::Unknown,
        weapon: unknown(UnknownReason::SourceFieldAbsent),
        distance_meters: unknown(UnknownReason::SourceFieldAbsent),
        vehicle_context: VehicleContext {
            is_kill_from_vehicle: false,
            raw_weapon: unknown(UnknownReason::SourceFieldAbsent),
            attacker_vehicle_entity_id: unknown(UnknownReason::SourceFieldAbsent),
            attacker_vehicle_name: unknown(UnknownReason::SourceFieldAbsent),
            attacker_vehicle_class: unknown(UnknownReason::SourceFieldAbsent),
            attacker_vehicle_category: present(VehicleScoreCategory::Unknown),
            target_category: present(VehicleScoreCategory::Unknown),
        },
        bounty: BountyEligibility {
            state: BountyEligibilityState::Excluded,
            exclusion_reasons: Vec::new(),
        },
        legacy_counter_effects: Vec::new(),
    };

    let serialized = serde_json::to_value(combat).expect("combat payload should serialize");

    assert_eq!(serialized["vehicle_context"]["attacker_vehicle_category"]["value"], "unknown");
    assert_eq!(serialized["vehicle_context"]["target_category"]["value"], "unknown");
}
