//! Parser-core vehicle score behavior tests.

#![allow(
    clippy::expect_used,
    clippy::float_cmp,
    clippy::panic,
    reason = "integration tests use expect messages as assertion context"
)]

use std::collections::BTreeSet;

use parser_contract::{
    aggregates::{AggregateContributionKind, VehicleScoreInputValue},
    artifact::{ParseArtifact, ParseStatus},
    events::VehicleScoreCategory,
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{
    ParserInput, ParserOptions, parse_replay,
    vehicle_score::{category_from_vehicle_class, teamkill_penalty_weight, vehicle_score_weight},
};
use serde_json::{Value, json};

const VEHICLE_SCORE_FIXTURE: &[u8] = include_bytes!("fixtures/vehicle-score.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-vehicle-score".to_owned()),
        source_file: "fixtures/vehicle-score.ocap.json".to_owned(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "4444444444444444444444444444444444444444444444444444444444444444",
            )
            .expect("test checksum should be valid"),
            source: None,
        },
    }
}

fn parser_input(bytes: &[u8]) -> ParserInput<'_> {
    ParserInput {
        bytes,
        source: replay_source(),
        parser: parser_info(),
        options: ParserOptions::default(),
    }
}

fn vehicle_score_artifact() -> ParseArtifact {
    parse_replay(parser_input(VEHICLE_SCORE_FIXTURE))
}

fn projection<'a>(artifact: &'a ParseArtifact, key: &str) -> &'a Value {
    artifact
        .summaries
        .projections
        .get(key)
        .unwrap_or_else(|| panic!("projection {key} should exist"))
}

fn projection_array<'a>(artifact: &'a ParseArtifact, key: &str) -> &'a Vec<Value> {
    projection(artifact, key)
        .as_array()
        .unwrap_or_else(|| panic!("projection {key} should be an array"))
}

fn vehicle_score_input_row<'a>(artifact: &'a ParseArtifact, event_id: &str) -> &'a Value {
    projection_array(artifact, "vehicle_score.inputs")
        .iter()
        .find(|row| row["event_id"] == event_id)
        .unwrap_or_else(|| panic!("vehicle score row for {event_id} should exist"))
}

fn vehicle_score_contribution_ids(artifact: &ParseArtifact) -> BTreeSet<String> {
    artifact
        .facts
        .aggregate_contributions
        .iter()
        .filter(|contribution| contribution.kind == AggregateContributionKind::VehicleScoreInput)
        .map(|contribution| contribution.contribution_id.clone())
        .collect()
}

#[test]
fn vehicle_score_should_map_issue_13_categories_from_raw_vehicle_class() {
    // Arrange
    let cases = [
        (Some("static-weapon"), VehicleScoreCategory::StaticWeapon),
        (Some("mortar"), VehicleScoreCategory::StaticWeapon),
        (Some("rhs_btr80_msv"), VehicleScoreCategory::Apc),
        (Some("rhs_t72ba_tv"), VehicleScoreCategory::Tank),
        (Some("mi24"), VehicleScoreCategory::Heli),
        (Some("su25"), VehicleScoreCategory::Plane),
        (Some("ural"), VehicleScoreCategory::Truck),
        (Some("CUP_B_UAZ_MG_CDF"), VehicleScoreCategory::Car),
        (Some("   "), VehicleScoreCategory::Unknown),
        (Some("sea"), VehicleScoreCategory::Unknown),
        (None, VehicleScoreCategory::Unknown),
    ];

    // Act + Assert
    for (raw_class, expected_category) in cases {
        assert_eq!(category_from_vehicle_class(raw_class), expected_category);
    }
}

