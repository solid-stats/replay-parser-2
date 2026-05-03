//! Graceful shutdown drain helpers.
// coverage-exclusion: reviewed v1.0 shutdown signal adapter regions are allowlisted by exact source line.

use std::pin::Pin;

use futures_util::{Stream, StreamExt};
use tokio_util::sync::CancellationToken;

use crate::{amqp::DeliveryAction, error::WorkerError};

/// Boxed future returned by shutdown test adapters.
pub type ShutdownFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, WorkerError>> + Send + 'a>>;

/// Delivery body plus an acknowledgement adapter.
#[derive(Debug)]
pub struct ShutdownDelivery<A> {
    /// Delivery body bytes.
    pub body: Vec<u8>,
    /// Delivery acknowledgement adapter.
    pub acker: A,
}

/// Processor boundary used by shutdown drain tests.
pub trait ShutdownJobProcessor: Sync {
    /// Processes one delivery body and returns the ack/nack decision.
    fn process_job<'a>(&'a self, body: &'a [u8]) -> ShutdownFuture<'a, DeliveryAction>;
}

/// Ack boundary used by shutdown drain tests.
pub trait ShutdownDeliveryAcker {
    /// Applies the processor delivery action.
    fn apply(&mut self, action: DeliveryAction) -> ShutdownFuture<'_, ()>;
}

/// Summary of one drain loop.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownDrainReport {
    /// Number of deliveries fully processed through ack/nack.
    pub processed: usize,
    /// Last delivery action applied by the drain.
    pub last_action: Option<DeliveryAction>,
}

/// Drains a delivery stream until cancellation or stream exhaustion.
///
/// The loop never starts another delivery after cancellation is observed, but a delivery already
/// being processed is allowed to finish through result publication and ack/nack.
///
/// # Errors
///
/// Returns [`WorkerError`] when the processor or acker fails.
pub async fn drain_until_cancelled<S, A>(
    deliveries: &mut S,
    token: CancellationToken,
    processor: &impl ShutdownJobProcessor,
) -> Result<ShutdownDrainReport, WorkerError>
where
    S: Stream<Item = ShutdownDelivery<A>> + Unpin,
    A: ShutdownDeliveryAcker,
{
    let mut report = ShutdownDrainReport::default();

    loop {
        if token.is_cancelled() {
            tracing::info!(event = "worker_shutdown_requested", "worker_shutdown_requested");
            break;
        }

        tokio::select! {
            () = token.cancelled() => {
                tracing::info!(event = "worker_shutdown_requested", "worker_shutdown_requested");
                break;
            }
            delivery = deliveries.next() => {
                let Some(mut delivery) = delivery else {
                    break;
                };
                let action = processor.process_job(&delivery.body).await?;
                delivery.acker.apply(action).await?;
                report.processed += 1;
                report.last_action = Some(action);

                if token.is_cancelled() {
                    tracing::info!(event = "worker_shutdown_requested", "worker_shutdown_requested");
                    break;
                }
            }
        }
    }

    tracing::info!(
        event = "worker_shutdown_complete",
        processed = report.processed,
        "worker_shutdown_complete"
    );
    Ok(report)
}
