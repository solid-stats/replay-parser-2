//! Parser-core observed entity normalization tests through minimal player rows.

#![allow(
    clippy::expect_used,
    clippy::too_many_lines,
    reason = "integration tests keep broad drift fixtures readable with inline assertions"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    identity::EntitySide,
    minimal::MinimalPlayerRow,
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::json;

const MIXED_UNSORTED_FIXTURE: &[u8] = include_bytes!("fixtures/entities-mixed-unsorted.ocap.json");
const ENTITY_DRIFT_FIXTURE: &[u8] = include_bytes!("fixtures/entities-drift.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-entities".to_string()),
        source_file: "fixtures/entities-mixed-unsorted.ocap.json".to_string(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "0000000000000000000000000000000000000000000000000000000000000000",
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

fn parse_fixture(bytes: &[u8]) -> ParseArtifact {
    parse_replay(parser_input(bytes))
}

fn player_by_id(artifact: &ParseArtifact, source_entity_id: i64) -> &MinimalPlayerRow {
    artifact
        .players
        .iter()
        .find(|player| player.source_entity_id == source_entity_id)
        .expect("player should be normalized")
}

#[test]
fn entity_normalization_should_extract_legacy_player_identity() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let unit = player_by_id(&artifact, 10);

    assert_eq!(unit.observed_name.as_deref(), Some("Alpha"));
    assert_eq!(unit.side, Some(EntitySide::West));
    assert_eq!(unit.group.as_deref(), Some("Alpha 1-1"));
    assert_eq!(unit.role.as_deref(), Some("Rifleman"));
    assert_eq!(unit.source_entity_id, 10);
}

#[test]
fn entity_normalization_should_preserve_steam_id_values_in_minimal_players() {
    let fixture = br#"{
        "missionName": "sg participant steam ids",
        "worldName": "Altis",
        "missionAuthor": "SolidGames",
        "playersCount": [0, 4],
        "captureDelay": 0.5,
        "endFrame": 10,
        "entities": [
            {
                "id": 41,
                "type": "unit",
                "name": "Steam Player",
                "steamID": "76561198000000001",
                "group": "Alpha 1-1",
                "side": "WEST",
                "description": "Rifleman",
                "isPlayer": 1,
                "positions": []
            },
            {
                "id": 42,
                "type": "unit",
                "name": "No Steam Player",
                "group": "Alpha 1-2",
                "side": "WEST",
                "description": "Medic",
                "isPlayer": 1,
                "positions": []
            },
            {
                "id": 43,
                "type": "unit",
                "name": "Camel Steam Player",
                "steamId": "76561198000000003",
                "group": "Alpha 1-3",
                "side": "WEST",
                "description": "Autorifleman",
                "isPlayer": 1,
                "positions": []
            },
            {
                "id": 44,
                "type": "unit",
                "name": "Snake Steam Player",
                "steam_id": "76561198000000004",
                "group": "Alpha 1-4",
                "side": "WEST",
                "description": "Grenadier",
                "isPlayer": 1,
                "positions": []
            }
        ],
        "events": [],
        "Markers": [],
        "EditorMarkers": []
    }"#;
    let artifact = parse_fixture(fixture);

    assert_eq!(player_by_id(&artifact, 41).steam_id.as_deref(), Some("76561198000000001"));
    assert_eq!(player_by_id(&artifact, 42).steam_id, None);
    assert_eq!(player_by_id(&artifact, 43).steam_id.as_deref(), Some("76561198000000003"));
    assert_eq!(player_by_id(&artifact, 44).steam_id.as_deref(), Some("76561198000000004"));
}

#[test]
fn entity_normalization_should_exclude_non_player_units_vehicles_and_static_weapons_from_players() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let player_ids =
        artifact.players.iter().map(|player| player.source_entity_id).collect::<Vec<_>>();

    assert_eq!(player_ids, vec![10]);
    assert!(!player_ids.contains(&20));
    assert!(!player_ids.contains(&30));
}

#[test]
fn entity_normalization_should_emit_unknown_player_flag_diagnostic_when_unit_flag_has_schema_drift()
{
    let fixture = br#"{
        "missionName": "sg player flag drift",
        "worldName": "Altis",
        "missionAuthor": "SolidGames",
        "playersCount": [0, 1],
        "captureDelay": 0.5,
        "endFrame": 10,
        "entities": [
            {
                "id": 45,
                "type": "unit",
                "name": "Flag Drift",
                "group": "Alpha 1-2",
                "side": "WEST",
                "description": "Rifleman",
                "isPlayer": "yes",
                "positions": []
            }
        ],
        "events": [],
        "Markers": [],
        "EditorMarkers": []
    }"#;
    let artifact = parse_fixture(fixture);

    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "schema.entity_is_player_shape")
    );
}

