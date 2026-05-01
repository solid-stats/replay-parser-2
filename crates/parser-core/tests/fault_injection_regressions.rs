//! Deterministic fault-injection regression tests for high-risk parser behavior.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    failure::ParseStage,
    minimal::{DestroyedVehicleClassification, KillClassification},
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::json;

const AGGREGATE_FIXTURE: &[u8] = include_bytes!("fixtures/aggregate-combat.ocap.json");
const COMBAT_EVENTS_FIXTURE: &[u8] = include_bytes!("fixtures/combat-events.ocap.json");
const INVALID_JSON_FIXTURE: &[u8] = include_bytes!("fixtures/invalid-json.ocap.json");
const VEHICLE_CONTEXT_FIXTURE: &[u8] = include_bytes!("fixtures/vehicle-score.ocap.json");

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

#[test]
fn fault_injection_regressions_should_catch_same_side_kills_counted_as_enemy_kills() {
    let artifact = parse_fixture(COMBAT_EVENTS_FIXTURE, "fixtures/combat-events.ocap.json");
    let teamkill = artifact
        .kills
        .iter()
        .find(|row| row.classification == KillClassification::Teamkill)
        .expect("same-side teamkill row should exist");

    assert_eq!(teamkill.killer_source_entity_id, Some(1));
    assert_eq!(teamkill.victim_source_entity_id, Some(3));
    assert!(!teamkill.bounty_eligible);
    assert_eq!(teamkill.bounty_exclusion_reasons, vec!["teamkill"]);
}

#[test]
fn fault_injection_regressions_should_catch_null_killer_deaths_producing_bounty_awards() {
    let artifact = parse_fixture(COMBAT_EVENTS_FIXTURE, "fixtures/combat-events.ocap.json");
    let null_killer = artifact
        .kills
        .iter()
        .find(|row| row.classification == KillClassification::NullKiller)
        .expect("null-killer row should exist");

    assert_eq!(null_killer.victim_source_entity_id, Some(5));
    assert!(!null_killer.bounty_eligible);
    assert_eq!(null_killer.bounty_exclusion_reasons, vec!["null_killer"]);
}

#[test]
fn fault_injection_regressions_should_catch_vehicle_context_loss_in_minimal_rows() {
    let artifact = parse_fixture(VEHICLE_CONTEXT_FIXTURE, "fixtures/vehicle-score.ocap.json");
    let vehicle_kill = artifact
        .kills
        .iter()
        .find(|row| row.classification == KillClassification::EnemyKill)
        .expect("enemy kill from vehicle should exist");
    let stats = artifact
        .player_stats
        .iter()
        .find(|row| row.source_entity_id == 1)
        .expect("attacker stats should exist");

    assert_eq!(vehicle_kill.attacker_vehicle_name.as_deref(), Some("T-72B"));
    assert_eq!(vehicle_kill.attacker_vehicle_class.as_deref(), Some("rhs_t72ba_tv"));
    assert!(stats.kills_from_vehicle > 0);
}

#[test]
fn fault_injection_regressions_should_catch_destroyed_vehicle_rows_dropped_from_default_output() {
    let artifact = parse_fixture(AGGREGATE_FIXTURE, "fixtures/aggregate-combat.ocap.json");
    let destroyed =
        artifact.destroyed_vehicles.first().expect("destroyed vehicle row should exist");

    assert_eq!(destroyed.classification, DestroyedVehicleClassification::Enemy);
    assert_eq!(destroyed.attacker_source_entity_id, Some(1));
    assert_eq!(destroyed.destroyed_entity_id, Some(20));
}

#[test]
fn fault_injection_regressions_should_catch_invalid_json_returning_success_or_partial() {
    let artifact = parse_fixture(INVALID_JSON_FIXTURE, "fixtures/invalid-json.ocap.json");
    let failure = artifact.failure.as_ref().expect("invalid JSON should include failure");

    assert_eq!(artifact.status, ParseStatus::Failed);
    assert_ne!(artifact.status, ParseStatus::Success);
    assert_ne!(artifact.status, ParseStatus::Partial);
    assert_eq!(failure.stage, ParseStage::JsonDecode);
    assert_eq!(failure.error_code.as_str(), "json.decode");
}
