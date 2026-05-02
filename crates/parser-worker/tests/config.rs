//! Worker configuration behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::collections::BTreeMap;

use parser_worker::config::{WorkerConfig, WorkerConfigOverrides};

fn config_from_pairs<const N: usize>(
    pairs: [(&str, &str); N],
) -> Result<WorkerConfig, parser_worker::error::WorkerError> {
    let env: BTreeMap<String, String> =
        pairs.map(|(key, value)| (key.to_owned(), value.to_owned())).into();
    WorkerConfig::from_env_and_overrides(
        |name| env.get(name).cloned(),
        WorkerConfigOverrides::default(),
    )
}

fn credentialed_amqp_url(password: &str) -> String {
    ["amqp://worker", ":", password, "@rabbitmq:5672/%2f"].concat()
}

#[test]
fn config_should_use_safe_defaults_when_only_required_bucket_is_set() {
    // Act
    let config = config_from_pairs([("REPLAY_PARSER_S3_BUCKET", "solid-replays")])
        .expect("required bucket should build config");

    // Assert
    assert_eq!(config.amqp_url, "amqp://127.0.0.1:5672/%2f");
    assert_eq!(config.job_queue, "parse.jobs");
    assert_eq!(config.result_exchange, "parse.results");
    assert_eq!(config.completed_routing_key, "parse.completed");
    assert_eq!(config.failed_routing_key, "parse.failed");
    assert_eq!(config.s3_bucket, "solid-replays");
    assert_eq!(config.s3_region, "us-east-1");
    assert_eq!(config.s3_endpoint, None);
    assert!(!config.s3_force_path_style);
    assert_eq!(config.artifact_prefix, "artifacts/v3");
    assert_eq!(config.prefetch, 1);
}

#[test]
fn config_should_apply_environment_overrides() {
    // Arrange
    let amqp_url = credentialed_amqp_url("redacted-test-password");

    // Act
    let config = config_from_pairs([
        ("REPLAY_PARSER_AMQP_URL", amqp_url.as_str()),
        ("REPLAY_PARSER_JOB_QUEUE", "custom.jobs"),
        ("REPLAY_PARSER_RESULT_EXCHANGE", "custom.results"),
        ("REPLAY_PARSER_COMPLETED_ROUTING_KEY", "custom.completed"),
        ("REPLAY_PARSER_FAILED_ROUTING_KEY", "custom.failed"),
        ("REPLAY_PARSER_S3_BUCKET", "custom-bucket"),
        ("AWS_REGION", "eu-central-1"),
        ("REPLAY_PARSER_S3_ENDPOINT", "http://127.0.0.1:9000"),
        ("REPLAY_PARSER_S3_FORCE_PATH_STYLE", "true"),
        ("REPLAY_PARSER_ARTIFACT_PREFIX", "custom/artifacts"),
        ("REPLAY_PARSER_PREFETCH", "2"),
    ])
    .expect("environment overrides should build config");

    // Assert
    assert_eq!(config.amqp_url, amqp_url);
    assert_eq!(config.job_queue, "custom.jobs");
    assert_eq!(config.result_exchange, "custom.results");
    assert_eq!(config.completed_routing_key, "custom.completed");
    assert_eq!(config.failed_routing_key, "custom.failed");
    assert_eq!(config.s3_bucket, "custom-bucket");
    assert_eq!(config.s3_region, "eu-central-1");
    assert_eq!(config.s3_endpoint.as_deref(), Some("http://127.0.0.1:9000"));
    assert!(config.s3_force_path_style);
    assert_eq!(config.artifact_prefix, "custom/artifacts");
    assert_eq!(config.prefetch, 2);
}

#[test]
fn config_should_apply_explicit_overrides_over_environment() {
    // Arrange
    let env = BTreeMap::from([
        ("REPLAY_PARSER_S3_BUCKET".to_owned(), "env-bucket".to_owned()),
        ("REPLAY_PARSER_PREFETCH".to_owned(), "2".to_owned()),
    ]);
    let overrides = WorkerConfigOverrides {
        s3_bucket: Some("flag-bucket".to_owned()),
        prefetch: Some(3),
        ..WorkerConfigOverrides::default()
    };

    // Act
    let config = WorkerConfig::from_env_and_overrides(|name| env.get(name).cloned(), overrides)
        .expect("explicit overrides should build config");

    // Assert
    assert_eq!(config.s3_bucket, "flag-bucket");
    assert_eq!(config.prefetch, 3);
}

#[test]
fn config_should_reject_zero_prefetch() {
    // Act
    let error = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_PREFETCH", "0"),
    ])
    .expect_err("prefetch zero should fail validation");

    // Assert
    let message = error.to_string();
    assert!(message.contains("REPLAY_PARSER_PREFETCH"));
    assert!(message.contains(">= 1"));
}

#[test]
fn config_should_reject_missing_s3_bucket() {
    // Act
    let error = config_from_pairs([]).expect_err("missing S3 bucket should fail validation");

    // Assert
    assert!(error.to_string().contains("REPLAY_PARSER_S3_BUCKET"));
}

#[test]
fn config_debug_should_redact_amqp_credentials_and_omit_aws_secrets() {
    // Arrange
    let amqp_url = credentialed_amqp_url("redacted-test-password");
    let config = config_from_pairs([
        ("REPLAY_PARSER_AMQP_URL", amqp_url.as_str()),
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
    ])
    .expect("required bucket should build config");

    // Act
    let debug = format!("{config:?}");

    // Assert
    assert!(debug.contains("amqp://***@rabbitmq:5672/%2f"));
    assert!(!debug.contains("worker"));
    assert!(!debug.contains("redacted-test-password"));
    assert!(!debug.contains("AWS_SECRET_ACCESS_KEY"));
}
