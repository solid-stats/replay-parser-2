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
    workspace_root().join("schemas/parse-artifact-v3.schema.json")
}

fn success_example_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/parse_artifact_success.v3.json")
}

fn failure_example_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/parse_failure.v3.json")
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
        "schemas/parse-artifact-v3.schema.json should be committed"
    );
}

#[test]
fn schema_contract_committed_schema_should_name_parse_artifact_and_minimal_fields() {
    let schema_text =
        fs::read_to_string(committed_schema_path()).expect("committed schema should be readable");

    assert!(schema_text.contains("ParseArtifact"));
    for expected_field in [
        "contract_version",
        "parser",
        "source",
        "status",
        "diagnostics",
        "players",
        "player_stats",
        "kills",
        "destroyed_vehicles",
        "failure",
    ] {
        assert!(schema_text.contains(expected_field), "schema should contain {expected_field}");
    }

    for removed_field in [
        "\"participants\"",
        "\"facts\"",
        "\"summaries\"",
        "\"entities\"",
        "\"events\"",
        "\"aggregates\"",
    ] {
        assert!(
            !schema_text.contains(removed_field),
            "schema should not contain removed top-level field {removed_field}"
        );
    }
}

#[test]
fn schema_contract_committed_schema_should_include_minimal_row_types() {
    let schema_text =
        fs::read_to_string(committed_schema_path()).expect("committed schema should be readable");

    for expected_fragment in [
        "MinimalPlayerRow",
        "MinimalPlayerStatsRow",
        "MinimalKillRow",
        "MinimalDestroyedVehicleRow",
        "vehicleKills",
        "killsFromVehicle",
        "bounty_eligible",
    ] {
        assert!(
            schema_text.contains(expected_fragment),
            "schema should contain {expected_fragment}"
        );
    }

    let retired_projection = ["vehicle", "_score"].concat();
    let retired_type = ["Vehicle", "Score"].concat();
    assert!(!schema_text.contains(&retired_projection));
    assert!(!schema_text.contains(&retired_type));
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
fn schema_contract_success_example_should_expose_minimal_tables_only() {
    let success_example = read_json(success_example_path());

    for expected_field in ["players", "player_stats", "kills", "destroyed_vehicles"] {
        assert!(success_example[expected_field].is_array());
    }

    for removed_field in ["participants", "facts", "summaries"] {
        assert!(success_example.get(removed_field).is_none());
    }
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
fn schema_contract_gap_regression_should_reject_player_row_without_player_id() {
    let mut success_example = read_json(success_example_path());
    let removed = success_example["players"][0]
        .as_object_mut()
        .expect("player row should be an object")
        .remove("player_id");

    assert!(removed.is_some(), "player row should include player_id");
    assert_committed_schema_rejects(&success_example);
}

#[test]
fn schema_contract_gap_regression_should_reject_string_counter_in_player_stats() {
    let mut success_example = read_json(success_example_path());
    success_example["player_stats"][0]["vehicleKills"] = json!("one");

    assert_committed_schema_rejects(&success_example);
}

#[test]
fn schema_contract_gap_regression_should_reject_invalid_kill_classification() {
    let mut success_example = read_json(success_example_path());
    success_example["kills"][0]["classification"] = json!("friendly_fire");

    assert_committed_schema_rejects(&success_example);
}

#[test]
fn schema_contract_gap_regression_should_reject_invalid_destroyed_vehicle_classification() {
    let mut success_example = read_json(success_example_path());
    success_example["destroyed_vehicles"][0]["classification"] = json!("neutral");

    assert_committed_schema_rejects(&success_example);
}
