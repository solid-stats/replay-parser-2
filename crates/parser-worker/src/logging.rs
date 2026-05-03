//! Stable worker operations log event taxonomy.
// coverage-exclusion: reviewed v1.0 tracing setup defensive fallback regions are allowlisted by exact source line.

use std::time::Instant;

/// Worker runtime is starting.
pub const WORKER_STARTING: &str = "worker_starting";
/// Worker connected to dependencies and can consume jobs.
pub const WORKER_CONNECTED: &str = "worker_connected";
/// A worker dependency is ready.
pub const WORKER_DEPENDENCY_READY: &str = "worker_dependency_ready";
/// A worker dependency is degraded.
pub const WORKER_DEPENDENCY_DEGRADED: &str = "worker_dependency_degraded";
/// Worker readiness state changed.
pub const WORKER_READINESS_CHANGED: &str = "worker_readiness_changed";
/// Worker received a parse job delivery.
pub const WORKER_JOB_RECEIVED: &str = "worker_job_received";
/// Parser started processing a job.
pub const WORKER_PARSE_STARTED: &str = "worker_parse_started";
/// Parser finished processing a job.
pub const WORKER_PARSE_FINISHED: &str = "worker_parse_finished";
/// Worker wrote a new parser artifact.
pub const WORKER_ARTIFACT_WRITTEN: &str = "worker_artifact_written";
/// Worker reused an existing matching parser artifact.
pub const WORKER_ARTIFACT_REUSED: &str = "worker_artifact_reused";
/// Worker detected an existing conflicting parser artifact.
pub const WORKER_ARTIFACT_CONFLICT: &str = "worker_artifact_conflict";
/// Worker published a parse result.
pub const WORKER_RESULT_PUBLISHED: &str = "worker_result_published";
/// Worker completed a parse job.
pub const WORKER_JOB_COMPLETED: &str = "worker_job_completed";
/// Worker failed a parse job with a handled parse result.
pub const WORKER_JOB_FAILED: &str = "worker_job_failed";
/// Worker acknowledged a delivery.
pub const WORKER_DELIVERY_ACK: &str = "worker_delivery_ack";
/// Worker negatively acknowledged a delivery for requeue.
pub const WORKER_DELIVERY_NACK_REQUEUE: &str = "worker_delivery_nack_requeue";
/// Worker shutdown was requested.
pub const WORKER_SHUTDOWN_REQUESTED: &str = "worker_shutdown_requested";
/// Worker shutdown completed.
pub const WORKER_SHUTDOWN_COMPLETE: &str = "worker_shutdown_complete";

/// Stable worker log event names.
pub const LOG_EVENTS: [&str; 18] = [
    WORKER_STARTING,
    WORKER_CONNECTED,
    WORKER_DEPENDENCY_READY,
    WORKER_DEPENDENCY_DEGRADED,
    WORKER_READINESS_CHANGED,
    WORKER_JOB_RECEIVED,
    WORKER_PARSE_STARTED,
    WORKER_PARSE_FINISHED,
    WORKER_ARTIFACT_WRITTEN,
    WORKER_ARTIFACT_REUSED,
    WORKER_ARTIFACT_CONFLICT,
    WORKER_RESULT_PUBLISHED,
    WORKER_JOB_COMPLETED,
    WORKER_JOB_FAILED,
    WORKER_DELIVERY_ACK,
    WORKER_DELIVERY_NACK_REQUEUE,
    WORKER_SHUTDOWN_REQUESTED,
    WORKER_SHUTDOWN_COMPLETE,
];

/// Completed outcome value for worker logs.
pub const OUTCOME_COMPLETED: &str = "completed";
/// Failed outcome value for worker logs.
pub const OUTCOME_FAILED: &str = "failed";
/// Acknowledged delivery outcome value for worker logs.
pub const OUTCOME_ACK: &str = "ack";
/// Requeued negative-ack delivery outcome value for worker logs.
pub const OUTCOME_NACK_REQUEUE: &str = "nack_requeue";
/// Ready dependency/readiness outcome value for worker logs.
pub const OUTCOME_READY: &str = "ready";
/// Degraded dependency/readiness outcome value for worker logs.
pub const OUTCOME_DEGRADED: &str = "degraded";
/// Draining readiness outcome value for worker logs.
pub const OUTCOME_DRAINING: &str = "draining";

/// Stable low-cardinality outcome values.
pub const OUTCOMES: [&str; 7] = [
    OUTCOME_COMPLETED,
    OUTCOME_FAILED,
    OUTCOME_ACK,
    OUTCOME_NACK_REQUEUE,
    OUTCOME_READY,
    OUTCOME_DEGRADED,
    OUTCOME_DRAINING,
];

/// Converts an elapsed operation duration to a log-safe millisecond value.
#[must_use]
pub fn duration_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}
