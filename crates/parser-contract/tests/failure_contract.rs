use std::collections::BTreeMap;

use parser_contract::{
    aggregates::AggregateSection,
    artifact::{ParseArtifact, ParseStatus},
    failure::{ErrorCode, ParseFailure, ParseStage, Retryability},
    source_ref::{ReplaySource, RuleId, SourceChecksum, SourceRef},
    version::{ContractVersion, ParserInfo},
};
use semver::Version;

fn checksum() -> SourceChecksum {
    SourceChecksum {
        algorithm: "sha256".to_string(),
        value: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
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
        checksum: checksum(),
    }
}

fn json_decode_failure() -> ParseFailure {
    ParseFailure {
        job_id: Some("job-0001".to_string()),
        replay_id: Some("replay-0001".to_string()),
        source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
        checksum: Some(checksum()),
        stage: ParseStage::JsonDecode,
        error_code: ErrorCode::new("json.decode").expect("test error code should be valid"),
        message: "Replay JSON could not be decoded".to_string(),
        retryability: Retryability::NotRetryable,
        source_cause: Some("expected value at line 1 column 1".to_string()),
        source_refs: vec![SourceRef {
            replay_id: Some("replay-0001".to_string()),
            source_file: Some("2025_04_05__23_27_21__1_ocap.json".to_string()),
            checksum: Some(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            ),
            frame: None,
            event_index: None,
            entity_id: None,
            json_path: Some("$".to_string()),
            rule_id: Some(
                RuleId::new("failure.json.decode").expect("test rule ID should be valid"),
            ),
        }],
    }
}

#[test]
fn failure_contract_parse_failure_should_serialize_structured_retryability_and_stage_when_json_decode_fails()
 {
    let failure = json_decode_failure();

    let serialized = serde_json::to_value(&failure).expect("failure should serialize");

    assert_eq!(serialized["job_id"], "job-0001");
    assert_eq!(serialized["replay_id"], "replay-0001");
    assert_eq!(
        serialized["source_file"],
        "2025_04_05__23_27_21__1_ocap.json"
    );
    assert_eq!(serialized["checksum"]["algorithm"], "sha256");
    assert_eq!(serialized["stage"], "json_decode");
    assert_eq!(serialized["error_code"], "json.decode");
    assert_eq!(serialized["retryability"], "not_retryable");
    assert_eq!(
        serialized["source_cause"],
        "expected value at line 1 column 1"
    );
    assert_eq!(serialized["source_refs"][0]["json_path"], "$");
}

#[test]
fn failure_contract_error_code_should_reject_unknown_family_when_value_is_not_namespaced() {
    assert_eq!(
        ErrorCode::new("json.decode")
            .expect("valid json error code should be accepted")
            .as_str(),
        "json.decode"
    );

    for invalid_error_code in [
        "",
        "decode",
        "network.timeout",
        "json.",
        "json.Decode",
        "json.decode!",
        "io.read.",
    ] {
        assert!(
            ErrorCode::new(invalid_error_code).is_err(),
            "{invalid_error_code:?} should be rejected"
        );
    }
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
    assert_eq!(
        deserialized
            .failure
            .expect("failed artifact should include failure")
            .retryability,
        Retryability::NotRetryable
    );
}
