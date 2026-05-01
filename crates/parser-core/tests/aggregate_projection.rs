//! Parser-core minimal aggregate row behavior tests.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    minimal::{DestroyedVehicleClassification, KillClassification, MinimalPlayerStatsRow},
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::json;

const AGGREGATE_FIXTURE: &[u8] = include_bytes!("fixtures/aggregate-combat.ocap.json");
const LEGACY_PLAYER_ELIGIBILITY_FIXTURE: &[u8] = br#"{
  "missionName": "sg legacy player eligibility",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [0, 3],
  "captureDelay": 0.5,
  "endFrame": 120,
  "entities": [
    {
      "id": 1,
      "type": "unit",
      "name": "Eligible Player",
      "group": "Alpha 1-1",
      "side": "WEST",
      "description": "Rifleman",
      "isPlayer": 1,
      "positions": []
    },
    {
      "id": 2,
      "type": "unit",
      "name": "AI Unit",
      "group": "Bravo 1-1",
      "side": "EAST",
      "description": "Rifleman",
      "isPlayer": 0,
      "positions": []
    },
    {
      "id": 3,
      "type": "unit",
      "name": "Empty Description",
      "group": "Bravo 1-2",
      "side": "EAST",
      "description": "",
      "isPlayer": 1,
      "positions": []
    }
  ],
  "events": [],
  "Markers": [],
  "EditorMarkers": []
}"#;

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-aggregate-projection".to_owned()),
        source_file: "fixtures/aggregate-combat.ocap.json".to_owned(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "3333333333333333333333333333333333333333333333333333333333333333",
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

fn aggregate_artifact() -> ParseArtifact {
    parse_replay(parser_input(AGGREGATE_FIXTURE))
}

fn parse_fixture(bytes: &[u8]) -> ParseArtifact {
    parse_replay(parser_input(bytes))
}

fn player_stats<'a>(
    artifact: &'a ParseArtifact,
    source_entity_id: i64,
) -> &'a MinimalPlayerStatsRow {
    artifact
        .player_stats
        .iter()
        .find(|row| row.source_entity_id == source_entity_id)
        .unwrap_or_else(|| panic!("player stats row {source_entity_id} should exist"))
}

#[test]
fn aggregate_projection_should_emit_minimal_players_and_zero_counter_rows() {
    let artifact = aggregate_artifact();
    let player_ids =
        artifact.players.iter().map(|player| player.source_entity_id).collect::<Vec<_>>();
    let stats_ids =
        artifact.player_stats.iter().map(|stats| stats.source_entity_id).collect::<Vec<_>>();

    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(player_ids, vec![1, 2, 4, 5, 6]);
    assert_eq!(stats_ids, player_ids);
    assert!(artifact.players.iter().all(|player| player.player_id.starts_with("entity:")));
}

#[test]
fn aggregate_projection_should_derive_replay_local_player_counters() {
    let artifact = aggregate_artifact();
    let alpha = player_stats(&artifact, 1);
    let bravo = player_stats(&artifact, 2);
    let delta = player_stats(&artifact, 4);
    let echo_killer = player_stats(&artifact, 5);
    let echo_victim = player_stats(&artifact, 6);

    assert_eq!(alpha.kills, 1);
    assert_eq!(alpha.deaths, 0);
    assert_eq!(alpha.teamkills, 0);
    assert_eq!(alpha.vehicle_kills, 1);
    assert_eq!(alpha.kills_from_vehicle, 1);
    let alpha_json = serde_json::to_value(alpha).expect("stats row should serialize");
    assert_eq!(alpha_json["vehicleKills"], 1);
    assert_eq!(alpha_json["killsFromVehicle"], 1);

    assert_eq!(bravo.deaths, 1);
    assert_eq!(delta.deaths, 1);
    assert_eq!(delta.null_killer_deaths, 1);
    assert_eq!(echo_killer.teamkills, 1);
    assert_eq!(echo_victim.deaths, 1);
}