#[test]
fn vehicle_score_should_return_issue_13_matrix_weights() {
    // Arrange
    use VehicleScoreCategory::{Apc, Car, Heli, Plane, Player, StaticWeapon, Tank, Truck, Unknown};

    let cases = [
        (Unknown, Player, None),
        (StaticWeapon, StaticWeapon, Some(1.0)),
        (StaticWeapon, Tank, Some(1.5)),
        (StaticWeapon, Player, Some(2.0)),
        (Car, Apc, Some(1.0)),
        (Truck, Plane, Some(2.0)),
        (Apc, StaticWeapon, Some(0.5)),
        (Apc, Tank, Some(1.0)),
        (Apc, Heli, Some(2.0)),
        (Tank, StaticWeapon, Some(0.25)),
        (Tank, Car, Some(0.5)),
        (Tank, Tank, Some(1.0)),
        (Tank, Heli, Some(1.5)),
        (Tank, Player, Some(2.0)),
        (Heli, StaticWeapon, Some(0.5)),
        (Heli, Truck, Some(1.0)),
        (Heli, Tank, Some(1.5)),
        (Heli, Player, Some(2.0)),
        (Plane, StaticWeapon, Some(0.25)),
        (Plane, Apc, Some(0.5)),
        (Plane, Tank, Some(1.0)),
        (Plane, Heli, Some(1.5)),
        (Plane, Player, Some(2.0)),
        (Player, Tank, None),
    ];

    // Act + Assert
    for (attacker, target, expected_weight) in cases {
        assert_eq!(vehicle_score_weight(attacker, target), expected_weight);
    }
}

#[test]
fn vehicle_score_should_clamp_teamkill_penalty_weight_below_one() {
    // Arrange
    let raw_weight = 0.25;

    // Act
    let applied_weight = teamkill_penalty_weight(raw_weight);

    // Assert
    assert_eq!(applied_weight, 1.0);
}

#[test]
fn vehicle_score_should_emit_award_contributions_for_kills_from_vehicle() {
    // Arrange
    let artifact = vehicle_score_artifact();

    // Act
    let inputs = projection_array(&artifact, "vehicle_score.inputs");
    let player_kill = vehicle_score_input_row(&artifact, "event.killed.0");
    let static_weapon_kill = vehicle_score_input_row(&artifact, "event.killed.1");

    // Assert
    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(inputs.len(), 5);
    assert_eq!(player_kill["sign"], "award");
    assert_eq!(player_kill["attacker_category"], "tank");
    assert_eq!(player_kill["target_category"], "player");
    assert_eq!(player_kill["matrix_weight"].as_f64(), Some(2.0));
    assert_eq!(player_kill["applied_weight"].as_f64(), Some(2.0));
    assert_eq!(player_kill["raw_attacker_vehicle_class"], "rhs_t72ba_tv");
    assert_eq!(static_weapon_kill["sign"], "award");
    assert_eq!(static_weapon_kill["target_category"], "static_weapon");
    assert_eq!(static_weapon_kill["raw_target_class"], "static-weapon");
    assert_eq!(static_weapon_kill["matrix_weight"].as_f64(), Some(0.25));
}

#[test]
fn vehicle_score_should_emit_clamped_penalty_for_friendly_vehicle_or_static_destruction() {
    // Arrange
    let artifact = vehicle_score_artifact();

    // Act
    let friendly_static = vehicle_score_input_row(&artifact, "event.killed.4");

    // Assert
    assert_eq!(friendly_static["sign"], "penalty");
    assert_eq!(friendly_static["rule_id"], "aggregate.vehicle_score.penalty");
    assert_eq!(friendly_static["attacker_category"], "tank");
    assert_eq!(friendly_static["target_category"], "static_weapon");
    assert_eq!(friendly_static["matrix_weight"].as_f64(), Some(0.25));
    assert_eq!(friendly_static["applied_weight"].as_f64(), Some(1.0));
    assert_eq!(friendly_static["teamkill_penalty_clamped"], true);
    assert_eq!(friendly_static["denominator_eligible"], false);
}

#[test]
fn vehicle_score_should_emit_penalty_contributions_for_teamkills_from_vehicle() {
    // Arrange
    let artifact = vehicle_score_artifact();

    // Act
    let teamkill = vehicle_score_input_row(&artifact, "event.killed.3");

    // Assert
    assert_eq!(teamkill["source_contribution_id"], "aggregate.vehicle_score.event.killed.3");
    assert_eq!(teamkill["sign"], "penalty");
    assert_eq!(teamkill["rule_id"], "aggregate.vehicle_score.penalty");
    assert_eq!(teamkill["attacker_category"], "tank");
    assert_eq!(teamkill["target_category"], "player");
    assert_eq!(teamkill["matrix_weight"].as_f64(), Some(2.0));
    assert_eq!(teamkill["applied_weight"].as_f64(), Some(2.0));
    assert_eq!(teamkill["teamkill_penalty_clamped"], false);
    assert_eq!(teamkill["denominator_eligible"], false);
}

