//! Parser-core replay metadata normalization tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    metadata::ReplayMetadata,
    presence::{FieldPresence, UnknownReason},
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay, parse_replay_debug};
use serde_json::json;

const VALID_MINIMAL_FIXTURE: &[u8] = include_bytes!("fixtures/valid-minimal.ocap.json");
const METADATA_DRIFT_FIXTURE: &[u8] = include_bytes!("fixtures/metadata-drift.ocap.json");

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

fn parse_fixture(bytes: &[u8]) -> ParseArtifact {
    parse_replay(parser_input(bytes))
}

const fn replay_metadata(artifact: &ParseArtifact) -> &ReplayMetadata {
    artifact.replay.as_ref().expect("artifact should include replay metadata")
}

#[test]
fn metadata_normalization_should_populate_observed_top_level_fields_when_fixture_is_valid() {
    let artifact = parse_fixture(VALID_MINIMAL_FIXTURE);
    let replay = replay_metadata(&artifact);

    assert_eq!(artifact.status, ParseStatus::Success);
    assert!(artifact.replay.is_some());
    assert!(artifact.diagnostics.is_empty());
    let mission_name_source = replay.mission_name_source();
    assert_eq!(
        mission_name_source
            .as_ref()
            .and_then(|source| source.rule_id.as_ref())
            .map(parser_contract::source_ref::RuleId::as_str),
        Some("metadata.mission_name.observed")
    );
    assert_eq!(
        replay.mission_name,
        FieldPresence::Present {
            value: "sg solid operation".to_string(),
            source: mission_name_source
        }
    );
    assert_eq!(
        replay.world_name,
        FieldPresence::Present { value: "Altis".to_string(), source: replay.world_name_source() }
    );
    assert_eq!(
        replay.players_count,
        FieldPresence::Present { value: vec![0, 12, 10], source: replay.players_count_source() }
    );
}

#[test]
fn metadata_normalization_should_emit_unknown_and_warning_when_metadata_field_has_schema_drift() {
    let artifact = parse_fixture(METADATA_DRIFT_FIXTURE);
    let replay = replay_metadata(&artifact);
    let diagnostic = artifact
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "schema.metadata_field")
        .expect("schema drift should emit a metadata diagnostic");
    let debug_artifact = parse_replay_debug(parser_input(METADATA_DRIFT_FIXTURE));
    let debug_diagnostic = debug_artifact
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "schema.metadata_field")
        .expect("debug artifact should keep full metadata diagnostic");

    assert_eq!(artifact.status, ParseStatus::Partial);
    assert_eq!(diagnostic.parser_action, "set_unknown");
    assert_eq!(debug_diagnostic.json_path.as_deref(), Some("$.playersCount"));
    assert_eq!(debug_diagnostic.expected_shape.as_deref(), Some("array<unsigned integer>"));
    assert_eq!(debug_diagnostic.observed_shape.as_deref(), Some("string"));
    assert_eq!(
        debug_diagnostic
            .source_refs
            .as_slice()
            .first()
            .and_then(|source_ref| source_ref.rule_id.as_ref())
            .map(parser_contract::source_ref::RuleId::as_str),
        Some("diagnostic.schema_drift.metadata")
    );
    assert!(matches!(
        replay.players_count,
        FieldPresence::Unknown { reason: UnknownReason::SchemaDrift, source: Some(_) }
    ));
}

#[test]
fn metadata_normalization_should_emit_unknown_when_metadata_field_is_absent() {
    let artifact = parse_fixture(METADATA_DRIFT_FIXTURE);
    let replay = replay_metadata(&artifact);

    assert!(matches!(
        replay.mission_author,
        FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: Some(_) }
    ));
}

#[test]
fn metadata_normalization_should_compute_frame_and_time_bounds_when_end_frame_and_capture_delay_are_present()
 {
    let artifact = parse_fixture(VALID_MINIMAL_FIXTURE);
    let replay = replay_metadata(&artifact);

    let (frame_bounds, time_bounds) = match (&replay.frame_bounds, &replay.time_bounds) {
        (
            FieldPresence::Present { value: frame_bounds, source: Some(_) },
            FieldPresence::Present { value: time_bounds, source: Some(_) },
        ) => Some((frame_bounds, time_bounds)),
        _ => None,
    }
    .expect("frame and time bounds should be present with source refs");

    assert_eq!(frame_bounds.end_frame, 120);
    assert_eq!(time_bounds.end_seconds.map(f64::to_bits), Some(60.0_f64.to_bits()));
}

trait ReplayMetadataSources {
    fn mission_name_source(&self) -> Option<parser_contract::source_ref::SourceRef>;
    fn players_count_source(&self) -> Option<parser_contract::source_ref::SourceRef>;
    fn world_name_source(&self) -> Option<parser_contract::source_ref::SourceRef>;
}

impl ReplayMetadataSources for ReplayMetadata {
    fn mission_name_source(&self) -> Option<parser_contract::source_ref::SourceRef> {
        match &self.mission_name {
            FieldPresence::Present { source, .. } => source.clone(),
            _ => None,
        }
    }

    fn players_count_source(&self) -> Option<parser_contract::source_ref::SourceRef> {
        match &self.players_count {
            FieldPresence::Present { source, .. } => source.clone(),
            _ => None,
        }
    }

    fn world_name_source(&self) -> Option<parser_contract::source_ref::SourceRef> {
        match &self.world_name {
            FieldPresence::Present { source, .. } => source.clone(),
            _ => None,
        }
    }
}
