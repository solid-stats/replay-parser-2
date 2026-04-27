//! Parser-core legacy entity compatibility behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    identity::{EntityCompatibilityHint, EntityCompatibilityHintKind, ObservedEntity},
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::json;

const CONNECTED_BACKFILL_FIXTURE: &[u8] = include_bytes!("fixtures/connected-backfill.ocap.json");
const DUPLICATE_SLOT_SAME_NAME_FIXTURE: &[u8] =
    include_bytes!("fixtures/duplicate-slot-same-name.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source(source_file: &str) -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-legacy-entities".to_string()),
        source_file: source_file.to_string(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect("test checksum should be valid"),
            source: None,
        },
    }
}

fn parser_input<'a>(bytes: &'a [u8], source_file: &str) -> ParserInput<'a> {
    ParserInput {
        bytes,
        source: replay_source(source_file),
        parser: parser_info(),
        options: ParserOptions::default(),
    }
}

fn parse_fixture(bytes: &[u8], source_file: &str) -> ParseArtifact {
    parse_replay(parser_input(bytes, source_file))
}

fn connected_artifact() -> ParseArtifact {
    parse_fixture(CONNECTED_BACKFILL_FIXTURE, "fixtures/connected-backfill.ocap.json")
}

fn duplicate_artifact() -> ParseArtifact {
    parse_fixture(DUPLICATE_SLOT_SAME_NAME_FIXTURE, "fixtures/duplicate-slot-same-name.ocap.json")
}

fn entity_by_id(artifact: &ParseArtifact, source_entity_id: i64) -> &ObservedEntity {
    artifact
        .entities
        .iter()
        .find(|entity| entity.source_entity_id == source_entity_id)
        .expect("entity should be normalized")
}

fn hint_by_kind(
    entity: &ObservedEntity,
    kind: EntityCompatibilityHintKind,
) -> &EntityCompatibilityHint {
    entity
        .compatibility_hints
        .iter()
        .find(|hint| hint.kind == kind)
        .expect("compatibility hint should exist")
}

#[test]
fn legacy_entity_compatibility_should_infer_player_name_from_connected_event_when_entity_name_is_missing()
 {
    let artifact = connected_artifact();
    let player = entity_by_id(&artifact, 11);

    assert!(matches!(
        &player.observed_name,
        FieldPresence::Inferred { value, reason, rule_id, .. }
            if value == "BackfilledName"
                && reason == "legacy connected event player backfill"
                && rule_id.as_str() == "entity.connected_player_backfill"
    ));
    assert!(matches!(
        &player.identity.nickname,
        FieldPresence::Inferred { value, rule_id, .. }
            if value == "BackfilledName"
                && rule_id.as_str() == "entity.connected_player_backfill"
    ));
}

#[test]
fn legacy_entity_compatibility_should_use_last_connected_name_for_player_nickname() {
    let fixture = br#"{
        "missionName": "sg connected overwrite",
        "worldName": "Altis",
        "missionAuthor": "SolidGames",
        "playersCount": [0, 1],
        "captureDelay": 0.5,
        "endFrame": 90,
        "entities": [
            {
                "id": 12,
                "type": "unit",
                "name": "StaleEntityName",
                "group": "Alpha 1-1",
                "side": "WEST",
                "description": "Rifleman",
                "isPlayer": 0,
                "positions": []
            }
        ],
        "events": [
            [4, "connected", "FirstConnectedName", 12],
            [8, "connected", "LastConnectedName", 12]
        ],
        "Markers": [],
        "EditorMarkers": []
    }"#;
    let artifact = parse_fixture(fixture, "fixtures/connected-overwrite.ocap.json");
    let player = entity_by_id(&artifact, 12);

    assert!(matches!(
        &player.observed_name,
        FieldPresence::Present { value, .. } if value == "StaleEntityName"
    ));
    assert!(matches!(
        &player.identity.nickname,
        FieldPresence::Inferred { value, rule_id, .. }
            if value == "LastConnectedName"
                && rule_id.as_str() == "entity.connected_player_backfill"
    ));
}

#[test]
fn legacy_entity_compatibility_should_skip_connected_backfill_for_vehicle_entities() {
    let artifact = connected_artifact();
    let vehicle = entity_by_id(&artifact, 99);

    assert!(matches!(
        &vehicle.observed_name,
        FieldPresence::Present { value, .. } if value == "OriginalVehicle"
    ));
    assert!(
        !vehicle
            .compatibility_hints
            .iter()
            .any(|hint| hint.kind == EntityCompatibilityHintKind::ConnectedPlayerBackfill)
    );
}

#[test]
fn legacy_entity_compatibility_should_attach_connected_backfill_hint_with_event_and_entity_source_refs()
 {
    let artifact = connected_artifact();
    let player = entity_by_id(&artifact, 11);
    let hint = hint_by_kind(player, EntityCompatibilityHintKind::ConnectedPlayerBackfill);
    let source_paths = hint
        .source_refs
        .as_slice()
        .iter()
        .filter_map(|source_ref| source_ref.json_path.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(hint.rule_id.as_str(), "entity.connected_player_backfill");
    assert!(source_paths.contains(&"$.events[0]"));
    assert!(source_paths.contains(&"$.entities[0]"));
    assert!(
        hint.source_refs
            .as_slice()
            .iter()
            .any(|source_ref| { source_ref.frame == Some(4) && source_ref.event_index == Some(0) })
    );
}

#[test]
fn legacy_entity_compatibility_should_add_duplicate_slot_hint_without_merging_same_name_entities() {
    let artifact = duplicate_artifact();
    let first_slot = entity_by_id(&artifact, 21);
    let second_slot = entity_by_id(&artifact, 22);
    let first_hint = hint_by_kind(first_slot, EntityCompatibilityHintKind::DuplicateSlotSameName);
    let second_hint = hint_by_kind(second_slot, EntityCompatibilityHintKind::DuplicateSlotSameName);

    assert_eq!(artifact.entities.len(), 3);
    assert_eq!(first_hint.rule_id.as_str(), "entity.duplicate_slot_same_name");
    assert_eq!(first_hint.related_entity_ids, vec![21, 22]);
    assert_eq!(second_hint.related_entity_ids, vec![21, 22]);
}

#[test]
fn legacy_entity_compatibility_should_keep_success_status_when_duplicate_slot_hint_has_no_conflict()
{
    let artifact = duplicate_artifact();

    assert_eq!(artifact.status, ParseStatus::Success);
}
