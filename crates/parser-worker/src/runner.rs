//! Worker runtime entrypoint.

use crate::{config::WorkerConfig, error::WorkerError};

/// Starts the worker runtime.
///
/// Later Phase 6 plans add RabbitMQ consumption and S3 object processing. This
/// shell validates the runtime boundary and emits redacted startup logging.
pub async fn run(config: WorkerConfig) -> Result<(), WorkerError> {
    config.validate()?;
    tracing::info!(config = ?config.redacted(), "starting parser worker runtime");
    Ok(())
}