#[test]
fn aggregate_projection_should_emit_minimal_kill_rows_without_vehicle_victims() {
    let artifact = aggregate_artifact();
    let classifications = artifact.kills.iter().map(|row| row.classification).collect::<Vec<_>>();

    assert_eq!(
        classifications,
        vec![
            KillClassification::EnemyKill,
            KillClassification::Teamkill,
            KillClassification::NullKiller,
        ]
    );
    assert!(artifact.kills.iter().all(|row| row.victim_source_entity_id != Some(20)));
    assert!(artifact.kills.iter().any(|row| {
        row.classification == KillClassification::EnemyKill
            && row.killer_source_entity_id == Some(1)
            && row.victim_source_entity_id == Some(2)
            && row.bounty_eligible
            && row.attacker_vehicle_name.as_deref() == Some("Offroad HMG")
    }));
}

#[test]
fn aggregate_projection_should_emit_minimal_destroyed_vehicle_rows() {
    let artifact = aggregate_artifact();
    let destroyed =
        artifact.destroyed_vehicles.first().expect("destroyed vehicle row should exist");

    assert_eq!(artifact.destroyed_vehicles.len(), 1);
    assert_eq!(destroyed.classification, DestroyedVehicleClassification::Enemy);
    assert_eq!(destroyed.attacker_source_entity_id, Some(1));
    assert_eq!(destroyed.weapon.as_deref(), Some("RPG-7"));
    assert_eq!(destroyed.destroyed_entity_id, Some(20));
    assert_eq!(destroyed.destroyed_entity_type.as_deref(), Some("vehicle"));
    assert_eq!(destroyed.destroyed_class.as_deref(), Some("apc"));
}

#[test]
fn aggregate_projection_should_preserve_duplicate_slot_players_without_merging_rows() {
    let artifact = aggregate_artifact();
    let duplicate_players = artifact
        .players
        .iter()
        .filter(|player| player.compatibility_key == "legacy_name:Echo")
        .collect::<Vec<_>>();

    assert_eq!(duplicate_players.len(), 2);
    assert!(duplicate_players.iter().any(|player| player.source_entity_id == 5));
    assert!(duplicate_players.iter().any(|player| player.source_entity_id == 6));
}

#[test]
fn aggregate_projection_should_emit_zero_counter_rows_for_eligible_players_without_contributions() {
    let artifact = parse_fixture(LEGACY_PLAYER_ELIGIBILITY_FIXTURE);
    let eligible = player_stats(&artifact, 1);

    assert_eq!(artifact.players.len(), 1);
    assert_eq!(eligible.kills, 0);
    assert_eq!(eligible.deaths, 0);
    assert_eq!(eligible.teamkills, 0);
    assert_eq!(eligible.vehicle_kills, 0);
    assert_eq!(eligible.kills_from_vehicle, 0);
}

#[test]
fn aggregate_projection_should_omit_debug_only_keys_from_default_success_json() {
    let artifact = aggregate_artifact();
    let default_rows = json!({
        "players": artifact.players,
        "player_stats": artifact.player_stats,
        "kills": artifact.kills,
        "destroyed_vehicles": artifact.destroyed_vehicles,
    });
    let serialized_rows =
        serde_json::to_string(&default_rows).expect("default rows should serialize");
    let serialized_artifact = serde_json::to_string(&artifact).expect("artifact should serialize");

    assert!(!serialized_artifact.contains("source_refs"));

    for forbidden_key in [
        "source_refs",
        "rule_id",
        "event_index",
        "event_id",
        "json_path",
        "aggregate_contributions",
        "normalized_event",
        "entity_snapshot",
    ] {
        assert!(
            !serialized_rows.contains(forbidden_key),
            "default row JSON should not contain {forbidden_key}"
        );
    }
    let retired_score_section = ["vehicle", "score"].join("_");
    assert!(!serialized_rows.contains(&retired_score_section));
}
