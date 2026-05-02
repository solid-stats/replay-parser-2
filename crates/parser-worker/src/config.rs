//! Worker configuration.
//!
//! Credentials are loaded by the AWS SDK from its standard environment/profile
//! chain, including `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and optional
//! `AWS_SESSION_TOKEN`. Secret values are intentionally not stored in this
//! configuration type.

use std::fmt;

use crate::error::WorkerError;

/// Environment variable for the `RabbitMQ` AMQP URL.
pub const ENV_AMQP_URL: &str = "REPLAY_PARSER_AMQP_URL";
/// Environment variable for the `RabbitMQ` job queue name.
pub const ENV_JOB_QUEUE: &str = "REPLAY_PARSER_JOB_QUEUE";
/// Environment variable for the `RabbitMQ` result exchange name.
pub const ENV_RESULT_EXCHANGE: &str = "REPLAY_PARSER_RESULT_EXCHANGE";
/// Environment variable for the successful-result routing key.
pub const ENV_COMPLETED_ROUTING_KEY: &str = "REPLAY_PARSER_COMPLETED_ROUTING_KEY";
/// Environment variable for the failed-result routing key.
pub const ENV_FAILED_ROUTING_KEY: &str = "REPLAY_PARSER_FAILED_ROUTING_KEY";
/// Environment variable for the raw replay/artifact S3 bucket.
pub const ENV_S3_BUCKET: &str = "REPLAY_PARSER_S3_BUCKET";
/// Environment variable for the S3 region.
pub const ENV_S3_REGION: &str = "AWS_REGION";
/// Environment variable for an optional custom S3 endpoint.
pub const ENV_S3_ENDPOINT: &str = "REPLAY_PARSER_S3_ENDPOINT";
/// Environment variable for S3 path-style addressing.
pub const ENV_S3_FORCE_PATH_STYLE: &str = "REPLAY_PARSER_S3_FORCE_PATH_STYLE";
/// Environment variable for the parser artifact key prefix.
pub const ENV_ARTIFACT_PREFIX: &str = "REPLAY_PARSER_ARTIFACT_PREFIX";
/// Environment variable for `RabbitMQ` prefetch.
pub const ENV_PREFETCH: &str = "REPLAY_PARSER_PREFETCH";
/// Environment variable for the HTTP probe bind address.
pub const ENV_PROBE_BIND: &str = "REPLAY_PARSER_PROBE_BIND";
/// Environment variable for the HTTP probe port.
pub const ENV_PROBE_PORT: &str = "REPLAY_PARSER_PROBE_PORT";
/// Environment variable for enabling HTTP probes.
pub const ENV_PROBES_ENABLED: &str = "REPLAY_PARSER_PROBES_ENABLED";
/// Environment variable for the operator-visible worker identity.
pub const ENV_WORKER_ID: &str = "REPLAY_PARSER_WORKER_ID";

/// Default local `RabbitMQ` AMQP URL.
pub const DEFAULT_AMQP_URL: &str = "amqp://127.0.0.1:5672/%2f";
/// Default `RabbitMQ` job queue name.
pub const DEFAULT_JOB_QUEUE: &str = "parse.jobs";
/// Default `RabbitMQ` result exchange name.
pub const DEFAULT_RESULT_EXCHANGE: &str = "parse.results";
/// Default routing key for successful parse results.
pub const DEFAULT_COMPLETED_ROUTING_KEY: &str = "parse.completed";
/// Default routing key for failed parse results.
pub const DEFAULT_FAILED_ROUTING_KEY: &str = "parse.failed";
/// Default S3 region used by local deployments when `AWS_REGION` is absent.
pub const DEFAULT_S3_REGION: &str = "us-east-1";
/// Default artifact key prefix.
pub const DEFAULT_ARTIFACT_PREFIX: &str = "artifacts/v3";
/// Default in-flight `RabbitMQ` job count.
pub const DEFAULT_PREFETCH: u16 = 1;
/// Default HTTP probe bind address.
pub const DEFAULT_PROBE_BIND: &str = "0.0.0.0";
/// Default HTTP probe port.
pub const DEFAULT_PROBE_PORT: u16 = 8080;
/// Default HTTP probe enabled flag.
pub const DEFAULT_PROBES_ENABLED: bool = true;
/// Worker identity used when neither env nor hostname is available.
pub const DEFAULT_WORKER_ID: &str = "replay-parser-worker";

