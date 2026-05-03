//! Parser-core combat semantic behavior through minimal default rows.

#![allow(
    clippy::expect_used,
    clippy::missing_const_for_fn,
    clippy::needless_collect,
    clippy::redundant_closure_for_method_calls,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    events::{
        BountyExclusionReason, CombatSemantic, CombatVictimKind, NormalizedEvent,
        NormalizedEventKind,
    },
    minimal::{DestroyedVehicleClassification, KillClassification, MinimalPlayerKillRow},
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{
    DebugParseArtifact, ParserInput, ParserOptions, parse_replay, parse_replay_debug,
};
use serde_json::json;

const COMBAT_EVENTS_FIXTURE: &[u8] = include_bytes!("fixtures/combat-events.ocap.json");
const MALFORMED_KILLED_EVENTS_FIXTURE: &[u8] = br#"{
  "missionName": "sg malformed killed events",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [0, 2],
  "captureDelay": 0.5,
  "endFrame": 120,
  "entities": [
    {
      "id": 1,
      "type": "unit",
      "name": "Alpha",
      "group": "Alpha 1-1",
      "side": "WEST",
      "description": "Rifleman",
      "isPlayer": 1,
      "positions": []
    },
    {
      "id": 2,
      "type": "unit",
      "name": "Bravo",
      "group": "Bravo 1-1",
      "side": "EAST",
      "description": "Rifleman",
      "isPlayer": 1,
      "positions": []
    }
  ],
  "events": [
    ["late", "killed", 2, [1, "AK-74"], 100],
    [20, "killed", 2, {"unexpected": true}, 100]
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
        replay_id: Some("replay-combat-events".to_owned()),
        source_file: "fixtures/combat-events.ocap.json".to_owned(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "2222222222222222222222222222222222222222222222222222222222222222",
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

fn combat_artifact() -> ParseArtifact {
    parse_replay(parser_input(COMBAT_EVENTS_FIXTURE))
}

fn parse_fixture(bytes: &[u8]) -> ParseArtifact {
    parse_replay(parser_input(bytes))
}

fn parse_debug_fixture(bytes: &[u8]) -> DebugParseArtifact {
    parse_replay_debug(parser_input(bytes))
}

fn debug_event_by_id<'a>(artifact: &'a DebugParseArtifact, event_id: &str) -> &'a NormalizedEvent {
    artifact
        .events
        .iter()
        .find(|event| event.event_id == event_id)
        .expect("requested debug event should exist")
}

fn kill_by_classification(
    artifact: &ParseArtifact,
    classification: KillClassification,
) -> &MinimalPlayerKillRow {
    artifact
        .players
        .iter()
        .flat_map(|player| player.kill_rows.iter())
        .find(|row| row.classification == classification)
        .expect("requested kill classification row should exist")
}

fn player_kill_rows(artifact: &ParseArtifact) -> Vec<&MinimalPlayerKillRow> {
    artifact.players.iter().flat_map(|player| player.kill_rows.iter()).collect()
}

fn player_by_id(
    artifact: &ParseArtifact,
    source_entity_id: i64,
) -> &parser_contract::minimal::MinimalPlayerRow {
    artifact
        .players
        .iter()
        .find(|player| player.source_entity_id == source_entity_id)
        .expect("requested player row should exist")
}

#[test]
fn combat_event_semantics_should_partition_player_deaths_and_destroyed_vehicles() {
    let artifact = combat_artifact();
    let kill_rows = player_kill_rows(&artifact);
    let classifications = kill_rows.iter().map(|row| row.classification).collect::<Vec<_>>();
    let delta = player_by_id(&artifact, 4);
    let echo = player_by_id(&artifact, 5);
    let bravo = player_by_id(&artifact, 2);
    let charlie = player_by_id(&artifact, 3);

    assert_eq!(kill_rows.len(), 2);
    assert_eq!(artifact.destroyed_vehicles.len(), 1);
    assert!(classifications.contains(&KillClassification::EnemyKill));
    assert!(classifications.contains(&KillClassification::Teamkill));
    assert_eq!(charlie.teamkill_deaths, 1);
    assert_eq!(delta.suicides, 1);
    assert_eq!(echo.null_killer_deaths, 1);
    assert_eq!(bravo.deaths, 1);
    assert_eq!(bravo.unknown_deaths, 1);
}

