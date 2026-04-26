//! Parse failure contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::collections::BTreeMap;

use parser_contract::{
    aggregates::AggregateSection,
    artifact::{ParseArtifact, ParseArtifactError, ParseStatus},
    failure::{ErrorCode, ParseFailure, ParseStage, Retryability},
    presence::{FieldPresence, UnknownReason},
    source_ref::{ReplaySource, RuleId, SourceChecksum, SourceRef, SourceRefs},
    version::{ContractVersion, ParserInfo},
};
use semver::Version;

fn checksum() -> SourceChecksum {
    SourceChecksum::sha256("0000000000000000000000000000000000000000000000000000000000000000")
        .expect("test checksum should be valid")
}

const fn present<T>(value: T) -> FieldPresence<T> {
    FieldPresence::Present { value, source: None }
}

fn parser_info() -> ParserInfo {
    ParserInfo {
        name: "replay-parser-2".to_string(),
        version: Version::parse("0.1.0").expect("test parser version should be valid"),
        build: None,
    }
}

fn replay_source() -> ReplaySource {
    ReplaySource {
        replay_id: Some("replay-0001".to_string()),
        source_file: "2025_04_05__23_27_21__1_ocap.json".to_string(),
        checksum: present(checksum()),
    }
}

fn source_ref() -> SourceRef {
    SourceRef {
        replay_id: Some("replay-0001".to_string()),
        source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
        checksum: Some(checksum()),
        frame: None,
        event_index: None,
        entity_id: None,
        json_path: Some("$".to_string()),
        rule_id: Some(RuleId::new("failure.json.decode").expect("test rule ID should be valid")),
    }
}

fn json_decode_failure() -> ParseFailure {
    ParseFailure {
        job_id: present("job-0001".to_string()),
        replay_id: present("replay-0001".to_string()),
        source_file: present("2025_04_05__23_27_21__1_ocap.json".to_string()),
        checksum: present(checksum()),
        stage: ParseStage::JsonDecode,
        error_code: ErrorCode::new("json.decode").expect("test error code should be valid"),
        message: "Replay JSON could not be decoded".to_string(),
        retryability: Retryability::NotRetryable,
        source_cause: present("expected value at line 1 column 1".to_string()),
        source_refs: SourceRefs::new(vec![source_ref()]).expect("source refs should be non-empty"),
    }
}

#[test]
fn failure_contract_parse_failure_should_serialize_structured_retryability_and_stage_when_json_decode_fails()
 {
    let failure = json_decode_failure();

    let serialized = serde_json::to_value(&failure).expect("failure should serialize");

    assert_eq!(serialized["job_id"]["value"], "job-0001");
    assert_eq!(serialized["replay_id"]["value"], "replay-0001");
    assert_eq!(serialized["source_file"]["value"], "2025_04_05__23_27_21__1_ocap.json");
    assert_eq!(serialized["checksum"]["value"]["algorithm"], "sha256");
    assert_eq!(serialized["stage"], "json_decode");
    assert_eq!(serialized["error_code"], "json.decode");
    assert_eq!(serialized["retryability"], "not_retryable");
    assert_eq!(serialized["source_cause"]["value"], "expected value at line 1 column 1");
    assert_eq!(serialized["source_refs"][0]["json_path"], "$");
}

#[test]
fn failure_contract_error_code_should_accept_checksum_and_output_families_when_stage_requires_them()
{
    for valid_error_code in ["json.decode", "checksum.mismatch", "output.write_failed"] {
        assert_eq!(
            ErrorCode::new(valid_error_code).expect("valid error code should be accepted").as_str(),
            valid_error_code
        );
    }
}

#[test]
fn failure_contract_error_code_should_reject_unknown_family_when_value_is_not_namespaced() {
    for invalid_error_code in
        ["", "decode", "network.timeout", "json.", "json.Decode", "json.decode!", "io.read."]
    {
        assert!(
            ErrorCode::new(invalid_error_code).is_err(),
            "{invalid_error_code:?} should be rejected"
        );
    }
}

