//! Parser-core observed entity normalization tests.

#![allow(
    clippy::expect_used,
    clippy::too_many_lines,
    reason = "integration tests keep broad drift fixtures readable with inline assertions"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    identity::{EntityKind, EntitySide, ObservedEntity},
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

fn entity_by_id(artifact: &ParseArtifact, source_entity_id: i64) -> &ObservedEntity {
    artifact
        .entities
        .iter()
        .find(|entity| entity.source_entity_id == source_entity_id)
        .expect("entity should be normalized")
}

#[test]
fn entity_normalization_should_extract_unit_identity_when_unit_player_entity_is_observed() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let unit = entity_by_id(&artifact, 10);

    assert_eq!(unit.kind, EntityKind::Unit);
    assert!(matches!(
        &unit.identity.nickname,
        FieldPresence::Present { value, source: Some(_) } if value == "Alpha"
    ));
    assert!(matches!(
        unit.identity.side,
        FieldPresence::Present { value: EntitySide::West, source: Some(_) }
    ));
    assert!(matches!(
        &unit.identity.group,
        FieldPresence::Present { value, source: Some(_) } if value == "Alpha 1-1"
    ));
    assert!(matches!(
        &unit.identity.description,
        FieldPresence::Present { value, source: Some(_) } if value == "Rifleman"
    ));
    assert!(matches!(unit.is_player, FieldPresence::Present { value: true, source: Some(_) }));
}

#[test]
fn entity_normalization_should_preserve_false_player_flag_when_unit_entity_is_ai() {
    let fixture = br#"{
        "missionName": "sg ai unit",
        "worldName": "Altis",
        "missionAuthor": "SolidGames",
        "playersCount": [0, 1],
        "captureDelay": 0.5,
        "endFrame": 10,
        "entities": [
            {
                "id": 44,
                "type": "unit",
                "name": "AI Rifleman",
                "group": "Alpha 1-2",
                "side": "WEST",
                "description": "Rifleman",
                "isPlayer": 0,
                "positions": []
            }
        ],
        "events": [],
        "Markers": [],
        "EditorMarkers": []
    }"#;
    let artifact = parse_fixture(fixture);
    let unit = entity_by_id(&artifact, 44);

    assert!(matches!(unit.is_player, FieldPresence::Present { value: false, source: Some(_) }));
}

#[test]
fn entity_normalization_should_emit_unknown_player_flag_when_unit_flag_has_schema_drift() {
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
    let unit = entity_by_id(&artifact, 45);

    assert!(matches!(
        unit.is_player,
        FieldPresence::Unknown {
            reason: parser_contract::presence::UnknownReason::SchemaDrift,
            source: Some(_)
        }
    ));
    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "schema.entity_is_player_shape")
    );
}

#[test]
fn entity_normalization_should_extract_vehicle_name_and_class_when_vehicle_entity_is_observed() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let vehicle = entity_by_id(&artifact, 20);

    assert_eq!(vehicle.kind, EntityKind::Vehicle);
    assert!(matches!(
        &vehicle.observed_name,
        FieldPresence::Present { value, source: Some(_) } if value == "BTR-80"
    ));
    assert!(matches!(
        &vehicle.observed_class,
        FieldPresence::Present { value, source: Some(_) } if value == "apc"
    ));
    assert!(matches!(vehicle.is_player, FieldPresence::NotApplicable { .. }));
}

#[test]
fn entity_normalization_should_classify_static_weapon_when_vehicle_class_is_static_weapon() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let static_weapon = entity_by_id(&artifact, 30);

    assert_eq!(static_weapon.kind, EntityKind::StaticWeapon);
    assert!(matches!(
        &static_weapon.observed_class,
        FieldPresence::Present { value, source: Some(_) } if value == "static-weapon"
    ));
}

#[test]
fn entity_normalization_should_sort_entities_by_source_entity_id() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let entity_ids =
        artifact.entities.iter().map(|entity| entity.source_entity_id).collect::<Vec<_>>();

    assert_eq!(entity_ids, vec![10, 20, 30]);
}

#[test]
fn entity_normalization_should_keep_original_json_path_after_sorting() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let static_weapon = entity_by_id(&artifact, 30);
    let source_paths = static_weapon
        .source_refs
        .as_slice()
        .iter()
        .filter_map(|source_ref| source_ref.json_path.as_deref())
        .collect::<Vec<_>>();

    assert!(source_paths.contains(&"$.entities[0]"));
    assert!(source_paths.contains(&"$.entities[0].positions"));
}