/// Validated worker runtime configuration.
#[derive(Clone, Eq, PartialEq)]
pub struct WorkerConfig {
    /// `RabbitMQ` AMQP URL.
    pub amqp_url: String,
    /// `RabbitMQ` job queue name.
    pub job_queue: String,
    /// `RabbitMQ` exchange used for completed and failed result messages.
    pub result_exchange: String,
    /// Routing key for completed parse results.
    pub completed_routing_key: String,
    /// Routing key for failed parse results.
    pub failed_routing_key: String,
    /// S3 bucket containing raw replays and parser artifacts.
    pub s3_bucket: String,
    /// S3 region.
    pub s3_region: String,
    /// Optional S3-compatible endpoint URL.
    pub s3_endpoint: Option<String>,
    /// Whether to force path-style S3 addressing.
    pub s3_force_path_style: bool,
    /// Prefix used for parser artifact object keys.
    pub artifact_prefix: String,
    /// `RabbitMQ` prefetch count. Phase 6 defaults this to one in-flight job.
    pub prefetch: u16,
    /// HTTP probe bind address.
    pub probe_bind: String,
    /// HTTP probe port.
    pub probe_port: u16,
    /// Whether HTTP probes are enabled.
    pub probes_enabled: bool,
    /// Operator-visible worker identity for probes and logs.
    pub worker_id: String,
}

/// Explicit configuration values supplied by the CLI.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkerConfigOverrides {
    /// `RabbitMQ` AMQP URL.
    pub amqp_url: Option<String>,
    /// `RabbitMQ` job queue name.
    pub job_queue: Option<String>,
    /// `RabbitMQ` exchange used for completed and failed result messages.
    pub result_exchange: Option<String>,
    /// Routing key for completed parse results.
    pub completed_routing_key: Option<String>,
    /// Routing key for failed parse results.
    pub failed_routing_key: Option<String>,
    /// S3 bucket containing raw replays and parser artifacts.
    pub s3_bucket: Option<String>,
    /// S3 region.
    pub s3_region: Option<String>,
    /// Optional S3-compatible endpoint URL.
    pub s3_endpoint: Option<String>,
    /// Whether to force path-style S3 addressing.
    pub s3_force_path_style: Option<bool>,
    /// Prefix used for parser artifact object keys.
    pub artifact_prefix: Option<String>,
    /// `RabbitMQ` prefetch count.
    pub prefetch: Option<u16>,
    /// HTTP probe bind address.
    pub probe_bind: Option<String>,
    /// HTTP probe port.
    pub probe_port: Option<u16>,
    /// Whether HTTP probes are enabled.
    pub probes_enabled: Option<bool>,
    /// Operator-visible worker identity for probes and logs.
    pub worker_id: Option<String>,
}

impl WorkerConfig {
    /// Builds worker configuration from process environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerError`] when required settings are missing or invalid.
    pub fn from_env() -> Result<Self, WorkerError> {
        Self::from_env_and_overrides(
            |name| std::env::var(name).ok(),
            WorkerConfigOverrides::default(),
        )
    }