#[test]
fn entity_normalization_should_emit_diagnostic_and_continue_when_entity_row_has_schema_drift() {
    let artifact = parse_fixture(ENTITY_DRIFT_FIXTURE);

    assert!(artifact.players.iter().any(|player| player.source_entity_id == 7));
    assert!(
        artifact.diagnostics.iter().any(|diagnostic| diagnostic.code.starts_with("schema.entity"))
    );
}

#[test]
fn entity_normalization_should_emit_partial_status_when_entities_section_is_absent_or_drifted() {
    let missing_entities = br#"{
        "missionName": "sg no entities",
        "events": []
    }"#;
    let drifted_entities = br#"{
        "missionName": "sg bad entities",
        "entities": {"not": "an array"},
        "events": []
    }"#;

    let missing_artifact = parse_fixture(missing_entities);
    let drifted_artifact = parse_fixture(drifted_entities);

    assert_eq!(missing_artifact.status, ParseStatus::Partial);
    assert_eq!(drifted_artifact.status, ParseStatus::Partial);
    assert!(missing_artifact.players.is_empty());
    assert!(drifted_artifact.players.is_empty());
    assert!(
        missing_artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "schema.entities_absent")
    );
    assert!(
        drifted_artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "schema.entities_shape")
    );
}

#[test]
fn entity_normalization_should_diagnose_entity_shape_and_value_drift_without_panicking() {
    let fixture = br#"{
        "missionName": "sg entity drift branches",
        "worldName": "Altis",
        "missionAuthor": "SolidGames",
        "playersCount": [0, 1],
        "captureDelay": 0.5,
        "endFrame": 10,
        "entities": [
            [],
            { "type": "unit", "name": "Missing Id", "side": "WEST", "isPlayer": 1 },
            {
                "id": 101,
                "type": "alien",
                "name": true,
                "class": 5,
                "side": "SIDEWAYS",
                "positions": []
            },
            {
                "id": 102,
                "type": "unit",
                "name": 4,
                "group": true,
                "side": 5,
                "description": false,
                "isPlayer": 2
            },
            {
                "id": 103,
                "type": "vehicle",
                "name": "Fallback Truck",
                "_class": "truck",
                "side": "EAST"
            },
            {
                "id": 104,
                "type": "vehicle",
                "name": "Bad Class",
                "class": 7,
                "_class": []
            },
            {
                "id": 105,
                "type": "unit",
                "name": "Civilian",
                "side": "CIV",
                "description": "Rifleman",
                "isPlayer": true
            },
            {
                "id": 106,
                "type": "unit",
                "name": "Guer",
                "side": "GUER",
                "description": "Rifleman",
                "isPlayer": true
            },
            {
                "id": 107,
                "type": "unit",
                "name": "Unknown Side",
                "side": "UNKNOWN",
                "description": "Rifleman",
                "isPlayer": true
            },
            {
                "id": 108,
                "type": "unit",
                "name": "Duplicate",
                "side": "EAST",
                "description": "Rifleman",
                "isPlayer": true
            },
            {
                "id": 109,
                "type": "unit",
                "name": "Duplicate",
                "side": "WEST",
                "description": "Rifleman",
                "isPlayer": true
            }
        ],
        "events": [
            [1, "connected", "", 102],
            [2, "connected", "Missing Entity", 999],
            [3, "connected", "Vehicle Name", 103]
        ],
        "Markers": [],
        "EditorMarkers": []
    }"#;

    let artifact = parse_fixture(fixture);
    let diagnostic_codes =
        artifact.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<Vec<_>>();

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(diagnostic_codes.contains(&"schema.entity_id_shape"));
    assert!(diagnostic_codes.contains(&"schema.entity_id_absent"));
    assert!(diagnostic_codes.contains(&"schema.entity_type_unknown"));
    assert!(diagnostic_codes.contains(&"schema.entity_field"));
    assert!(diagnostic_codes.contains(&"schema.entity_side_unknown"));
    assert!(diagnostic_codes.contains(&"schema.entity_side_shape"));
    assert!(diagnostic_codes.contains(&"schema.entity_is_player_shape"));
    assert!(diagnostic_codes.contains(&"compat.entity_duplicate_side_conflict"));
    assert_eq!(player_by_id(&artifact, 105).side, Some(EntitySide::Civ));
    assert_eq!(player_by_id(&artifact, 106).side, Some(EntitySide::Guer));
    assert_eq!(player_by_id(&artifact, 107).side, Some(EntitySide::Unknown));
}

#[test]
fn entity_normalization_should_not_emit_forbidden_identity_fields() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let serialized = serde_json::to_string(&artifact.players).expect("players should serialize");
    let forbidden_fields =
        [("canonical", "player"), ("canonical", "id"), ("account", "id"), ("user", "id")]
            .map(|(prefix, suffix)| format!("{prefix}_{suffix}"));

    for forbidden_field in forbidden_fields {
        assert!(
            !serialized.contains(&forbidden_field),
            "players should not contain {forbidden_field}"
        );
    }
}
