//! Worker command behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::process::Output;

use assert_cmd::Command;

fn run_worker(args: &[&str]) -> Output {
    let mut command =
        Command::cargo_bin("replay-parser-2").expect("replay-parser-2 binary should build");
    for env_name in [
        "REPLAY_PARSER_AMQP_URL",
        "REPLAY_PARSER_JOB_QUEUE",
        "REPLAY_PARSER_RESULT_EXCHANGE",
        "REPLAY_PARSER_COMPLETED_ROUTING_KEY",
        "REPLAY_PARSER_FAILED_ROUTING_KEY",
        "REPLAY_PARSER_S3_BUCKET",
        "AWS_REGION",
        "REPLAY_PARSER_S3_ENDPOINT",
        "REPLAY_PARSER_S3_FORCE_PATH_STYLE",
        "REPLAY_PARSER_ARTIFACT_PREFIX",
        "REPLAY_PARSER_PREFETCH",
        "REPLAY_PARSER_PROBE_BIND",
        "REPLAY_PARSER_PROBE_PORT",
        "REPLAY_PARSER_PROBES_ENABLED",
        "REPLAY_PARSER_WORKER_ID",
        "HOSTNAME",
    ] {
        _ = command.env_remove(env_name);
    }
    command.arg("worker").args(args).output().expect("worker command should run")
}

fn credentialed_amqp_url(password: &str) -> String {
    ["amqp://worker", ":", password, "@rabbitmq:5672/%2f"].concat()
}

#[test]
fn worker_command_help_should_list_runtime_configuration_flags() {
    // Act
    let command_output = run_worker(&["--help"]);
    let stdout =
        String::from_utf8(command_output.stdout).expect("stdout should be valid UTF-8 text");

    // Assert
    assert!(command_output.status.success());
    for expected_flag in [
        "--amqp-url",
        "--s3-bucket",
        "--s3-endpoint",
        "--artifact-prefix",
        "--prefetch",
        "--probe-bind",
        "--probe-port",
        "--probes-enabled",
        "--worker-id",
    ] {
        assert!(stdout.contains(expected_flag), "worker help should contain {expected_flag}");
    }
}

#[test]
fn worker_command_missing_s3_bucket_should_fail_without_printing_secrets() {
    // Arrange
    let amqp_url = credentialed_amqp_url("redacted-test-password");

    // Act
    let command_output = run_worker(&["--amqp-url", amqp_url.as_str()]);
    let stderr =
        String::from_utf8(command_output.stderr).expect("stderr should be valid UTF-8 text");

    // Assert
    assert!(!command_output.status.success());
    assert!(stderr.contains("REPLAY_PARSER_S3_BUCKET"));
    assert!(!stderr.contains("AWS_SECRET_ACCESS_KEY"));
    assert!(!stderr.contains(&amqp_url));
    assert!(!stderr.contains("redacted-test-password"));
}
