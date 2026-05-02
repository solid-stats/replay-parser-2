//! Worker configuration shell.

/// Validated worker runtime configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerConfig {
    /// RabbitMQ AMQP URL.
    pub amqp_url: String,
}

impl WorkerConfig {
    /// Returns a redacted representation suitable for logs.
    #[must_use]
    pub fn redacted(&self) -> RedactedWorkerConfig<'_> {
        RedactedWorkerConfig { config: self }
    }
}

/// Redacted worker configuration for logs and diagnostics.
#[derive(Clone, Copy, Debug)]
pub struct RedactedWorkerConfig<'a> {
    config: &'a WorkerConfig,
}

impl RedactedWorkerConfig<'_> {
    /// Returns the configured AMQP URL as currently redacted.
    #[must_use]
    pub fn amqp_url(&self) -> &str {
        &self.config.amqp_url
    }
}
