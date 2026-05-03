//! Worker runtime entrypoint. coverage-exclusion: reviewed live S3/RabbitMQ runner and signal/tracing regions are allowlisted by exact source line.

use futures_util::StreamExt;
use parser_contract::{version::ParserInfo, worker::ParseJobMessage};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::{
    amqp::{RabbitMqClient, apply_lapin_delivery_action},
    config::WorkerConfig,
    error::WorkerError,
    health::{HealthState, spawn_probe_server},
    logging::{
        OUTCOME_DEGRADED, OUTCOME_DRAINING, OUTCOME_READY, WORKER_CONNECTED,
        WORKER_DEPENDENCY_DEGRADED, WORKER_DEPENDENCY_READY, WORKER_JOB_RECEIVED,
        WORKER_READINESS_CHANGED, WORKER_SHUTDOWN_COMPLETE, WORKER_SHUTDOWN_REQUESTED,
        WORKER_STARTING,
    },
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
    let shutdown = CancellationToken::new();
    run_with_shutdown(config, shutdown, true).await
}

/// Starts the worker runtime and exits when the supplied cancellation token is cancelled.
///
/// This entrypoint is used by deployment smoke tests that run the worker against live
/// RabbitMQ/S3-compatible services and need deterministic shutdown without sending a process signal.
///
/// # Errors
///
/// Returns [`WorkerError`] when configuration, storage, AMQP, processing, or ack/nack
/// operations fail.
pub async fn run_until_cancelled(
    config: WorkerConfig,
    shutdown: CancellationToken,
) -> Result<(), WorkerError> {
    run_with_shutdown(config, shutdown, false).await
}

async fn run_with_shutdown(
    config: WorkerConfig,
    shutdown: CancellationToken,
    listen_for_ctrl_c: bool,
) -> Result<(), WorkerError> {
    init_tracing();
    let health = HealthState::new(config.worker_id.clone());
    if let Err(error) = config.validate() {
        health.mark_fatal("config_validation");
        return Err(error);
    }
    health.mark_starting();
    if listen_for_ctrl_c {
        spawn_ctrl_c_listener(shutdown.clone(), health.clone());
    }
    tracing::info!(
        event = WORKER_STARTING,
        worker_id = %config.worker_id,
        config = ?config.redacted(),
        "worker_starting"
    );

    let probe_server = match spawn_probe_server(&config, health.clone(), shutdown.clone()).await {
        Ok(handle) => handle,
        Err(error) => {
            health.mark_fatal("probe_bind");
            return Err(error);
        }
    };

    let store = match S3ObjectStore::from_config(&config).await {
        Ok(store) => store,
        Err(error) => {
            health.mark_fatal("config_validation");
            stop_probe_server(shutdown, probe_server).await?;
            return Err(error);
        }
    };
    if let Err(error) = store.check_ready().await {
        health.mark_degraded("s3_ready");
        log_dependency_degraded(&config, "s3", "s3_ready", &error);
        stop_probe_server(shutdown, probe_server).await?;
        return Err(error);
    }
    log_dependency_ready(&config, "s3");

    let mut rabbit = match RabbitMqClient::connect(&config).await {
        Ok(rabbit) => rabbit,
        Err(error) => {
            health.mark_degraded("amqp_connect");
            log_dependency_degraded(&config, "rabbitmq", "amqp_connect", &error);
            stop_probe_server(shutdown, probe_server).await?;
            return Err(error);
        }
    };
    health.mark_ready();
    log_dependency_ready(&config, "rabbitmq");
    log_readiness_changed(&config, OUTCOME_READY, "ready");
    tracing::info!(
        event = WORKER_CONNECTED,
        worker_id = %config.worker_id,
        job_queue = %config.job_queue,
        result_exchange = %config.result_exchange,
        prefetch = config.prefetch,
        "worker_connected"
    );

    let result = consume_until_shutdown(
        &config,
        &mut rabbit,
        &store,
        shutdown.clone(),
        health.clone(),
        parser_info()?,
    )
    .await;
    if result.is_err() {
        health.mark_fatal("worker_runtime");
    }
    stop_probe_server(shutdown, probe_server).await?;
    result
}

