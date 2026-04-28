//! Parser-core aggregate projection behavior tests.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests use expect messages as assertion context"
)]

use std::collections::BTreeSet;

use parser_contract::{
    aggregates::AggregateContributionKind,
    artifact::{ParseArtifact, ParseStatus},
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::{Value, json};

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
  "events": [
    [10, "killed", 2, [1, "AK-74"], 100],
    [20, "killed", 1, [2, "AK-74"], 100]
  ],
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

fn projection_object<'a>(
    artifact: &'a ParseArtifact,
    key: &str,
) -> &'a serde_json::Map<String, Value> {
    projection(artifact, key)
        .as_object()
        .unwrap_or_else(|| panic!("projection {key} should be an object"))
}

fn player_row<'a>(artifact: &'a ParseArtifact, compatibility_key: &str) -> &'a Value {
    projection_array(artifact, "legacy.player_game_results")
        .iter()
        .find(|row| row["compatibility_key"] == compatibility_key)
        .unwrap_or_else(|| panic!("player row {compatibility_key} should exist"))
}

fn source_contribution_ids(row: &Value) -> BTreeSet<String> {
    row["source_contribution_ids"]
        .as_array()
        .expect("row should contain source_contribution_ids")
        .iter()
        .map(|value| value.as_str().expect("contribution ID should be a string").to_owned())
        .collect()
}

fn aggregate_contribution_ids(artifact: &ParseArtifact) -> BTreeSet<String> {
    artifact
        .aggregates
        .contributions
        .iter()
        .map(|contribution| contribution.contribution_id.clone())
        .collect()
}

#[test]
fn aggregate_projection_should_emit_namespaced_legacy_and_bounty_projection_keys() {
    let artifact = aggregate_artifact();

    let projection_keys =
        artifact.aggregates.projections.keys().map(String::as_str).collect::<Vec<_>>();

    assert_eq!(artifact.status, ParseStatus::Success);
    for expected_key in [
        "legacy.player_game_results",
        "legacy.relationships",
        "legacy.game_type_compatibility",
        "legacy.squad_inputs",
        "legacy.rotation_inputs",
        "bounty.inputs",
    ] {
        assert!(
            projection_keys.contains(&expected_key),
            "projection should contain {expected_key}"
        );
    }
}

#[test]
fn aggregate_projection_should_derive_legacy_counters_from_contributions() {
    let artifact = aggregate_artifact();

    let alpha = player_row(&artifact, "entity:1");
    let echo = player_row(&artifact, "legacy_name:Echo");

    assert_eq!(alpha["kills"], 1);
    assert_eq!(alpha["killsFromVehicle"], 1);
    assert_eq!(alpha["vehicleKills"], 1);
    assert_eq!(alpha["teamkills"], 0);
    assert_eq!(alpha["deaths"], json!({ "total": 0, "byTeamkills": 0 }));
    assert_eq!(alpha["kdRatio"].as_f64(), Some(1.0));
    assert_eq!(alpha["killsFromVehicleCoef"].as_f64(), Some(1.0));
    assert_eq!(alpha["score"].as_f64(), Some(1.0));
    assert_eq!(alpha["totalPlayedGames"], 1);
    assert!(!source_contribution_ids(alpha).is_empty());

    assert_eq!(echo["teamkills"], 1);
    assert_eq!(echo["isDead"], true);
    assert_eq!(echo["isDeadByTeamkill"], true);
    assert_eq!(echo["observed_entity_ids"], json!([5, 6]));
    assert!(!source_contribution_ids(echo).is_empty());
}

#[test]
fn aggregate_projection_should_emit_relationship_summaries() {
    let artifact = aggregate_artifact();
    let relationships = projection_object(&artifact, "legacy.relationships");
    let killed = relationships["killed"].as_array().expect("killed rows should be an array");
    let teamkilled =
        relationships["teamkilled"].as_array().expect("teamkilled rows should be an array");

    let enemy_row = killed
        .iter()
        .find(|row| {
            row["source_compatibility_key"] == "entity:1"
                && row["target_compatibility_key"] == "entity:2"
        })
        .expect("enemy kill relationship should exist");
    let teamkill_row = teamkilled
        .iter()
        .find(|row| {
            row["source_compatibility_key"] == "legacy_name:Echo"
                && row["target_compatibility_key"] == "legacy_name:Echo"
        })
        .expect("same-name teamkill relationship should exist");

    assert_eq!(enemy_row["count"], 1);
    assert_eq!(enemy_row["event_ids"], json!(["event.killed.0"]));
    assert!(!source_contribution_ids(enemy_row).is_empty());

    assert_eq!(teamkill_row["count"], 1);
    assert_eq!(teamkill_row["event_ids"], json!(["event.killed.1"]));
    assert!(!source_contribution_ids(teamkill_row).is_empty());
}

#[test]
fn aggregate_projection_should_exclude_teamkill_suicide_null_and_vehicle_events_from_bounty_inputs()
{
    let artifact = aggregate_artifact();
    let bounty_inputs = projection_array(&artifact, "bounty.inputs");
    let bounty_event_ids =
        bounty_inputs.iter().map(|row| row["event_id"].as_str()).collect::<Vec<_>>();

    assert_eq!(bounty_inputs.len(), 1);
    assert_eq!(bounty_inputs[0]["event_id"], "event.killed.0");
    assert_eq!(bounty_inputs[0]["eligible"], true);
    assert!(!bounty_event_ids.contains(&Some("event.killed.1")));
    assert!(!bounty_event_ids.contains(&Some("event.killed.2")));
    assert!(!bounty_event_ids.contains(&Some("event.killed.3")));
}