#[test]
fn vehicle_score_should_emit_denominator_input_only_for_players_with_vehicle_kill_awards() {
    // Arrange
    let artifact = vehicle_score_artifact();

    // Act
    let denominator_inputs = projection_array(&artifact, "vehicle_score.denominator_inputs");
    let source_contribution_ids = denominator_inputs[0]["source_contribution_ids"]
        .as_array()
        .expect("denominator row should carry source contribution IDs")
        .iter()
        .map(|value| value.as_str().expect("contribution ID should be a string"))
        .collect::<BTreeSet<_>>();

    // Assert
    assert_eq!(denominator_inputs.len(), 1);
    assert_eq!(denominator_inputs[0]["compatibility_key"], "entity:1");
    assert_eq!(denominator_inputs[0]["observed_entity_ids"], json!([1]));
    assert_eq!(denominator_inputs[0]["has_vehicle_kill"], true);
    assert!(source_contribution_ids.contains("aggregate.vehicle_score.event.killed.0"));
    assert!(source_contribution_ids.contains("aggregate.vehicle_score.event.killed.1"));
    assert!(source_contribution_ids.contains("aggregate.vehicle_score.event.killed.2"));
    assert!(!source_contribution_ids.contains("aggregate.vehicle_score.event.killed.3"));
    assert!(!source_contribution_ids.contains("aggregate.vehicle_score.event.killed.4"));
}

#[test]
fn vehicle_score_should_emit_inputs_without_final_cross_replay_score() {
    // Arrange
    let artifact = vehicle_score_artifact();
    let inputs = projection_array(&artifact, "vehicle_score.inputs");

    // Assert
    assert!(!artifact.summaries.projections.contains_key("vehicle_score.score"));
    assert!(!artifact.summaries.projections.contains_key("vehicle_score.final_score"));
    assert!(inputs.iter().all(|row| row.get("score").is_none()));
    assert!(inputs.iter().all(|row| row.get("final_score").is_none()));
}

#[test]
fn vehicle_score_should_include_source_refs_on_every_vehicle_score_contribution() {
    // Arrange
    let artifact = vehicle_score_artifact();
    let projection_contribution_ids = projection_array(&artifact, "vehicle_score.inputs")
        .iter()
        .map(|row| {
            row["source_contribution_id"]
                .as_str()
                .expect("vehicle score row should carry source contribution ID")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();

    // Act
    let contribution_ids = vehicle_score_contribution_ids(&artifact);

    // Assert
    assert_eq!(contribution_ids, projection_contribution_ids);
    for contribution in
        artifact.facts.aggregate_contributions.iter().filter(|contribution| {
            contribution.kind == AggregateContributionKind::VehicleScoreInput
        })
    {
        let value = serde_json::from_value::<VehicleScoreInputValue>(contribution.value.clone())
            .expect("vehicle score contribution should use VehicleScoreInputValue");

        assert!(!contribution.source_refs.as_slice().is_empty());
        assert!(contribution.event_id.is_some());
        assert_eq!(
            contribution.contribution_id,
            format!("aggregate.vehicle_score.{}", value.event_id)
        );
        assert!(projection_contribution_ids.contains(&contribution.contribution_id));
    }
}

#[test]
fn vehicle_score_should_include_event_and_vehicle_entity_source_refs_for_category_evidence() {
    // Arrange
    let artifact = vehicle_score_artifact();
    let contribution = artifact
        .facts
        .aggregate_contributions
        .iter()
        .find(|contribution| {
            contribution.contribution_id == "aggregate.vehicle_score.event.killed.0"
        })
        .expect("vehicle score contribution should exist");

    // Act
    let json_paths = contribution
        .source_refs
        .as_slice()
        .iter()
        .filter_map(|source_ref| source_ref.json_path.as_deref())
        .collect::<BTreeSet<_>>();

    // Assert
    assert!(json_paths.contains("$.events[0]"));
    assert!(json_paths.contains("$.entities[4]"));
    assert!(json_paths.contains("$.entities[4].class"));
    assert!(json_paths.contains("$.entities[1]"));
}