#[test]
fn combat_event_semantics_should_classify_enemy_kill_as_bounty_eligible() {
    let artifact = combat_artifact();
    let enemy_kill = kill_by_classification(&artifact, KillClassification::EnemyKill);

    assert_eq!(enemy_kill.victim_source_entity_id, Some(2));
    assert_eq!(enemy_kill.classification, KillClassification::EnemyKill);
}

#[test]
fn combat_event_semantics_should_classify_teamkill_suicide_and_null_killer_as_excluded() {
    let artifact = combat_artifact();
    let teamkill = kill_by_classification(&artifact, KillClassification::Teamkill);
    let delta = player_by_id(&artifact, 4);
    let echo = player_by_id(&artifact, 5);

    assert_eq!(teamkill.classification, KillClassification::Teamkill);
    assert_eq!(delta.suicides, 1);
    assert_eq!(echo.null_killer_deaths, 1);
}

#[test]
fn combat_event_semantics_should_classify_vehicle_destroyed_event() {
    let artifact = combat_artifact();
    let destroyed =
        artifact.destroyed_vehicles.first().expect("destroyed vehicle row should exist");

    assert_eq!(destroyed.classification, DestroyedVehicleClassification::Enemy);
    assert_eq!(destroyed.attacker_source_entity_id, Some(1));
    assert_eq!(destroyed.destroyed_entity_id, Some(20));
}

#[test]
fn combat_event_semantics_should_emit_unknown_player_death_and_partial_status_for_missing_actor() {
    let artifact = combat_artifact();
    let bravo = player_by_id(&artifact, 2);
    let diagnostic_codes =
        artifact.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<Vec<_>>();

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert_eq!(bravo.deaths, 1);
    assert_eq!(bravo.unknown_deaths, 1);
    assert!(diagnostic_codes.contains(&"event.killed_actor_unknown"));
}

#[test]
fn combat_event_semantics_should_emit_diagnostics_without_stats_for_malformed_killed_tuples() {
    let artifact = parse_fixture(MALFORMED_KILLED_EVENTS_FIXTURE);
    let diagnostic_codes =
        artifact.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<Vec<_>>();
    let bravo = player_by_id(&artifact, 2);

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(player_kill_rows(&artifact).is_empty());
    assert_eq!(bravo.unknown_deaths, 0);
    assert_eq!(
        diagnostic_codes.iter().filter(|code| **code == "event.killed_shape_unknown").count(),
        2
    );
}

#[test]
fn combat_event_semantics_should_keep_malformed_killed_tuples_as_unknown_debug_events() {
    let fixture = br#"{
      "missionName": "sg malformed debug killed events",
      "worldName": "Altis",
      "missionAuthor": "SolidGames",
      "playersCount": [0, 2],
      "captureDelay": 0.5,
      "endFrame": 120,
      "entities": [
        {
          "id": 1,
          "type": "unit",
          "name": "Alpha",
          "group": "Alpha 1-1",
          "side": "WEST",
          "description": "Rifleman",
          "isPlayer": 1,
          "positions": []
        },
        {
          "id": 2,
          "type": "unit",
          "name": "Bravo",
          "group": "Bravo 1-1",
          "side": "EAST",
          "description": "Rifleman",
          "isPlayer": 1,
          "positions": []
        }
      ],
      "events": [
        ["late", "killed", 2, [1, "AK-74"], 100],
        [20, "killed", 2, {"unexpected": true}],
        [21, "killed", "bad-victim", [1, "AK-74"], 75]
      ],
      "Markers": [],
      "EditorMarkers": []
    }"#;

    let artifact = parse_debug_fixture(fixture);
    let shape_diagnostics = artifact
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "event.killed_shape_unknown")
        .count();

    assert_eq!(artifact.events.len(), 3);
    assert_eq!(shape_diagnostics, 3);
    for event in &artifact.events {
        let combat = event.combat.as_ref().expect("debug killed event should carry combat data");

        assert_eq!(event.kind, NormalizedEventKind::Unknown);
        assert_eq!(combat.semantic, CombatSemantic::Unknown);
        assert_eq!(combat.bounty.exclusion_reasons, vec![BountyExclusionReason::UnknownActor]);
    }

    let drifted_frame = debug_event_by_id(&artifact, "event.killed.0");
    let malformed_info = debug_event_by_id(&artifact, "event.killed.1");
    let missing_victim = debug_event_by_id(&artifact, "event.killed.2");

    assert!(matches!(&drifted_frame.frame, FieldPresence::Unknown { .. }));
    assert!(matches!(
        &malformed_info
            .combat
            .as_ref()
            .expect("malformed kill-info event should carry combat data")
            .distance_meters,
        FieldPresence::Unknown { .. }
    ));
    assert!(matches!(
        &missing_victim
            .combat
            .as_ref()
            .expect("missing-victim event should carry combat data")
            .victim,
        FieldPresence::Unknown { .. }
    ));
}