#[test]
fn aggregate_projection_should_emit_game_type_squad_and_rotation_inputs_without_filtering_replay() {
    let artifact = aggregate_artifact();
    let game_type = projection_object(&artifact, "legacy.game_type_compatibility");
    let squad_inputs = projection_array(&artifact, "legacy.squad_inputs");
    let rotation = projection_object(&artifact, "legacy.rotation_inputs");

    let alpha_squad = squad_inputs
        .iter()
        .find(|row| row["compatibility_key"] == "entity:1")
        .expect("alpha squad input should exist");

    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(game_type["mission_name"], "sg aggregate projection");
    assert_eq!(game_type["prefix_bucket"], "sg");
    assert_eq!(game_type["parser_action"], "emit_filter_metadata_only");
    assert_eq!(alpha_squad["squad_prefix"], "[A]");
    assert_eq!(alpha_squad["source_entity_ids"], json!([1]));
    assert_eq!(rotation["requires_downstream_replay_date"], true);
    assert_eq!(rotation["parser_action"], "server_or_parity_harness_groups_by_replay_date");
}

#[test]
fn aggregate_projection_should_group_duplicate_same_name_projection_without_merging_entities() {
    let artifact = aggregate_artifact();
    let duplicate_entities = artifact
        .entities
        .iter()
        .filter(|entity| entity.source_entity_id == 5 || entity.source_entity_id == 6)
        .count();
    let duplicate_rows = projection_array(&artifact, "legacy.player_game_results")
        .iter()
        .filter(|row| row["compatibility_key"] == "legacy_name:Echo")
        .count();
    let duplicate_row = player_row(&artifact, "legacy_name:Echo");

    assert_eq!(duplicate_entities, 2);
    assert_eq!(duplicate_rows, 1);
    assert_eq!(duplicate_row["observed_entity_ids"], json!([5, 6]));
    assert_eq!(duplicate_row["observed_name"], "Echo");
}

#[test]
fn aggregate_projection_should_emit_zero_counter_rows_for_eligible_players_without_contributions() {
    let artifact = parse_fixture(LEGACY_PLAYER_ELIGIBILITY_FIXTURE);
    let eligible = player_row(&artifact, "entity:1");

    assert_eq!(eligible["kills"], 0);
    assert_eq!(eligible["killsFromVehicle"], 0);
    assert_eq!(eligible["vehicleKills"], 0);
    assert_eq!(eligible["teamkills"], 0);
    assert_eq!(eligible["totalPlayedGames"], 1);
    assert_eq!(eligible["source_contribution_ids"], json!([]));
}

#[test]
fn aggregate_projection_should_exclude_non_player_units_from_legacy_and_bounty_rows() {
    let artifact = parse_fixture(LEGACY_PLAYER_ELIGIBILITY_FIXTURE);
    let player_rows = projection_array(&artifact, "legacy.player_game_results");
    let compatibility_keys =
        player_rows.iter().map(|row| row["compatibility_key"].as_str()).collect::<Vec<_>>();
    let bounty_inputs = projection_array(&artifact, "bounty.inputs");

    assert_eq!(compatibility_keys, vec![Some("entity:1")]);
    assert!(bounty_inputs.is_empty());
    assert!(artifact.aggregates.contributions.is_empty());
}

#[test]
fn aggregate_projection_should_keep_every_counter_traceable_to_source_refs() {
    let artifact = aggregate_artifact();
    let contribution_ids = aggregate_contribution_ids(&artifact);

    for contribution in &artifact.aggregates.contributions {
        assert!(
            !contribution.source_refs.as_slice().is_empty(),
            "contribution {} should have source refs",
            contribution.contribution_id
        );
        assert!(!contribution.rule_id.as_str().is_empty());
        assert!(contribution.event_id.is_some());
    }

    for row in projection_array(&artifact, "legacy.player_game_results") {
        let row_contribution_ids = source_contribution_ids(row);

        assert!(!row_contribution_ids.is_empty());
        assert!(row_contribution_ids.iter().all(|id| contribution_ids.contains(id)));
    }

    let relationships = projection_object(&artifact, "legacy.relationships");
    for relationship_rows in relationships.values().filter_map(Value::as_array) {
        for row in relationship_rows {
            let row_contribution_ids = source_contribution_ids(row);

            assert!(!row_contribution_ids.is_empty());
            assert!(row_contribution_ids.iter().all(|id| contribution_ids.contains(id)));
        }
    }

    for bounty_row in projection_array(&artifact, "bounty.inputs") {
        let contribution_id = bounty_row["source_contribution_id"]
            .as_str()
            .expect("bounty row should carry contribution ID");

        assert!(contribution_ids.contains(contribution_id));
        assert!(artifact.aggregates.contributions.iter().any(|contribution| {
            contribution.contribution_id == contribution_id
                && contribution.kind == AggregateContributionKind::BountyInput
        }));
    }
}
