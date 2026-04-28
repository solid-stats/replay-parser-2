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
    events::{
        BountyEligibilityState, BountyExclusionReason, CombatEventAttributes, CombatSemantic,
        NormalizedEvent, NormalizedEventKind,
    },
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

fn event_by_id<'a>(artifact: &'a ParseArtifact, event_id: &str) -> &'a NormalizedEvent {
    artifact
        .events
        .iter()
        .find(|event| event.event_id == event_id)
        .expect("combat event should be normalized")
}

fn combat(event: &NormalizedEvent) -> &CombatEventAttributes {
    event.combat.as_ref().expect("combat event should include combat attributes")
}

fn has_exclusion(combat: &CombatEventAttributes, exclusion: BountyExclusionReason) -> bool {
    combat.bounty.exclusion_reasons.contains(&exclusion)
}

#[test]
fn combat_event_semantics_should_emit_one_event_per_source_killed_tuple() {
    let artifact = combat_artifact();
    let event_ids = artifact.events.iter().map(|event| event.event_id.as_str()).collect::<Vec<_>>();

    assert_eq!(artifact.events.len(), 6);
    assert!(event_ids.contains(&"event.killed.0"));
}

#[test]
fn combat_event_semantics_should_classify_enemy_kill_as_bounty_eligible() {
    let artifact = combat_artifact();
    let enemy_kill = event_by_id(&artifact, "event.killed.0");
    let combat = combat(enemy_kill);

    assert_eq!(enemy_kill.kind, NormalizedEventKind::Kill);
    assert_eq!(combat.semantic, CombatSemantic::EnemyKill);
    assert_eq!(combat.bounty.state, BountyEligibilityState::Eligible);
    assert!(combat.bounty.exclusion_reasons.is_empty());
}

#[test]
fn combat_event_semantics_should_classify_teamkill_suicide_and_null_killer_as_bounty_excluded() {
    let artifact = combat_artifact();
    let teamkill = combat(event_by_id(&artifact, "event.killed.1"));
    let suicide = combat(event_by_id(&artifact, "event.killed.2"));
    let null_killer = combat(event_by_id(&artifact, "event.killed.3"));

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
    let combat = combat(vehicle_destroyed_event);

    assert_eq!(vehicle_destroyed_event.kind, NormalizedEventKind::VehicleKilled);
    assert_eq!(combat.semantic, CombatSemantic::VehicleDestroyed);
    assert_eq!(combat.bounty.state, BountyEligibilityState::Excluded);
    assert!(has_exclusion(combat, BountyExclusionReason::VehicleVictim));
}

#[test]
fn combat_event_semantics_should_emit_unknown_event_and_partial_status_for_missing_actor() {
    let artifact = combat_artifact();
    let unknown_event = event_by_id(&artifact, "event.killed.5");
    let combat = combat(unknown_event);
    let diagnostic_codes =
        artifact.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<Vec<_>>();

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert_eq!(unknown_event.kind, NormalizedEventKind::Unknown);
    assert_eq!(combat.semantic, CombatSemantic::Unknown);
    assert!(has_exclusion(combat, BountyExclusionReason::UnknownActor));
    assert!(combat.legacy_counter_effects.is_empty());
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
    assert_eq!(malformed_frame.kind, NormalizedEventKind::Unknown);
    assert_eq!(combat(malformed_frame).semantic, CombatSemantic::Unknown);
    assert!(matches!(
        malformed_frame.frame,
        FieldPresence::Unknown {
            reason: parser_contract::presence::UnknownReason::SchemaDrift,
            source: Some(_)
        }
    ));
    assert_eq!(malformed_kill_info.kind, NormalizedEventKind::Unknown);
    assert_eq!(combat(malformed_kill_info).semantic, CombatSemantic::Unknown);
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
        enemy_kill.source_refs.as_slice().first().expect("combat event should include source ref");

    assert_eq!(source_ref.frame, Some(10));
    assert_eq!(source_ref.event_index, Some(0));
    assert_eq!(source_ref.json_path.as_deref(), Some("$.events[0]"));
    assert_eq!(
        source_ref.rule_id.as_ref().map(|rule_id| rule_id.as_str()),
        Some("event.killed.enemy")
    );
}
