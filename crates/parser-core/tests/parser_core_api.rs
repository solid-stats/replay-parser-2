//! Parser-core public API behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::ParseStatus,
    failure::ParseStage,
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::json;

const INVALID_JSON_FIXTURE: &[u8] = include_bytes!("fixtures/invalid-json.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-0001".to_string()),
        source_file: "fixtures/replay-0001.ocap.json".to_string(),
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

#[test]
fn parser_core_api_should_return_success_shell_when_root_object_is_valid() {
    let input = parser_input(br#"{"missionName":"Operation Copper","entities":[]}"#);

    let artifact = parse_replay(input);
    let player_results = artifact
        .aggregates
        .projections
        .get("legacy.player_game_results")
        .and_then(serde_json::Value::as_array)
        .expect("player game result projection should be present");
    let bounty_inputs = artifact
        .aggregates
        .projections
        .get("bounty.inputs")
        .and_then(serde_json::Value::as_array)
        .expect("bounty input projection should be present");
    let game_type = artifact
        .aggregates
        .projections
        .get("legacy.game_type_compatibility")
        .and_then(serde_json::Value::as_object)
        .expect("game type compatibility projection should be present");

    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(artifact.produced_at, None);
    assert!(artifact.replay.is_some());
    assert!(artifact.diagnostics.is_empty());
    assert!(artifact.entities.is_empty());
    assert!(artifact.events.is_empty());
    assert!(artifact.aggregates.contributions.is_empty());
    assert!(player_results.is_empty());
    assert!(bounty_inputs.is_empty());
    assert_eq!(game_type["mission_name"], "Operation Copper");
    assert_eq!(game_type["prefix_bucket"], "other");
    assert_eq!(artifact.failure, None);
}

#[test]
fn parser_core_failure_should_return_structured_failure_when_json_is_invalid() {
    let input = parser_input(INVALID_JSON_FIXTURE);

    let artifact = parse_replay(input);
    let failure = artifact.failure.expect("failed artifact should include failure details");
    let source_ref = failure
        .source_refs
        .as_slice()
        .first()
        .expect("failure should include one source reference");

    assert_eq!(artifact.status, ParseStatus::Failed);
    assert_eq!(artifact.produced_at, None);
    assert_eq!(failure.stage, ParseStage::JsonDecode);
    assert_eq!(failure.error_code.as_str(), "json.decode");
    assert_eq!(source_ref.json_path.as_deref(), Some("$"));
    assert_eq!(
        source_ref.rule_id.as_ref().map(parser_contract::source_ref::RuleId::as_str),
        Some("failure.json.decode")
    );
    assert!(matches!(failure.source_cause, FieldPresence::Present { .. }));
}

#[test]
fn parser_core_failure_should_return_structured_failure_when_root_is_not_object() {
    let input = parser_input(br#"["not", "an", "object"]"#);

    let artifact = parse_replay(input);
    let failure = artifact.failure.expect("failed artifact should include failure details");
    let source_ref = failure
        .source_refs
        .as_slice()
        .first()
        .expect("failure should include one source reference");

    assert_eq!(artifact.status, ParseStatus::Failed);
    assert_eq!(failure.stage, ParseStage::Schema);
    assert_eq!(failure.error_code.as_str(), "schema.root_object");
    assert_eq!(
        failure.source_cause,
        FieldPresence::Present {
            value: "OCAP replay root must be a JSON object".to_string(),
            source: None
        }
    );
    assert_eq!(source_ref.json_path.as_deref(), Some("$"));
    assert_eq!(
        source_ref.rule_id.as_ref().map(parser_contract::source_ref::RuleId::as_str),
        Some("failure.schema.root_object")
    );
}

#[test]
fn parser_core_api_should_not_populate_produced_at_when_parser_core_runs() {
    let input = parser_input(br#"{"entities":[]}"#);

    let artifact = parse_replay(input);

    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(artifact.produced_at, None);
}
