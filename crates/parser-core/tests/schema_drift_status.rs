//! Parser-core schema drift status and diagnostic cap tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay, parse_replay_debug};
use serde_json::json;

const VALID_MINIMAL_FIXTURE: &[u8] = include_bytes!("fixtures/valid-minimal.ocap.json");
const METADATA_DRIFT_FIXTURE: &[u8] = include_bytes!("fixtures/metadata-drift.ocap.json");
const ENTITY_DRIFT_FIXTURE: &[u8] = include_bytes!("fixtures/entities-drift.ocap.json");
const DIAGNOSTIC_CAP_FIXTURE: &[u8] = include_bytes!("fixtures/diagnostic-cap.ocap.json");

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": "0.1.0"
    }))
    .expect("test parser info should be valid")
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-schema-drift".to_string()),
        source_file: "fixtures/schema-drift.ocap.json".to_string(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .expect("test checksum should be valid"),
            source: None,
        },
    }
}

fn parser_input(bytes: &[u8], options: ParserOptions) -> ParserInput<'_> {
    ParserInput { bytes, source: replay_source(), parser: parser_info(), options }
}

fn parse_fixture(bytes: &[u8]) -> ParseArtifact {
    parse_replay(parser_input(bytes, ParserOptions::default()))
}

fn parse_fixture_with_options(bytes: &[u8], options: ParserOptions) -> ParseArtifact {
    parse_replay(parser_input(bytes, options))
}

#[test]
fn schema_drift_status_should_mark_artifact_partial_when_metadata_drift_causes_unknown() {
    let artifact = parse_fixture(METADATA_DRIFT_FIXTURE);

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(
        artifact.diagnostics.iter().any(|diagnostic| diagnostic.code == "schema.metadata_field")
    );
}

#[test]
fn schema_drift_status_should_mark_artifact_partial_when_entity_row_is_dropped() {
    let artifact = parse_fixture(ENTITY_DRIFT_FIXTURE);

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert!(
        artifact.diagnostics.iter().any(|diagnostic| diagnostic.parser_action == "drop_entity")
    );
}

#[test]
fn schema_drift_status_should_keep_artifact_success_when_no_data_loss_diagnostics_exist() {
    let artifact = parse_fixture(VALID_MINIMAL_FIXTURE);

    assert_eq!(artifact.status, ParseStatus::Success);
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn schema_drift_status_should_append_summary_diagnostic_when_diagnostic_limit_is_exceeded() {
    let artifact =
        parse_fixture_with_options(DIAGNOSTIC_CAP_FIXTURE, ParserOptions { diagnostic_limit: 2 });
    let debug_artifact = parse_replay_debug(parser_input(
        DIAGNOSTIC_CAP_FIXTURE,
        ParserOptions { diagnostic_limit: 2 },
    ));
    let summary = artifact
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "diagnostic.limit_exceeded")
        .expect("diagnostic cap should emit a summary diagnostic");
    let debug_summary = debug_artifact
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "diagnostic.limit_exceeded")
        .expect("debug artifact should keep full summary diagnostic");

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert_eq!(artifact.diagnostics.len(), 3);
    assert!(summary.message.contains('3'));
    assert_eq!(debug_summary.json_path.as_deref(), Some("$"));
    assert_eq!(summary.parser_action, "summarized_repeated_diagnostics");
}
