//! Parser-core public API behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::collections::BTreeMap;

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    failure::ParseStage,
    metadata::{FrameBounds, ReplayMetadata},
    presence::{Confidence, FieldPresence, NullReason, UnknownReason},
    side_facts::ReplaySideFacts,
    source_ref::{ReplaySource, RuleId, SourceChecksum, SourceRef},
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

fn source_ref() -> SourceRef {
    SourceRef {
        replay_id: Some("replay-0001".to_owned()),
        source_file: Some("fixtures/replay-0001.ocap.json".to_owned()),
        checksum: None,
        frame: None,
        event_index: None,
        entity_id: None,
        json_path: Some("$".to_owned()),
        rule_id: Some(RuleId::new("metadata.test").expect("test rule ID should be valid")),
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
fn public_parse_artifact_should_strip_all_source_bearing_metadata_presence_variants() {
    let source = source_ref();
    let artifact = ParseArtifact {
        contract_version: parser_contract::version::ContractVersion::current(),
        parser: parser_info(),
        source: replay_source(),
        status: ParseStatus::Partial,
        produced_at: None,
        diagnostics: Vec::new(),
        replay: Some(ReplayMetadata {
            mission_name: FieldPresence::Present {
                value: "Operation Copper".to_owned(),
                source: Some(source.clone()),
            },
            world_name: FieldPresence::ExplicitNull {
                reason: NullReason::EmptyValue,
                source: Some(source.clone()),
            },
            mission_author: FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: Some(source.clone()),
            },
            players_count: FieldPresence::Inferred {
                value: vec![2],
                reason: "test inference".to_owned(),
                confidence: Some(Confidence::new(0.5).expect("test confidence should be valid")),
                source: Some(source.clone()),
                rule_id: RuleId::new("metadata.players_count.inferred")
                    .expect("test rule ID should be valid"),
            },
            capture_delay: FieldPresence::NotApplicable {
                reason: "not present in this fixture".to_owned(),
            },
            end_frame: FieldPresence::Present { value: 120, source: Some(source.clone()) },
            time_bounds: FieldPresence::Unknown {
                reason: UnknownReason::SourceFieldAbsent,
                source: Some(source.clone()),
            },
            frame_bounds: FieldPresence::Inferred {
                value: FrameBounds { start_frame: 0, end_frame: 120 },
                reason: "test bounds".to_owned(),
                confidence: None,
                source: Some(source),
                rule_id: RuleId::new("metadata.frame_bounds.inferred")
                    .expect("test rule ID should be valid"),
            },
        }),
        players: Vec::new(),
        weapons: Vec::new(),
        destroyed_vehicles: Vec::new(),
        side_facts: ReplaySideFacts::default(),
        failure: None,
        extensions: BTreeMap::new(),
    };

    let public_artifact = public_parse_artifact(artifact);
    let replay = public_artifact.replay.expect("public replay metadata should remain present");

    assert!(!field_has_source(&replay.mission_name));
    assert!(!field_has_source(&replay.world_name));
    assert!(!field_has_source(&replay.mission_author));
    assert!(!field_has_source(&replay.players_count));
    assert!(!field_has_source(&replay.capture_delay));
    assert!(!field_has_source(&replay.end_frame));
    assert!(!field_has_source(&replay.time_bounds));
    assert!(!field_has_source(&replay.frame_bounds));
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
    assert_eq!(source_ref.rule_id.as_ref().map(RuleId::as_str), Some("failure.json.decode"));
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
    assert_eq!(source_ref.rule_id.as_ref().map(RuleId::as_str), Some("failure.schema.root_object"));
}

#[test]
fn parser_core_api_should_not_populate_produced_at_when_parser_core_runs() {
    let input = parser_input(br#"{"entities":[]}"#);

    let artifact = parse_replay(input);

    assert_eq!(artifact.status, ParseStatus::Success);
    assert_eq!(artifact.produced_at, None);
}

const fn field_has_source<T>(presence: &FieldPresence<T>) -> bool {
    match presence {
        FieldPresence::Present { source, .. }
        | FieldPresence::ExplicitNull { source, .. }
        | FieldPresence::Unknown { source, .. }
        | FieldPresence::Inferred { source, .. } => source.is_some(),
        FieldPresence::NotApplicable { .. } => false,
    }
}
