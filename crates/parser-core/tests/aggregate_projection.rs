//! Parser-core minimal aggregate row behavior tests.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    minimal::{
        DestroyedVehicleClassification, KillClassification, MinimalPlayerKillRow, MinimalPlayerRow,
    },
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

fn player_row(artifact: &ParseArtifact, source_entity_id: i64) -> &MinimalPlayerRow {
    artifact
        .players
        .iter()
        .find(|row| row.source_entity_id == source_entity_id)
        .unwrap_or_else(|| panic!("player row {source_entity_id} should exist"))
}

fn player_kill_rows(artifact: &ParseArtifact) -> Vec<&MinimalPlayerKillRow> {
    artifact.players.iter().flat_map(|player| player.kill_rows.iter()).collect()
}

#[test]
fn aggregate_projection_should_emit_compact_players_with_merged_counter_rows() {
    let artifact = aggregate_artifact();
    let player_ids =
        artifact.players.iter().map(|player| player.source_entity_id).collect::<Vec<_>>();

    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(player_ids, vec![1, 2, 4, 6]);
    assert_eq!(
        serde_json::to_value(&artifact).expect("artifact should serialize").get("player_stats"),
        None
    );
    assert!(
        artifact
            .players
            .iter()
            .filter(|player| player.source_entity_id <= 4)
            .all(|player| { player.compatibility_key.is_none() })
    );
}

#[test]
fn aggregate_projection_should_derive_replay_local_player_counters() {
    let artifact = aggregate_artifact();
    let alpha = player_row(&artifact, 1);
    let bravo = player_row(&artifact, 2);
    let delta = player_row(&artifact, 4);
    let echo = player_row(&artifact, 6);

    assert_eq!(alpha.kills, 1);
    assert_eq!(alpha.deaths, 0);
    assert_eq!(alpha.teamkills, 0);
    assert_eq!(alpha.vehicle_kills, 1);
    assert_eq!(alpha.kills_from_vehicle, 1);
    let alpha_json = serde_json::to_value(alpha).expect("stats row should serialize");
    assert_eq!(alpha_json["vk"], 1);
    assert_eq!(alpha_json["kfv"], 1);
    assert!(alpha_json.get("d").is_none());

    assert_eq!(bravo.deaths, 1);
    assert_eq!(delta.deaths, 1);
    assert_eq!(delta.null_killer_deaths, 1);
    assert_eq!(echo.teamkills, 1);
    assert_eq!(echo.deaths, 1);
}

#[test]
fn aggregate_projection_should_emit_deterministic_weapon_dictionary() {
    let artifact = aggregate_artifact();
    let weapons =
        artifact.weapons.iter().map(|row| (row.id, row.name.as_str())).collect::<Vec<_>>();

    assert_eq!(weapons, vec![(1, "AK-74"), (2, "Offroad HMG"), (3, "RPG-7")]);
}

#[test]
fn aggregate_projection_should_emit_compact_kill_rows_without_vehicle_victims() {
    let artifact = aggregate_artifact();
    let kill_rows = player_kill_rows(&artifact);
    let classifications = kill_rows.iter().map(|row| row.classification).collect::<Vec<_>>();

    assert_eq!(classifications, vec![KillClassification::EnemyKill, KillClassification::Teamkill]);
    assert!(kill_rows.iter().all(|row| row.victim_source_entity_id != Some(20)));
    assert!(player_row(&artifact, 1).kill_rows.iter().any(|row| {
        row.classification == KillClassification::EnemyKill
            && row.victim_source_entity_id == Some(2)
            && row.weapon_id == Some(2)
            && row.attacker_vehicle_entity_id == Some(30)
            && row.attacker_vehicle_class.as_deref() == Some("car")
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
    assert_eq!(destroyed.weapon_id, Some(3));
    assert_eq!(destroyed.attacker_vehicle_entity_id, None);
    assert_eq!(destroyed.attacker_vehicle_class, None);
    assert_eq!(destroyed.destroyed_entity_id, Some(20));
    assert_eq!(destroyed.destroyed_entity_type.as_deref(), Some("vehicle"));
    assert_eq!(destroyed.destroyed_class.as_deref(), Some("apc"));
}

#[test]
fn aggregate_projection_should_merge_duplicate_slot_players_like_legacy_parser() {
    let artifact = aggregate_artifact();
    let duplicate_player = player_row(&artifact, 6);

    assert_eq!(duplicate_player.compatibility_key.as_deref(), Some("legacy_name:Echo"));
    assert_eq!(duplicate_player.source_entity_ids, vec![5, 6]);
    assert_eq!(duplicate_player.teamkills, 1);
    assert_eq!(duplicate_player.deaths, 1);
}

#[test]
fn aggregate_projection_should_emit_zero_counter_rows_for_eligible_players_without_contributions() {
    let artifact = parse_fixture(LEGACY_PLAYER_ELIGIBILITY_FIXTURE);
    let eligible = player_row(&artifact, 1);
    let serialized = serde_json::to_value(eligible).expect("player row should serialize");

    assert_eq!(artifact.players.len(), 1);
    assert!(artifact.weapons.is_empty());
    assert_eq!(eligible.kills, 0);
    assert_eq!(eligible.deaths, 0);
    assert_eq!(eligible.teamkills, 0);
    assert_eq!(eligible.vehicle_kills, 0);
    assert_eq!(eligible.kills_from_vehicle, 0);
    assert!(serialized.get("k").is_none());
    assert!(serialized.get("d").is_none());
    assert!(serialized.get("vk").is_none());
    assert!(serialized.get("kfv").is_none());
}

#[test]
fn aggregate_projection_should_omit_debug_only_keys_from_default_success_json() {
    let artifact = aggregate_artifact();
    let default_rows = json!({
        "players": artifact.players,
        "weapons": artifact.weapons,
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
        "killer_name",
        "victim_name",
        "attacker_vehicle_name",
        "destroyed_name",
        "bounty_eligible",
        "bounty_exclusion_reasons",
    ] {
        assert!(
            !serialized_rows.contains(forbidden_key),
            "default row JSON should not contain {forbidden_key}"
        );
    }
    let retired_score_section = ["vehicle", "score"].join("_");
    assert!(!serialized_rows.contains(&retired_score_section));
}
