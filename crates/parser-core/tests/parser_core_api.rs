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
use parser_core::{
    ParserInput, ParserOptions, parse_replay, public_parse_artifact, public_parse_replay,
};
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

    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(artifact.produced_at, None);
    assert!(artifact.replay.is_some());
    assert!(artifact.diagnostics.is_empty());
    assert!(artifact.players.is_empty());
    assert!(artifact.weapons.is_empty());
    assert!(artifact.destroyed_vehicles.is_empty());
    assert_eq!(artifact.failure, None);
}

#[test]
fn public_parse_replay_should_match_public_parse_artifact_wrapper() {
    let bytes = br#"{"missionName":"Operation Copper","worldName":"Altis","entities":[]}"#;

    let public_from_replay = public_parse_replay(parser_input(bytes));
    let public_from_artifact = public_parse_artifact(parse_replay(parser_input(bytes)));

    assert_eq!(public_from_replay, public_from_artifact);
}

#[test]
fn public_parse_replay_should_strip_replay_metadata_sources() {
    let bytes = br#"{"missionName":"Operation Copper","worldName":"Altis","entities":[]}"#;

    let internal_artifact = parse_replay(parser_input(bytes));
    let public_artifact = public_parse_replay(parser_input(bytes));
    let internal_replay = internal_artifact.replay.expect("internal replay metadata should exist");
    let public_replay = public_artifact.replay.expect("public replay metadata should exist");

    assert!(field_has_source(&internal_replay.mission_name));
    assert!(!field_has_source(&public_replay.mission_name));
    assert!(!field_has_source(&public_replay.world_name));
    assert!(!field_has_source(&public_replay.mission_author));
    assert!(!field_has_source(&public_replay.players_count));
    assert!(!field_has_source(&public_replay.capture_delay));
    assert!(!field_has_source(&public_replay.end_frame));
    assert!(!field_has_source(&public_replay.time_bounds));
    assert!(!field_has_source(&public_replay.frame_bounds));
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

fn field_has_source<T>(presence: &FieldPresence<T>) -> bool {
    match presence {
        FieldPresence::Present { source, .. }
        | FieldPresence::ExplicitNull { source, .. }
        | FieldPresence::Unknown { source, .. }
        | FieldPresence::Inferred { source, .. } => source.is_some(),
        FieldPresence::NotApplicable { .. } => false,
    }
}
