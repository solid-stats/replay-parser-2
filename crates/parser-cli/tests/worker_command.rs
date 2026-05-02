//! Worker command behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{
    io::{Read, Write},
    net::TcpListener,
    process::Output,
    thread,
};

use assert_cmd::Command;

fn replay_parser_command() -> Command {
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
    command
}

fn run_cli(args: &[&str]) -> Output {
    replay_parser_command().args(args).output().expect("CLI command should run")
}

fn run_worker(args: &[&str]) -> Output {
    let mut command = replay_parser_command();
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

#[test]
fn healthcheck_command_should_be_hidden_from_top_level_help() {
    // Act
    let command_output = run_cli(&["--help"]);
    let stdout =
        String::from_utf8(command_output.stdout).expect("stdout should be valid UTF-8 text");

    // Assert
    assert!(command_output.status.success());
    assert!(!stdout.contains("healthcheck"));
}

#[test]
fn healthcheck_command_should_exit_success_for_http_200() {
    // Arrange
    let url = one_response_probe_url("HTTP/1.1 200 OK");

    // Act
    let command_output = run_cli(&["healthcheck", "--url", &url]);

    // Assert
    assert!(command_output.status.success());
}

#[test]
fn healthcheck_command_should_exit_failure_for_non_200() {
    // Arrange
    let url = one_response_probe_url("HTTP/1.1 503 Service Unavailable");

    // Act
    let command_output = run_cli(&["healthcheck", "--url", &url]);

    // Assert
    assert_eq!(command_output.status.code(), Some(1));
}

#[test]
fn healthcheck_command_bad_url_should_exit_two() {
    // Act
    let command_output = run_cli(&["healthcheck", "--url", "https://127.0.0.1:8080/readyz"]);

    // Assert
    assert_eq!(command_output.status.code(), Some(2));
}

fn one_response_probe_url(status_line: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("probe listener should bind");
    let address = listener.local_addr().expect("probe listener should have a local address");
    let _server = thread::spawn(move || {
        let (mut stream, _peer) = listener.accept().expect("healthcheck should connect");
        let mut request = [0_u8; 512];
        let _read = stream.read(&mut request);
        let response = format!("{status_line}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        stream.write_all(response.as_bytes()).expect("probe response should write");
    });
    format!("http://{}:{}/readyz", address.ip(), address.port())
}
