//! Compare command behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Output,
    sync::atomic::{AtomicU64, Ordering},
};

use assert_cmd::Command;
use serde_json::json;

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
        .join(format!("replay-parser-2-compare-{test_name}-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).expect("test temp directory should be created");
    dir.join(file_name)
}

fn run_compare(args: &[&str]) -> Output {
    let mut command =
        Command::cargo_bin("replay-parser-2").expect("replay-parser-2 binary should build");
    command.arg("compare").args(args).output().expect("compare command should run")
}

fn run_parse(input: &Path, output: &Path) -> Output {
    Command::cargo_bin("replay-parser-2")
        .expect("replay-parser-2 binary should build")
        .arg("parse")
        .arg(input)
        .arg("--output")
        .arg(output)
        .output()
        .expect("parse command should run")
}

fn write_selected_artifact(path: &Path, status: &str) {
    let artifact = json!({
        "status": status,
        "replay": {
            "mission_name": "SolidGames"
        },
        "participants": [],
        "facts": {
            "combat": [],
            "aggregate_contributions": []
        },
        "summaries": {
            "projections": {
                "legacy.player_game_results": [],
                "legacy.relationships": [],
                "bounty.inputs": [],
                "vehicle_score.inputs": []
            }
        }
    });
    let bytes = serde_json::to_vec_pretty(&artifact).expect("test artifact should serialize");
    fs::write(path, bytes).expect("test artifact should be writable");
}

fn read_report(path: &Path) -> String {
    fs::read_to_string(path).expect("comparison report should be readable")
}

#[test]
fn compare_command_should_write_compatible_report_when_saved_artifacts_match() {
    // Arrange
    let old_artifact = temp_output_path("compatible", "old.json");
    let new_artifact = temp_output_path("compatible", "new.json");
    let report_path = temp_output_path("compatible", "report.md");
    write_selected_artifact(&old_artifact, "success");
    write_selected_artifact(&new_artifact, "success");

    // Act
    let command_output = run_compare(&[
        "--new-artifact",
        new_artifact.to_str().expect("new artifact path should be valid UTF-8"),
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
    ]);
    let report = read_report(&report_path);

    // Assert
    assert!(command_output.status.success());
    assert!(report.starts_with("# Comparison Summary"));
    assert!(report.contains("## Counts by Category"));
    assert!(report.contains("## Counts by Impact"));
    assert!(report.contains("## Top Diffs"));
    assert!(report.contains("## Next Action"));
    assert!(report.contains("`compatible`"));
}

#[test]
fn compare_command_should_write_review_report_when_saved_artifacts_differ() {
    // Arrange
    let old_artifact = temp_output_path("different", "old.json");
    let new_artifact = temp_output_path("different", "new.json");
    let report_path = temp_output_path("different", "report.md");
    write_selected_artifact(&old_artifact, "success");
    write_selected_artifact(&new_artifact, "failed");

    // Act
    let command_output = run_compare(&[
        "--new-artifact",
        new_artifact.to_str().expect("new artifact path should be valid UTF-8"),
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
    ]);
    let report = read_report(&report_path);

    // Assert
    assert!(command_output.status.success());
    assert!(report.starts_with("# Comparison Summary"));
    assert!(report.contains("Review top human_review diffs before accepting parity"));
}

#[test]
fn compare_command_should_write_json_detail_when_markdown_output_requests_detail() {
    // Arrange
    let old_artifact = temp_output_path("detail", "old.json");
    let new_artifact = temp_output_path("detail", "new.json");
    let report_path = temp_output_path("detail", "report.md");
    let detail_path = temp_output_path("detail", "report.json");
    write_selected_artifact(&old_artifact, "success");
    write_selected_artifact(&new_artifact, "failed");

    // Act
    let command_output = run_compare(&[
        "--new-artifact",
        new_artifact.to_str().expect("new artifact path should be valid UTF-8"),
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
        "--detail-output",
        detail_path.to_str().expect("detail path should be valid UTF-8"),
    ]);
    let report = read_report(&report_path);
    let detail = read_report(&detail_path);

    // Assert
    assert!(command_output.status.success());
    assert!(report.starts_with("# Comparison Summary"));
    assert!(detail.contains("\"findings\""));
}

#[test]
fn compare_command_should_write_json_report_when_format_json_is_requested() {
    // Arrange
    let old_artifact = temp_output_path("json-format", "old.json");
    let new_artifact = temp_output_path("json-format", "new.json");
    let report_path = temp_output_path("json-format", "report.json");
    write_selected_artifact(&old_artifact, "success");
    write_selected_artifact(&new_artifact, "success");

    // Act
    let command_output = run_compare(&[
        "--new-artifact",
        new_artifact.to_str().expect("new artifact path should be valid UTF-8"),
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
        "--format",
        "json",
    ]);
    let report = read_report(&report_path);

    // Assert
    assert!(command_output.status.success());
    assert!(report.contains("\"findings\""));
    assert!(!report.starts_with("# Comparison Summary"));
}

#[test]
fn compare_command_should_reject_json_format_with_detail_output() {
    // Arrange
    let old_artifact = temp_output_path("json-detail", "old.json");
    let new_artifact = temp_output_path("json-detail", "new.json");
    let report_path = temp_output_path("json-detail", "report.json");
    let detail_path = temp_output_path("json-detail", "detail.json");
    write_selected_artifact(&old_artifact, "success");
    write_selected_artifact(&new_artifact, "success");

    // Act
    let command_output = run_compare(&[
        "--new-artifact",
        new_artifact.to_str().expect("new artifact path should be valid UTF-8"),
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
        "--format",
        "json",
        "--detail-output",
        detail_path.to_str().expect("detail path should be valid UTF-8"),
    ]);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert!(stderr.contains("compare --format json cannot be combined with --detail-output"));
    assert!(!report_path.exists());
    assert!(!detail_path.exists());
}

#[test]
fn compare_command_should_parse_replay_when_new_artifact_is_absent() {
    // Arrange
    let replay = parser_core_fixture("valid-minimal.ocap.json");
    let old_artifact = temp_output_path("replay", "old.json");
    let report_path = temp_output_path("replay", "report.json");
    let parse_output = run_parse(&replay, &old_artifact);

    // Act
    let command_output = run_compare(&[
        "--replay",
        replay.to_str().expect("replay path should be valid UTF-8"),
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
    ]);
    let report = read_report(&report_path);

    // Assert
    assert!(parse_output.status.success());
    assert!(command_output.status.success());
    assert!(report.starts_with("# Comparison Summary"));
    assert!(report.contains("`compatible`"));
}

#[test]
fn compare_command_should_fail_when_replay_and_new_artifact_are_missing() {
    // Arrange
    let old_artifact = temp_output_path("invalid", "old.json");
    let report_path = temp_output_path("invalid", "report.json");
    write_selected_artifact(&old_artifact, "success");

    // Act
    let command_output = run_compare(&[
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
    ]);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert!(stderr.contains("compare requires --replay or --new-artifact"));
}

#[test]
fn compare_command_should_fail_when_replay_and_new_artifact_are_both_present() {
    // Arrange
    let replay = parser_core_fixture("valid-minimal.ocap.json");
    let old_artifact = temp_output_path("conflicting", "old.json");
    let new_artifact = temp_output_path("conflicting", "new.json");
    let report_path = temp_output_path("conflicting", "report.json");
    write_selected_artifact(&old_artifact, "success");
    write_selected_artifact(&new_artifact, "success");

    // Act
    let command_output = run_compare(&[
        "--replay",
        replay.to_str().expect("replay path should be valid UTF-8"),
        "--new-artifact",
        new_artifact.to_str().expect("new artifact path should be valid UTF-8"),
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
    ]);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert!(stderr.contains("compare accepts only one of --replay or --new-artifact"));
    assert!(!report_path.exists());
}

#[test]
fn compare_command_should_fail_when_old_artifact_is_not_valid_json() {
    // Arrange
    let old_artifact = temp_output_path("invalid-old", "old.json");
    let new_artifact = temp_output_path("invalid-old", "new.json");
    let report_path = temp_output_path("invalid-old", "report.json");
    fs::write(&old_artifact, b"{").expect("invalid old artifact should be writable");
    write_selected_artifact(&new_artifact, "success");

    // Act
    let command_output = run_compare(&[
        "--new-artifact",
        new_artifact.to_str().expect("new artifact path should be valid UTF-8"),
        "--old-artifact",
        old_artifact.to_str().expect("old artifact path should be valid UTF-8"),
        "--output",
        report_path.to_str().expect("report path should be valid UTF-8"),
    ]);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert!(stderr.contains("could not compare artifacts"));
    assert!(!report_path.exists());
}
