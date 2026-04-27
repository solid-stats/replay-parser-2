//! Parser-core observed entity normalization tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::ParseArtifact,
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
