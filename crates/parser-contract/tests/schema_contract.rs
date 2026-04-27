//! JSON Schema contract tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{fs, path::PathBuf};

use parser_contract::{artifact::ParseArtifact, schema::parse_artifact_schema};
use serde_json::{Value, json};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate should live under crates/")
        .parent()
        .expect("crates/ should live under the workspace root")
        .to_path_buf()
}

fn committed_schema_path() -> PathBuf {
    workspace_root().join("schemas/parse-artifact-v1.schema.json")
}

fn success_example_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/parse_artifact_success.v1.json")
}

fn failure_example_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/parse_failure.v1.json")
}

fn read_json(path: PathBuf) -> Value {
    let text = fs::read_to_string(path).expect("JSON fixture should be readable");
    serde_json::from_str(&text).expect("JSON fixture should parse")
}

fn assert_committed_schema_rejects(candidate: &Value) {
    let schema = read_json(committed_schema_path());
    let validator = jsonschema::draft202012::new(&schema).expect("committed schema should compile");

    assert!(
        validator.validate(candidate).is_err(),
        "candidate should be rejected by committed schema"
    );
}

fn freshly_generated_schema_text() -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(&parse_artifact_schema())
            .expect("parse artifact schema should serialize")
    )
}

#[test]
fn schema_contract_committed_parse_artifact_schema_should_exist() {
    assert!(
        committed_schema_path().is_file(),
        "schemas/parse-artifact-v1.schema.json should be committed"
    );
}

#[test]
fn schema_contract_committed_schema_should_name_parse_artifact_and_contract_fields() {
    let schema_text =
        fs::read_to_string(committed_schema_path()).expect("committed schema should be readable");

    assert!(schema_text.contains("ParseArtifact"));
    for expected_field in [
        "contract_version",
        "parser",
        "status",
        "diagnostics",
        "replay",
        "entities",
        "events",
        "aggregates",
        "failure",
    ] {
        assert!(schema_text.contains(expected_field), "schema should contain {expected_field}");
    }
}

#[test]
fn schema_contract_committed_schema_should_match_fresh_generation() {
    let committed_schema_text =
        fs::read_to_string(committed_schema_path()).expect("committed schema should be readable");
    let fresh_schema_text = freshly_generated_schema_text();

    assert_eq!(committed_schema_text, fresh_schema_text);
}

#[test]
fn schema_contract_success_and_failure_examples_should_deserialize_into_parse_artifact() {
    for example_path in [success_example_path(), failure_example_path()] {
        let example = read_json(example_path);
        let _artifact: ParseArtifact =
            serde_json::from_value(example).expect("example should deserialize into ParseArtifact");
    }
}

#[test]
fn schema_contract_success_and_failure_examples_should_validate_against_committed_schema() {
    let schema = read_json(committed_schema_path());
    let validator = jsonschema::draft202012::new(&schema).expect("committed schema should compile");

    for example_path in [success_example_path(), failure_example_path()] {
        let example = read_json(example_path);
        let validation = validator.validate(&example);
        assert!(
            validation.is_ok(),
            "example should validate against committed schema: {:?}",
            validation.err()
        );
    }
}

#[test]
fn schema_contract_failure_example_should_include_required_structured_failure_fields() {
    let failure_example = read_json(failure_example_path());
    let failure = &failure_example["failure"];

    for expected_field in [
        "job_id",
        "replay_id",
        "source_file",
        "stage",
        "error_code",
        "message",
        "retryability",
        "source_cause",
    ] {
        assert!(
            failure.get(expected_field).is_some(),
            "failure example should include {expected_field}"
        );
    }
    assert_eq!(failure_example["status"], "failed");
    assert_eq!(failure["stage"], "json_decode");
    assert_eq!(failure["error_code"], "json.decode");
    assert_eq!(failure["retryability"], "not_retryable");
}

#[test]
fn schema_contract_gap_regression_should_reject_invalid_checksum_algorithm_and_value() {
    let mut success_example = read_json(success_example_path());
    success_example["source"]["checksum"]["value"] = json!({
        "algorithm": "md5",
        "value": "not-a-hash"
    });

    assert_committed_schema_rejects(&success_example);
}

#[test]
fn schema_contract_gap_regression_should_reject_failed_artifact_without_failure() {
    let mut failure_example = read_json(failure_example_path());
    failure_example["failure"] = Value::Null;

    assert_committed_schema_rejects(&failure_example);
}

#[test]
fn schema_contract_gap_regression_should_reject_non_failed_artifact_with_failure() {
    let mut failure_example = read_json(failure_example_path());
    failure_example["status"] = json!("success");

    assert_committed_schema_rejects(&failure_example);
}

#[test]
fn schema_contract_gap_regression_should_reject_empty_event_source_refs() {
    let mut success_example = read_json(success_example_path());
    success_example["events"][0]["source_refs"] = json!([]);

    assert_committed_schema_rejects(&success_example);
}

#[test]
fn schema_contract_gap_regression_should_reject_empty_entity_source_refs() {
    let mut success_example = read_json(success_example_path());
    success_example["entities"][0]["source_refs"] = json!([]);

    assert_committed_schema_rejects(&success_example);
}

#[test]
fn schema_contract_should_include_entity_compatibility_hint_shape() {
    let schema_text =
        fs::read_to_string(committed_schema_path()).expect("committed schema should be readable");

    for expected_fragment in [
        "EntityCompatibilityHint",
        "EntityCompatibilityHintKind",
        "connected_player_backfill",
        "duplicate_slot_same_name",
        "related_entity_ids",
        "observed_name",
        "rule_id",
        "source_refs",
    ] {
        assert!(
            schema_text.contains(expected_fragment),
            "schema should contain {expected_fragment}"
        );
    }
}

#[test]
fn schema_contract_gap_regression_should_reject_hollow_source_ref_objects() {
    let mut success_example = read_json(success_example_path());
    success_example["events"][0]["source_refs"][0] = json!({});

    assert_committed_schema_rejects(&success_example);
}

#[test]
fn schema_contract_gap_regression_should_reject_out_of_range_inferred_confidence() {
    let mut success_example = read_json(success_example_path());
    success_example["replay"]["mission_name"] = json!({
        "state": "inferred",
        "value": "Operation Solid",
        "reason": "test fixture",
        "confidence": 1.1,
        "source": null,
        "rule_id": "metadata.mission_name.inferred"
    });

    assert_committed_schema_rejects(&success_example);
}
