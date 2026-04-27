//! Parser-core deterministic output tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::ParseArtifact,
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::json;

const MIXED_UNSORTED_FIXTURE: &[u8] = include_bytes!("fixtures/entities-mixed-unsorted.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-deterministic".to_string()),
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

#[test]
fn deterministic_output_should_serialize_identically_when_same_input_is_parsed_twice() {
    let first_artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let second_artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);

    let first_serialized =
        serde_json::to_string(&first_artifact).expect("first artifact should serialize");
    let second_serialized =
        serde_json::to_string(&second_artifact).expect("second artifact should serialize");

    assert_eq!(first_serialized, second_serialized);
}

#[test]
fn deterministic_output_should_keep_entities_ordered_after_input_entities_are_unsorted() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);
    let entity_ids =
        artifact.entities.iter().map(|entity| entity.source_entity_id).collect::<Vec<_>>();

    assert_eq!(entity_ids, vec![10, 20, 30]);
}

#[test]
fn deterministic_output_should_not_include_parser_core_timestamp() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);

    assert!(artifact.produced_at.is_none());
}
