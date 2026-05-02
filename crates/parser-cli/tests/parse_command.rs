//! Parse command behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{
    fs,
    path::PathBuf,
    process::Output,
    sync::atomic::{AtomicU64, Ordering},
};

use assert_cmd::Command;
use parser_contract::{
    presence::FieldPresence,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, public_parse_replay};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn parser_core_fixture(name: &str) -> PathBuf {
    let crates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parser-cli should live under crates/")
        .to_path_buf();

    crates_dir.join("parser-core/tests/fixtures").join(name)
}

fn temp_output_path(test_name: &str, file_name: &str) -> PathBuf {
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir()
        .join(format!("replay-parser-2-{test_name}-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).expect("test temp directory should be created");
    dir.join(file_name)
}

fn run_parse(input: &PathBuf, output: &PathBuf) -> Output {
    run_parse_with_args(input, output, [])
}

fn run_parse_with_args<const N: usize>(
    input: &PathBuf,
    output: &PathBuf,
    extra_args: [&str; N],
) -> Output {
    Command::cargo_bin("replay-parser-2")
        .expect("replay-parser-2 binary should build")
        .arg("parse")
        .arg(input)
        .arg("--output")
        .arg(output)
        .args(extra_args)
        .output()
        .expect("parse command should run")
}

fn read_json(path: &PathBuf) -> Value {
    let text = fs::read_to_string(path).expect("output artifact should be readable");
    serde_json::from_str(&text).expect("output artifact should be valid JSON")
}

fn public_parser_core_output_bytes(input: &PathBuf) -> Vec<u8> {
    let bytes = fs::read(input).expect("fixture should be readable");
    let checksum =
        SourceChecksum::sha256(sha256_hex(&bytes)).expect("fixture checksum should be valid");
    let artifact = public_parse_replay(ParserInput {
        bytes: &bytes,
        source: ReplaySource {
            replay_id: None,
            source_file: input.display().to_string(),
            checksum: FieldPresence::Present { value: checksum, source: None },
        },
        parser: parser_info(),
        options: ParserOptions::default(),
    });
    let mut output = serde_json::to_vec(&artifact).expect("public artifact should serialize");
    output.push(b'\n');
    output
}

fn parser_info() -> ParserInfo {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": env!("CARGO_PKG_VERSION")
    }))
    .expect("test parser info should be valid")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn assert_compact_artifact_root(artifact: &Value) {
    for expected_field in ["contract_version", "parser", "source", "status"] {
        assert!(artifact.get(expected_field).is_some(), "artifact should contain {expected_field}");
    }

    for removed_field in ["participants", "facts", "summaries", "player_stats"] {
        assert!(
            artifact.get(removed_field).is_none(),
            "artifact should not contain removed top-level field {removed_field}"
        );
    }
}

