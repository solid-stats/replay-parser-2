//! Deterministic fault-injection regression tests for high-risk parser behavior.

#![allow(
    clippy::expect_used,
    clippy::float_cmp,
    reason = "integration tests use expect messages as assertion context"
)]

use std::collections::BTreeSet;

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    events::{BountyEligibilityState, BountyExclusionReason, CombatSemantic},
    failure::ParseStage,
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::{Value, json};

const AGGREGATE_FIXTURE: &[u8] = include_bytes!("fixtures/aggregate-combat.ocap.json");
const COMBAT_EVENTS_FIXTURE: &[u8] = include_bytes!("fixtures/combat-events.ocap.json");
const INVALID_JSON_FIXTURE: &[u8] = include_bytes!("fixtures/invalid-json.ocap.json");
const VEHICLE_SCORE_FIXTURE: &[u8] = include_bytes!("fixtures/vehicle-score.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source(source_file: &str) -> ReplaySource {
    ReplaySource {
        replay_id: Some("fault-injection-regression".to_owned()),
        source_file: source_file.to_owned(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "5555555555555555555555555555555555555555555555555555555555555555",
            )
            .expect("test checksum should be valid"),
            source: None,
        },
    }
}

fn parse_fixture(bytes: &[u8], source_file: &str) -> ParseArtifact {
    parse_replay(ParserInput {
        bytes,
        source: replay_source(source_file),
        parser: parser_info(),
        options: ParserOptions::default(),
    })
}

fn projection_array<'a>(artifact: &'a ParseArtifact, key: &str) -> &'a Vec<Value> {
    artifact
        .summaries
        .projections
        .get(key)
        .and_then(Value::as_array)
        .expect("projection should be an array")
}

fn vehicle_score_input_row<'a>(artifact: &'a ParseArtifact, event_id: &str) -> &'a Value {
    projection_array(artifact, "vehicle_score.inputs")
        .iter()
        .find(|row| row["event_id"] == event_id)
        .expect("vehicle score row should exist")
}

#[test]
fn fault_injection_regressions_should_catch_teamkill_penalty_clamp_using_raw_weight_below_one() {
    // Arrange
    // This fault class guards the teamkill penalty clamp when the matrix weight is below 1.
    let artifact = parse_fixture(VEHICLE_SCORE_FIXTURE, "fixtures/vehicle-score.ocap.json");

    // Act
    let friendly_static_teamkill = vehicle_score_input_row(&artifact, "event.killed.4");

    // Assert
    assert_eq!(friendly_static_teamkill["sign"], "penalty");
    assert_eq!(friendly_static_teamkill["matrix_weight"].as_f64(), Some(0.25));
    assert_eq!(friendly_static_teamkill["applied_weight"].as_f64(), Some(1.0));
    assert_eq!(friendly_static_teamkill["teamkill_penalty_clamped"], true);
}

#[test]
fn fault_injection_regressions_should_catch_vehicle_score_attacker_and_target_category_swaps() {
    // Arrange
    let artifact = parse_fixture(VEHICLE_SCORE_FIXTURE, "fixtures/vehicle-score.ocap.json");

    // Act
    let static_weapon_kill = vehicle_score_input_row(&artifact, "event.killed.1");

    // Assert
    assert_eq!(static_weapon_kill["sign"], "award");
    assert_eq!(static_weapon_kill["attacker_category"], "tank");
    assert_eq!(static_weapon_kill["target_category"], "static_weapon");
    assert_eq!(static_weapon_kill["matrix_weight"].as_f64(), Some(0.25));
    assert_eq!(static_weapon_kill["raw_attacker_vehicle_class"], "rhs_t72ba_tv");
    assert_eq!(static_weapon_kill["raw_target_class"], "static-weapon");
}

#[test]
fn fault_injection_regressions_should_catch_same_side_kills_counted_as_enemy_kills() {
    // Arrange
    let artifact = parse_fixture(COMBAT_EVENTS_FIXTURE, "fixtures/combat-events.ocap.json");

    // Act
    let teamkill_event = artifact
        .facts
        .combat
        .iter()
        .find(|event| event.fact_id == "event.killed.1")
        .expect("same-side event should exist");

    // Assert
    assert_eq!(teamkill_event.semantic, CombatSemantic::Teamkill);
    assert_eq!(teamkill_event.bounty.state, BountyEligibilityState::Excluded);
    assert!(teamkill_event.bounty.exclusion_reasons.contains(&BountyExclusionReason::Teamkill));
}

#[test]
fn fault_injection_regressions_should_catch_null_killer_deaths_producing_bounty_inputs() {
    // Arrange
    let artifact = parse_fixture(COMBAT_EVENTS_FIXTURE, "fixtures/combat-events.ocap.json");

    // Act
    let null_killer_event = artifact
        .facts
        .combat
        .iter()
        .find(|event| event.fact_id == "event.killed.3")
        .expect("null-killer event should exist");
    let bounty_event_ids = projection_array(&artifact, "bounty.inputs")
        .iter()
        .filter_map(|row| row["event_id"].as_str())
        .collect::<BTreeSet<_>>();

    // Assert
    assert_eq!(null_killer_event.semantic, CombatSemantic::NullKillerDeath);
    assert_eq!(null_killer_event.bounty.state, BountyEligibilityState::Excluded);
    assert!(
        null_killer_event.bounty.exclusion_reasons.contains(&BountyExclusionReason::NullKiller)
    );
    assert!(!bounty_event_ids.contains("event.killed.3"));
}

#[test]
fn fault_injection_regressions_should_catch_aggregate_contributions_without_source_refs() {
    // Arrange
    let artifact = parse_fixture(AGGREGATE_FIXTURE, "fixtures/aggregate-combat.ocap.json");

    // Act
    let every_contribution_has_source_refs = artifact
        .facts
        .aggregate_contributions
        .iter()
        .all(|contribution| !contribution.source_refs.as_slice().is_empty());

    // Assert
    assert!(every_contribution_has_source_refs, "aggregate contributions must keep source refs");
}

#[test]
fn fault_injection_regressions_should_catch_invalid_json_returning_success_or_partial() {
    // Arrange
    let artifact = parse_fixture(INVALID_JSON_FIXTURE, "fixtures/invalid-json.ocap.json");

    // Act
    let failure = artifact.failure.as_ref().expect("invalid JSON should include failure");

    // Assert
    assert_eq!(artifact.status, ParseStatus::Failed);
    assert_ne!(artifact.status, ParseStatus::Success);
    assert_ne!(artifact.status, ParseStatus::Partial);
    assert_eq!(failure.stage, ParseStage::JsonDecode);
    assert_eq!(failure.error_code.as_str(), "json.decode");
}
