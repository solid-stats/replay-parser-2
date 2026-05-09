//! Parser-core replay side facts behavior tests.

#![allow(
    clippy::expect_used,
    clippy::missing_const_for_fn,
    clippy::needless_collect,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    events::EventActorRef,
    identity::EntitySide,
    presence::{FieldPresence, UnknownReason},
    side_facts::{CommanderFactKind, OutcomeStatus},
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{
    DebugParseArtifact, ParserInput, ParserOptions, parse_replay, parse_replay_debug,
};
use serde_json::json;

const SIDE_FACTS_FIXTURE: &[u8] = include_bytes!("fixtures/side-facts.ocap.json");
const MISSING_WINNER_FIXTURE: &[u8] = br#"{
  "missionName": "sg missing side facts",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [0, 0],
  "captureDelay": 0.5,
  "endFrame": 120,
  "entities": [],
  "events": [],
  "Markers": [],
  "EditorMarkers": []
}"#;
const UNRECOGNIZED_WINNER_FIXTURE: &[u8] = br#"{
  "missionName": "sg unrecognized side facts",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [0, 0],
  "captureDelay": 0.5,
  "endFrame": 120,
  "winner": "DRAW",
  "entities": [],
  "events": [],
  "Markers": [],
  "EditorMarkers": []
}"#;
const PADDED_ALIAS_WINNER_FIXTURE: &[u8] = br#"{
  "missionName": "sg padded alias side facts",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [0, 0],
  "captureDelay": 0.5,
  "endFrame": 120,
  "winner": " BLUFOR ",
  "entities": [],
  "events": [],
  "Markers": [],
  "EditorMarkers": []
}"#;
const CONFLICTING_WINNER_FIXTURE: &[u8] = br#"{
  "missionName": "sg conflicting side facts",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [0, 0],
  "captureDelay": 0.5,
  "endFrame": 120,
  "winner": "WEST",
  "outcome": "EAST",
  "entities": [],
  "events": [],
  "Markers": [],
  "EditorMarkers": []
}"#;
const COMMANDER_FALSE_POSITIVE_FIXTURE: &[u8] = br#"{
  "missionName": "sg commander false positives",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [0, 2],
  "captureDelay": 0.5,
  "endFrame": 120,
  "entities": [
    {
      "id": 1,
      "type": "unit",
      "name": "Maksim",
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
      "group": "Alpha 1-2",
      "side": "WEST",
      "description": "Marksman",
      "isPlayer": 1,
      "positions": []
    }
  ],
  "events": [],
  "Markers": [],
  "EditorMarkers": []
}"#;
const KS_MISSION_MESSAGE_FIXTURE: &str = r#"{
  "missionName": "sg ks mission message",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [1, 1],
  "captureDelay": 0.5,
  "endFrame": 120,
  "entities": [
    {
      "id": 1,
      "type": "unit",
      "name": "[SHK]Sota",
      "group": "Alpha 1-1",
      "side": "WEST",
      "description": "Командир",
      "isPlayer": 1,
      "positions": []
    },
    {
      "id": 2,
      "type": "unit",
      "name": "[JTF2]Bas",
      "group": "Bravo 1-1",
      "side": "EAST",
      "description": "Командир",
      "isPlayer": 1,
      "positions": []
    }
  ],
  "events": [
    [902, "mission_message", "Победа КС: [SHK]Sota. Поражение КС: [JTF2]Bas"]
  ],
  "Markers": [],
  "EditorMarkers": []
}"#;
const MULTI_KS_MISSION_MESSAGE_FIXTURE: &str = r#"{
  "missionName": "sg multi ks mission message",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [3, 1],
  "captureDelay": 0.5,
  "endFrame": 120,
  "entities": [
    {
      "id": 1,
      "type": "unit",
      "name": "[RAF]Valar",
      "group": "Alpha 1-1",
      "side": "WEST",
      "description": "Командир",
      "isPlayer": 1,
      "positions": []
    },
    {
      "id": 2,
      "type": "unit",
      "name": "Rollan",
      "group": "Alpha 1-2",
      "side": "WEST",
      "description": "Заместитель командира",
      "isPlayer": 1,
      "positions": []
    },
    {
      "id": 3,
      "type": "unit",
      "name": "[RT]Raiden",
      "group": "Bravo 1-1",
      "side": "EAST",
      "description": "Командир",
      "isPlayer": 1,
      "positions": []
    },
    {
      "id": 4,
      "type": "unit",
      "name": "[RAF]baptized",
      "group": "Bravo 1-2",
      "side": "EAST",
      "description": "Заместитель командира",
      "isPlayer": 1,
      "positions": []
    }
  ],
  "events": [
    [54, "mission_message", "Победа КС: [RAF]Valar, Rollan. Поражение КС: [RT]Raiden, [RAF]baptized"],
    [54, "mission_message", "Победа КС: [RAF]Valar, Rollan. Поражение КС: [RT]Raiden, [RAF]baptized"]
  ],
  "Markers": [],
  "EditorMarkers": []
}"#;
const CONFLICTING_KS_MISSION_MESSAGE_FIXTURE: &str = r#"{
  "missionName": "sg conflicting ks mission message",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [1, 1],
  "captureDelay": 0.5,
  "endFrame": 120,
  "entities": [
    {
      "id": 1,
      "type": "unit",
      "name": "Alpha",
      "group": "Alpha 1-1",
      "side": "WEST",
      "description": "Командир",
      "isPlayer": 1,
      "positions": []
    },
    {
      "id": 2,
      "type": "unit",
      "name": "Bravo",
      "group": "Bravo 1-1",
      "side": "EAST",
      "description": "Командир",
      "isPlayer": 1,
      "positions": []
    }
  ],
  "events": [
    [10, "mission_message", "Победа КС: Alpha, Bravo. Поражение КС: "]
  ],
  "Markers": [],
  "EditorMarkers": []
}"#;
const UNMATCHED_KS_MISSION_MESSAGE_FIXTURE: &str = r#"{
  "missionName": "sg unmatched ks mission message",
  "worldName": "Altis",
  "missionAuthor": "SolidGames",
  "playersCount": [1, 0],
  "captureDelay": 0.5,
  "endFrame": 120,
  "entities": [
    {
      "id": 1,
      "type": "unit",
      "name": "Known",
      "group": "Alpha 1-1",
      "side": "WEST",
      "description": "Командир",
      "isPlayer": 1,
      "positions": []
    }
  ],
  "events": [
    [87, "mission_message", "Победа КС: Missing. Поражение КС: "]
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
        replay_id: Some("replay-side-facts".to_owned()),
        source_file: "fixtures/side-facts.ocap.json".to_owned(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "5555555555555555555555555555555555555555555555555555555555555555",
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

fn parse_fixture(bytes: &[u8]) -> DebugParseArtifact {
    parse_replay_debug(parser_input(bytes))
}

fn present_winner_side(artifact: &DebugParseArtifact) -> Option<EntitySide> {
    match &artifact.side_facts.outcome.winner_side {
        FieldPresence::Present { value, source: Some(_) }
        | FieldPresence::Inferred { value, source: Some(_), .. } => Some(*value),
        FieldPresence::Present { source: None, .. }
        | FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { source: None, .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

fn actor_source_entity_id(actor: &EventActorRef) -> Option<i64> {
    match &actor.source_entity_id {
        FieldPresence::Present { value, source: Some(_) } => Some(*value),
        FieldPresence::Present { source: None, .. }
        | FieldPresence::ExplicitNull { .. }
        | FieldPresence::Unknown { .. }
        | FieldPresence::Inferred { .. }
        | FieldPresence::NotApplicable { .. } => None,
    }
}

#[test]
fn side_facts_should_emit_known_outcome_from_explicit_winner_field() {
    let artifact = parse_fixture(SIDE_FACTS_FIXTURE);

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Known);
    assert_eq!(present_winner_side(&artifact), Some(EntitySide::West));
    assert_eq!(artifact.side_facts.outcome.rule_id.as_str(), "side_facts.outcome.explicit_field");
    assert!(artifact.side_facts.outcome.source_refs.is_some());
}

#[test]
fn side_facts_should_emit_unknown_outcome_without_partial_status_when_winner_missing() {
    let artifact = parse_fixture(MISSING_WINNER_FIXTURE);

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Unknown);
    assert!(matches!(
        artifact.side_facts.outcome.winner_side,
        FieldPresence::Unknown { reason: UnknownReason::MissingWinner, source: None }
    ));
    assert!(artifact.side_facts.outcome.source_refs.is_none());
}

#[test]
fn side_facts_should_emit_commander_candidate_with_confidence_rule_and_source_refs() {
    let artifact = parse_fixture(SIDE_FACTS_FIXTURE);
    let candidate =
        artifact.side_facts.commanders.first().expect("commander candidate should be emitted");
    let source_paths = candidate
        .source_refs
        .as_slice()
        .iter()
        .filter_map(|source_ref| source_ref.json_path.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(artifact.side_facts.commanders.len(), 1);
    assert_eq!(candidate.kind, CommanderFactKind::Candidate);
    assert_eq!(
        candidate.confidence.map(parser_contract::presence::Confidence::get).map(f32::to_bits),
        Some(0.6_f32.to_bits())
    );
    assert_eq!(candidate.rule_id.as_str(), "side_facts.commander.keyword_candidate");
    assert!(source_paths.contains(&"$.entities[0]"));
    assert!(matches!(
        &candidate.side_name,
        FieldPresence::Present { value, source: Some(_) } if value == "west"
    ));
    assert!(matches!(
        &candidate.commander,
        FieldPresence::Present { value, source: Some(_) }
            if actor_source_entity_id(value) == Some(1)
    ));
}

#[test]
fn side_facts_should_not_emit_canonical_commander_identity() {
    let artifact = parse_fixture(SIDE_FACTS_FIXTURE);
    let serialized =
        serde_json::to_string(&artifact.side_facts).expect("side facts should serialize");

    assert!(!serialized.contains("canonical_player_id"));
}

#[test]
fn side_facts_should_warn_but_not_data_loss_for_unrecognized_outcome_value() {
    let artifact = parse_fixture(UNRECOGNIZED_WINNER_FIXTURE);

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Unknown);
    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "side_facts.outcome_unrecognized")
    );
}

#[test]
fn side_facts_should_accept_trimmed_case_insensitive_winner_aliases() {
    let artifact = parse_fixture(PADDED_ALIAS_WINNER_FIXTURE);

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Known);
    assert_eq!(present_winner_side(&artifact), Some(EntitySide::West));
}