#[test]
fn entity_normalization_should_emit_diagnostic_and_continue_when_entity_row_has_schema_drift() {
    let artifact = parse_fixture(ENTITY_DRIFT_FIXTURE);

    assert!(artifact.entities.iter().any(|entity| entity.source_entity_id == 7));
    assert!(
        artifact.diagnostics.iter().any(|diagnostic| diagnostic.code.starts_with("schema.entity"))
    );
}

#[test]
fn entity_normalization_should_emit_partial_status_when_entities_section_is_absent_or_drifted() {
    // Arrange
    let missing_entities = br#"{
        "missionName": "sg no entities",
        "events": []
    }"#;
    let drifted_entities = br#"{
        "missionName": "sg bad entities",
        "entities": {"not": "an array"},
        "events": []
    }"#;

    // Act
    let missing_artifact = parse_fixture(missing_entities);
    let drifted_artifact = parse_fixture(drifted_entities);

    // Assert
    assert_eq!(missing_artifact.status, ParseStatus::Partial);
    assert_eq!(drifted_artifact.status, ParseStatus::Partial);
    assert!(missing_artifact.entities.is_empty());
    assert!(drifted_artifact.entities.is_empty());
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
    // Arrange
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
                "isPlayer": true
            },
            {
                "id": 106,
                "type": "unit",
                "name": "Guer",
                "side": "GUER",
                "isPlayer": true
            },
            {
                "id": 107,
                "type": "unit",
                "name": "Unknown Side",
                "side": "UNKNOWN",
                "isPlayer": true
            },
            {
                "id": 108,
                "type": "unit",
                "name": "Duplicate",
                "side": "EAST",
                "isPlayer": true
            },
            {
                "id": 109,
                "type": "unit",
                "name": "Duplicate",
                "side": "WEST",
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

    // Act
    let artifact = parse_fixture(fixture);
    let drifted_unit = entity_by_id(&artifact, 102);
    let fallback_vehicle = entity_by_id(&artifact, 103);
    let civ = entity_by_id(&artifact, 105);
    let guer = entity_by_id(&artifact, 106);
    let unknown_side = entity_by_id(&artifact, 107);
    let diagnostic_codes =
        artifact.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<Vec<_>>();

    // Assert
    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(diagnostic_codes.contains(&"schema.entity_id_shape"));
    assert!(diagnostic_codes.contains(&"schema.entity_id_absent"));
    assert!(diagnostic_codes.contains(&"schema.entity_type_unknown"));
    assert!(diagnostic_codes.contains(&"schema.entity_field"));
    assert!(diagnostic_codes.contains(&"schema.entity_side_unknown"));
    assert!(diagnostic_codes.contains(&"schema.entity_side_shape"));
    assert!(diagnostic_codes.contains(&"schema.entity_is_player_shape"));
    assert!(diagnostic_codes.contains(&"compat.entity_duplicate_side_conflict"));
    assert!(matches!(
        drifted_unit.identity.nickname,
        FieldPresence::Unknown {
            reason: parser_contract::presence::UnknownReason::SchemaDrift,
            source: Some(_)
        }
    ));
    assert!(matches!(
        &fallback_vehicle.observed_class,
        FieldPresence::Present { value, source: Some(_) } if value == "truck"
    ));
    assert!(matches!(civ.identity.side, FieldPresence::Present { value: EntitySide::Civ, .. }));
    assert!(matches!(guer.identity.side, FieldPresence::Present { value: EntitySide::Guer, .. }));
    assert!(matches!(
        unknown_side.identity.side,
        FieldPresence::Present { value: EntitySide::Unknown, .. }
    ));
}

#[test]
fn entity_normalization_should_not_emit_forbidden_identity_fields() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let serialized = serde_json::to_string(&artifact.entities).expect("entities should serialize");
    let forbidden_fields =
        [("canonical", "player"), ("canonical", "id"), ("account", "id"), ("user", "id")]
            .map(|(prefix, suffix)| format!("{prefix}_{suffix}"));

    for forbidden_field in forbidden_fields {
        assert!(
            !serialized.contains(&forbidden_field),
            "entities should not contain {forbidden_field}"
        );
    }
}