fn assert_no_key_recursive(value: &Value, forbidden_key: &str) {
    match value {
        Value::Object(map) => {
            assert!(
                !map.contains_key(forbidden_key),
                "artifact should not contain {forbidden_key}"
            );
            for nested in map.values() {
                assert_no_key_recursive(nested, forbidden_key);
            }
        }
        Value::Array(items) => {
            for nested in items {
                assert_no_key_recursive(nested, forbidden_key);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn contains_null_recursive(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.values().any(contains_null_recursive),
        Value::Array(items) => items.iter().any(contains_null_recursive),
        Value::Null => true,
        Value::Bool(_) | Value::Number(_) | Value::String(_) => false,
    }
}

fn assert_no_null_recursive(value: &Value) {
    assert!(
        !contains_null_recursive(value),
        "default artifact should not contain JSON null values"
    );
}

#[test]
fn parse_command_should_write_success_artifact_when_input_is_valid() {
    // Arrange
    let input = parser_core_fixture("valid-minimal.ocap.json");
    let output_path = temp_output_path("valid", "artifact.json");

    // Act
    let command_output = run_parse(&input, &output_path);
    let artifact = read_json(&output_path);

    // Assert
    assert!(command_output.status.success());
    assert!(artifact.get("contract_version").is_some());
    assert_eq!(artifact["source"]["source_file"], input.display().to_string());
    assert!(artifact["source"].get("checksum").is_some());
    assert!(artifact.get("replay").is_some());
    assert_compact_artifact_root(&artifact);
    assert!(artifact.get("players").is_none());
    assert!(artifact.get("weapons").is_none());
    assert!(artifact.get("kills").is_none());
    assert!(artifact.get("destroyed_vehicles").is_none());
    assert!(artifact.get("failure").is_none());
    assert_no_null_recursive(&artifact);
}

#[test]
fn parse_command_should_write_minified_minimal_json_by_default() {
    // Arrange
    let input = parser_core_fixture("valid-minimal.ocap.json");
    let output_path = temp_output_path("minified", "artifact.json");

    // Act
    let command_output = run_parse(&input, &output_path);
    let artifact_text =
        fs::read_to_string(&output_path).expect("output artifact should be readable");
    let artifact: Value =
        serde_json::from_str(&artifact_text).expect("output artifact should be valid JSON");

    // Assert
    assert!(command_output.status.success());
    assert!(artifact_text.ends_with('\n'));
    assert!(artifact_text.trim_end().lines().count() == 1);
    assert_compact_artifact_root(&artifact);
    let retired_score_section = ["vehicle", "score"].join("_");
    for forbidden_key in [
        "participants",
        "facts",
        "summaries",
        "player_stats",
        "bounty_eligible",
        "bounty_exclusion_reasons",
        "killer_name",
        "victim_name",
        "attacker_vehicle_name",
        "destroyed_name",
        "source_refs",
        "rule_id",
        "frame",
        "event_index",
        "json_path",
    ] {
        assert_no_key_recursive(&artifact, forbidden_key);
    }
    assert_no_key_recursive(&artifact, &retired_score_section);
    assert_no_null_recursive(&artifact);
}

#[test]
fn parse_command_default_output_should_match_public_parser_core_bytes() {
    // Arrange
    let input = parser_core_fixture("metadata-drift.ocap.json");
    let output_path = temp_output_path("public-core-bytes", "artifact.json");

    // Act
    let command_output = run_parse(&input, &output_path);
    let cli_bytes = fs::read(&output_path).expect("output artifact should be readable");
    let core_bytes = public_parser_core_output_bytes(&input);

    // Assert
    assert!(command_output.status.success());
    assert_eq!(cli_bytes, core_bytes);
}

#[test]
fn parse_command_should_write_pretty_minimal_json_when_requested() {
    // Arrange
    let input = parser_core_fixture("valid-minimal.ocap.json");
    let output_path = temp_output_path("pretty", "artifact.json");

    // Act
    let command_output = run_parse_with_args(&input, &output_path, ["--pretty"]);
    let artifact_text =
        fs::read_to_string(&output_path).expect("output artifact should be readable");
    let artifact: Value =
        serde_json::from_str(&artifact_text).expect("output artifact should be valid JSON");

    // Assert
    assert!(command_output.status.success());
    assert!(artifact_text.ends_with('\n'));
    assert!(artifact_text.trim_end().lines().count() > 1);
    assert_compact_artifact_root(&artifact);
    assert!(artifact.get("player_stats").is_none());
}

#[test]
fn parse_command_should_omit_debug_provenance_from_partial_default_artifact() {
    // Arrange
    let input = parser_core_fixture("metadata-drift.ocap.json");
    let output_path = temp_output_path("partial-public", "artifact.json");

    // Act
    let command_output = run_parse(&input, &output_path);
    let artifact = read_json(&output_path);

    // Assert
    assert!(command_output.status.success());
    assert_eq!(artifact["status"], "partial");
    assert_eq!(artifact["diagnostics"][0]["code"], "schema.metadata_field");
    assert_compact_artifact_root(&artifact);
    for forbidden_key in ["source_refs", "rule_id", "frame", "event_index", "json_path"] {
        assert_no_key_recursive(&artifact, forbidden_key);
    }
    assert_no_null_recursive(&artifact);
}

#[test]
fn parse_command_should_write_debug_artifact_sidecar_when_requested() {
    // Arrange
    let input = parser_core_fixture("aggregate-combat.ocap.json");
    let output_path = temp_output_path("debug-sidecar", "artifact.json");
    let debug_path = temp_output_path("debug-sidecar", "debug.json");
    let debug_arg = debug_path.to_str().expect("debug artifact temp path should be valid UTF-8");

    // Act
    let command_output = run_parse_with_args(&input, &output_path, ["--debug-artifact", debug_arg]);
    let artifact_text =
        fs::read_to_string(&output_path).expect("output artifact should be readable");
    let debug_text = fs::read_to_string(&debug_path).expect("debug artifact should be readable");
    let artifact: Value =
        serde_json::from_str(&artifact_text).expect("output artifact should be valid JSON");
    let debug_artifact: Value =
        serde_json::from_str(&debug_text).expect("debug artifact should be valid JSON");

    // Assert
    assert!(command_output.status.success());
    assert!(output_path.is_file());
    assert!(debug_path.is_file());
    assert_no_key_recursive(&artifact, "source_refs");
    assert_no_key_recursive(&artifact, "player_stats");
    assert!(debug_text.contains("\"source_refs\""));
    assert!(debug_text.contains("\"rule_id\""));
    assert!(debug_text.contains("\"frame\""));
    assert!(debug_text.contains("\"event_index\""));
    assert!(debug_artifact.get("entities").is_some());
    assert!(debug_artifact.get("events").is_some());
}

#[test]
fn parse_command_should_reject_debug_artifact_path_that_matches_output_path() {
    // Arrange
    let input = parser_core_fixture("aggregate-combat.ocap.json");
    let output_path = temp_output_path("debug-conflict", "artifact.json");
    let debug_arg = output_path.to_str().expect("output temp path should be valid UTF-8");

    // Act
    let command_output = run_parse_with_args(&input, &output_path, ["--debug-artifact", debug_arg]);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert!(stderr.contains("parse --debug-artifact must not be the same path as --output"));
    assert!(!output_path.exists());
}

#[test]
fn parse_command_should_not_create_debug_artifact_without_explicit_flag() {
    // Arrange
    let input = parser_core_fixture("aggregate-combat.ocap.json");
    let output_path = temp_output_path("debug-not-requested", "artifact.json");
    let debug_path = output_path.with_file_name("debug.json");

    // Act
    let command_output = run_parse(&input, &output_path);

    // Assert
    assert!(command_output.status.success());
    assert!(output_path.is_file());
    assert!(!debug_path.exists());
}

#[test]
fn parse_command_should_write_compact_failure_artifact_and_stderr_summary_when_input_is_invalid() {
    // Arrange
    let input = parser_core_fixture("invalid-json.ocap.json");
    let output_path = temp_output_path("invalid", "artifact.json");

    // Act
    let command_output = run_parse(&input, &output_path);
    let artifact = read_json(&output_path);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert_eq!(artifact["status"], "failed");
    assert!(artifact.get("failure").is_some());
    assert_compact_artifact_root(&artifact);
    assert!(artifact.get("players").is_none());
    assert!(artifact.get("weapons").is_none());
    assert!(artifact.get("kills").is_none());
    assert!(artifact.get("destroyed_vehicles").is_none());
    assert!(artifact["failure"].is_object());
    assert_no_null_recursive(&artifact);
    assert!(stderr.contains("parse failed:"));
}

#[test]
fn parse_command_should_write_byte_identical_artifacts_when_same_input_runs_twice() {
    // Arrange
    let input = parser_core_fixture("valid-minimal.ocap.json");
    let first_output_path = temp_output_path("determinism", "first.json");
    let second_output_path = temp_output_path("determinism", "second.json");

    // Act
    let first_command_output = run_parse(&input, &first_output_path);
    let second_command_output = run_parse(&input, &second_output_path);
    let first_artifact =
        fs::read_to_string(&first_output_path).expect("first output artifact should be readable");
    let second_artifact =
        fs::read_to_string(&second_output_path).expect("second output artifact should be readable");

    // Assert
    assert!(first_command_output.status.success());
    assert!(second_command_output.status.success());
    assert_eq!(first_artifact, second_artifact);
}

#[test]
fn parse_command_should_fail_with_human_error_when_input_file_is_missing() {
    // Arrange
    let input = temp_output_path("missing-input", "missing.ocap.json");
    let output_path = temp_output_path("missing-input", "artifact.json");

    // Act
    let command_output = run_parse(&input, &output_path);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert!(stderr.contains("could not read input"));
    assert!(!output_path.exists());
}

#[test]
fn parse_command_should_fail_with_human_error_when_output_path_parent_is_missing() {
    // Arrange
    let input = parser_core_fixture("valid-minimal.ocap.json");
    let output_path = temp_output_path("missing-output-parent", "missing").join("artifact.json");

    // Act
    let command_output = run_parse(&input, &output_path);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert!(stderr.contains("could not write output"));
    assert!(!output_path.exists());
}