#[test]
fn side_facts_should_infer_ks_winner_from_mission_message_event() {
    let artifact = parse_fixture(KS_MISSION_MESSAGE_FIXTURE.as_bytes());
    let commander_entity_ids = artifact
        .side_facts
        .commanders
        .iter()
        .filter_map(|commander| match &commander.commander {
            FieldPresence::Present { value, .. } => actor_source_entity_id(value),
            FieldPresence::ExplicitNull { .. }
            | FieldPresence::Unknown { .. }
            | FieldPresence::Inferred { .. }
            | FieldPresence::NotApplicable { .. } => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Inferred);
    assert_eq!(present_winner_side(&artifact), Some(EntitySide::West));
    assert!(artifact.side_facts.outcome.source_refs.is_some());
    assert!(commander_entity_ids.contains(&1));
    assert!(commander_entity_ids.contains(&2));
    assert!(artifact.side_facts.commanders.iter().any(|commander| {
        commander.kind == CommanderFactKind::Observed
            && commander.rule_id.as_str() == "side_facts.commander.mission_message"
    }));
}

#[test]
fn side_facts_should_emit_mission_message_facts_in_default_artifact() {
    let artifact = parse_replay(parser_input(KS_MISSION_MESSAGE_FIXTURE.as_bytes()));

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Inferred);
    assert_eq!(artifact.side_facts.commanders.len(), 2);
}

