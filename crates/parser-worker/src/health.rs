//! Cached HTTP liveness and readiness probe state.

use std::sync::{Arc, RwLock};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{config::WorkerConfig, error::WorkerError};

/// Cached worker readiness state exposed through probe responses.
#[derive(Clone, Debug)]
pub struct HealthState {
    worker_id: Arc<str>,
    inner: Arc<RwLock<HealthInner>>,
}

impl HealthState {
    /// Creates a health state initialized as `starting`.
    #[must_use]
    pub fn new(worker_id: impl Into<String>) -> Self {
        Self {
            worker_id: Arc::from(worker_id.into()),
            inner: Arc::new(RwLock::new(HealthInner::new(ReadinessState::Starting, None))),
        }
    }

    /// Marks the worker as starting and not ready.
    pub fn mark_starting(&self) {
        self.store(ReadinessState::Starting, None);
    }

    /// Marks the worker as ready to consume parse jobs.
    pub fn mark_ready(&self) {
        self.store(ReadinessState::Ready, None);
    }

    /// Marks the worker as live but temporarily not ready.
    pub fn mark_degraded(&self, reason: impl Into<String>) {
        self.store(ReadinessState::Degraded, Some(reason.into()));
    }

    /// Marks the worker as draining in-flight work and not ready.
    pub fn mark_draining(&self) {
        self.store(ReadinessState::Draining, None);
    }

    /// Marks the worker as fatally unhealthy.
    pub fn mark_fatal(&self, reason: impl Into<String>) {
        self.store(ReadinessState::Fatal, Some(reason.into()));
    }

    /// Returns a liveness probe snapshot.
    #[must_use]
    pub fn livez_snapshot(&self) -> HealthSnapshot {
        let inner = self.load();
        HealthSnapshot {
            status: if inner.state.is_live() { "live" } else { "fatal" },
            ready: inner.state.is_ready(),
            worker_id: self.worker_id.to_string(),
            state: inner.state,
            reason: inner.reason,
        }
    }

    /// Returns a readiness probe snapshot.
    #[must_use]
    pub fn readyz_snapshot(&self) -> HealthSnapshot {
        let inner = self.load();
        HealthSnapshot {
            status: if inner.state.is_ready() { "ready" } else { "not_ready" },
            ready: inner.state.is_ready(),
            worker_id: self.worker_id.to_string(),
            state: inner.state,
            reason: inner.reason,
        }
    }

    /// Returns the operator-visible worker identity.
    #[must_use]
    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }

    fn store(&self, state: ReadinessState, reason: Option<String>) {
        let mut guard = self.inner.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = HealthInner::new(state, reason);
    }

    fn load(&self) -> HealthInner {
        self.inner.read().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }
}

/// Probe JSON response body.
#[derive(Clone, Debug, Serialize)]
pub struct HealthSnapshot {
    status: &'static str,
    ready: bool,
    worker_id: String,
    state: ReadinessState,
    reason: Option<String>,
}

/// Cached readiness state names returned by `/readyz` and `/livez`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessState {
    /// Worker has started but dependencies are not ready.
    Starting,
    /// Worker dependencies are active and shutdown has not been requested.
    Ready,
    /// Worker is live but a dependency path is not ready.
    Degraded,
    /// Worker is draining in-flight work after shutdown was requested.
    Draining,
    /// Worker hit a fatal internal state.
    Fatal,
}

impl ReadinessState {
    const fn is_live(self) -> bool {
        matches!(self, Self::Starting | Self::Ready | Self::Degraded | Self::Draining)
    }

    const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Clone, Debug)]
struct HealthInner {
    state: ReadinessState,
    reason: Option<String>,
}

impl HealthInner {
    const fn new(state: ReadinessState, reason: Option<String>) -> Self {
        Self { state, reason }
    }
}

/// Builds the HTTP probe router.
pub fn probe_router(state: HealthState) -> Router {
    Router::new().route("/livez", get(livez)).route("/readyz", get(readyz)).with_state(state)
}

/// Spawns the HTTP probe server when probes are enabled.
///
/// # Errors
///
/// Returns [`WorkerError`] when the configured bind address or port cannot be bound.
pub async fn spawn_probe_server(
    config: &WorkerConfig,
    state: HealthState,
    shutdown: CancellationToken,
) -> Result<Option<JoinHandle<Result<(), WorkerError>>>, WorkerError> {
    if !config.probes_enabled {
        return Ok(None);
    }

    let listener = tokio::net::TcpListener::bind((config.probe_bind.as_str(), config.probe_port))
        .await
        .map_err(|source| {
            WorkerError::ConfigValidation(format!(
                "could not bind probe listener on {}:{}: {source}",
                config.probe_bind, config.probe_port
            ))
        })?;
    let router = probe_router(state);
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                shutdown.cancelled().await;
            })
            .await
            .map_err(|source| {
                WorkerError::ConfigValidation(format!("probe server failed: {source}"))
            })
    });

    Ok(Some(handle))
}

async fn livez(State(state): State<HealthState>) -> (StatusCode, Json<HealthSnapshot>) {
    let snapshot = state.livez_snapshot();
    let status =
        if snapshot.state.is_live() { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, Json(snapshot))
}

async fn readyz(State(state): State<HealthState>) -> (StatusCode, Json<HealthSnapshot>) {
    let snapshot = state.readyz_snapshot();
    let status = if snapshot.ready { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, Json(snapshot))
}
