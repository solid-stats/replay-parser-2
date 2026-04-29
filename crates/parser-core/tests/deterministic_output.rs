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

const AGGREGATE_FIXTURE: &[u8] = include_bytes!("fixtures/aggregate-combat.ocap.json");
const MIXED_UNSORTED_FIXTURE: &[u8] = include_bytes!("fixtures/entities-mixed-unsorted.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source(source_file: &str) -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-deterministic".to_string()),
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

fn parse_fixture(bytes: &[u8]) -> ParseArtifact {
    parse_replay(parser_input(bytes, "fixtures/entities-mixed-unsorted.ocap.json"))
}

fn parse_aggregate_fixture() -> ParseArtifact {
    parse_replay(parser_input(AGGREGATE_FIXTURE, "fixtures/aggregate-combat.ocap.json"))
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
    let entity_ids = artifact
        .participants
        .iter()
        .map(|participant| participant.source_entity_id)
        .collect::<Vec<_>>();

    assert_eq!(entity_ids, vec![10, 20, 30]);
}

#[test]
fn deterministic_output_should_not_include_parser_core_timestamp() {
    let artifact = parse_fixture(MIXED_UNSORTED_FIXTURE);

    assert!(artifact.produced_at.is_none());
}

#[test]
fn deterministic_output_should_serialize_populated_events_and_aggregates_identically() {
    // Arrange
    let first_artifact = parse_aggregate_fixture();
    let second_artifact = parse_aggregate_fixture();

    // Act
    let first_serialized =
        serde_json::to_string(&first_artifact).expect("first artifact should serialize");
    let second_serialized =
        serde_json::to_string(&second_artifact).expect("second artifact should serialize");

    // Assert
    assert_eq!(first_serialized, second_serialized);
    assert!(!first_artifact.facts.combat.is_empty());
    assert!(!first_artifact.facts.aggregate_contributions.is_empty());
    assert!(first_artifact.produced_at.is_none());
}

#[test]
fn deterministic_output_should_keep_aggregate_contributions_sorted_by_id() {
    // Arrange
    let artifact = parse_aggregate_fixture();

    // Act
    let contribution_ids = artifact
        .facts
        .aggregate_contributions
        .iter()
        .map(|contribution| contribution.contribution_id.as_str())
        .collect::<Vec<_>>();
    let mut sorted_contribution_ids = contribution_ids.clone();
    sorted_contribution_ids.sort_unstable();

    // Assert
    assert!(!contribution_ids.is_empty());
    assert_eq!(contribution_ids, sorted_contribution_ids);
}

#[test]
fn deterministic_output_should_keep_projection_keys_sorted() {
    // Arrange
    let artifact = parse_aggregate_fixture();

    // Act
    let projection_keys =
        artifact.summaries.projections.keys().map(String::as_str).collect::<Vec<_>>();
    let mut sorted_projection_keys = projection_keys.clone();
    sorted_projection_keys.sort_unstable();

    // Assert
    assert!(!projection_keys.is_empty());
    assert_eq!(projection_keys, sorted_projection_keys);
}