    /// Builds worker configuration from an environment source and explicit overrides.
    ///
    /// Explicit overrides take precedence over environment variables, and
    /// environment variables take precedence over safe defaults.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerError`] when required settings are missing or invalid.
    pub fn from_env_and_overrides(
        mut env: impl FnMut(&str) -> Option<String>,
        overrides: WorkerConfigOverrides,
    ) -> Result<Self, WorkerError> {
        let config = Self {
            amqp_url: overrides
                .amqp_url
                .or_else(|| env(ENV_AMQP_URL))
                .unwrap_or_else(|| DEFAULT_AMQP_URL.to_owned()),
            job_queue: overrides
                .job_queue
                .or_else(|| env(ENV_JOB_QUEUE))
                .unwrap_or_else(|| DEFAULT_JOB_QUEUE.to_owned()),
            result_exchange: overrides
                .result_exchange
                .or_else(|| env(ENV_RESULT_EXCHANGE))
                .unwrap_or_else(|| DEFAULT_RESULT_EXCHANGE.to_owned()),
            completed_routing_key: overrides
                .completed_routing_key
                .or_else(|| env(ENV_COMPLETED_ROUTING_KEY))
                .unwrap_or_else(|| DEFAULT_COMPLETED_ROUTING_KEY.to_owned()),
            failed_routing_key: overrides
                .failed_routing_key
                .or_else(|| env(ENV_FAILED_ROUTING_KEY))
                .unwrap_or_else(|| DEFAULT_FAILED_ROUTING_KEY.to_owned()),
            s3_bucket: overrides
                .s3_bucket
                .or_else(|| env(ENV_S3_BUCKET))
                .ok_or_else(|| validation_error(format!("missing required {ENV_S3_BUCKET}")))?,
            s3_region: overrides
                .s3_region
                .or_else(|| env(ENV_S3_REGION))
                .unwrap_or_else(|| DEFAULT_S3_REGION.to_owned()),
            s3_endpoint: overrides.s3_endpoint.or_else(|| env(ENV_S3_ENDPOINT)),
            s3_force_path_style: match overrides.s3_force_path_style {
                Some(value) => value,
                None => env(ENV_S3_FORCE_PATH_STYLE)
                    .map(|value| parse_bool(ENV_S3_FORCE_PATH_STYLE, &value))
                    .transpose()?
                    .unwrap_or(false),
            },
            artifact_prefix: overrides
                .artifact_prefix
                .or_else(|| env(ENV_ARTIFACT_PREFIX))
                .unwrap_or_else(|| DEFAULT_ARTIFACT_PREFIX.to_owned()),
            prefetch: match overrides.prefetch {
                Some(value) => validate_prefetch(value)?,
                None => env(ENV_PREFETCH)
                    .map(|value| parse_prefetch(&value))
                    .transpose()?
                    .unwrap_or(DEFAULT_PREFETCH),
            },
            probe_bind: overrides
                .probe_bind
                .or_else(|| env(ENV_PROBE_BIND))
                .unwrap_or_else(|| DEFAULT_PROBE_BIND.to_owned()),
            probe_port: match overrides.probe_port {
                Some(value) => value,
                None => env(ENV_PROBE_PORT)
                    .map(|value| parse_probe_port(&value))
                    .transpose()?
                    .unwrap_or(DEFAULT_PROBE_PORT),
            },
            probes_enabled: match overrides.probes_enabled {
                Some(value) => value,
                None => env(ENV_PROBES_ENABLED)
                    .map(|value| parse_bool(ENV_PROBES_ENABLED, &value))
                    .transpose()?
                    .unwrap_or(DEFAULT_PROBES_ENABLED),
            },
            worker_id: overrides
                .worker_id
                .or_else(|| env(ENV_WORKER_ID))
                .or_else(|| env("HOSTNAME"))
                .unwrap_or_else(|| DEFAULT_WORKER_ID.to_owned()),
        };
        config.validate()?;
        Ok(config)
    }

