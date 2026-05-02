//! Worker runtime entrypoint.

use futures_util::StreamExt;
use parser_contract::{version::ParserInfo, worker::ParseJobMessage};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::{
    amqp::{RabbitMqClient, apply_lapin_delivery_action},
    config::WorkerConfig,
    error::WorkerError,
    processor::process_job_body,
    storage::S3ObjectStore,
};

/// Starts the worker runtime.
///
/// The runner owns live AMQP/S3 clients, shutdown signal handling, and applying the
/// processor's ack/nack decision to each delivery.
///
/// # Errors
///
/// Returns [`WorkerError`] when configuration, storage, AMQP, processing, or ack/nack
/// operations fail.
pub async fn run(config: WorkerConfig) -> Result<(), WorkerError> {
    init_tracing();
    config.validate()?;
    tracing::info!(
        event = "worker_starting",
        config = ?config.redacted(),
        "worker_starting"
    );

    let store = S3ObjectStore::from_config(&config).await?;
    let mut rabbit = RabbitMqClient::connect(&config).await?;
    tracing::info!(
        event = "worker_connected",
        job_queue = %config.job_queue,
        result_exchange = %config.result_exchange,
        prefetch = config.prefetch,
        "worker_connected"
    );

    let shutdown = CancellationToken::new();
    spawn_ctrl_c_listener(shutdown.clone());
    consume_until_shutdown(&config, &mut rabbit, &store, shutdown, parser_info()?).await
}

async fn consume_until_shutdown(
    config: &WorkerConfig,
    rabbit: &mut RabbitMqClient,
    store: &S3ObjectStore,
    shutdown: CancellationToken,
    parser: ParserInfo,
) -> Result<(), WorkerError> {
    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let delivery = tokio::select! {
            () = shutdown.cancelled() => {
                break;
            }
            delivery = rabbit.consumer_mut().next() => delivery,
        };

        let Some(delivery) = delivery else {
            break;
        };
        let delivery = delivery?;
        let fields = log_fields(&delivery.data);
        tracing::info!(
            event = "worker_job_received",
            job_id = ?fields.job_id.as_deref(),
            replay_id = ?fields.replay_id.as_deref(),
            object_key = ?fields.object_key.as_deref(),
            "worker_job_received"
        );

        let action =
            process_job_body(&delivery.data, config, store, rabbit, parser.clone()).await?;
        apply_lapin_delivery_action(&delivery, action).await?;

        if shutdown.is_cancelled() {
            break;
        }
    }

    tracing::info!(event = "worker_shutdown_complete", "worker_shutdown_complete");
    Ok(())
}

fn spawn_ctrl_c_listener(shutdown: CancellationToken) {
    let _shutdown_task = tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!(event = "worker_shutdown_requested", "worker_shutdown_requested");
                shutdown.cancel();
            }
            Err(error) => {
                tracing::warn!(
                    event = "worker_shutdown_requested",
                    error = %error,
                    "worker_shutdown_requested"
                );
                shutdown.cancel();
            }
        }
    });
}

fn parser_info() -> Result<ParserInfo, WorkerError> {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": env!("CARGO_PKG_VERSION")
    }))
    .map_err(|source| WorkerError::ParserMetadata(source.to_string()))
}

fn init_tracing() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .finish();
    let _set_result = tracing::subscriber::set_global_default(subscriber);
}

#[derive(Debug, Default)]
struct LogFields {
    job_id: Option<String>,
    replay_id: Option<String>,
    object_key: Option<String>,
}

fn log_fields(body: &[u8]) -> LogFields {
    serde_json::from_slice::<ParseJobMessage>(body).map_or_else(
        |_error| LogFields::default(),
        |message| LogFields {
            job_id: Some(message.job_id),
            replay_id: Some(message.replay_id),
            object_key: Some(message.object_key),
        },
    )
}
