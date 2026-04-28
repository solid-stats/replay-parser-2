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
use serde_json::Value;

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
    Command::cargo_bin("replay-parser-2")
        .expect("replay-parser-2 binary should build")
        .arg("parse")
        .arg(input)
        .arg("--output")
        .arg(output)
        .output()
        .expect("parse command should run")
}

fn read_json(path: &PathBuf) -> Value {
    let text = fs::read_to_string(path).expect("output artifact should be readable");
    serde_json::from_str(&text).expect("output artifact should be valid JSON")
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
}

#[test]
fn parse_command_should_write_failure_artifact_and_stderr_summary_when_input_is_invalid() {
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