#[test]
fn side_facts_should_dedupe_duplicate_mission_message_commanders() {
    let artifact = parse_fixture(MULTI_KS_MISSION_MESSAGE_FIXTURE.as_bytes());
    let commander_entity_ids = artifact
        .side_facts
        .commanders
        .iter()
        .filter_map(|commander| match &commander.commander {
            FieldPresence::Present { value, .. } => actor_source_entity_id(value),
            FieldPresence::ExplicitNull { .. }
            | FieldPresence::Unknown { .. }
            | FieldPresence::Inferred { .. }
            | FieldPresence::NotApplicable { .. } => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Inferred);
    assert_eq!(present_winner_side(&artifact), Some(EntitySide::West));
    assert_eq!(commander_entity_ids, vec![1, 2, 3, 4]);
}

#[test]
fn side_facts_should_emit_partial_unknown_outcome_when_recognized_fields_conflict() {
    let artifact = parse_fixture(CONFLICTING_WINNER_FIXTURE);

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Unknown);
    assert!(matches!(
        artifact.side_facts.outcome.winner_side,
        FieldPresence::Unknown { reason: UnknownReason::MissingWinner, source: None }
    ));
    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "side_facts.outcome_conflict")
    );
}

#[test]
fn side_facts_should_keep_mission_message_outcome_unknown_when_winner_sides_conflict() {
    let artifact = parse_fixture(CONFLICTING_KS_MISSION_MESSAGE_FIXTURE.as_bytes());

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Unknown);
    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "side_facts.outcome_conflict")
    );
}

#[test]
fn side_facts_should_not_fabricate_unmatched_mission_message_commanders() {
    let artifact = parse_fixture(UNMATCHED_KS_MISSION_MESSAGE_FIXTURE.as_bytes());

    assert_eq!(artifact.side_facts.outcome.status, OutcomeStatus::Unknown);
    assert!(artifact.side_facts.commanders.is_empty());
}

#[test]
fn side_facts_should_not_emit_commander_candidate_for_embedded_ks_substrings() {
    let artifact = parse_fixture(COMMANDER_FALSE_POSITIVE_FIXTURE);

    assert!(artifact.side_facts.commanders.is_empty());
}
