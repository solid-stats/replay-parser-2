//! HTTP probe behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use parser_worker::{
    config::{WorkerConfig, WorkerConfigOverrides},
    health::{HealthState, probe_router, spawn_probe_server},
};
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

mod health {
    use super::*;

    fn worker_config(overrides: WorkerConfigOverrides) -> WorkerConfig {
        WorkerConfig::from_env_and_overrides(|_| None, overrides)
            .expect("test worker config should be valid")
    }

    async fn get_json(app: Router, path: &str) -> (StatusCode, Value) {
        let response = app
            .oneshot(
                Request::builder().uri(path).body(Body::empty()).expect("request should build"),
            )
            .await
            .expect("probe request should be handled");
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("probe body should be readable");
        let json = serde_json::from_slice(&body).expect("probe body should be JSON");
        (status, json)
    }

    #[tokio::test]
    async fn readyz_should_return_503_until_ready() {
        // Arrange
        let state = HealthState::new("worker-health-test");
        state.mark_starting();

        // Act
        let (status, body) = get_json(probe_router(state), "/readyz").await;

        // Assert
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["status"], "not_ready");
        assert_eq!(body["ready"], Value::Bool(false));
        assert_eq!(body["worker_id"], "worker-health-test");
        assert_eq!(body["state"], "starting");
        assert!(body["reason"].is_null());
    }

    #[tokio::test]
    async fn readyz_should_return_200_when_state_is_ready() {
        // Arrange
        let state = HealthState::new("worker-health-test");
        state.mark_ready();

        // Act
        let (status, body) = get_json(probe_router(state), "/readyz").await;

        // Assert
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ready");
        assert_eq!(body["ready"], Value::Bool(true));
        assert_eq!(body["state"], "ready");
    }

    #[tokio::test]
    async fn livez_should_remain_200_when_dependencies_are_degraded() {
        // Arrange
        let state = HealthState::new("worker-health-test");
        state.mark_degraded("s3_ready");

        // Act
        let (status, body) = get_json(probe_router(state), "/livez").await;

        // Assert
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "live");
        assert_eq!(body["ready"], Value::Bool(false));
        assert_eq!(body["state"], "degraded");
        assert_eq!(body["reason"], "s3_ready");
    }

    #[tokio::test]
    async fn livez_should_return_503_when_state_is_fatal() {
        // Arrange
        let state = HealthState::new("worker-health-test");
        state.mark_fatal("probe_bind");

        // Act
        let (status, body) = get_json(probe_router(state), "/livez").await;

        // Assert
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["status"], "fatal");
        assert_eq!(body["ready"], Value::Bool(false));
        assert_eq!(body["state"], "fatal");
        assert_eq!(body["reason"], "probe_bind");
    }

    #[tokio::test]
    async fn runner_readiness_should_start_false_before_dependency_checks_pass() {
        // Arrange
        let state = HealthState::new("worker-health-test");
        state.mark_starting();

        // Act
        let (status, body) = get_json(probe_router(state), "/readyz").await;

        // Assert
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["ready"], Value::Bool(false));
        assert_eq!(body["state"], "starting");
    }

    #[tokio::test]
    async fn runner_readiness_should_flip_true_after_dependency_ready_transition() {
        // Arrange
        let state = HealthState::new("worker-health-test");
        state.mark_starting();
        state.mark_ready();

        // Act
        let (status, body) = get_json(probe_router(state), "/readyz").await;

        // Assert
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["ready"], Value::Bool(true));
        assert_eq!(body["state"], "ready");
    }

    #[tokio::test]
    async fn readiness_should_flip_false_when_shutdown_is_requested() {
        // Arrange
        let state = HealthState::new("worker-health-test");
        state.mark_ready();
        state.mark_draining();

        // Act
        let (status, body) = get_json(probe_router(state), "/readyz").await;

        // Assert
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["ready"], Value::Bool(false));
        assert_eq!(body["state"], "draining");
    }

    #[tokio::test]
    async fn probe_server_should_not_spawn_when_probes_are_disabled() {
        // Arrange
        let config = worker_config(WorkerConfigOverrides {
            s3_bucket: Some("solid-replays".to_owned()),
            probes_enabled: Some(false),
            ..WorkerConfigOverrides::default()
        });
        let state = HealthState::new("worker-health-test");

        // Act
        let handle = spawn_probe_server(&config, state, CancellationToken::new())
            .await
            .expect("disabled probes should not fail");

        // Assert
        assert!(handle.is_none());
    }

    #[tokio::test]
    async fn probe_server_should_spawn_and_shutdown_when_probes_are_enabled() {
        // Arrange
        let mut config = worker_config(WorkerConfigOverrides {
            s3_bucket: Some("solid-replays".to_owned()),
            probes_enabled: Some(false),
            ..WorkerConfigOverrides::default()
        });
        config.probes_enabled = true;
        config.probe_bind = "127.0.0.1".to_owned();
        config.probe_port = 0;
        let state = HealthState::new("worker-health-test");
        let shutdown = CancellationToken::new();

        // Act
        let handle = spawn_probe_server(&config, state.clone(), shutdown.clone())
            .await
            .expect("enabled probes should bind")
            .expect("enabled probes should spawn");
        shutdown.cancel();
        let server_result = handle.await.expect("probe task should join");

        // Assert
        assert_eq!(state.worker_id(), "worker-health-test");
        server_result.expect("probe server should shut down cleanly");
    }

    #[tokio::test]
    async fn probe_server_should_return_config_error_when_bind_address_is_invalid() {
        // Arrange
        let config = worker_config(WorkerConfigOverrides {
            s3_bucket: Some("solid-replays".to_owned()),
            probe_bind: Some("256.256.256.256".to_owned()),
            probe_port: Some(18080),
            probes_enabled: Some(true),
            ..WorkerConfigOverrides::default()
        });
        let state = HealthState::new("worker-health-test");

        // Act
        let error = spawn_probe_server(&config, state, CancellationToken::new())
            .await
            .expect_err("invalid bind address should fail");

        // Assert
        assert!(error.to_string().contains("could not bind probe listener"));
    }
}
