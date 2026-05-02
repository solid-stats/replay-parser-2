//! Parser-core legacy entity compatibility behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    minimal::MinimalPlayerRow,
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

fn player_by_id(artifact: &ParseArtifact, source_entity_id: i64) -> &MinimalPlayerRow {
    artifact
        .players
        .iter()
        .find(|player| player.source_entity_id == source_entity_id)
        .expect("player should be normalized")
}

#[test]
fn legacy_entity_compatibility_should_infer_player_name_from_connected_event_when_entity_name_is_missing()
 {
    let artifact = connected_artifact();
    let player = player_by_id(&artifact, 11);

    assert_eq!(player.observed_name.as_deref(), Some("BackfilledName"));
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
    let player = player_by_id(&artifact, 12);

    assert_eq!(player.observed_name.as_deref(), Some("LastConnectedName"));
}

#[test]
fn legacy_entity_compatibility_should_skip_connected_backfill_for_vehicle_entities() {
    let artifact = connected_artifact();

    assert!(!artifact.players.iter().any(|player| player.source_entity_id == 99));
}

#[test]
fn legacy_entity_compatibility_should_not_serialize_source_refs_in_default_players() {
    let artifact = connected_artifact();
    let serialized = serde_json::to_string(&artifact.players).expect("players should serialize");

    assert!(!serialized.contains("source_refs"));
    assert!(!serialized.contains("rule_id"));
}

#[test]
fn legacy_entity_compatibility_should_merge_same_name_slot_entities_like_legacy_parser() {
    let artifact = duplicate_artifact();
    let duplicate_players = artifact
        .players
        .iter()
        .filter(|player| player.compatibility_key.as_deref() == Some("legacy_name:SameName"))
        .collect::<Vec<_>>();

    assert_eq!(artifact.players.len(), 2);
    assert_eq!(duplicate_players.len(), 1);
    assert_eq!(duplicate_players[0].source_entity_id, 22);
    assert_eq!(duplicate_players[0].source_entity_ids, vec![21, 22]);
}

#[test]
fn legacy_entity_compatibility_should_strip_tags_from_duplicate_slot_compatibility_keys() {
    let fixture = br#"{
        "missionName": "sg tagged duplicate slot same name",
        "worldName": "Altis",
        "missionAuthor": "SolidGames",
        "playersCount": [0, 2],
        "captureDelay": 0.5,
        "endFrame": 90,
        "entities": [
            {
                "id": 31,
                "type": "unit",
                "name": "[TAG]TaggedName",
                "group": "Alpha 1-1",
                "side": "WEST",
                "description": "Rifleman",
                "isPlayer": 1,
                "positions": []
            },
            {
                "id": 32,
                "type": "unit",
                "name": "[TAG]TaggedName",
                "group": "Alpha 1-2",
                "side": "WEST",
                "description": "Autorifleman",
                "isPlayer": 1,
                "positions": []
            }
        ],
        "events": [],
        "Markers": [],
        "EditorMarkers": []
    }"#;

    let artifact = parse_fixture(fixture, "fixtures/tagged-duplicate-slot-same-name.ocap.json");
    let player = player_by_id(&artifact, 32);

    assert_eq!(artifact.players.len(), 1);
    assert_eq!(player.observed_name.as_deref(), Some("TaggedName"));
    assert_eq!(player.observed_tag.as_deref(), Some("[TAG]"));
    assert_eq!(player.compatibility_key.as_deref(), Some("legacy_name:TaggedName"));
}

#[test]
fn legacy_entity_compatibility_should_not_override_connected_names_with_stale_duplicate_slot_names()
{
    let fixture = br#"{
        "missionName": "sg stale duplicate observed name",
        "worldName": "Altis",
        "missionAuthor": "SolidGames",
        "playersCount": [0, 2],
        "captureDelay": 0.5,
        "endFrame": 90,
        "entities": [
            {
                "id": 41,
                "type": "unit",
                "name": "[SKIF]Gre4a",
                "group": "Alpha 1-1",
                "side": "WEST",
                "description": "Rifleman",
                "isPlayer": 1,
                "positions": []
            },
            {
                "id": 42,
                "type": "unit",
                "name": "[SKIF]Gre4a",
                "group": "Alpha 1-2",
                "side": "WEST",
                "description": "Autorifleman",
                "isPlayer": 1,
                "positions": []
            }
        ],
        "events": [
            [1, "connected", "[SKIF]QuadRat", 41],
            [2, "connected", "[SKIF]Gre4a", 42]
        ],
        "Markers": [],
        "EditorMarkers": []
    }"#;

    let artifact = parse_fixture(fixture, "fixtures/stale-duplicate-observed-name.ocap.json");
    let quadrat = player_by_id(&artifact, 41);
    let gre4a = player_by_id(&artifact, 42);

    assert_eq!(artifact.players.len(), 2);
    assert_eq!(quadrat.observed_name.as_deref(), Some("QuadRat"));
    assert_eq!(quadrat.compatibility_key, None);
    assert_eq!(gre4a.observed_name.as_deref(), Some("Gre4a"));
    assert_eq!(gre4a.compatibility_key.as_deref(), Some("legacy_name:Gre4a"));
}

#[test]
fn legacy_entity_compatibility_should_keep_success_status_when_duplicate_slot_hint_has_no_conflict()
{
    let artifact = duplicate_artifact();

    assert_eq!(artifact.status, ParseStatus::Success);
}
