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
    assert_eq!(config.probe_bind, "0.0.0.0");
    assert_eq!(config.probe_port, 8080);
    assert!(config.probes_enabled);
    assert_eq!(config.worker_id, "replay-parser-worker");
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
        ("REPLAY_PARSER_PROBE_BIND", "127.0.0.1"),
        ("REPLAY_PARSER_PROBE_PORT", "9090"),
        ("REPLAY_PARSER_PROBES_ENABLED", "false"),
        ("REPLAY_PARSER_WORKER_ID", "worker-env-1"),
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
    assert_eq!(config.probe_bind, "127.0.0.1");
    assert_eq!(config.probe_port, 9090);
    assert!(!config.probes_enabled);
    assert_eq!(config.worker_id, "worker-env-1");
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
        probe_bind: Some("127.0.0.2".to_owned()),
        probe_port: Some(9091),
        probes_enabled: Some(true),
        worker_id: Some("worker-flag-1".to_owned()),
        ..WorkerConfigOverrides::default()
    };

    // Act
    let config = WorkerConfig::from_env_and_overrides(|name| env.get(name).cloned(), overrides)
        .expect("explicit overrides should build config");

    // Assert
    assert_eq!(config.s3_bucket, "flag-bucket");
    assert_eq!(config.prefetch, 3);
    assert_eq!(config.probe_bind, "127.0.0.2");
    assert_eq!(config.probe_port, 9091);
    assert!(config.probes_enabled);
    assert_eq!(config.worker_id, "worker-flag-1");
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
fn config_should_accept_boolean_false_aliases() {
    // Act
    let config = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_S3_FORCE_PATH_STYLE", "no"),
        ("REPLAY_PARSER_PROBES_ENABLED", "0"),
    ])
    .expect("false aliases should build config");

    // Assert
    assert!(!config.s3_force_path_style);
    assert!(!config.probes_enabled);
}

#[test]
fn config_should_reject_invalid_boolean_value() {
    // Act
    let error = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_S3_FORCE_PATH_STYLE", "maybe"),
    ])
    .expect_err("invalid boolean should fail validation");

    // Assert
    assert!(error.to_string().contains("must be a boolean"));
}

#[test]
fn config_should_reject_non_integer_prefetch() {
    // Act
    let error = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_PREFETCH", "many"),
    ])
    .expect_err("non-integer prefetch should fail validation");

    // Assert
    assert!(error.to_string().contains("REPLAY_PARSER_PREFETCH"));
    assert!(error.to_string().contains("integer"));
}

#[test]
fn config_should_reject_missing_s3_bucket() {
    // Act
    let error = config_from_pairs([]).expect_err("missing S3 bucket should fail validation");

    // Assert
    assert!(error.to_string().contains("REPLAY_PARSER_S3_BUCKET"));
}

#[test]
fn worker_identity_should_fall_back_to_hostname_before_literal_default() {
    // Act
    let config = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("HOSTNAME", "container-hostname"),
    ])
    .expect("hostname fallback should build config");

    // Assert
    assert_eq!(config.worker_id, "container-hostname");
}

#[test]
fn worker_identity_should_prefer_explicit_env_over_hostname() {
    // Act
    let config = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_WORKER_ID", "worker-env-2"),
        ("HOSTNAME", "container-hostname"),
    ])
    .expect("explicit worker id should build config");

    // Assert
    assert_eq!(config.worker_id, "worker-env-2");
}

#[test]
fn probe_config_should_reject_zero_port_when_probes_are_enabled() {
    // Act
    let error = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_PROBE_PORT", "0"),
    ])
    .expect_err("zero probe port should fail validation");

    // Assert
    let message = error.to_string();
    assert!(message.contains("REPLAY_PARSER_PROBE_PORT"));
    assert!(message.contains(">= 1"));
}

#[test]
fn probe_config_should_reject_non_integer_port_when_probes_are_enabled() {
    // Act
    let error = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_PROBE_PORT", "not-a-port"),
    ])
    .expect_err("non-integer probe port should fail validation");

    // Assert
    assert!(error.to_string().contains("REPLAY_PARSER_PROBE_PORT"));
    assert!(error.to_string().contains("integer"));
}

