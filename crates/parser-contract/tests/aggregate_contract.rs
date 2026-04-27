//! Aggregate contribution value contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    aggregates::{
        AggregateContributionKind, AggregateContributionRef, VehicleScoreInputValue,
        VehicleScoreSign,
    },
    events::VehicleScoreCategory,
    source_ref::{RuleId, SourceChecksum, SourceRef, SourceRefs},
};

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

fn vehicle_score_input_value() -> VehicleScoreInputValue {
    VehicleScoreInputValue {
        player_entity_id: 101,
        event_id: "event-0007".to_string(),
        sign: VehicleScoreSign::Penalty,
        attacker_category: VehicleScoreCategory::Tank,
        target_category: VehicleScoreCategory::StaticWeapon,
        raw_attacker_vehicle_name: Some("Tank 1".to_string()),
        raw_attacker_vehicle_class: Some("rhs_t72ba_tv".to_string()),
        raw_target_class: Some("rhsgref_ins_d30".to_string()),
        matrix_weight: 0.25,
        applied_weight: 1.0,
        teamkill_penalty_clamped: true,
        denominator_eligible: false,
    }
}

#[test]
fn aggregate_contract_should_serialize_vehicle_score_input_evidence_when_penalty_is_clamped() {
    let value = vehicle_score_input_value();

    let serialized =
        serde_json::to_value(value).expect("vehicle score input value should serialize");
    let serialized_text =
        serde_json::to_string(&serialized).expect("vehicle score input JSON should stringify");

    assert_eq!(serialized["sign"], "penalty");
    assert_eq!(serialized["attacker_category"], "tank");
    assert_eq!(serialized["target_category"], "static_weapon");
    assert_eq!(serialized["matrix_weight"], 0.25);
    assert_eq!(serialized["applied_weight"], 1.0);
    assert_eq!(serialized["teamkill_penalty_clamped"], true);
    for expected_fragment in [
        "\"penalty\"",
        "\"tank\"",
        "\"static_weapon\"",
        "\"matrix_weight\":0.25",
        "\"applied_weight\":1.0",
        "\"teamkill_penalty_clamped\":true",
    ] {
        assert!(
            serialized_text.contains(expected_fragment),
            "vehicle score input JSON should contain {expected_fragment}"
        );
    }
}

#[test]
fn aggregate_contract_should_wrap_vehicle_score_input_value_in_source_backed_contribution_ref() {
    let value = serde_json::to_value(vehicle_score_input_value())
        .expect("vehicle score input value should serialize");
    let contribution = AggregateContributionRef {
        contribution_id: "contribution-vehicle-score-0007".to_string(),
        kind: AggregateContributionKind::VehicleScoreInput,
        event_id: Some("event-0007".to_string()),
        source_refs: SourceRefs::new(vec![source_ref("aggregate.vehicle_score.source")])
            .expect("source refs should be non-empty"),
        rule_id: RuleId::new("aggregate.vehicle_score.input")
            .expect("test contribution rule ID should be valid"),
        value,
    };

    let serialized = serde_json::to_value(contribution).expect("contribution should serialize");

    assert_eq!(serialized["kind"], "vehicle_score_input");
    assert_eq!(serialized["event_id"], "event-0007");
    assert_eq!(serialized["source_refs"][0]["rule_id"], "aggregate.vehicle_score.source");
    assert_eq!(serialized["rule_id"], "aggregate.vehicle_score.input");
    assert_eq!(serialized["value"]["teamkill_penalty_clamped"], true);
    assert_eq!(serialized["value"]["matrix_weight"], 0.25);
    assert_eq!(serialized["value"]["applied_weight"], 1.0);
}

#[test]
fn aggregate_contract_should_serialize_bounty_input_kind_as_snake_case_when_kind_is_bounty_input() {
    let serialized = serde_json::to_value(AggregateContributionKind::BountyInput)
        .expect("aggregate contribution kind should serialize");

    assert_eq!(serialized, "bounty_input");
}
