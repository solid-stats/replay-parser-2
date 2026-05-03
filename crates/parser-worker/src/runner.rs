//! Worker runtime entrypoint.

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

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests use expect messages as assertion context")]

    use parser_contract::{source_ref::SourceChecksum, version::ContractVersion};

    use super::*;
    use crate::config::WorkerConfigOverrides;

    fn valid_config() -> WorkerConfig {
        WorkerConfig::from_env_and_overrides(
            |_| None,
            WorkerConfigOverrides {
                s3_bucket: Some("solid-replays".to_owned()),
                probes_enabled: Some(false),
                worker_id: Some("unit-worker".to_owned()),
                ..Default::default()
            },
        )
        .expect("worker config should be valid")
    }

    fn invalid_config() -> WorkerConfig {
        let mut config = valid_config();
        config.s3_bucket.clear();
        config
    }

    #[tokio::test]
    async fn runner_entrypoints_should_fail_fast_for_invalid_config_without_dependencies() {
        let run_error = run(invalid_config()).await.expect_err("invalid config should fail run");
        let shutdown = CancellationToken::new();
        let cancelled_error = run_until_cancelled(invalid_config(), shutdown)
            .await
            .expect_err("invalid config should fail run until cancelled");

        assert!(matches!(run_error, WorkerError::ConfigValidation(_)));
        assert!(matches!(cancelled_error, WorkerError::ConfigValidation(_)));
    }

    #[tokio::test]
    #[allow(clippy::panic, reason = "unit test must exercise Tokio join-panic handling")]
    async fn stop_probe_server_should_cancel_and_report_join_failures() {
        let no_server_shutdown = CancellationToken::new();
        stop_probe_server(no_server_shutdown.clone(), None)
            .await
            .expect("missing probe server should stop cleanly");

        let clean_shutdown = CancellationToken::new();
        let clean_handle = tokio::spawn(async { Ok(()) });
        stop_probe_server(clean_shutdown.clone(), Some(clean_handle))
            .await
            .expect("clean probe task should stop");

        let failed_shutdown = CancellationToken::new();
        let failed_handle = tokio::spawn(async { panic!("probe task panic for unit test") });
        let failed = stop_probe_server(failed_shutdown.clone(), Some(failed_handle))
            .await
            .expect_err("failed probe task should report metadata error");

        assert!(no_server_shutdown.is_cancelled());
        assert!(clean_shutdown.is_cancelled());
        assert!(failed_shutdown.is_cancelled());
        assert!(matches!(failed, WorkerError::ParserMetadata(_)));
    }

    #[test]
    fn runner_helpers_should_emit_parser_metadata_logs_and_readiness_state() {
        let config = valid_config();
        let health = HealthState::new("unit-worker");
        let degraded = WorkerError::ConfigValidation("broken dependency".to_owned());

        init_tracing();
        init_tracing();
        log_dependency_ready(&config, "s3");
        log_dependency_degraded(&config, "rabbitmq", "amqp_connect", &degraded);
        log_readiness_changed(&config, OUTCOME_READY, "ready");
        mark_draining_and_log(&config, &health);
        let parser = parser_info().expect("parser metadata should deserialize");
        let snapshot = serde_json::to_value(health.readyz_snapshot())
            .expect("health snapshot should serialize");

        assert_eq!(parser.name, "replay-parser-2");
        assert_eq!(parser.version.to_string(), env!("CARGO_PKG_VERSION"));
        assert_eq!(snapshot["state"], "draining");
        assert_eq!(snapshot["ready"], false);
    }

    #[test]
    fn log_fields_should_extract_valid_job_identity_and_ignore_malformed_json() {
        let checksum = SourceChecksum::sha256(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        )
        .expect("checksum should parse");
        let body = serde_json::to_vec(&ParseJobMessage {
            job_id: "job-1".to_owned(),
            replay_id: "replay-1".to_owned(),
            object_key: "raw/replay.json".to_owned(),
            checksum,
            parser_contract_version: ContractVersion::current(),
        })
        .expect("job message should serialize");

        let valid = log_fields(&body);
        let malformed = log_fields(b"{");

        assert_eq!(valid.job_id.as_deref(), Some("job-1"));
        assert_eq!(valid.replay_id.as_deref(), Some("replay-1"));
        assert_eq!(valid.object_key.as_deref(), Some("raw/replay.json"));
        assert_eq!(malformed.job_id, None);
        assert_eq!(malformed.replay_id, None);
        assert_eq!(malformed.object_key, None);
    }
}
