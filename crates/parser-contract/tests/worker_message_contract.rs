//! Worker message contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use parser_contract::{
    failure::{ErrorCode, ParseFailure, ParseStage, Retryability},
    presence::{FieldPresence, UnknownReason},
    source_ref::{RuleId, SourceChecksum, SourceRef, SourceRefs},
    version::{ContractVersion, ParserInfo},
    worker::{
        ArtifactReference, ParseCompletedMessage, ParseFailedMessage, ParseJobMessage,
        ParseResultMessage,
    },
};
use semver::Version;
use serde_json::{Value, json};

fn checksum(value: &str) -> SourceChecksum {
    SourceChecksum::sha256(value).expect("test checksum should be valid")
}

fn source_checksum() -> SourceChecksum {
    checksum("1111111111111111111111111111111111111111111111111111111111111111")
}

fn artifact_checksum() -> SourceChecksum {
    checksum("2222222222222222222222222222222222222222222222222222222222222222")
}

const fn present<T>(value: T) -> FieldPresence<T> {
    FieldPresence::Present { value, source: None }
}

const fn unknown<T>() -> FieldPresence<T> {
    FieldPresence::Unknown { reason: UnknownReason::SourceFieldAbsent, source: None }
}

fn parser_info() -> ParserInfo {
    ParserInfo {
        name: "replay-parser-2".to_owned(),
        version: Version::parse("0.1.0").expect("test parser version should be valid"),
        build: None,
    }
}

fn artifact_reference() -> ArtifactReference {
    ArtifactReference {
        bucket: "solid-stats-replays".to_owned(),
        key: "artifacts/v3/replay-0001/1111111111111111111111111111111111111111111111111111111111111111.json"
            .to_owned(),
    }
}

fn failure_source_ref() -> SourceRef {
    SourceRef {
        replay_id: Some("replay-0001".to_owned()),
        source_file: Some("raw/replay-0001.ocap.json".to_owned()),
        checksum: Some(source_checksum()),
        frame: None,
        event_index: None,
        entity_id: None,
        json_path: Some("$".to_owned()),
        rule_id: Some(
            RuleId::new("failure.schema.unsupported_contract_version")
                .expect("test rule ID should be valid"),
        ),
    }
}

fn schema_failure() -> ParseFailure {
    ParseFailure {
        job_id: present("job-0001".to_owned()),
        replay_id: present("replay-0001".to_owned()),
        source_file: present("raw/replay-0001.ocap.json".to_owned()),
        checksum: present(source_checksum()),
        stage: ParseStage::Schema,
        error_code: ErrorCode::new("unsupported.contract_version")
            .expect("test error code should be valid"),
        message: "unsupported parser contract version".to_owned(),
        retryability: Retryability::NotRetryable,
        source_cause: unknown(),
        source_refs: SourceRefs::new(vec![failure_source_ref()])
            .expect("source refs should be non-empty"),
    }
}

fn valid_job_json() -> Value {
    json!({
        "job_id": "job-0001",
        "replay_id": "replay-0001",
        "object_key": "raw/replay-0001.ocap.json",
        "checksum": {
            "algorithm": "sha256",
            "value": "1111111111111111111111111111111111111111111111111111111111111111"
        },
        "parser_contract_version": "3.0.0"
    })
}

#[test]
fn worker_message_contract_valid_parse_job_json_should_deserialize_with_required_fields() {
    let job: ParseJobMessage =
        serde_json::from_value(valid_job_json()).expect("valid parse job should deserialize");

    assert_eq!(job.job_id, "job-0001");
    assert_eq!(job.replay_id, "replay-0001");
    assert_eq!(job.object_key, "raw/replay-0001.ocap.json");
    assert_eq!(job.checksum, source_checksum());
    assert_eq!(job.parser_contract_version, ContractVersion::current());
}

#[test]
fn worker_message_contract_parse_job_json_should_reject_missing_required_fields() {
    for missing_field in
        ["job_id", "replay_id", "object_key", "checksum", "parser_contract_version"]
    {
        let mut job = valid_job_json();
        let removed =
            job.as_object_mut().expect("job fixture should be an object").remove(missing_field);

        assert!(removed.is_some(), "fixture should include {missing_field}");
        assert!(
            serde_json::from_value::<ParseJobMessage>(job).is_err(),
            "missing {missing_field} should be rejected"
        );
    }
}