#[test]
fn combat_event_semantics_should_keep_unknown_actor_context_in_debug_events() {
    let fixture = br#"{
      "missionName": "sg unknown actor debug killed events",
      "worldName": "Altis",
      "missionAuthor": "SolidGames",
      "playersCount": [0, 2],
      "captureDelay": 0.5,
      "endFrame": 120,
      "entities": [
        {
          "id": 1,
          "type": "unit",
          "name": "Known Side",
          "group": "Alpha 1-1",
          "side": "WEST",
          "description": "Rifleman",
          "isPlayer": 1,
          "positions": []
        },
        {
          "id": 2,
          "type": "unit",
          "name": "No Side",
          "group": "Bravo 1-1",
          "description": "Rifleman",
          "isPlayer": 1,
          "positions": []
        },
        {
          "id": 3,
          "type": "vehicle",
          "name": "Truck",
          "class": "truck",
          "side": "EAST",
          "positions": []
        }
      ],
      "events": [
        [10, "killed", 99, ["null"], 50],
        [11, "killed", 3, ["null"], 50],
        [12, "killed", 1, [3, ""], 15],
        [13, "killed", 2, [1, "AK-74"], 25]
      ],
      "Markers": [],
      "EditorMarkers": []
    }"#;

    let artifact = parse_debug_fixture(fixture);
    let diagnostic_messages = artifact
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(artifact.events.len(), 4);
    assert!(
        diagnostic_messages
            .iter()
            .any(|message| { message.contains("explicit null killer but no known player victim") })
    );
    assert!(
        diagnostic_messages
            .iter()
            .any(|message| message.contains("explicit null killer and a non-player victim"))
    );
    assert!(
        diagnostic_messages
            .iter()
            .any(|message| message.contains("not auditable as a player combat event"))
    );
    assert!(
        diagnostic_messages.iter().any(|message| message.contains("player sides are incomplete"))
    );

    let missing_victim = debug_event_by_id(&artifact, "event.killed.0")
        .combat
        .as_ref()
        .expect("missing-victim event should carry combat data");
    let null_vehicle_victim = debug_event_by_id(&artifact, "event.killed.1")
        .combat
        .as_ref()
        .expect("null-killer vehicle event should carry combat data");
    let vehicle_killer = debug_event_by_id(&artifact, "event.killed.2");
    let incomplete_sides = debug_event_by_id(&artifact, "event.killed.3");

    assert!(matches!(&missing_victim.killer, FieldPresence::Unknown { .. }));
    assert!(matches!(&missing_victim.victim, FieldPresence::Unknown { .. }));
    assert_eq!(missing_victim.victim_kind, CombatVictimKind::Unknown);
    assert!(matches!(&null_vehicle_victim.victim, FieldPresence::Present { .. }));
    assert_eq!(null_vehicle_victim.victim_kind, CombatVictimKind::Vehicle);
    assert_eq!(vehicle_killer.actors.len(), 2);
    assert_eq!(incomplete_sides.actors.len(), 2);
    assert_eq!(incomplete_sides.kind, NormalizedEventKind::Unknown);
}

#[test]
fn combat_event_semantics_should_omit_event_coordinates_from_default_rows() {
    let artifact = combat_artifact();
    let default_rows = json!({
        "players": artifact.players,
        "destroyed_vehicles": artifact.destroyed_vehicles,
    });
    let serialized = serde_json::to_string(&default_rows).expect("default rows should serialize");

    assert!(!serialized.contains("source_refs"));
    assert!(!serialized.contains("rule_id"));
    assert!(!serialized.contains("event_index"));
    assert!(!serialized.contains("\"frame\""));
}