    /// Validates required non-empty fields and bounded runtime settings.
    ///
    /// # Errors
    ///
    /// Returns [`WorkerError`] when any required setting is empty or invalid.
    pub fn validate(&self) -> Result<(), WorkerError> {
        validate_non_empty("amqp_url", &self.amqp_url)?;
        validate_non_empty("job_queue", &self.job_queue)?;
        validate_non_empty("result_exchange", &self.result_exchange)?;
        validate_non_empty("completed_routing_key", &self.completed_routing_key)?;
        validate_non_empty("failed_routing_key", &self.failed_routing_key)?;
        validate_non_empty(ENV_S3_BUCKET, &self.s3_bucket)?;
        validate_non_empty("s3_region", &self.s3_region)?;
        validate_non_empty("artifact_prefix", &self.artifact_prefix)?;
        _ = validate_prefetch(self.prefetch)?;
        if self.probes_enabled {
            validate_non_empty("probe_bind", &self.probe_bind)?;
            validate_probe_port(self.probe_port)?;
            validate_non_empty("worker_id", &self.worker_id)?;
        }
        Ok(())
    }

    /// Returns a redacted representation suitable for logs.
    #[must_use]
    pub const fn redacted(&self) -> RedactedWorkerConfig<'_> {
        RedactedWorkerConfig { config: self }
    }
}

impl fmt::Debug for WorkerConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted().fmt(formatter)
    }
}

/// Redacted worker configuration for logs and diagnostics.
#[derive(Clone, Copy)]
pub struct RedactedWorkerConfig<'a> {
    config: &'a WorkerConfig,
}

impl fmt::Debug for RedactedWorkerConfig<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkerConfig")
            .field("amqp_url", &redact_amqp_url(&self.config.amqp_url))
            .field("job_queue", &self.config.job_queue)
            .field("result_exchange", &self.config.result_exchange)
            .field("completed_routing_key", &self.config.completed_routing_key)
            .field("failed_routing_key", &self.config.failed_routing_key)
            .field("s3_bucket", &self.config.s3_bucket)
            .field("s3_region", &self.config.s3_region)
            .field("s3_endpoint", &self.config.s3_endpoint)
            .field("s3_force_path_style", &self.config.s3_force_path_style)
            .field("artifact_prefix", &self.config.artifact_prefix)
            .field("prefetch", &self.config.prefetch)
            .field("probe_bind", &self.config.probe_bind)
            .field("probe_port", &self.config.probe_port)
            .field("probes_enabled", &self.config.probes_enabled)
            .field("worker_id", &self.config.worker_id)
            .finish()
    }
}

fn parse_bool(name: &str, value: &str) -> Result<bool, WorkerError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(validation_error(format!("{name} must be a boolean"))),
    }
}

fn parse_prefetch(value: &str) -> Result<u16, WorkerError> {
    let parsed = value
        .parse::<u16>()
        .map_err(|_source| validation_error(format!("{ENV_PREFETCH} must be an integer >= 1")))?;
    validate_prefetch(parsed)
}

fn validate_prefetch(value: u16) -> Result<u16, WorkerError> {
    if value == 0 {
        return Err(validation_error(format!("{ENV_PREFETCH} must be >= 1")));
    }
    Ok(value)
}

fn parse_probe_port(value: &str) -> Result<u16, WorkerError> {
    let parsed = value
        .parse::<u16>()
        .map_err(|_source| validation_error(format!("{ENV_PROBE_PORT} must be an integer >= 1")))?;
    validate_probe_port(parsed)?;
    Ok(parsed)
}

fn validate_probe_port(value: u16) -> Result<(), WorkerError> {
    if value == 0 {
        return Err(validation_error(format!("{ENV_PROBE_PORT} must be >= 1")));
    }
    Ok(())
}

fn validate_non_empty(name: &str, value: &str) -> Result<(), WorkerError> {
    if value.trim().is_empty() {
        return Err(validation_error(format!("{name} must not be empty")));
    }
    Ok(())
}

const fn validation_error(message: String) -> WorkerError {
    WorkerError::ConfigValidation(message)
}

fn redact_amqp_url(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return redact_userinfo(url);
    };
    let scheme = &url[..scheme_end];
    let rest = &url[scheme_end + "://".len()..];
    format!("{scheme}://{}", redact_userinfo(rest))
}

fn redact_userinfo(value: &str) -> String {
    value.rfind('@').map_or_else(
        || value.to_owned(),
        |userinfo_end| format!("***@{}", &value[userinfo_end + 1..]),
    )
}