#[test]
fn worker_message_contract_completed_constructor_should_serialize_artifact_reference_and_proof() {
    let message = ParseCompletedMessage::new(
        "job-0001".to_owned(),
        "replay-0001".to_owned(),
        ContractVersion::current(),
        source_checksum(),
        artifact_reference(),
        artifact_checksum(),
        1234,
        parser_info(),
    );

    let serialized = serde_json::to_value(message).expect("completed message should serialize");

    assert_eq!(serialized["message_type"], "parse.completed");
    assert_eq!(serialized["artifact"]["bucket"], "solid-stats-replays");
    assert_eq!(
        serialized["artifact"]["key"],
        "artifacts/v3/replay-0001/1111111111111111111111111111111111111111111111111111111111111111.json"
    );
    assert_eq!(serialized["artifact_checksum"]["algorithm"], "sha256");
    assert_eq!(
        serialized["artifact_checksum"]["value"],
        "2222222222222222222222222222222222222222222222222222222222222222"
    );
    assert_eq!(serialized["artifact_size_bytes"], 1234);
}

#[test]
fn worker_message_contract_completed_should_reject_failed_message_type() {
    let message = ParseCompletedMessage::new(
        "job-0001".to_owned(),
        "replay-0001".to_owned(),
        ContractVersion::current(),
        source_checksum(),
        artifact_reference(),
        artifact_checksum(),
        1234,
        parser_info(),
    );
    let mut serialized = serde_json::to_value(message).expect("completed message should serialize");
    serialized["message_type"] = json!("parse.failed");

    assert!(serde_json::from_value::<ParseCompletedMessage>(serialized.clone()).is_err());
    assert!(serde_json::from_value::<ParseResultMessage>(serialized).is_err());
}

#[test]
fn worker_message_contract_failed_constructor_should_preserve_unknown_malformed_job_fields() {
    let message = ParseFailedMessage::new(
        unknown(),
        unknown(),
        unknown(),
        unknown(),
        unknown(),
        schema_failure(),
        parser_info(),
    );

    let serialized = serde_json::to_value(message).expect("failed message should serialize");

    assert_eq!(serialized["message_type"], "parse.failed");
    for field in ["job_id", "replay_id", "object_key", "parser_contract_version", "source_checksum"]
    {
        assert_eq!(serialized[field]["state"], "unknown");
        assert_eq!(serialized[field]["reason"], "source_field_absent");
    }
}

#[test]
fn worker_message_contract_failed_should_reject_completed_message_type() {
    let message = ParseFailedMessage::new(
        present("job-0001".to_owned()),
        present("replay-0001".to_owned()),
        present("raw/replay-0001.ocap.json".to_owned()),
        present(ContractVersion::current()),
        present(source_checksum()),
        schema_failure(),
        parser_info(),
    );
    let mut serialized = serde_json::to_value(message).expect("failed message should serialize");
    serialized["message_type"] = json!("parse.completed");

    assert!(serde_json::from_value::<ParseFailedMessage>(serialized.clone()).is_err());
    assert!(serde_json::from_value::<ParseResultMessage>(serialized).is_err());
}

#[test]
fn worker_message_contract_unsupported_version_helper_should_emit_non_retryable_schema_failure() {
    let message = ParseFailedMessage::unsupported_contract_version(
        present("job-0001".to_owned()),
        present("replay-0001".to_owned()),
        present("raw/replay-0001.ocap.json".to_owned()),
        present(ContractVersion::parse("9.0.0").expect("test contract version should parse")),
        present(source_checksum()),
        parser_info(),
    )
    .expect("unsupported-version failure should be constructed");

    let serialized = serde_json::to_value(message).expect("failed message should serialize");

    assert_eq!(serialized["message_type"], "parse.failed");
    assert_eq!(serialized["failure"]["error_code"], "unsupported.contract_version");
    assert_eq!(serialized["failure"]["stage"], "schema");
    assert_eq!(serialized["failure"]["retryability"], "not_retryable");
}
