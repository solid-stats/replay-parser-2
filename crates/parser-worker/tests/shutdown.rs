//! Shutdown drain behavior tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use futures_util::stream;
use parser_worker::{
    amqp::DeliveryAction,
    error::WorkerError,
    shutdown::{
        ShutdownDelivery, ShutdownDeliveryAcker, ShutdownFuture, ShutdownJobProcessor,
        drain_until_cancelled,
    },
};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
struct FakeAcker {
    calls: Arc<Mutex<Vec<DeliveryAction>>>,
}

impl FakeAcker {
    fn new(calls: Arc<Mutex<Vec<DeliveryAction>>>) -> Self {
        Self { calls }
    }
}

impl ShutdownDeliveryAcker for FakeAcker {
    fn apply<'a>(&'a mut self, action: DeliveryAction) -> ShutdownFuture<'a, ()> {
        Box::pin(async move {
            self.calls.lock().expect("fake acker calls lock should not be poisoned").push(action);
            Ok(())
        })
    }
}

#[derive(Debug)]
struct CancelAfterFirstProcessor {
    token: CancellationToken,
    calls: Arc<AtomicUsize>,
}

impl ShutdownJobProcessor for CancelAfterFirstProcessor {
    fn process_job<'a>(&'a self, _body: &'a [u8]) -> ShutdownFuture<'a, DeliveryAction> {
        Box::pin(async move {
            let _previous_calls = self.calls.fetch_add(1, Ordering::SeqCst);
            self.token.cancel();
            Ok(DeliveryAction::Ack)
        })
    }
}

#[derive(Debug)]
struct StaticProcessor {
    token: Option<CancellationToken>,
    action: DeliveryAction,
}

impl ShutdownJobProcessor for StaticProcessor {
    fn process_job<'a>(&'a self, _body: &'a [u8]) -> ShutdownFuture<'a, DeliveryAction> {
        Box::pin(async move {
            if let Some(token) = &self.token {
                token.cancel();
            }
            Ok(self.action)
        })
    }
}

#[derive(Debug)]
struct DelayedProcessor {
    started: Mutex<Option<oneshot::Sender<()>>>,
    release: Mutex<Option<oneshot::Receiver<DeliveryAction>>>,
}

impl DelayedProcessor {
    fn new(started: oneshot::Sender<()>, release: oneshot::Receiver<DeliveryAction>) -> Self {
        Self { started: Mutex::new(Some(started)), release: Mutex::new(Some(release)) }
    }
}

impl ShutdownJobProcessor for DelayedProcessor {
    fn process_job<'a>(&'a self, _body: &'a [u8]) -> ShutdownFuture<'a, DeliveryAction> {
        let started = self.started.lock().expect("started lock should not be poisoned").take();
        let release = self
            .release
            .lock()
            .expect("release lock should not be poisoned")
            .take()
            .expect("release receiver should be available once");

        Box::pin(async move {
            if let Some(started) = started {
                _ = started.send(());
            }
            release.await.map_err(|error| WorkerError::ParserMetadata(error.to_string()))
        })
    }
}

fn delivery(calls: Arc<Mutex<Vec<DeliveryAction>>>, body: &[u8]) -> ShutdownDelivery<FakeAcker> {
    ShutdownDelivery { body: body.to_vec(), acker: FakeAcker::new(calls) }
}

fn acker_calls(calls: &Arc<Mutex<Vec<DeliveryAction>>>) -> Vec<DeliveryAction> {
    calls.lock().expect("fake acker calls lock should not be poisoned").clone()
}

#[tokio::test]
async fn shutdown_cancellation_before_next_delivery_should_stop_after_first_ack() {
    // Arrange
    let token = CancellationToken::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let process_calls = Arc::new(AtomicUsize::new(0));
    let processor =
        CancelAfterFirstProcessor { token: token.clone(), calls: Arc::clone(&process_calls) };
    let mut deliveries = stream::iter(vec![
        delivery(Arc::clone(&calls), b"first"),
        delivery(Arc::clone(&calls), b"second"),
    ]);

    // Act
    let report = drain_until_cancelled(&mut deliveries, token, &processor)
        .await
        .expect("drain should finish cleanly");

    // Assert
    assert_eq!(report.processed, 1);
    assert_eq!(process_calls.load(Ordering::SeqCst), 1);
    assert_eq!(acker_calls(&calls), vec![DeliveryAction::Ack]);
}

#[tokio::test]
async fn shutdown_cancellation_during_in_flight_job_should_wait_for_ack() {
    // Arrange
    let token = CancellationToken::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let (started_tx, started_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let processor = DelayedProcessor::new(started_tx, release_rx);
    let mut deliveries = stream::iter(vec![delivery(Arc::clone(&calls), b"in-flight")]);
    let drain_token = token.clone();

    // Act
    let handle = tokio::spawn(async move {
        drain_until_cancelled(&mut deliveries, drain_token, &processor).await
    });
    started_rx.await.expect("processor should start");
    token.cancel();
    assert!(acker_calls(&calls).is_empty());
    release_tx.send(DeliveryAction::Ack).expect("release should send action");
    let report = handle.await.expect("drain task should join").expect("drain should pass");

    // Assert
    assert_eq!(report.processed, 1);
    assert_eq!(report.last_action, Some(DeliveryAction::Ack));
    assert_eq!(acker_calls(&calls), vec![DeliveryAction::Ack]);
}

#[tokio::test]
async fn shutdown_publish_failure_action_should_apply_nack_requeue() {
    // Arrange
    let token = CancellationToken::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let processor =
        StaticProcessor { token: Some(token.clone()), action: DeliveryAction::NackRequeue };
    let mut deliveries = stream::iter(vec![delivery(Arc::clone(&calls), b"failed-publish")]);

    // Act
    let report = drain_until_cancelled(&mut deliveries, token, &processor)
        .await
        .expect("drain should finish cleanly");

    // Assert
    assert_eq!(report.processed, 1);
    assert_eq!(report.last_action, Some(DeliveryAction::NackRequeue));
    assert_eq!(acker_calls(&calls), vec![DeliveryAction::NackRequeue]);
}

#[test]
fn shutdown_should_not_add_http_probe_surface() {
    let source = [include_str!("../src/runner.rs"), include_str!("../src/shutdown.rs")].join("\n");
    for needle in [
        concat!("he", "alth"),
        concat!("read", "iness"),
        concat!("HEALTH", "CHECK"),
        concat!("/", "he", "alth"),
    ] {
        assert!(!source.contains(needle));
    }
}
