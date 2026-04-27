//! Parser-core vehicle score behavior tests.

#![allow(
    clippy::expect_used,
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
        .aggregates
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
        .aggregates
        .contributions
        .iter()
        .filter(|contribution| contribution.kind == AggregateContributionKind::VehicleScoreInput)
        .map(|contribution| contribution.contribution_id.clone())
        .collect()
}

#[test]
fn vehicle_score_should_map_issue_13_categories_from_raw_vehicle_class() {
    // Arrange
    let raw_tank_class = Some("tank");
    let raw_static_weapon_class = Some("static-weapon");

    // Act
    let tank_category = category_from_vehicle_class(raw_tank_class);
    let static_weapon_category = category_from_vehicle_class(raw_static_weapon_class);
    let unknown_category = category_from_vehicle_class(Some("sea"));
    let absent_category = category_from_vehicle_class(None);

    // Assert
    assert_eq!(tank_category, VehicleScoreCategory::Tank);
    assert_eq!(static_weapon_category, VehicleScoreCategory::StaticWeapon);
    assert_eq!(unknown_category, VehicleScoreCategory::Unknown);
    assert_eq!(absent_category, VehicleScoreCategory::Unknown);
}

#[test]
fn vehicle_score_should_return_issue_13_matrix_weights() {
    // Arrange
    let attacker = VehicleScoreCategory::Tank;
    let static_weapon_target = VehicleScoreCategory::StaticWeapon;
    let player_target = VehicleScoreCategory::Player;

    // Act
    let static_weapon_weight = vehicle_score_weight(attacker, static_weapon_target);
    let player_weight = vehicle_score_weight(attacker, player_target);
    let unknown_weight = vehicle_score_weight(VehicleScoreCategory::Unknown, player_target);

    // Assert
    assert_eq!(static_weapon_weight, Some(0.25));
    assert_eq!(player_weight, Some(2.0));
    assert_eq!(unknown_weight, None);
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
    assert_eq!(inputs.len(), 4);
    assert_eq!(player_kill["sign"], "award");
    assert_eq!(player_kill["attacker_category"], "tank");
    assert_eq!(player_kill["target_category"], "player");
    assert_eq!(player_kill["matrix_weight"].as_f64(), Some(2.0));
    assert_eq!(player_kill["applied_weight"].as_f64(), Some(2.0));
    assert_eq!(player_kill["raw_attacker_vehicle_class"], "tank");
    assert_eq!(static_weapon_kill["sign"], "award");
    assert_eq!(static_weapon_kill["target_category"], "static_weapon");
    assert_eq!(static_weapon_kill["raw_target_class"], "static-weapon");
    assert_eq!(static_weapon_kill["matrix_weight"].as_f64(), Some(0.25));
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
    assert!(!source_contribution_ids.contains("aggregate.vehicle_score.event.killed.3"));
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
        artifact.aggregates.contributions.iter().filter(|contribution| {
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