#[test]
fn failure_contract_error_code_should_reject_empty_dotted_segments() {
    for invalid_error_code in ["json..decode", ".json.decode", "json.decode."] {
        assert!(
            ErrorCode::new(invalid_error_code).is_err(),
            "{invalid_error_code:?} should be rejected"
        );
    }
}

#[test]
fn failure_contract_input_stage_failure_should_represent_unavailable_checksum_as_unknown() {
    let failure = ParseFailure {
        job_id: present("job-0002".to_string()),
        replay_id: present("replay-0002".to_string()),
        source_file: FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: None,
        },
        checksum: FieldPresence::Unknown {
            reason: UnknownReason::ChecksumUnavailable,
            source: None,
        },
        stage: ParseStage::Input,
        error_code: ErrorCode::new("io.read").expect("test error code should be valid"),
        message: "Replay file could not be read".to_string(),
        retryability: Retryability::Retryable,
        source_cause: FieldPresence::Unknown {
            reason: UnknownReason::SourceFieldAbsent,
            source: None,
        },
        source_refs: SourceRefs::new(vec![SourceRef {
            replay_id: Some("replay-0002".to_string()),
            source_file: None,
            checksum: None,
            frame: None,
            event_index: None,
            entity_id: None,
            json_path: None,
            rule_id: Some(RuleId::new("failure.io.read").expect("test rule ID should be valid")),
        }])
        .expect("source refs should be non-empty"),
    };

    let serialized = serde_json::to_value(&failure).expect("failure should serialize");

    assert_eq!(serialized["checksum"]["state"], "unknown");
    assert_eq!(serialized["checksum"]["reason"], "checksum_unavailable");
}

#[test]
fn failure_contract_failed_artifact_should_carry_status_failed_and_failure_object_together() {
    let artifact = ParseArtifact {
        contract_version: ContractVersion::current(),
        parser: parser_info(),
        source: replay_source(),
        status: ParseStatus::Failed,
        produced_at: None,
        diagnostics: Vec::new(),
        replay: None,
        entities: Vec::new(),
        events: Vec::new(),
        aggregates: AggregateSection::default(),
        failure: Some(json_decode_failure()),
        extensions: BTreeMap::new(),
    };

    let serialized = serde_json::to_value(&artifact).expect("artifact should serialize");
    let deserialized: ParseArtifact =
        serde_json::from_value(serialized.clone()).expect("artifact should deserialize");

    assert_eq!(serialized["status"], "failed");
    assert_eq!(serialized["failure"]["error_code"], "json.decode");
    assert_eq!(serialized["failure"]["stage"], "json_decode");
    assert!(deserialized.validate_status_payload().is_ok());
    assert_eq!(
        deserialized.failure.expect("failed artifact should include failure").retryability,
        Retryability::NotRetryable
    );
}

#[test]
fn artifact_envelope_failed_artifact_should_require_failure_when_status_is_failed() {
    let artifact = ParseArtifact {
        contract_version: ContractVersion::current(),
        parser: parser_info(),
        source: replay_source(),
        status: ParseStatus::Failed,
        produced_at: None,
        diagnostics: Vec::new(),
        replay: None,
        entities: Vec::new(),
        events: Vec::new(),
        aggregates: AggregateSection::default(),
        failure: None,
        extensions: BTreeMap::new(),
    };

    let result = artifact.validate_status_payload();

    assert_eq!(result, Err(ParseArtifactError::MissingFailure));
}

#[test]
fn artifact_envelope_success_artifact_should_reject_failure_when_status_is_not_failed() {
    let artifact = ParseArtifact {
        contract_version: ContractVersion::current(),
        parser: parser_info(),
        source: replay_source(),
        status: ParseStatus::Success,
        produced_at: None,
        diagnostics: Vec::new(),
        replay: None,
        entities: Vec::new(),
        events: Vec::new(),
        aggregates: AggregateSection::default(),
        failure: Some(json_decode_failure()),
        extensions: BTreeMap::new(),
    };

    let result = artifact.validate_status_payload();

    assert_eq!(result, Err(ParseArtifactError::UnexpectedFailure));
}