async fn consume_until_shutdown(
    config: &WorkerConfig,
    rabbit: &mut RabbitMqClient,
    store: &S3ObjectStore,
    shutdown: CancellationToken,
    health: HealthState,
    parser: ParserInfo,
) -> Result<(), WorkerError> {
    loop {
        if shutdown.is_cancelled() {
            mark_draining_and_log(config, &health);
            break;
        }

        let delivery = tokio::select! {
            () = shutdown.cancelled() => {
                mark_draining_and_log(config, &health);
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
            event = WORKER_JOB_RECEIVED,
            worker_id = %config.worker_id,
            job_id = ?fields.job_id.as_deref(),
            replay_id = ?fields.replay_id.as_deref(),
            object_key = ?fields.object_key.as_deref(),
            "messaging.destination.name" = %config.job_queue,
            "messaging.rabbitmq.message.delivery_tag" = delivery.delivery_tag,
            "messaging.rabbitmq.destination.routing_key" = %delivery.routing_key,
            "worker_job_received"
        );

        let action =
            process_job_body(&delivery.data, config, store, rabbit, parser.clone()).await?;
        apply_lapin_delivery_action(&delivery, action, &config.worker_id).await?;

        if shutdown.is_cancelled() {
            mark_draining_and_log(config, &health);
            break;
        }
    }

    tracing::info!(
        event = WORKER_SHUTDOWN_COMPLETE,
        worker_id = %config.worker_id,
        "worker_shutdown_complete"
    );
    Ok(())
}

fn log_dependency_ready(config: &WorkerConfig, dependency: &str) {
    tracing::info!(
        event = WORKER_DEPENDENCY_READY,
        worker_id = %config.worker_id,
        dependency = %dependency,
        outcome = OUTCOME_READY,
        "worker_dependency_ready"
    );
}

fn log_dependency_degraded(
    config: &WorkerConfig,
    dependency: &str,
    error_type: &str,
    error: &WorkerError,
) {
    tracing::warn!(
        event = WORKER_DEPENDENCY_DEGRADED,
        worker_id = %config.worker_id,
        dependency = %dependency,
        outcome = OUTCOME_DEGRADED,
        error_type = %error_type,
        error = %error,
        "worker_dependency_degraded"
    );
}

fn mark_draining_and_log(config: &WorkerConfig, health: &HealthState) {
    health.mark_draining();
    log_readiness_changed(config, OUTCOME_DRAINING, "draining");
}

fn log_readiness_changed(config: &WorkerConfig, outcome: &str, state: &str) {
    tracing::info!(
        event = WORKER_READINESS_CHANGED,
        worker_id = %config.worker_id,
        outcome = %outcome,
        state = %state,
        "worker_readiness_changed"
    );
}

async fn stop_probe_server(
    shutdown: CancellationToken,
    probe_server: Option<tokio::task::JoinHandle<Result<(), WorkerError>>>,
) -> Result<(), WorkerError> {
    shutdown.cancel();
    let Some(probe_server) = probe_server else {
        return Ok(());
    };

    probe_server.await.map_err(|source| {
        WorkerError::ParserMetadata(format!("probe server task failed: {source}"))
    })?
}

fn spawn_ctrl_c_listener(shutdown: CancellationToken, health: HealthState) {
    let _shutdown_task = tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!(
                    event = WORKER_SHUTDOWN_REQUESTED,
                    worker_id = %health.worker_id(),
                    "worker_shutdown_requested"
                );
                health.mark_draining();
                shutdown.cancel();
            }
            Err(error) => {
                tracing::warn!(
                    event = WORKER_SHUTDOWN_REQUESTED,
                    worker_id = %health.worker_id(),
                    error = %error,
                    "worker_shutdown_requested"
                );
                health.mark_draining();
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
