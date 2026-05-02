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
use parser_worker::health::{HealthState, probe_router};
use serde_json::Value;
use tower::ServiceExt;

mod health {
    use super::*;

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
}