#[test]
fn combat_event_semantics_should_keep_ambiguous_or_non_player_actor_cases_as_minimal_rows() {
    let fixture = br#"{
      "missionName": "sg ambiguous combat",
      "worldName": "Altis",
      "missionAuthor": "SolidGames",
      "playersCount": [0, 2],
      "captureDelay": 0.5,
      "endFrame": 120,
      "entities": [
        {
          "id": 1,
          "type": "unit",
          "name": "No Side",
          "group": "Alpha",
          "description": "Rifleman",
          "isPlayer": 1
        },
        {
          "id": 2,
          "type": "unit",
          "name": "Known Side",
          "group": "Bravo",
          "side": "WEST",
          "description": "Rifleman",
          "isPlayer": 1
        },
        {
          "id": 3,
          "type": "vehicle",
          "name": "Truck",
          "class": "truck"
        }
      ],
      "events": [
        [10, "killed", "bad-victim", [1, "AK-74"], 100],
        [11, "killed", 3, ["null"], 50],
        [12, "killed", 1, [2, ""], 25],
        [13, "killed", 3, [2, "AK-74"], 20],
        [14, "killed", 1, [3, ""], 15]
      ],
      "Markers": [],
      "EditorMarkers": []
    }"#;

    let artifact = parse_fixture(fixture);
    let unknown_deaths = artifact.players.iter().map(|player| player.unknown_deaths).sum::<u64>();
    let deaths = artifact.players.iter().map(|player| player.deaths).sum::<u64>();

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert_eq!(unknown_deaths, 2);
    assert_eq!(deaths, 0);
    assert!(player_kill_rows(&artifact).is_empty());
    assert_eq!(artifact.destroyed_vehicles.len(), 1);
    assert!(
        artifact.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("not auditable as a player combat event"))
    );
}

#[test]
fn combat_event_semantics_should_preserve_non_empty_weapon_statistics() {
    let fixture = br#"{
      "missionName": "sg binocular weapon",
      "worldName": "Altis",
      "missionAuthor": "SolidGames",
      "playersCount": [0, 2],
      "captureDelay": 0.5,
      "endFrame": 120,
      "entities": [
        {
          "id": 1,
          "type": "unit",
          "name": "Spotter",
          "group": "Alpha",
          "side": "WEST",
          "description": "Rifleman",
          "isPlayer": 1
        },
        {
          "id": 2,
          "type": "unit",
          "name": "Target",
          "group": "Bravo",
          "side": "EAST",
          "description": "Rifleman",
          "isPlayer": 1
        }
      ],
      "events": [
        [10, "killed", 2, [1, "Binoculars"], 100]
      ],
      "Markers": [],
      "EditorMarkers": []
    }"#;

    let artifact = parse_fixture(fixture);
    let spotter = player_by_id(&artifact, 1);

    assert_eq!(spotter.kills, 1);
    assert_eq!(artifact.weapons.len(), 1);
    assert_eq!(artifact.weapons[0].name, "Binoculars");
    assert_eq!(spotter.kill_rows.len(), 1);
    assert_eq!(spotter.kill_rows[0].weapon_id, Some(1));
}

#[test]
fn combat_event_semantics_should_use_latest_death_for_teamkill_death_marker() {
    let fixture = br#"{
      "missionName": "sg respawned after teamkill",
      "worldName": "Altis",
      "missionAuthor": "SolidGames",
      "playersCount": [0, 3],
      "captureDelay": 0.5,
      "endFrame": 120,
      "entities": [
        {
          "id": 1,
          "type": "unit",
          "name": "Driver",
          "group": "Alpha",
          "side": "WEST",
          "description": "Driver",
          "isPlayer": 1
        },
        {
          "id": 2,
          "type": "unit",
          "name": "Passenger",
          "group": "Alpha",
          "side": "WEST",
          "description": "Passenger",
          "isPlayer": 1
        },
        {
          "id": 3,
          "type": "unit",
          "name": "Enemy",
          "group": "Bravo",
          "side": "EAST",
          "description": "Rifleman",
          "isPlayer": 1
        }
      ],
      "events": [
        [10, "killed", 2, [1, "Car"], 100],
        [80, "killed", 2, [3, "Rifle"], 100]
      ],
      "Markers": [],
      "EditorMarkers": []
    }"#;

    let artifact = parse_fixture(fixture);
    let passenger = player_by_id(&artifact, 2);

    assert_eq!(passenger.deaths, 2);
    assert_eq!(passenger.teamkill_deaths, 0);
}