#[test]
fn probe_config_should_allow_empty_probe_fields_when_probes_are_disabled() {
    // Arrange
    let overrides = WorkerConfigOverrides {
        s3_bucket: Some("solid-replays".to_owned()),
        probes_enabled: Some(false),
        probe_bind: Some(" ".to_owned()),
        worker_id: Some(" ".to_owned()),
        ..WorkerConfigOverrides::default()
    };

    // Act
    let config = WorkerConfig::from_env_and_overrides(|_| None, overrides)
        .expect("disabled probes should not validate probe-only fields");

    // Assert
    assert!(!config.probes_enabled);
    assert_eq!(config.probe_bind, " ");
    assert_eq!(config.worker_id, " ");
}

#[test]
fn probe_config_should_reject_empty_bind_when_probes_are_enabled() {
    // Act
    let error = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_PROBE_BIND", " "),
    ])
    .expect_err("empty probe bind should fail validation");

    // Assert
    assert!(error.to_string().contains("probe_bind"));
}

#[test]
fn probe_config_should_reject_empty_worker_id_when_probes_are_enabled() {
    // Act
    let error = config_from_pairs([
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_WORKER_ID", " "),
    ])
    .expect_err("empty worker id should fail validation");

    // Assert
    assert!(error.to_string().contains("worker_id"));
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
    assert!(debug.contains("probe_bind"));
    assert!(debug.contains("probe_port"));
    assert!(debug.contains("probes_enabled"));
    assert!(debug.contains("worker_id"));
    assert!(!debug.contains("worker:redacted-test-password"));
    assert!(!debug.contains("redacted-test-password"));
    assert!(!debug.contains("AWS_SECRET_ACCESS_KEY"));
}

#[test]
fn worker_config_redaction_should_keep_worker_and_probe_fields_but_hide_amqp_credentials() {
    // Arrange
    let password = "phase-seven-secret";
    let amqp_url = credentialed_amqp_url(password);
    let config = config_from_pairs([
        ("REPLAY_PARSER_AMQP_URL", amqp_url.as_str()),
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
        ("REPLAY_PARSER_PROBE_BIND", "127.0.0.1"),
        ("REPLAY_PARSER_PROBE_PORT", "18080"),
        ("REPLAY_PARSER_PROBES_ENABLED", "true"),
        ("REPLAY_PARSER_WORKER_ID", "worker-redaction-test"),
    ])
    .expect("required bucket should build config");

    // Act
    let redacted = format!("{:?}", config.redacted());

    // Assert
    assert!(redacted.contains("worker_id"));
    assert!(redacted.contains("worker-redaction-test"));
    assert!(redacted.contains("probe_bind"));
    assert!(redacted.contains("127.0.0.1"));
    assert!(redacted.contains("probe_port"));
    assert!(redacted.contains("18080"));
    assert!(redacted.contains("probes_enabled"));
    assert!(redacted.contains("amqp://***@rabbitmq:5672/%2f"));
    assert!(!redacted.contains(password));
    assert!(!redacted.contains("worker:phase-seven-secret"));
}

#[test]
fn config_debug_should_redact_amqp_password_containing_at_sign() {
    // Arrange
    let amqp_url = credentialed_amqp_url("p@ss");
    let config = config_from_pairs([
        ("REPLAY_PARSER_AMQP_URL", amqp_url.as_str()),
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
    ])
    .expect("required bucket should build config");

    // Act
    let debug = format!("{config:?}");

    // Assert
    assert!(debug.contains("amqp://***@rabbitmq:5672/%2f"));
    assert!(!debug.contains("p@ss"));
    assert!(!debug.contains("ss@rabbitmq"));
}

#[test]
fn config_debug_should_redact_userinfo_when_amqp_url_has_no_scheme() {
    // Arrange
    let config = config_from_pairs([
        ("REPLAY_PARSER_AMQP_URL", "worker:p@ss@rabbitmq:5672"),
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
    ])
    .expect("required bucket should build config");

    // Act
    let debug = format!("{config:?}");

    // Assert
    assert!(debug.contains("***@rabbitmq:5672"));
    assert!(!debug.contains("p@ss"));
}

#[test]
fn config_debug_should_keep_amqp_url_without_userinfo_unchanged() {
    // Arrange
    let config = config_from_pairs([
        ("REPLAY_PARSER_AMQP_URL", "amqp://rabbitmq:5672/%2f"),
        ("REPLAY_PARSER_S3_BUCKET", "solid-replays"),
    ])
    .expect("required bucket should build config");

    // Act
    let debug = format!("{config:?}");

    // Assert
    assert!(debug.contains("amqp://rabbitmq:5672/%2f"));
}
