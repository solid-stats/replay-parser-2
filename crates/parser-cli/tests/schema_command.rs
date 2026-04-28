//! Schema command behavior tests.

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

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parser-cli should live under crates/")
        .parent()
        .expect("crates/ should live under workspace root")
        .to_path_buf()
}

fn committed_schema_path() -> PathBuf {
    workspace_root().join("schemas/parse-artifact-v1.schema.json")
}

fn temp_output_path(test_name: &str, file_name: &str) -> PathBuf {
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir()
        .join(format!("replay-parser-2-schema-{test_name}-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).expect("test temp directory should be created");
    dir.join(file_name)
}

fn run_schema(args: &[&str]) -> Output {
    let mut command =
        Command::cargo_bin("replay-parser-2").expect("replay-parser-2 binary should build");
    command.arg("schema").args(args).output().expect("schema command should run")
}

#[test]
fn schema_command_should_write_current_schema_to_stdout_when_output_is_absent() {
    // Arrange and Act
    let command_output = run_schema(&[]);
    let stdout =
        String::from_utf8(command_output.stdout).expect("stdout should be valid UTF-8 text");

    // Assert
    assert!(command_output.status.success());
    assert!(stdout.contains("ParseArtifact"));
    assert!(stdout.contains("ReplaySideFacts"));
    assert!(stdout.contains("vehicle_score.denominator_inputs"));
}

#[test]
fn schema_command_should_write_current_schema_to_file_when_output_is_present() {
    // Arrange
    let output_path = temp_output_path("file", "schema.json");

    // Act
    let command_output = run_schema(&[
        "--output",
        output_path.to_str().expect("test schema output path should be valid UTF-8"),
    ]);
    let file_text =
        fs::read_to_string(&output_path).expect("schema output file should be readable");

    // Assert
    assert!(command_output.status.success());
    assert!(command_output.stdout.is_empty());
    assert!(file_text.contains("ParseArtifact"));
}

#[test]
fn schema_command_should_match_committed_parse_artifact_schema_exactly() {
    // Arrange
    let output_path = temp_output_path("freshness", "schema.json");

    // Act
    let command_output = run_schema(&[
        "--output",
        output_path.to_str().expect("test schema output path should be valid UTF-8"),
    ]);
    let fresh_schema = fs::read(&output_path).expect("schema output file should be readable");
    let committed_schema =
        fs::read(committed_schema_path()).expect("committed schema should be readable");

    // Assert
    assert!(command_output.status.success());
    assert_eq!(fresh_schema, committed_schema);
}
