//! Parser-core combat event semantic behavior tests.

#![allow(
    clippy::expect_used,
    clippy::missing_const_for_fn,
    clippy::needless_collect,
    clippy::redundant_closure_for_method_calls,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    compact::CombatFact,
    events::{BountyEligibilityState, BountyExclusionReason, CombatSemantic},
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
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

fn event_by_id<'a>(artifact: &'a ParseArtifact, event_id: &str) -> &'a CombatFact {
    artifact
        .facts
        .combat
        .iter()
        .find(|event| event.fact_id == event_id)
        .expect("combat fact should be normalized")
}

fn has_exclusion(combat: &CombatFact, exclusion: BountyExclusionReason) -> bool {
    combat.bounty.exclusion_reasons.contains(&exclusion)
}

#[test]
fn combat_event_semantics_should_emit_one_event_per_source_killed_tuple() {
    let artifact = combat_artifact();
    let event_ids =
        artifact.facts.combat.iter().map(|event| event.fact_id.as_str()).collect::<Vec<_>>();

    assert_eq!(artifact.facts.combat.len(), 6);
    assert!(event_ids.contains(&"event.killed.0"));
}

#[test]
fn combat_event_semantics_should_classify_enemy_kill_as_bounty_eligible() {
    let artifact = combat_artifact();
    let enemy_kill = event_by_id(&artifact, "event.killed.0");

    assert_eq!(enemy_kill.semantic, CombatSemantic::EnemyKill);
    assert_eq!(enemy_kill.bounty.state, BountyEligibilityState::Eligible);
    assert!(enemy_kill.bounty.exclusion_reasons.is_empty());
}

#[test]
fn combat_event_semantics_should_classify_teamkill_suicide_and_null_killer_as_bounty_excluded() {
    let artifact = combat_artifact();
    let teamkill = event_by_id(&artifact, "event.killed.1");
    let suicide = event_by_id(&artifact, "event.killed.2");
    let null_killer = event_by_id(&artifact, "event.killed.3");

    assert_eq!(teamkill.semantic, CombatSemantic::Teamkill);
    assert_eq!(teamkill.bounty.state, BountyEligibilityState::Excluded);
    assert!(has_exclusion(teamkill, BountyExclusionReason::Teamkill));

    assert_eq!(suicide.semantic, CombatSemantic::Suicide);
    assert_eq!(suicide.bounty.state, BountyEligibilityState::Excluded);
    assert!(has_exclusion(suicide, BountyExclusionReason::Suicide));

    assert_eq!(null_killer.semantic, CombatSemantic::NullKillerDeath);
    assert_eq!(null_killer.bounty.state, BountyEligibilityState::Excluded);
    assert!(has_exclusion(null_killer, BountyExclusionReason::NullKiller));
}

#[test]
fn combat_event_semantics_should_classify_vehicle_destroyed_event() {
    let artifact = combat_artifact();
    let vehicle_destroyed_event = event_by_id(&artifact, "event.killed.4");

    assert_eq!(vehicle_destroyed_event.semantic, CombatSemantic::VehicleDestroyed);
    assert_eq!(vehicle_destroyed_event.bounty.state, BountyEligibilityState::Excluded);
    assert!(has_exclusion(vehicle_destroyed_event, BountyExclusionReason::VehicleVictim));
}

#[test]
fn combat_event_semantics_should_emit_unknown_event_and_partial_status_for_missing_actor() {
    let artifact = combat_artifact();
    let unknown_event = event_by_id(&artifact, "event.killed.5");
    let diagnostic_codes =
        artifact.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<Vec<_>>();

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert_eq!(unknown_event.semantic, CombatSemantic::Unknown);
    assert!(has_exclusion(unknown_event, BountyExclusionReason::UnknownActor));
    assert!(unknown_event.legacy_counter_effects.is_empty());
    assert!(diagnostic_codes.contains(&"event.killed_actor_unknown"));
}

#[test]
fn combat_event_semantics_should_emit_unknown_events_and_diagnostics_for_malformed_killed_tuples() {
    let artifact = parse_fixture(MALFORMED_KILLED_EVENTS_FIXTURE);
    let malformed_frame = event_by_id(&artifact, "event.killed.0");
    let malformed_kill_info = event_by_id(&artifact, "event.killed.1");
    let diagnostic_codes =
        artifact.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<Vec<_>>();

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert_eq!(malformed_frame.semantic, CombatSemantic::Unknown);
    assert!(matches!(
        malformed_frame.frame,
        FieldPresence::Unknown {
            reason: parser_contract::presence::UnknownReason::SchemaDrift,
            source: Some(_)
        }
    ));
    assert_eq!(malformed_kill_info.semantic, CombatSemantic::Unknown);
    assert_eq!(
        diagnostic_codes.iter().filter(|code| **code == "event.killed_shape_unknown").count(),
        2
    );
}

#[test]
fn combat_event_semantics_should_include_source_refs_with_event_coordinates() {
    let artifact = combat_artifact();
    let enemy_kill = event_by_id(&artifact, "event.killed.0");
    let source_ref =
        enemy_kill.source_refs.as_slice().first().expect("combat fact should include source ref");

    assert_eq!(source_ref.frame, Some(10));
    assert_eq!(source_ref.event_index, Some(0));
    assert_eq!(source_ref.json_path.as_deref(), Some("$.events[0]"));
    assert_eq!(
        source_ref.rule_id.as_ref().map(|rule_id| rule_id.as_str()),
        Some("event.killed.enemy")
    );
}

#[test]
fn combat_event_semantics_should_keep_ambiguous_or_non_player_actor_cases_auditable() {
    // Arrange
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

    // Act
    let artifact = parse_fixture(fixture);
    let missing_victim = event_by_id(&artifact, "event.killed.0");
    let null_non_player_victim = event_by_id(&artifact, "event.killed.1");
    let incomplete_side = event_by_id(&artifact, "event.killed.2");
    let vehicle_destroyed = event_by_id(&artifact, "event.killed.3");
    let non_player_killer = event_by_id(&artifact, "event.killed.4");
    let diagnostic_messages = artifact
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(artifact.status, ParseStatus::Partial);
    assert_eq!(missing_victim.semantic, CombatSemantic::Unknown);
    assert_eq!(null_non_player_victim.semantic, CombatSemantic::Unknown);
    assert_eq!(incomplete_side.semantic, CombatSemantic::Unknown);
    assert_eq!(vehicle_destroyed.semantic, CombatSemantic::VehicleDestroyed);
    assert_eq!(non_player_killer.semantic, CombatSemantic::Unknown);
    assert!(has_exclusion(missing_victim, BountyExclusionReason::UnknownActor));
    assert!(has_exclusion(null_non_player_victim, BountyExclusionReason::UnknownActor));
    assert!(has_exclusion(incomplete_side, BountyExclusionReason::UnknownActor));
    assert!(has_exclusion(non_player_killer, BountyExclusionReason::UnknownActor));
    assert!(
        diagnostic_messages
            .iter()
            .any(|message| { message.contains("explicit null killer and a non-player victim") })
    );
    assert!(
        diagnostic_messages
            .iter()
            .any(|message| { message.contains("no numeric victim entity identifier") })
    );
    assert!(
        diagnostic_messages
            .iter()
            .any(|message| { message.contains("player sides are incomplete") })
    );
    assert!(
        diagnostic_messages
            .iter()
            .any(|message| { message.contains("not auditable as a player combat event") })
    );
}
